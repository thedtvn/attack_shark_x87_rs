//! Physical keys of the X87 (from `Dev/kb/KB.ini`).
//!
//! [`Key`] names every physical key; [`Key::index`] is its LED / key-matrix
//! index (the same index is used for per-key colour planes and for remapping).
//!
//! ```
//! use attack_shark_x87::layout::Key;
//! assert_eq!(Key::CapsLock.index(), 3);
//! assert_eq!(Key::from_name("capslock"), Some(Key::CapsLock));
//! assert_eq!(Key::Digit1.name(), "1");
//! assert_eq!("Space".parse::<Key>(), Ok(Key::Space));
//! ```

/// Number of LED slots in a colour plane (`0x7e`).
pub const LED_SLOTS: usize = 126;

/// A physical key on the X87. Variants are in keyboard order (row by row).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Key {
    /// `Esc`, index 0.
    Esc,
    /// `F1`, index 12.
    F1,
    /// `F2`, index 18.
    F2,
    /// `F3`, index 24.
    F3,
    /// `F4`, index 30.
    F4,
    /// `F5`, index 36.
    F5,
    /// `F6`, index 42.
    F6,
    /// `F7`, index 48.
    F7,
    /// `F8`, index 54.
    F8,
    /// `F9`, index 60.
    F9,
    /// `F10`, index 66.
    F10,
    /// `F11`, index 72.
    F11,
    /// `F12`, index 78.
    F12,
    /// `PrtSc`, index 84.
    PrtSc,
    /// `ScrLock`, index 90.
    ScrLock,
    /// `Pause`, index 96.
    Pause,
    /// ```, index 1.
    Grave,
    /// `1`, index 7.
    Digit1,
    /// `2`, index 13.
    Digit2,
    /// `3`, index 19.
    Digit3,
    /// `4`, index 25.
    Digit4,
    /// `5`, index 31.
    Digit5,
    /// `6`, index 37.
    Digit6,
    /// `7`, index 43.
    Digit7,
    /// `8`, index 49.
    Digit8,
    /// `9`, index 55.
    Digit9,
    /// `0`, index 61.
    Digit0,
    /// `-`, index 67.
    Minus,
    /// `=`, index 73.
    Equal,
    /// `Backspace`, index 79.
    Backspace,
    /// `Insert`, index 85.
    Insert,
    /// `Home`, index 91.
    Home,
    /// `PgUp`, index 97.
    PgUp,
    /// `Tab`, index 2.
    Tab,
    /// `Q`, index 8.
    Q,
    /// `W`, index 14.
    W,
    /// `E`, index 20.
    E,
    /// `R`, index 26.
    R,
    /// `T`, index 32.
    T,
    /// `Y`, index 38.
    Y,
    /// `U`, index 44.
    U,
    /// `I`, index 50.
    I,
    /// `O`, index 56.
    O,
    /// `P`, index 62.
    P,
    /// `[`, index 68.
    BracketLeft,
    /// `]`, index 74.
    BracketRight,
    /// `\`, index 80.
    Backslash,
    /// `Delete`, index 86.
    Delete,
    /// `End`, index 92.
    End,
    /// `PgDn`, index 98.
    PgDn,
    /// `CapsLock`, index 3.
    CapsLock,
    /// `A`, index 9.
    A,
    /// `S`, index 15.
    S,
    /// `D`, index 21.
    D,
    /// `F`, index 27.
    F,
    /// `G`, index 33.
    G,
    /// `H`, index 39.
    H,
    /// `J`, index 45.
    J,
    /// `K`, index 51.
    K,
    /// `L`, index 57.
    L,
    /// `;`, index 63.
    Semicolon,
    /// `'`, index 69.
    Quote,
    /// `Enter`, index 81.
    Enter,
    /// `LShift`, index 4.
    LShift,
    /// `Z`, index 16.
    Z,
    /// `X`, index 22.
    X,
    /// `C`, index 28.
    C,
    /// `V`, index 34.
    V,
    /// `B`, index 40.
    B,
    /// `N`, index 46.
    N,
    /// `M`, index 52.
    M,
    /// `,`, index 58.
    Comma,
    /// `.`, index 64.
    Period,
    /// `/`, index 70.
    Slash,
    /// `RShift`, index 82.
    RShift,
    /// `Up`, index 94.
    Up,
    /// `LCtrl`, index 5.
    LCtrl,
    /// `LWin`, index 11.
    LWin,
    /// `LAlt`, index 17.
    LAlt,
    /// `Space`, index 35.
    Space,
    /// `RAlt`, index 53.
    RAlt,
    /// `Fn`, index 59.
    Fn,
    /// `App`, index 65.
    App,
    /// `RCtrl`, index 83.
    RCtrl,
    /// `Left`, index 89.
    Left,
    /// `Down`, index 95.
    Down,
    /// `Right`, index 101.
    Right,
}

