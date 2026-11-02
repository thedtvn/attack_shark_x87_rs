//! 2.4G dongle transport (`CDev3632` in OemDrv.exe).
//!
//! The dongle (`3554:FA09`) exposes a vendor collection (usage page 0xFF02,
//! usage 2) with 20 byte input/output reports, report id 0x13.
//! The same data blocks as the wired protocol are tunnelled through it in
//! 14 byte pieces.
//!
//! Packet (`CDev3632::AccessData` 0x00487530 / `SendCMD` 0x004878d0):
//!
//! | off | meaning                                     |
//! |-----|---------------------------------------------|
//! | 0   | 0x13 (report id)                            |
//! | 1   | command                                     |
//! | 2   | total packets                               |
//! | 3   | packet index                                |
//! | 4   | `sub << 4 | len` (len <= 14)                |
//! | 5   | 14 data bytes                               |
//! | 19  | checksum = sum of bytes 0..19 (mod 256)     |
//!
//! Writes: each packet is acknowledged by an input report with the same
//! command at [1] and the packet index at [3]; [2] & 0x80 means checksum
//! error, [0] == 0xFF means failure.
//!
//! Reads: send `[0x13, cmd, 1, 0, sub << 4, 0.., csum]`, then the dongle
//! streams input reports `[_, cmd, total, index, len, data[14], _]` until
//! `index == total - 1`.

use std::thread::sleep;
use std::time::Duration;

use hidapi::HidDevice;

use crate::{Error, Result};

pub const REPORT_ID: u8 = 0x13;
pub const PACKET_LEN: usize = 20;
pub const DATA_LEN: usize = 14;
pub const USAGE_PAGE: u16 = 0xFF02;
pub const USAGE: u16 = 0x0002;

/// Wireless command numbers (differ from the wired ones).
pub mod cmd {
    pub const SET_KEY_MATRIX: u8 = 0x01;
    pub const SET_PER_KEY: u8 = 0x02;
    pub const SET_MACRO: u8 = 0x03;
    pub const SET_LED: u8 = 0x04;
    pub const RESET: u8 = 0x06;
    pub const SET_RGB_TABLE: u8 = 0x09;
    pub const SET_ONBOARD: u8 = 0x0A;
    pub const SET_SCREEN: u8 = 0x0B;
    /// Realtime / music frame (`W3632_SetMusicRealData`, 0x004886d0). Not acked.
    pub const REALTIME: u8 = 0x88;

    pub const GET_KEY_MATRIX: u8 = 0x41;
    pub const GET_PER_KEY: u8 = 0x42;
    pub const GET_MACRO: u8 = 0x43;
    pub const GET_LED: u8 = 0x44;
    pub const GET_RGB_TABLE: u8 = 0x49;
}

pub fn checksum(p: &[u8; PACKET_LEN]) -> u8 {
    p[..PACKET_LEN - 1].iter().fold(0u8, |a, &b| a.wrapping_add(b))
}

pub fn build_packet(cmd: u8, total: u8, index: u8, sub: u8, data: &[u8]) -> [u8; PACKET_LEN] {
    let n = data.len().min(DATA_LEN);
    let mut p = [0u8; PACKET_LEN];
    p[0] = REPORT_ID;
    p[1] = cmd;
    p[2] = total;
    p[3] = index;
    p[4] = (sub << 4) | n as u8;
    p[5..5 + n].copy_from_slice(&data[..n]);
    p[19] = checksum(&p);
    p
}

pub(crate) struct Wireless {
    pub dev: HidDevice,
    /// Delay before each packet (driver: 15 ms).
    pub delay: Duration,
    pub timeout_ms: i32,
}

impl Wireless {
    fn recv(&self) -> Result<Option<[u8; PACKET_LEN]>> {
        self.recv_timeout(self.timeout_ms)
    }

    fn recv_timeout(&self, timeout_ms: i32) -> Result<Option<[u8; PACKET_LEN]>> {
        let mut buf = [0u8; PACKET_LEN];
        let n = self.dev.read_timeout(&mut buf, timeout_ms)?;
        if n == 0 {
            return Ok(None);
        }
        if n != PACKET_LEN { return Err(Error::BadResponse); }
        Ok(Some(buf))
    }

