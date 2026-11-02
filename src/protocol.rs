//! Low level wire format, reverse engineered from `CDevG5KB::AccessData`
//! (`0x0048c9e0` in OemDrv.exe).
//!
//! Every transfer is a 520 (0x208) byte HID *feature* report:
//!
//! | offset | meaning                                   |
//! |--------|-------------------------------------------|
//! | 0      | report id (6 for the X87)                 |
//! | 1      | command (`| 0x80` for reads)              |
//! | 2      | sub index (layer / colour group)          |
//! | 3      | always 0                                  |
//! | 4      | total number of chunks                    |
//! | 5      | chunk index                               |
//! | 6..8   | payload length of this chunk (u16 LE)     |
//! | 8..520 | payload (max 512 bytes)                   |
//!
//! Writes: one `SetFeature` per chunk.
//! Reads: `SetFeature` with the read command, then `GetFeature` returns the
//! same header with the payload filled in.

pub const REPORT_LEN: usize = 0x208;
pub const HEADER_LEN: usize = 8;
pub const CHUNK_LEN: usize = 0x200;

/// Report id used by devices of type 0x18 (the X87). Other G5KB devices use 9.
pub const REPORT_ID_X87: u8 = 0x06;
pub const REPORT_ID_OTHER: u8 = 0x09;

pub const READ_FLAG: u8 = 0x80;

/// Command bytes understood by the keyboard firmware.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Command {
    /// 6 byte device identification ("password"), read only, sub = 1.
    Password = 0x02,
    /// Key remap matrix, sub = layer (0..4), `MatrixLen * 4` bytes.
    KeyMatrix = 0x03,
    /// 128 byte lighting / settings block.
    LedConfig = 0x04,
    /// Macro buffer.
    Macro = 0x05,
    /// Per-key colours for "self-define" mode, sub = colour group.
    PerKeyColors = 0x06,
    /// Music / real time data (used by the audio visualiser).
    MusicRealtime = 0x08,
    /// Per-effect colour palette table.
    LedRgbTable = 0x0A,
    /// Screen parameters (models with a display).
    ScreenParam = 0x0B,
    /// GIF parameters (models with a display).
    GifParam = 0x0D,
    /// Second real time command (vtable slot 0x3c).
    Realtime2 = 0x0E,
    /// Factory reset, 1 byte payload.
    Reset = 0x11,
}

/// Build one feature report.
pub fn build_packet(
    report_id: u8,
    cmd: u8,
    sub: u8,
    chunk_count: u8,
    chunk_index: u8,
    payload_len: u16,
    payload: &[u8],
) -> [u8; REPORT_LEN] {
    let mut p = [0u8; REPORT_LEN];
    p[0] = report_id;
    p[1] = cmd;
    p[2] = sub;
    p[3] = 0;
    p[4] = chunk_count;
    p[5] = chunk_index;
    p[6..8].copy_from_slice(&payload_len.to_le_bytes());
    let n = payload.len().min(CHUNK_LEN);
    p[HEADER_LEN..HEADER_LEN + n].copy_from_slice(&payload[..n]);
    p
}

/// Number of 512 byte chunks needed for `len` bytes (as computed by the driver).
pub fn chunk_count(len: usize) -> usize {
    len.div_ceil(CHUNK_LEN)
}

/// Validate a numbered USB reply. Short (136-byte) settings replies still
/// include the report ID and the complete eight-byte header.
pub fn parse_reply<'a>(data: &'a [u8], request: &[u8; REPORT_LEN]) -> crate::Result<&'a [u8]> {
    let size = u16::from_le_bytes([request[6], request[7]]) as usize;
    if size > CHUNK_LEN || data.len() < HEADER_LEN + size || data[..HEADER_LEN] != request[..HEADER_LEN] {
        return Err(crate::Error::BadResponse);
    }
    Ok(&data[HEADER_LEN..HEADER_LEN + size])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packet_layout() {
        let p = build_packet(6, 0x04, 0, 1, 0, 128, &[0xAA; 128]);
        assert_eq!(&p[..8], &[6, 4, 0, 0, 1, 0, 128, 0]);
        assert_eq!(p[8], 0xAA);
        assert_eq!(p[8 + 127], 0xAA);
        assert_eq!(p[8 + 128], 0);
        assert_eq!(p.len(), 520);
    }

    #[test]
    fn chunks() {
        assert_eq!(chunk_count(1), 1);
        assert_eq!(chunk_count(512), 1);
        assert_eq!(chunk_count(513), 2);
    }
}