impl Key {
    /// Every key, in keyboard order.
    pub const ALL: [Key; 87] = [
        Key::Esc,
        Key::F1,
        Key::F2,
        Key::F3,
        Key::F4,
        Key::F5,
        Key::F6,
        Key::F7,
        Key::F8,
        Key::F9,
        Key::F10,
        Key::F11,
        Key::F12,
        Key::PrtSc,
        Key::ScrLock,
        Key::Pause,
        Key::Grave,
        Key::Digit1,
        Key::Digit2,
        Key::Digit3,
        Key::Digit4,
        Key::Digit5,
        Key::Digit6,
        Key::Digit7,
        Key::Digit8,
        Key::Digit9,
        Key::Digit0,
        Key::Minus,
        Key::Equal,
        Key::Backspace,
        Key::Insert,
        Key::Home,
        Key::PgUp,
        Key::Tab,
        Key::Q,
        Key::W,
        Key::E,
        Key::R,
        Key::T,
        Key::Y,
        Key::U,
        Key::I,
        Key::O,
        Key::P,
        Key::BracketLeft,
        Key::BracketRight,
        Key::Backslash,
        Key::Delete,
        Key::End,
        Key::PgDn,
        Key::CapsLock,
        Key::A,
        Key::S,
        Key::D,
        Key::F,
        Key::G,
        Key::H,
        Key::J,
        Key::K,
        Key::L,
        Key::Semicolon,
        Key::Quote,
        Key::Enter,
        Key::LShift,
        Key::Z,
        Key::X,
        Key::C,
        Key::V,
        Key::B,
        Key::N,
        Key::M,
        Key::Comma,
        Key::Period,
        Key::Slash,
        Key::RShift,
        Key::Up,
        Key::LCtrl,
        Key::LWin,
        Key::LAlt,
        Key::Space,
        Key::RAlt,
        Key::Fn,
        Key::App,
        Key::RCtrl,
        Key::Left,
        Key::Down,
        Key::Right,
    ];

    /// LED / key-matrix index.
    pub const fn index(self) -> u8 {
        match self {
            Key::Esc          => 0,
            Key::F1           => 12,
            Key::F2           => 18,
            Key::F3           => 24,
            Key::F4           => 30,
            Key::F5           => 36,
            Key::F6           => 42,
            Key::F7           => 48,
            Key::F8           => 54,
            Key::F9           => 60,
            Key::F10          => 66,
            Key::F11          => 72,
            Key::F12          => 78,
            Key::PrtSc        => 84,
            Key::ScrLock      => 90,
            Key::Pause        => 96,
            Key::Grave        => 1,
            Key::Digit1       => 7,
            Key::Digit2       => 13,
            Key::Digit3       => 19,
            Key::Digit4       => 25,
            Key::Digit5       => 31,
            Key::Digit6       => 37,
            Key::Digit7       => 43,
            Key::Digit8       => 49,
            Key::Digit9       => 55,
            Key::Digit0       => 61,
            Key::Minus        => 67,
            Key::Equal        => 73,
            Key::Backspace    => 79,
            Key::Insert       => 85,
            Key::Home         => 91,
            Key::PgUp         => 97,
            Key::Tab          => 2,
            Key::Q            => 8,
            Key::W            => 14,
            Key::E            => 20,
            Key::R            => 26,
            Key::T            => 32,
            Key::Y            => 38,
            Key::U            => 44,
            Key::I            => 50,
            Key::O            => 56,
            Key::P            => 62,
            Key::BracketLeft  => 68,
            Key::BracketRight => 74,
            Key::Backslash    => 80,
            Key::Delete       => 86,
            Key::End          => 92,
            Key::PgDn         => 98,
            Key::CapsLock     => 3,
            Key::A            => 9,
            Key::S            => 15,
            Key::D            => 21,
            Key::F            => 27,
            Key::G            => 33,
            Key::H            => 39,
            Key::J            => 45,
            Key::K            => 51,
            Key::L            => 57,
            Key::Semicolon    => 63,
            Key::Quote        => 69,
            Key::Enter        => 81,
            Key::LShift       => 4,
            Key::Z            => 16,
            Key::X            => 22,
            Key::C            => 28,
            Key::V            => 34,
            Key::B            => 40,
            Key::N            => 46,
            Key::M            => 52,
            Key::Comma        => 58,
            Key::Period       => 64,
            Key::Slash        => 70,
            Key::RShift       => 82,
            Key::Up           => 94,
            Key::LCtrl        => 5,
            Key::LWin         => 11,
            Key::LAlt         => 17,
            Key::Space        => 35,
            Key::RAlt         => 53,
            Key::Fn           => 59,
            Key::App          => 65,
            Key::RCtrl        => 83,
            Key::Left         => 89,
            Key::Down         => 95,
            Key::Right        => 101,
        }
    }

