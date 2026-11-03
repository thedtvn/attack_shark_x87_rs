//! Physical keys of the X87 (from `Dev/kb/KB.ini`).
//!
//! - [`Key`] names every physical key; [`Key::index`] is its LED / key-matrix
//!   index (used both for per-key colours and for remapping).
//! - [`LAYOUT`] is the physical layout as keyboard → row → group, where a group
//!   is a cluster of adjacent keys separated from the next by a gap.
//! - [`KeyGroup`] names the key areas (function row, main block, navigation,
//!   arrows), each as a list of its clusters. Each physical cluster is also a
//!   variant, e.g. [`KeyGroup::F1F4`], [`KeyGroup::NumberRow`];
//!   [`KeyGroup::AREAS`] and [`KeyGroup::CLUSTERS`] list them.
//!
//! The key table is defined once with [`define_keys!`](crate::rs_macro), so
//! names and indices are never repeated by hand.
//!
//! ```
//! use attack_shark_x87::layout::{Key, KeyGroup, LAYOUT};
//!
//! assert_eq!(Key::CapsLock.index(), 3);
//! assert_eq!(Key::from_name("capslock"), Some(Key::CapsLock));
//! assert_eq!("Space".parse::<Key>(), Ok(Key::Space));
//!
//! // keyboard -> row -> group
//! assert_eq!(LAYOUT[0][0], &[Key::Esc]);
//! assert_eq!(LAYOUT[0][1], &[Key::F1, Key::F2, Key::F3, Key::F4]);
//!
//! // Areas
//! assert_eq!(KeyGroup::FKey.layout()[0], &[Key::Esc]);
//! assert_eq!(KeyGroup::Main.layout()[0][0], Key::Grave);   // first row starts with `
//! assert_eq!(Key::Up.group(), KeyGroup::Arrows);
//! assert_eq!(Key::Up.row(), 4);
//! assert_eq!(KeyGroup::F1F4.keys().collect::<Vec<_>>(), [Key::F1, Key::F2, Key::F3, Key::F4]);
//! assert_eq!(Key::F2.cluster(), KeyGroup::F1F4);
//! let all_f_keys: Vec<Key> = KeyGroup::FKey.keys().collect();
//! assert_eq!(all_f_keys.len(), 13);
//! ```

use crate::rs_macro::define_keys;

/// Number of LED slots in a colour plane (`0x7e`).
pub const LED_SLOTS: usize = 126;

define_keys! {
    /// A physical key on the X87, in keyboard order (row by row).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    #[non_exhaustive]
    pub enum Key {
        Esc          => ("Esc", 0),
        F1           => ("F1", 12),
        F2           => ("F2", 18),
        F3           => ("F3", 24),
        F4           => ("F4", 30),
        F5           => ("F5", 36),
        F6           => ("F6", 42),
        F7           => ("F7", 48),
        F8           => ("F8", 54),
        F9           => ("F9", 60),
        F10          => ("F10", 66),
        F11          => ("F11", 72),
        F12          => ("F12", 78),
        PrtSc        => ("PrtSc", 84),
        ScrLock      => ("ScrLock", 90),
        Pause        => ("Pause", 96),
        Grave        => ("`", 1),
        Digit1       => ("1", 7),
        Digit2       => ("2", 13),
        Digit3       => ("3", 19),
        Digit4       => ("4", 25),
        Digit5       => ("5", 31),
        Digit6       => ("6", 37),
        Digit7       => ("7", 43),
        Digit8       => ("8", 49),
        Digit9       => ("9", 55),
        Digit0       => ("0", 61),
        Minus        => ("-", 67),
        Equal        => ("=", 73),
        Backspace    => ("Backspace", 79),
        Insert       => ("Insert", 85),
        Home         => ("Home", 91),
        PgUp         => ("PgUp", 97),
        Tab          => ("Tab", 2),
        Q            => ("Q", 8),
        W            => ("W", 14),
        E            => ("E", 20),
        R            => ("R", 26),
        T            => ("T", 32),
        Y            => ("Y", 38),
        U            => ("U", 44),
        I            => ("I", 50),
        O            => ("O", 56),
        P            => ("P", 62),
        BracketLeft  => ("[", 68),
        BracketRight => ("]", 74),
        Backslash    => ("\\", 80),
        Delete       => ("Delete", 86),
        End          => ("End", 92),
        PgDn         => ("PgDn", 98),
        CapsLock     => ("CapsLock", 3),
        A            => ("A", 9),
        S            => ("S", 15),
        D            => ("D", 21),
        F            => ("F", 27),
        G            => ("G", 33),
        H            => ("H", 39),
        J            => ("J", 45),
        K            => ("K", 51),
        L            => ("L", 57),
        Semicolon    => (";", 63),
        Quote        => ("'", 69),
        Enter        => ("Enter", 81),
        LShift       => ("LShift", 4),
        Z            => ("Z", 16),
        X            => ("X", 22),
        C            => ("C", 28),
        V            => ("V", 34),
        B            => ("B", 40),
        N            => ("N", 46),
        M            => ("M", 52),
        Comma        => (",", 58),
        Period       => (".", 64),
        Slash        => ("/", 70),
        RShift       => ("RShift", 82),
        Up           => ("Up", 94),
        LCtrl        => ("LCtrl", 5),
        LWin         => ("LWin", 11),
        LAlt         => ("LAlt", 17),
        Space        => ("Space", 35),
        RAlt         => ("RAlt", 53),
        Fn           => ("Fn", 59),
        App          => ("App", 65),
        RCtrl        => ("RCtrl", 83),
        Left         => ("Left", 89),
        Down         => ("Down", 95),
        Right        => ("Right", 101),
    }
}

