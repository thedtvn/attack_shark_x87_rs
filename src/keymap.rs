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
    /// Mouse button code 1..=5 (1 = left, then right/middle/back/forward).
    Mouse(u8),
    /// The Fn key.
    Fn,
    /// Second Fn layer key (vendor virtual key FB; G5KB_ConvertKeyCode).
    Fn2,
    /// Onboard macro slot (zero-based), with playback mode.
    Macro { slot: u8, repeat: crate::macros::MacroRepeat },
    /// Built-in firmware controls (Bluetooth/receiver, lighting, battery, reset).
    Controller(crate::controller::ControllerAction),
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
    /// Validate macro bindings instead of panicking on untrusted input.
    pub fn try_encode(self) -> crate::Result<u32> {
        if let Self::Macro { slot, repeat } = self {
            if slot as usize >= crate::macros::MAX_MACROS {
                return Err(crate::Error::InvalidArgument("macro slot out of range"));
            }
            repeat.encode()?;
        }
        Ok(self.encode())
    }

    /// Encode a trusted action. Use `try_encode` for externally supplied bindings.
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
            KeyAction::Fn2 => 0x010D,
            KeyAction::Macro { slot, repeat } => {
                let (mode,count) = repeat.encode().expect("macro repeat count must be nonzero");
                assert!((slot as usize) < crate::macros::MAX_MACROS, "macro slot out of range");
                u32::from_le_bytes([3,mode,count,slot])
            }
            KeyAction::Controller(action) => action.encode(),
            KeyAction::Raw(v) => v,
        }
    }

    pub fn decode(v: u32) -> KeyAction {
        if let Some(action) = crate::controller::ControllerAction::decode(v) {
            return KeyAction::Controller(action);
        }
        let b = v.to_le_bytes();
        match b {
            [0, 0, 0, 0] => KeyAction::None,
            [0x0D, 0, 0, 0] => KeyAction::Fn,
            [0x0D, 1, 0, 0] => KeyAction::Fn2,
            [0x00, m, u2, u] => KeyAction::Key { modifiers: m, usage: u, usage2: u2 },
            [0x02, 0x00, hi, lo] => KeyAction::Consumer(u16::from_be_bytes([hi, lo])),
            [0x01, b, 0x01, 0x00] => KeyAction::Mouse(b),
            [3, mode, count, slot] if (slot as usize) < crate::macros::MAX_MACROS => {
                match crate::macros::MacroRepeat::decode(mode,count) {
                    Some(repeat) => KeyAction::Macro {slot, repeat},
                    None => KeyAction::Raw(v),
                }
            }
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
    fn macro_bindings() {
        use crate::macros::MacroRepeat;
        for (repeat, bytes) in [(MacroRepeat::Times(7),[3,1,7,2]),(MacroRepeat::UntilReleased,[3,4,1,2]),(MacroRepeat::UntilAnyKey,[3,2,1,2])] {
            let a=KeyAction::Macro{slot:2,repeat};
            assert_eq!(a.encode().to_le_bytes(),bytes);
            assert_eq!(KeyAction::decode(a.encode()),a);
        }
        assert!(matches!(KeyAction::decode(u32::from_le_bytes([3,8,1,2])),KeyAction::Raw(_)));
    }

    #[test]
    fn known_encodings() {
        // Values seen in the driver's conversion table.
        assert_eq!(KeyAction::Consumer(0xE9).encode(), 0xE900_0002);
        assert_eq!(KeyAction::Consumer(0x183).encode(), 0x8301_0002);
        assert_eq!(KeyAction::Mouse(1).encode(), 0x0001_0101);
        assert_eq!(KeyAction::Fn.encode(), 0x0D);
        assert_eq!(KeyAction::Fn2.encode().to_le_bytes(), [0x0D, 1, 0, 0]);
        assert_eq!(KeyAction::decode(0x010D), KeyAction::Fn2);
        assert!(KeyAction::Macro { slot: 32, repeat: crate::macros::MacroRepeat::Times(1) }.try_encode().is_err());
        assert!(KeyAction::Macro { slot: 0, repeat: crate::macros::MacroRepeat::Times(0) }.try_encode().is_err());
        // HID 0x2C (space) alone, as returned for internal code 0x40.
        assert_eq!(KeyAction::Key { modifiers: 0, usage: 0x2C, usage2: 0 }.encode(), 0x2C00_0000);
        let k = KeyAction::Key { modifiers: modifier::LCTRL, usage: 0x06, usage2: 0 };
        assert_eq!(KeyAction::decode(k.encode()), k);
    }
}