    /// Display name, as used by the vendor app and the web app (e.g. `"1"`, `"CapsLock"`).
    pub const fn name(self) -> &'static str {
        match self {
            Key::Esc          => "Esc",
            Key::F1           => "F1",
            Key::F2           => "F2",
            Key::F3           => "F3",
            Key::F4           => "F4",
            Key::F5           => "F5",
            Key::F6           => "F6",
            Key::F7           => "F7",
            Key::F8           => "F8",
            Key::F9           => "F9",
            Key::F10          => "F10",
            Key::F11          => "F11",
            Key::F12          => "F12",
            Key::PrtSc        => "PrtSc",
            Key::ScrLock      => "ScrLock",
            Key::Pause        => "Pause",
            Key::Grave        => "`",
            Key::Digit1       => "1",
            Key::Digit2       => "2",
            Key::Digit3       => "3",
            Key::Digit4       => "4",
            Key::Digit5       => "5",
            Key::Digit6       => "6",
            Key::Digit7       => "7",
            Key::Digit8       => "8",
            Key::Digit9       => "9",
            Key::Digit0       => "0",
            Key::Minus        => "-",
            Key::Equal        => "=",
            Key::Backspace    => "Backspace",
            Key::Insert       => "Insert",
            Key::Home         => "Home",
            Key::PgUp         => "PgUp",
            Key::Tab          => "Tab",
            Key::Q            => "Q",
            Key::W            => "W",
            Key::E            => "E",
            Key::R            => "R",
            Key::T            => "T",
            Key::Y            => "Y",
            Key::U            => "U",
            Key::I            => "I",
            Key::O            => "O",
            Key::P            => "P",
            Key::BracketLeft  => "[",
            Key::BracketRight => "]",
            Key::Backslash    => "\\",
            Key::Delete       => "Delete",
            Key::End          => "End",
            Key::PgDn         => "PgDn",
            Key::CapsLock     => "CapsLock",
            Key::A            => "A",
            Key::S            => "S",
            Key::D            => "D",
            Key::F            => "F",
            Key::G            => "G",
            Key::H            => "H",
            Key::J            => "J",
            Key::K            => "K",
            Key::L            => "L",
            Key::Semicolon    => ";",
            Key::Quote        => "'",
            Key::Enter        => "Enter",
            Key::LShift       => "LShift",
            Key::Z            => "Z",
            Key::X            => "X",
            Key::C            => "C",
            Key::V            => "V",
            Key::B            => "B",
            Key::N            => "N",
            Key::M            => "M",
            Key::Comma        => ",",
            Key::Period       => ".",
            Key::Slash        => "/",
            Key::RShift       => "RShift",
            Key::Up           => "Up",
            Key::LCtrl        => "LCtrl",
            Key::LWin         => "LWin",
            Key::LAlt         => "LAlt",
            Key::Space        => "Space",
            Key::RAlt         => "RAlt",
            Key::Fn           => "Fn",
            Key::App          => "App",
            Key::RCtrl        => "RCtrl",
            Key::Left         => "Left",
            Key::Down         => "Down",
            Key::Right        => "Right",
        }
    }

    /// Look up a key by display name (case-insensitive), e.g. from a config file.
    pub fn from_name(name: &str) -> Option<Key> {
        Key::ALL.into_iter().find(|k| k.name().eq_ignore_ascii_case(name))
    }

    /// The key at an LED / key-matrix index, if any.
    pub fn from_index(index: u8) -> Option<Key> {
        Key::ALL.into_iter().find(|k| k.index() == index)
    }
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

/// Error from parsing an unknown key name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownKey;

impl std::fmt::Display for UnknownKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("unknown key name")
    }
}

impl std::error::Error for UnknownKey {}

impl std::str::FromStr for Key {
    type Err = UnknownKey;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Key::from_name(s).ok_or(UnknownKey)
    }
}

/// Look up a key's LED/matrix index by display name (case-insensitive).
/// Prefer [`Key`]; this is for names coming from text.
pub fn key_index(name: &str) -> Option<u8> {
    Key::from_name(name).map(Key::index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indices_and_names_are_unique_and_in_range() {
        let mut idx: Vec<u8> = Key::ALL.iter().map(|k| k.index()).collect();
        idx.sort_unstable();
        idx.dedup();
        assert_eq!(idx.len(), Key::ALL.len());
        assert!(idx.iter().all(|&i| (i as usize) < LED_SLOTS));
        let mut names: Vec<String> = Key::ALL.iter().map(|k| k.name().to_ascii_lowercase()).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), Key::ALL.len());
    }

    #[test]
    fn round_trips() {
        for k in Key::ALL {
            assert_eq!(Key::from_name(k.name()), Some(k));
            assert_eq!(Key::from_index(k.index()), Some(k));
            assert_eq!(k.to_string().parse::<Key>(), Ok(k));
            assert_eq!(key_index(k.name()), Some(k.index()));
        }
        assert_eq!(Key::from_name("nope"), None);
        assert_eq!("nope".parse::<Key>(), Err(UnknownKey));
        assert_eq!(Key::from_index(6), None);
    }

    #[test]
    fn matches_vendor_table() {
        // Spot checks against KB.ini / the previous (name, index) table.
        assert_eq!(Key::Esc.index(), 0);
        assert_eq!(Key::Grave.index(), 1);
        assert_eq!(Key::Digit4.index(), 25);
        assert_eq!(Key::Backslash.index(), 80);
        assert_eq!(Key::Backslash.name(), "\\");
        assert_eq!(Key::Fn.index(), 59);
        assert_eq!(Key::Right.index(), 101);
    }
}