/// A named set of keys: one of the four key areas, or one physical cluster.
///
/// - **Areas** ([`KeyGroup::AREAS`]): `FKey`, `Main`, `Navigation`, `Arrows`.
///   Together they cover every key once. `layout()` returns the area's
///   clusters, e.g. `FKey` = `[[Esc], [F1..F4], [F5..F8], [F9..F12]]`.
/// - **Clusters** ([`KeyGroup::CLUSTERS`]): runs of adjacent keys separated
///   from the next by a gap, e.g. `F1F4` = `[F1, F2, F3, F4]`. `layout()`
///   returns the single cluster, `[[F1, F2, F3, F4]]`.
///
/// [`LAYOUT`] arranges the clusters physically: keyboard → row → cluster.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum KeyGroup {
    // ---- areas ----
    /// Area: Esc and the function keys, `[Esc] [F1–F4] [F5–F8] [F9–F12]`.
    FKey,
    /// Area: the main block, one cluster per row from the number row down.
    Main,
    /// Area: `PrtSc ScrLock Pause`, `Insert Home PgUp`, `Delete End PgDn`.
    Navigation,
    /// Area: the arrow keys, `[Up]` then `[Left Down Right]`.
    Arrows,

    // ---- physical clusters ----
    /// Cluster `[Esc]`.
    Esc,
    /// Cluster `[F1 F2 F3 F4]`.
    F1F4,
    /// Cluster `[F5 F6 F7 F8]`.
    F5F8,
    /// Cluster `[F9 F10 F11 F12]`.
    F9F12,
    /// Cluster `[PrtSc ScrLock Pause]`.
    PrintScrollPause,
    /// Cluster `` [` 1 2 3 4 5 6 7 8 9 0 - = Backspace] ``.
    NumberRow,
    /// Cluster `[Insert Home PgUp]`.
    InsertHomePgUp,
    /// Cluster `[Tab Q W E R T Y U I O P [ ] \]`.
    TopRow,
    /// Cluster `[Delete End PgDn]`.
    DeleteEndPgDn,
    /// Cluster `[CapsLock A S D F G H J K L ; ' Enter]`.
    HomeRow,
    /// Cluster `[LShift Z X C V B N M , . / RShift]`.
    ShiftRow,
    /// Cluster `[Up]`.
    ArrowUp,
    /// Cluster `[LCtrl LWin LAlt Space RAlt Fn App RCtrl]`.
    BottomRow,
    /// Cluster `[Left Down Right]`.
    ArrowLeftDownRight,
}

