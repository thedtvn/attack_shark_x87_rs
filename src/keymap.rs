//! Key remap matrix entries (cmd 0x03), encoded as in `0x0048dc80`.
//!
//! Each key has a 32-bit little-endian entry at its matrix index; a layer holds
//! `MatrixLen` (128) entries = 512 bytes.

/// What a key does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    /// Disabled / no action.
    None,
    /// Keyboard key: HID modifier bitmask + up to two HID usages.
    Key { modifiers: u8, usage: u8, usage2: u8 },
    /// Consumer-control usage (media keys), e.g. 0xE9 volume up.
    Consumer(u16),
    /// Mouse button code 1..=5 (driver uses 1 = left; the others are
    /// right/middle/back/forward in an order not yet confirmed on hardware).
    Mouse(u8),
    /// The Fn key.
    Fn,
    /// Any raw 32-bit value.
    Raw(u32),
}

pub mod modifier {
    pub const LCTRL: u8 = 0x01;
    pub const LSHIFT: u8 = 0x02;
    pub const LALT: u8 = 0x04;
    pub const LGUI: u8 = 0x08;
    pub const RCTRL: u8 = 0x10;
    pub const RSHIFT: u8 = 0x20;
    pub const RALT: u8 = 0x40;
    pub const RGUI: u8 = 0x80;
}

impl KeyAction {
    pub fn encode(self) -> u32 {
        match self {
            KeyAction::None => 0,
            KeyAction::Key { modifiers, usage, usage2 } => {
                u32::from_le_bytes([0x00, modifiers, usage2, usage])
            }
            KeyAction::Consumer(u) => {
                let [lo, hi] = u.to_le_bytes();
                u32::from_le_bytes([0x02, 0x00, hi, lo])
            }
            KeyAction::Mouse(b) => u32::from_le_bytes([0x01, b, 0x01, 0x00]),
            KeyAction::Fn => 0x0D,
            KeyAction::Raw(v) => v,
        }
    }

    pub fn decode(v: u32) -> KeyAction {
        let b = v.to_le_bytes();
        match b {
            [0, 0, 0, 0] => KeyAction::None,
            [0x0D, 0, 0, 0] => KeyAction::Fn,
            [0x00, m, u2, u] => KeyAction::Key { modifiers: m, usage: u, usage2: u2 },
            [0x02, 0x00, hi, lo] => KeyAction::Consumer(u16::from_be_bytes([hi, lo])),
            [0x01, b, 0x01, 0x00] => KeyAction::Mouse(b),
            _ => KeyAction::Raw(v),
        }
    }
}

/// One layer of the key matrix.
#[derive(Clone, PartialEq, Eq)]
pub struct KeyMatrix(pub Vec<u32>);

impl KeyMatrix {
    pub fn from_bytes(b: &[u8]) -> Self {
        KeyMatrix(
            b.chunks_exact(4)
                .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                .collect(),
        )
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.iter().flat_map(|v| v.to_le_bytes()).collect()
    }

    pub fn get(&self, index: u8) -> KeyAction {
        KeyAction::decode(self.0[index as usize])
    }

    pub fn set(&mut self, index: u8, a: KeyAction) {
        self.0[index as usize] = a.encode();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_encodings() {
        // Values seen in the driver's conversion table.
        assert_eq!(KeyAction::Consumer(0xE9).encode(), 0xE900_0002);
        assert_eq!(KeyAction::Consumer(0x183).encode(), 0x8301_0002);
        assert_eq!(KeyAction::Mouse(1).encode(), 0x0001_0101);
        assert_eq!(KeyAction::Fn.encode(), 0x0D);
        // HID 0x2C (space) alone, as returned for internal code 0x40.
        assert_eq!(KeyAction::Key { modifiers: 0, usage: 0x2C, usage2: 0 }.encode(), 0x2C00_0000);
        let k = KeyAction::Key { modifiers: modifier::LCTRL, usage: 0x06, usage2: 0 };
        assert_eq!(KeyAction::decode(k.encode()), k);
    }
}
