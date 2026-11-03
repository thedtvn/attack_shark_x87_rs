//! Built-in keyboard controls, correlated with vendor KB.ini [FN1], the X87
//! manual and G5KB_ConvertKeyCode. Creating/encoding a binding does not execute
//! it. Remapped-key hold behavior and FN2 activation need hardware validation.
//! Host media controls remain [`crate::KeyAction::Consumer`], not controllers.
use crate::KeyAction;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ControllerAction {
    Bluetooth1 = 0x05000007,
    Bluetooth2 = 0x06000007,
    Bluetooth3 = 0x07000007,
    Receiver24G = 0x08000007,
    BatteryLevel = 0x11000007,
    WindowsKeyLock = 0x01000007,
    /// Destructive. Factory shortcut: hold Fn + Space for three seconds.
    FactoryReset = 0x04000007,
    BacklightEffect = 0x00000008,
    BacklightColor = 0x00000208,
    LightingPowerSaving = 0x0a000007,
    KeyboardLighting = 0x0b000007,
    BacklightBrightnessUp = 0x00010308,
    BacklightBrightnessDown = 0x00020308,
    BacklightSpeedDown = 0x00020408,
    BacklightSpeedUp = 0x00010408,
    LightBarMode = 0x01000008,
    LightBarColor = 0x01000208,
}

impl ControllerAction {
    pub const ALL: [Self; 17] = [
        Self::Bluetooth1,
        Self::Bluetooth2,
        Self::Bluetooth3,
        Self::Receiver24G,
        Self::BatteryLevel,
        Self::WindowsKeyLock,
        Self::FactoryReset,
        Self::BacklightEffect,
        Self::BacklightColor,
        Self::LightingPowerSaving,
        Self::KeyboardLighting,
        Self::BacklightBrightnessUp,
        Self::BacklightBrightnessDown,
        Self::BacklightSpeedDown,
        Self::BacklightSpeedUp,
        Self::LightBarMode,
        Self::LightBarColor,
    ];
    pub fn encode(self) -> u32 {
        self as u32
    }
    pub fn decode(word: u32) -> Option<Self> {
        Self::ALL.into_iter().find(|action| action.encode() == word)
    }
    /// Triggering these controls can disconnect the device or erase settings.
    pub fn is_sensitive(self) -> bool {
        matches!(
            self,
            Self::Bluetooth1
                | Self::Bluetooth2
                | Self::Bluetooth3
                | Self::Receiver24G
                | Self::FactoryReset
        )
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Bluetooth1 => "Bluetooth 1",
            Self::Bluetooth2 => "Bluetooth 2",
            Self::Bluetooth3 => "Bluetooth 3",
            Self::Receiver24G => "2.4 GHz receiver",
            Self::BatteryLevel => "Battery level",
            Self::WindowsKeyLock => "Windows-key lock",
            Self::FactoryReset => "Factory reset (hold 3 seconds)",
            Self::BacklightEffect => "Backlight effect",
            Self::BacklightColor => "Backlight colour",
            Self::LightingPowerSaving => "Lighting / power-saving switch",
            Self::KeyboardLighting => "Keyboard lighting switch",
            Self::BacklightBrightnessUp => "Backlight brightness up",
            Self::BacklightBrightnessDown => "Backlight brightness down",
            Self::BacklightSpeedDown => "Backlight speed down",
            Self::BacklightSpeedUp => "Backlight speed up",
            Self::LightBarMode => "Light-bar mode",
            Self::LightBarColor => "Light-bar colour",
        }
    }
    pub fn factory_key(self) -> &'static str {
        match self {
            Self::Bluetooth1 => "1",
            Self::Bluetooth2 => "2",
            Self::Bluetooth3 => "3",
            Self::Receiver24G => "4",
            Self::BatteryLevel => "Backspace",
            Self::WindowsKeyLock => "LWin",
            Self::FactoryReset => "Space",
            Self::BacklightEffect => "\\",
            Self::BacklightColor => "Enter",
            Self::LightingPowerSaving => "G",
            Self::KeyboardLighting => "X",
            Self::BacklightBrightnessUp => "Up",
            Self::BacklightBrightnessDown => "Down",
            Self::BacklightSpeedDown => "Left",
            Self::BacklightSpeedUp => "Right",
            Self::LightBarMode => "CapsLock",
            Self::LightBarColor => "A",
        }
    }
}