// Each cluster's keys, written once. Every area, LAYOUT and the cluster
// variants are built from these.
const ESC: &[Key] = &[Key::Esc];
const F1_F4: &[Key] = &[Key::F1, Key::F2, Key::F3, Key::F4];
const F5_F8: &[Key] = &[Key::F5, Key::F6, Key::F7, Key::F8];
const F9_F12: &[Key] = &[Key::F9, Key::F10, Key::F11, Key::F12];
const PRINT_SCROLL_PAUSE: &[Key] = &[Key::PrtSc, Key::ScrLock, Key::Pause];
const NUMBER_ROW: &[Key] = &[
    Key::Grave, Key::Digit1, Key::Digit2, Key::Digit3, Key::Digit4, Key::Digit5, Key::Digit6,
    Key::Digit7, Key::Digit8, Key::Digit9, Key::Digit0, Key::Minus, Key::Equal, Key::Backspace,
];
const INSERT_HOME_PGUP: &[Key] = &[Key::Insert, Key::Home, Key::PgUp];
const TOP_ROW: &[Key] = &[
    Key::Tab, Key::Q, Key::W, Key::E, Key::R, Key::T, Key::Y, Key::U, Key::I, Key::O, Key::P,
    Key::BracketLeft, Key::BracketRight, Key::Backslash,
];
const DELETE_END_PGDN: &[Key] = &[Key::Delete, Key::End, Key::PgDn];
const HOME_ROW: &[Key] = &[
    Key::CapsLock, Key::A, Key::S, Key::D, Key::F, Key::G, Key::H, Key::J, Key::K, Key::L,
    Key::Semicolon, Key::Quote, Key::Enter,
];
const SHIFT_ROW: &[Key] = &[
    Key::LShift, Key::Z, Key::X, Key::C, Key::V, Key::B, Key::N, Key::M, Key::Comma, Key::Period,
    Key::Slash, Key::RShift,
];
const ARROW_UP: &[Key] = &[Key::Up];
const BOTTOM_ROW: &[Key] = &[
    Key::LCtrl, Key::LWin, Key::LAlt, Key::Space, Key::RAlt, Key::Fn, Key::App, Key::RCtrl,
];
const ARROW_LEFT_DOWN_RIGHT: &[Key] = &[Key::Left, Key::Down, Key::Right];

/// The physical layout: keyboard → row → cluster of adjacent keys.
///
/// Rows go top to bottom; clusters and keys go left to right. Every key
/// appears exactly once, and reading it row by row gives [`Key::ALL`].
pub const LAYOUT: &[&[&[Key]]] = &[
    &[ESC, F1_F4, F5_F8, F9_F12, PRINT_SCROLL_PAUSE],
    &[NUMBER_ROW, INSERT_HOME_PGUP],
    &[TOP_ROW, DELETE_END_PGDN],
    &[HOME_ROW],
    &[SHIFT_ROW, ARROW_UP],
    &[BOTTOM_ROW, ARROW_LEFT_DOWN_RIGHT],
];

impl KeyGroup {
    /// The four areas, top-left to bottom-right. They cover every key once.
    pub const AREAS: [KeyGroup; 4] = [KeyGroup::FKey, KeyGroup::Main, KeyGroup::Navigation, KeyGroup::Arrows];

    /// The 14 physical clusters, in keyboard order (row by row, left to right).
    pub const CLUSTERS: [KeyGroup; 14] = [
        KeyGroup::Esc,
        KeyGroup::F1F4,
        KeyGroup::F5F8,
        KeyGroup::F9F12,
        KeyGroup::PrintScrollPause,
        KeyGroup::NumberRow,
        KeyGroup::InsertHomePgUp,
        KeyGroup::TopRow,
        KeyGroup::DeleteEndPgDn,
        KeyGroup::HomeRow,
        KeyGroup::ShiftRow,
        KeyGroup::ArrowUp,
        KeyGroup::BottomRow,
        KeyGroup::ArrowLeftDownRight,
    ];

    /// Whether this is one of the four areas (as opposed to a single cluster).
    pub const fn is_area(self) -> bool {
        matches!(self, KeyGroup::FKey | KeyGroup::Main | KeyGroup::Navigation | KeyGroup::Arrows)
    }

