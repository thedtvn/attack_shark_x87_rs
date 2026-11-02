//! Experimental onboard macro codecs, reverse engineered in MACROS.md.
//! No automatic uploads: replacing the bank affects macro slots in ALL layers.
use crate::{Error, Result};

pub const BANK_SIZE: usize = 4096;
/// Conservative client-side bound, not a measured firmware slot limit.
pub const MAX_MACROS: usize = 32;
pub const MAX_DELAY_MS: u32 = 0xfffff;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacroRepeat {
    Times(u8),
    UntilReleased,
    UntilAnyKey,
}
impl MacroRepeat {
    pub fn encode(self) -> Result<(u8, u8)> {
        match self {
            Self::Times(0) => Err(Error::InvalidArgument("repeat count must be 1..255")),
            Self::Times(n) => Ok((1, n)),
            Self::UntilReleased => Ok((4, 1)),
            Self::UntilAnyKey => Ok((2, 1)),
        }
    }
    pub fn decode(mode: u8, count: u8) -> Option<Self> {
        match (mode, count) {
            (1, 1..=255) => Some(Self::Times(count)),
            (4, 1) => Some(Self::UntilReleased),
            (2, 1) => Some(Self::UntilAnyKey),
            _ => None,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacroAction {
    Key { usage: u8, down: bool },
    MouseButton { mask: u8, down: bool },
    MoveX(i8),
    MoveY(i8),
    Wheel(i8),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacroEvent {
    pub action: MacroAction,
    pub delay_ms: u32,
}
impl MacroEvent {
    pub fn encode(self) -> Result<[u8; 4]> {
        if self.delay_ms > MAX_DELAY_MS {
            return Err(Error::InvalidArgument("macro delay exceeds 20 bits"));
        }
        let (ty, down, val) = match self.action {
            MacroAction::Key { usage, down } if usage > 0 && usage <= 0xe7 => {
                (if usage >= 0xe0 { 1 } else { 0 }, down, usage)
            }
            MacroAction::MouseButton { mask, down } if mask > 0 && mask <= 31 => (2, down, mask),
            MacroAction::MoveX(n) => (3, false, n as u8),
            MacroAction::MoveY(n) => (4, false, n as u8),
            MacroAction::Wheel(n) => (5, false, n as u8),
            _ => return Err(Error::InvalidArgument("invalid macro key or button")),
        };
        Ok([
            (ty << 4) | (if down { 0x80 } else { 0 }) | (self.delay_ms >> 16) as u8,
            (self.delay_ms >> 8) as u8,
            self.delay_ms as u8,
            val,
        ])
    }
    pub fn decode(b: [u8; 4]) -> Result<Self> {
        let ty = (b[0] >> 4) & 7;
        let down = b[0] & 0x80 != 0;
        let action = match ty {
            0 => MacroAction::Key { usage: b[3], down },
            1 if b[3] >= 0xe0 && b[3] <= 0xe7 => MacroAction::Key { usage: b[3], down },
            2 => MacroAction::MouseButton { mask: b[3], down },
            3 if !down => MacroAction::MoveX(b[3] as i8),
            4 if !down => MacroAction::MoveY(b[3] as i8),
            5 if !down => MacroAction::Wheel(b[3] as i8),
            _ => return Err(Error::BadResponse),
        };
        let e = Self {
            action,
            delay_ms: ((b[0] as u32 & 15) << 16) | ((b[1] as u32) << 8) | b[2] as u32,
        };
        if e.encode()? != b {
            return Err(Error::BadResponse);
        }
        Ok(e)
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Macro {
    pub name: String,
    pub events: Vec<MacroEvent>,
}
impl Macro {
    pub fn encode(&self) -> Result<Vec<u8>> {
        let name: Vec<u16> = self.name.encode_utf16().collect();
        if name.is_empty()
            || name.len() > 30
            || self.name.trim().is_empty()
            || self.name.contains('\0')
        {
            return Err(Error::InvalidArgument(
                "macro name must be 1..30 UTF16 units",
            ));
        }
        if self.events.is_empty() || self.events.len() > 1023 {
            return Err(Error::InvalidArgument("macro needs 1..1023 events"));
        }
        let mut b = Vec::with_capacity(1 + name.len() * 2 + self.events.len() * 4);
        b.push((name.len() * 2) as u8);
        for c in name {
            b.extend(c.to_le_bytes());
        }
        for e in &self.events {
            b.extend(e.encode()?);
        }
        Ok(b)
    }
    pub fn decode(b: &[u8]) -> Result<Self> {
        if b.is_empty() || b[0] > 60 || b[0] % 2 != 0 {
            return Err(Error::BadResponse);
        }
        let start = 1 + b[0] as usize;
        if start > b.len() || (b.len() - start) % 4 != 0 {
            return Err(Error::BadResponse);
        }
        let name = String::from_utf16(
            &b[1..start]
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect::<Vec<_>>(),
        )
        .map_err(|_| Error::BadResponse)?;
        let events = b[start..]
            .chunks_exact(4)
            .map(|c| MacroEvent::decode([c[0], c[1], c[2], c[3]]))
            .collect::<Result<Vec<_>>>()?;
        let m = Self { name, events };
        if m.encode()?.as_slice() != b {
            return Err(Error::BadResponse);
        }
        Ok(m)
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MacroBank {
    pub macros: Vec<Macro>,
}
impl MacroBank {
    pub fn encode(&self) -> Result<Vec<u8>> {
        if self.macros.len() > MAX_MACROS {
            return Err(Error::InvalidArgument("client limit is 32 macros"));
        }
        let records = self
            .macros
            .iter()
            .map(Macro::encode)
            .collect::<Result<Vec<_>>>()?;
        let mut offset = records.len() * 4;
        if offset + records.iter().map(Vec::len).sum::<usize>() > BANK_SIZE {
            return Err(Error::InvalidArgument("macro bank exceeds 4096 bytes"));
        }
        let mut out = vec![0u8; BANK_SIZE];
        for (i, b) in records.iter().enumerate() {
            out[i * 4..i * 4 + 2].copy_from_slice(&(offset as u16).to_le_bytes());
            out[i * 4 + 2..i * 4 + 4].copy_from_slice(&(b.len() as u16).to_le_bytes());
            out[offset..offset + b.len()].copy_from_slice(b);
            offset += b.len();
        }
        Ok(out)
    }
    pub fn decode(b: &[u8]) -> Result<Self> {
        if b.len() != BANK_SIZE {
            return Err(Error::BadResponse);
        }
        if b.iter().all(|&x| x == 0) || b.iter().all(|&x| x == 255) {
            return Ok(Self::default());
        }
        let first = u16::from_le_bytes([b[0], b[1]]) as usize;
        if first == 0 || first % 4 != 0 || first / 4 > MAX_MACROS {
            return Err(Error::BadResponse);
        }
        let mut offset = first;
        let mut macros = Vec::new();
        for i in 0..first / 4 {
            let start = u16::from_le_bytes([b[i * 4], b[i * 4 + 1]]) as usize;
            let size = u16::from_le_bytes([b[i * 4 + 2], b[i * 4 + 3]]) as usize;
            if start != offset || size < 5 || start + size > b.len() {
                return Err(Error::BadResponse);
            }
            macros.push(Macro::decode(&b[start..start + size])?);
            offset += size;
        }
        Ok(Self { macros })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn sample() -> Macro {
        Macro {
            name: "A".into(),
            events: vec![
                MacroEvent {
                    action: MacroAction::Key {
                        usage: 4,
                        down: true,
                    },
                    delay_ms: 0,
                },
                MacroEvent {
                    action: MacroAction::Key {
                        usage: 4,
                        down: false,
                    },
                    delay_ms: 50,
                },
            ],
        }
    }
    #[test]
    fn golden_record() {
        let m = sample();
        assert_eq!(
            m.encode().unwrap(),
            vec![2, 65, 0, 128, 0, 0, 4, 0, 0, 50, 4]
        );
        assert_eq!(Macro::decode(&m.encode().unwrap()).unwrap(), m);
    }
    #[test]
    fn bank_roundtrip() {
        let bank = MacroBank {
            macros: vec![sample(), sample()],
        };
        let b = bank.encode().unwrap();
        assert_eq!(&b[..8], &[8, 0, 11, 0, 19, 0, 11, 0]);
        assert_eq!(MacroBank::decode(&b).unwrap(), bank);
    }
    #[test]
    fn malformed() {
        assert!(MacroBank::decode(&[0; 12]).is_err());
        let mut b = MacroBank {
            macros: vec![sample()],
        }
        .encode()
        .unwrap();
        b[0] = 3;
        assert!(MacroBank::decode(&b).is_err());
    }
    #[test]
    fn limits() {
        let mut m = sample();
        m.name = "a".repeat(31);
        assert!(m.encode().is_err());
        let e = MacroEvent {
            action: MacroAction::Wheel(-1),
            delay_ms: 0x100000,
        };
        assert!(e.encode().is_err());
        assert!(MacroRepeat::Times(0).encode().is_err());
    }
    #[test]
    fn signed_motion() {
        let e = MacroEvent {
            action: MacroAction::MoveX(-4),
            delay_ms: 0xabcde,
        };
        assert_eq!(e.encode().unwrap(), [0x3a, 0xbc, 0xde, 0xfc]);
        assert_eq!(MacroEvent::decode(e.encode().unwrap()).unwrap(), e);
    }
}