    fn drain(&self) {
        let mut buf = [0u8; PACKET_LEN];
        while matches!(self.dev.read_timeout(&mut buf, 0), Ok(n) if n > 0) {}
    }

    /// Wait for the ack of packet `index` of `cmd`.
    fn wait_ack(&self, cmd: u8, index: u8) -> Result<()> {
        for _ in 0..20 {
            let Some(r) = self.recv()? else {
                return Err(Error::Timeout);
            };
            if r[0] == 0xFF {
                return Err(Error::BadResponse);
            }
            if r[2] & 0x80 != 0 {
                return Err(Error::Checksum);
            }
            if r[1] != cmd {
                continue;
            }
            return if r[3] == index { Ok(()) } else { Err(Error::BadResponse) };
        }
        Err(Error::BadResponse)
    }

    pub fn write(&self, cmd: u8, sub: u8, data: &[u8], retries: usize) -> Result<()> {
        if data.is_empty() {
            return Err(Error::InvalidArgument("empty payload"));
        }
        let total = data.len().div_ceil(DATA_LEN);
        if total > 255 {
            return Err(Error::InvalidArgument("payload too large"));
        }
        self.drain();
        for (i, chunk) in data.chunks(DATA_LEN).enumerate() {
            let pkt = build_packet(cmd, total as u8, i as u8, sub, chunk);
            let mut attempt = 0;
            loop {
                sleep(self.delay);
                self.dev.write(&pkt)?;
                // Reset (6) and commands with the high bit are not acknowledged.
                if cmd & 0x80 != 0 || cmd == cmd::RESET {
                    break;
                }
                match self.wait_ack(cmd, i as u8) {
                    Ok(()) => break,
                    Err(Error::Timeout | Error::Checksum) if attempt < retries => attempt += 1,
                    Err(e) => return Err(e),
                }
            }
        }
        Ok(())
    }

    /// Fire-and-forget write (realtime frames): no ack, fixed gap per packet.
    /// Without the gap the dongle drops packets and the frame turns to noise.
    pub fn stream(&self, cmd: u8, sub: u8, data: &[u8], gap: Duration) -> Result<()> {
        let total = data.len().div_ceil(DATA_LEN).max(1);
        for (i, chunk) in data.chunks(DATA_LEN).enumerate() {
            sleep(gap);
            self.dev.write(&build_packet(cmd, total as u8, i as u8, sub, chunk))?;
        }
        Ok(())
    }

    /// Read one block. Measured on hardware: a 512-byte page streams in about
    /// 160 ms and the receiver cannot pipeline requests (a new request aborts
    /// the previous stream). Radio drops are common, so identical packets are
    /// kept across re-requests of the same block and a stream that goes idle is
    /// re-requested quickly; conflicting data restarts assembly from scratch.
    pub fn read(&self, cmd: u8, sub: u8, len: usize) -> Result<Vec<u8>> {
        let mut stream = crate::wireless_read::ReadStream::new(cmd, sub, len);
        for _ in 0..8 {
            self.drain();
            let mut req = [0u8; PACKET_LEN];
            req[0] = REPORT_ID;
            req[1] = cmd;
            req[2] = 1;
            req[3] = 0;
            req[4] = sub << 4;
            req[19] = checksum(&req);
            self.dev.write(&req)?;

            let mut first = true;
            for _ in 0..512 {
                let Some(r) = self.recv_timeout(if first { 700 } else { 120 })? else { break };
                if r[1] == cmd {
                    first = false;
                }
                match stream.push(&r) {
                    Ok(Some(out)) => return Ok(out),
                    Ok(None) => {}
                    Err(_) => {
                        stream = crate::wireless_read::ReadStream::new(cmd, sub, len);
                        break;
                    }
                }
            }
            sleep(Duration::from_millis(20));
        }
        Err(Error::Timeout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packet() {
        let p = build_packet(0x04, 10, 0, 0, &[1, 2, 3]);
        assert_eq!(&p[..8], &[0x13, 0x04, 10, 0, 3, 1, 2, 3]);
        assert_eq!(p[19], (0x13 + 4 + 10 + 3 + 1 + 2 + 3) as u8);
    }
}