    /// The group's clusters of adjacent keys.
    ///
    /// For an area, its clusters (for [`KeyGroup::Main`] and
    /// [`KeyGroup::Navigation`], one per row). For a cluster, just itself.
    pub const fn layout(self) -> &'static [&'static [Key]] {
        match self {
            KeyGroup::FKey => &[ESC, F1_F4, F5_F8, F9_F12],
            KeyGroup::Main => &[NUMBER_ROW, TOP_ROW, HOME_ROW, SHIFT_ROW, BOTTOM_ROW],
            KeyGroup::Navigation => &[PRINT_SCROLL_PAUSE, INSERT_HOME_PGUP, DELETE_END_PGDN],
            KeyGroup::Arrows => &[ARROW_UP, ARROW_LEFT_DOWN_RIGHT],
            KeyGroup::Esc => &[ESC],
            KeyGroup::F1F4 => &[F1_F4],
            KeyGroup::F5F8 => &[F5_F8],
            KeyGroup::F9F12 => &[F9_F12],
            KeyGroup::PrintScrollPause => &[PRINT_SCROLL_PAUSE],
            KeyGroup::NumberRow => &[NUMBER_ROW],
            KeyGroup::InsertHomePgUp => &[INSERT_HOME_PGUP],
            KeyGroup::TopRow => &[TOP_ROW],
            KeyGroup::DeleteEndPgDn => &[DELETE_END_PGDN],
            KeyGroup::HomeRow => &[HOME_ROW],
            KeyGroup::ShiftRow => &[SHIFT_ROW],
            KeyGroup::ArrowUp => &[ARROW_UP],
            KeyGroup::BottomRow => &[BOTTOM_ROW],
            KeyGroup::ArrowLeftDownRight => &[ARROW_LEFT_DOWN_RIGHT],
        }
    }

    /// Every key in the group, left to right and top to bottom.
    pub fn keys(self) -> impl Iterator<Item = Key> {
        self.layout().iter().flat_map(|cluster| cluster.iter().copied())
    }

    /// Whether `key` is in this group.
    pub fn contains(self, key: Key) -> bool {
        self.keys().any(|k| k == key)
    }
}

impl Key {
    /// The area this key belongs to (`FKey`, `Main`, `Navigation` or `Arrows`).
    pub fn group(self) -> KeyGroup {
        KeyGroup::AREAS
            .into_iter()
            .find(|g| g.contains(self))
            .expect("every key is in an area")
    }

    /// The physical cluster this key belongs to, e.g. `Key::F2` → `KeyGroup::F1F4`.
    pub fn cluster(self) -> KeyGroup {
        KeyGroup::CLUSTERS
            .into_iter()
            .find(|g| g.contains(self))
            .expect("every key is in a cluster")
    }