// These Controller-picker choices are ordinary keyboard chords, not proprietary
// commands. Preserve their normal KeyAction::Key representation on read-back.
pub const DISPLAY_TASK_SWITCH: KeyAction = KeyAction::Key {
    modifiers: 8,
    usage: 0x2b,
    usage2: 0,
};
pub const APPLICATION_SWITCH: KeyAction = KeyAction::Key {
    modifiers: 4,
    usage: 0x2b,
    usage2: 0,
};
/// Vendor default is Win + Space (read from hardware); the manual calls it Input method.
pub const INPUT_METHOD: KeyAction = KeyAction::Key {
    modifiers: 8,
    usage: 0x2c,
    usage2: 0,
};

/// Factory FN1 mapping only. FN2/Tap defaults are empty in the supplied KB.ini.
/// None means there is no explicit factory layer action, not necessarily disabled.
pub fn factory_fn1_action(key: &str) -> Option<KeyAction> {
    if let Some(action) = ControllerAction::ALL
        .into_iter()
        .find(|a| a.factory_key().eq_ignore_ascii_case(key))
    {
        return Some(KeyAction::Controller(action));
    }
    Some(match key.to_ascii_uppercase().as_str() {
        "F1" => KeyAction::Consumer(0x70),
        "F2" => KeyAction::Consumer(0x6f),
        "F3" => DISPLAY_TASK_SWITCH,
        "F4" => APPLICATION_SWITCH,
        "F5" => KeyAction::Consumer(0x221),
        "F6" => INPUT_METHOD,
        "F7" => KeyAction::Consumer(0xb6),
        "F8" => KeyAction::Consumer(0xcd),
        "F9" => KeyAction::Consumer(0xb5),
        "F10" => KeyAction::Consumer(0xe2),
        "F11" => KeyAction::Consumer(0xea),
        "F12" => KeyAction::Consumer(0xe9),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn controller_fixtures_roundtrip() {
        let words = [
            0x05000007, 0x06000007, 0x07000007, 0x08000007, 0x11000007, 0x01000007, 0x04000007,
            0x00000008, 0x00000208, 0x0a000007, 0x0b000007, 0x00010308, 0x00020308, 0x00020408,
            0x00010408, 0x01000008, 0x01000208,
        ];
        for (action, word) in ControllerAction::ALL.into_iter().zip(words) {
            assert_eq!(action.encode(), word);
            assert_eq!(KeyAction::decode(word), KeyAction::Controller(action));
            assert_eq!(
                factory_fn1_action(action.factory_key()),
                Some(KeyAction::Controller(action))
            );
            assert!(!action.name().is_empty());
        }
        assert_eq!(ControllerAction::decode(0x09000007), None);
        assert_eq!(KeyAction::decode(0x09000007), KeyAction::Raw(0x09000007));
    }
    #[test]
    fn host_controls_remain_keyboard_or_media() {
        assert_eq!(factory_fn1_action("F11"), Some(KeyAction::Consumer(0xea)));
        assert_eq!(factory_fn1_action("F3"), Some(DISPLAY_TASK_SWITCH));
        assert_eq!(DISPLAY_TASK_SWITCH.encode(), 0x2b000800);
        assert_eq!(APPLICATION_SWITCH.encode(), 0x2b000400);
        assert_eq!(INPUT_METHOD.encode(), 0x2c000800);
        assert_eq!(factory_fn1_action("5"), None);
        assert!(ControllerAction::Receiver24G.is_sensitive());
        assert!(ControllerAction::FactoryReset.is_sensitive());
        assert!(!ControllerAction::LightBarColor.is_sensitive());
    }
}
