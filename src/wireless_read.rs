//! Checked receiver stream assembly shared by real reads and offline fixtures.
use crate::wireless::{DATA_LEN, PACKET_LEN, REPORT_ID};
use crate::{Error, Result};

pub(crate) struct ReadStream {
    cmd: u8,
    sub: u8,
    len: usize,
    minimum: usize,
    total: Option<usize>,
    parts: Vec<Option<Vec<u8>>>,
}
impl ReadStream {
    pub(crate) fn new(cmd: u8, sub: u8, len: usize) -> Self {
        let minimum = match cmd {
            0x41 => 504.min(len),
            0x42 => 378.min(len),
            0x49 => 420.min(len),
            _ => len,
        };
        Self {
            cmd,
            sub,
            len,
            minimum,
            total: None,
            parts: vec![None; len.div_ceil(DATA_LEN)],
        }
    }
    pub(crate) fn push(&mut self, r: &[u8]) -> Result<Option<Vec<u8>>> {
        if r.len() != PACKET_LEN {
            return Err(Error::BadResponse);
        }
        if r[0] == 0xff {
            return Err(Error::BadResponse);
        }
        if r[0] != REPORT_ID || r[1] != self.cmd {
            return Ok(None);
        }
        if r[2] & 0x80 != 0 {
            return Err(Error::Checksum);
        }
        if r[4] >> 4 != self.sub {
            return Ok(None);
        }
        let total = r[2] as usize;
        let index = r[3] as usize;
        let n = (r[4] & 15) as usize;
        if total == 0
            || total > self.parts.len()
            || index >= total
            || n == 0
            || n > DATA_LEN
            || (index + 1 < total && n != DATA_LEN)
            || index * DATA_LEN + n > self.len
            || self.total.is_some_and(|old| old != total)
        {
            return Err(Error::BadResponse);
        }
        self.total = Some(total);
        let data = r[5..5 + n].to_vec();
        if self.parts[index].as_ref().is_some_and(|old| old != &data) {
            return Err(Error::BadResponse);
        }
        self.parts[index] = Some(data);
        if self.parts[..total].iter().any(Option::is_none) {
            return Ok(None);
        }
        let actual = (total - 1) * DATA_LEN + self.parts[total - 1].as_ref().unwrap().len();
        if actual < self.minimum {
            return Err(Error::BadResponse);
        }
        let mut out = vec![0; self.len];
        for (i, data) in self.parts[..total].iter().enumerate() {
            let data = data.as_ref().unwrap();
            out[i * DATA_LEN..i * DATA_LEN + data.len()].copy_from_slice(data);
        }
        Ok(Some(out))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn packet(cmd: u8, sub: u8, total: u8, index: u8, n: u8) -> [u8; 20] {
        let mut p = [0; 20];
        p[..5].copy_from_slice(&[19, cmd, total, index, sub << 4 | n]);
        p[5..5 + n as usize].fill(index);
        p
    }
    #[test]
    fn physical_lengths() {
        for (cmd, actual, len) in [(0x41, 504usize, 512), (0x49, 490, 512), (0x42, 378, 378)] {
            let mut s = ReadStream::new(cmd, 0, len);
            let total = actual.div_ceil(14);
            let mut out = None;
            for i in 0..total {
                out = s
                    .push(&packet(
                        cmd,
                        0,
                        total as u8,
                        i as u8,
                        (actual - i * 14).min(14) as u8,
                    ))
                    .unwrap();
            }
            let out = out.unwrap();
            assert_eq!(out.len(), len);
            assert!(out[actual..].iter().all(|&b| b == 0));
        }
    }
    #[test]
    fn page_tags_and_complete_packet_sets() {
        let mut s = ReadStream::new(0x43, 1, 512);
        assert!(s.push(&packet(0x43, 0, 37, 0, 14)).unwrap().is_none());
        assert!(s.push(&packet(0x43, 1, 37, 36, 8)).unwrap().is_none());
        for i in 0..35 {
            let p = packet(0x43, 1, 37, i, 14);
            assert!(s.push(&p).unwrap().is_none());
            assert!(s.push(&p).unwrap().is_none());
        }
        assert_eq!(
            s.push(&packet(0x43, 1, 37, 35, 14)).unwrap().unwrap().len(),
            512
        );
    }
    #[test]
    fn rejects_truncated_and_short_streams() {
        let mut s = ReadStream::new(0x44, 0, 128);
        assert!(s.push(&[0; 19]).is_err());
        assert!(s.push(&packet(0x44, 0, 1, 0, 14)).is_err());
    }
}