    /// Physical row, 0 (function row) to 5 (bottom row).
    pub fn row(self) -> usize {
        LAYOUT
            .iter()
            .position(|row| row.iter().any(|cluster| cluster.contains(&self)))
            .expect("every key is in LAYOUT")
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
        assert_eq!(Key::COUNT, 87);
        let mut idx: Vec<u8> = Key::ALL.iter().map(|k| k.index()).collect();
        idx.sort_unstable();
        idx.dedup();
        assert_eq!(idx.len(), Key::COUNT);
        assert!(idx.iter().all(|&i| (i as usize) < LED_SLOTS));
        let mut names: Vec<String> = Key::ALL.iter().map(|k| k.name().to_ascii_lowercase()).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), Key::COUNT);
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
        assert_eq!(Key::Esc.index(), 0);
        assert_eq!(Key::Grave.index(), 1);
        assert_eq!(Key::Digit4.index(), 25);
        assert_eq!(Key::Backslash.index(), 80);
        assert_eq!(Key::Backslash.name(), "\\");
        assert_eq!(Key::Fn.index(), 59);
        assert_eq!(Key::Right.index(), 101);
    }

    #[test]
    fn layout_lists_every_key_once_in_keyboard_order() {
        let flat: Vec<Key> = LAYOUT.iter().flat_map(|r| r.iter()).flat_map(|c| c.iter().copied()).collect();
        assert_eq!(flat, Key::ALL.to_vec());
        assert_eq!(LAYOUT.len(), 6);
        assert_eq!(LAYOUT[0].len(), 5, "Esc, F1-F4, F5-F8, F9-F12, PrtSc-Pause");
        assert_eq!(LAYOUT[0][4], &[Key::PrtSc, Key::ScrLock, Key::Pause]);
        assert_eq!(LAYOUT[1][0].last(), Some(&Key::Backspace));
        assert_eq!(LAYOUT[5][1], &[Key::Left, Key::Down, Key::Right]);
    }

    #[test]
    fn groups_partition_the_layout() {
        // Every group cluster is a cluster of LAYOUT and every LAYOUT cluster
        // is in exactly one group.
        let layout_clusters: Vec<&[Key]> = LAYOUT.iter().flat_map(|r| r.iter().copied()).collect();
        let group_clusters: Vec<&[Key]> = KeyGroup::AREAS.iter().flat_map(|g| g.layout().iter().copied()).collect();
        assert_eq!(layout_clusters.len(), group_clusters.len());
        for c in &layout_clusters {
            assert_eq!(group_clusters.iter().filter(|g| *g == c).count(), 1);
        }
        let total: usize = KeyGroup::AREAS.iter().map(|g| g.keys().count()).sum();
        assert_eq!(total, Key::COUNT);
    }

    #[test]
    fn cluster_variants_match_the_layout_once() {
        // CLUSTERS, in order, are exactly the clusters of LAYOUT.
        let layout: Vec<&[Key]> = LAYOUT.iter().flat_map(|r| r.iter().copied()).collect();
        let clusters: Vec<&[Key]> = KeyGroup::CLUSTERS.iter().map(|c| c.layout()[0]).collect();
        assert_eq!(layout, clusters);
        for c in KeyGroup::CLUSTERS {
            assert!(!c.is_area());
            assert_eq!(c.layout().len(), 1, "a cluster is one run of keys");
        }
        for a in KeyGroup::AREAS {
            assert!(a.is_area());
        }
        // Each cluster belongs to exactly one area.
        for c in KeyGroup::CLUSTERS {
            let owners = KeyGroup::AREAS.iter().filter(|a| a.layout().contains(&c.layout()[0])).count();
            assert_eq!(owners, 1, "{c:?}");
        }
        // Every key's cluster contains it, and clusters partition the keyboard.
        for k in Key::ALL {
            assert!(k.cluster().contains(k));
        }
        let total: usize = KeyGroup::CLUSTERS.iter().map(|c| c.keys().count()).sum();
        assert_eq!(total, Key::COUNT);
    }

    #[test]
    fn named_clusters() {
        let keys = |g: KeyGroup| g.keys().collect::<Vec<_>>();
        assert_eq!(keys(KeyGroup::F1F4), [Key::F1, Key::F2, Key::F3, Key::F4]);
        assert_eq!(keys(KeyGroup::NumberRow).last(), Some(&Key::Backspace));
        assert_eq!(keys(KeyGroup::HomeRow)[0], Key::CapsLock);
        assert_eq!(keys(KeyGroup::ArrowLeftDownRight), [Key::Left, Key::Down, Key::Right]);
        assert_eq!(KeyGroup::FKey.layout(), [KeyGroup::Esc, KeyGroup::F1F4, KeyGroup::F5F8, KeyGroup::F9F12].map(|c| c.layout()[0]));
        assert_eq!(KeyGroup::Arrows.layout(), [KeyGroup::ArrowUp, KeyGroup::ArrowLeftDownRight].map(|c| c.layout()[0]));
        assert_eq!(Key::F2.cluster(), KeyGroup::F1F4);
        assert_eq!(Key::Space.cluster(), KeyGroup::BottomRow);
        assert_eq!(Key::Up.cluster(), KeyGroup::ArrowUp);
        assert_eq!(Key::Esc.cluster(), KeyGroup::Esc);
        assert_eq!(Key::Esc.group(), KeyGroup::FKey);
    }

    #[test]
    fn group_and_row_lookup() {
        assert_eq!(KeyGroup::FKey.layout(), &[&[Key::Esc][..],
            &[Key::F1, Key::F2, Key::F3, Key::F4], &[Key::F5, Key::F6, Key::F7, Key::F8],
            &[Key::F9, Key::F10, Key::F11, Key::F12]]);
        assert_eq!(KeyGroup::Main.layout().len(), 5);
        assert_eq!(KeyGroup::Main.layout()[0].len(), 14);
        assert_eq!(Key::Esc.group(), KeyGroup::FKey);
        assert_eq!(Key::Space.group(), KeyGroup::Main);
        assert_eq!(Key::PgDn.group(), KeyGroup::Navigation);
        assert_eq!(Key::Left.group(), KeyGroup::Arrows);
        assert_eq!(Key::Esc.row(), 0);
        assert_eq!(Key::Grave.row(), 1);
        assert_eq!(Key::Enter.row(), 3);
        assert_eq!(Key::Space.row(), 5);
        assert!(KeyGroup::Navigation.contains(Key::PrtSc));
        assert!(!KeyGroup::Main.contains(Key::Up));
    }
}
