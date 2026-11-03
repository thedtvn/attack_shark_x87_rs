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

// ---- geometry -----------------------------------------------------------------

/// Width of the vendor keyboard image (`Dev/kb/keyimg.png`) in pixels.
/// [`Key::rect`] coordinates use this image's pixel space.
pub const IMAGE_WIDTH: u16 = 669;
/// Height of the vendor keyboard image in pixels.
pub const IMAGE_HEIGHT: u16 = 259;

/// A key's rectangle in the vendor image's pixel space, from `KB.ini`.
/// `x1, y1` is the top-left corner; `x2, y2` is the bottom-right (exclusive).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rect {
    pub x1: u16,
    pub y1: u16,
    pub x2: u16,
    pub y2: u16,
}

impl Rect {
    pub const fn new(x1: u16, y1: u16, x2: u16, y2: u16) -> Self {
        Self { x1, y1, x2, y2 }
    }

    pub const fn width(self) -> u16 {
        self.x2 - self.x1
    }

    pub const fn height(self) -> u16 {
        self.y2 - self.y1
    }

    /// Centre point `(x, y)`.
    pub fn center(self) -> (f32, f32) {
        ((self.x1 + self.x2) as f32 / 2.0, (self.y1 + self.y2) as f32 / 2.0)
    }

    /// The smallest rectangle containing both `self` and `other`.
    pub const fn union(self, other: Rect) -> Rect {
        const fn min(a: u16, b: u16) -> u16 { if a < b { a } else { b } }
        const fn max(a: u16, b: u16) -> u16 { if a > b { a } else { b } }
        Rect::new(min(self.x1, other.x1), min(self.y1, other.y1), max(self.x2, other.x2), max(self.y2, other.y2))
    }

    /// Whether the point `(x, y)` is inside the rectangle (for hit-testing).
    pub const fn contains(self, x: u16, y: u16) -> bool {
        x >= self.x1 && x < self.x2 && y >= self.y1 && y < self.y2
    }

    /// The rectangle scaled to a drawing area of `width × height`, as
    /// `(x, y, w, h)`, keeping the image's proportions per axis.
    pub fn scaled(self, width: f32, height: f32) -> (f32, f32, f32, f32) {
        let sx = width / IMAGE_WIDTH as f32;
        let sy = height / IMAGE_HEIGHT as f32;
        (
            self.x1 as f32 * sx,
            self.y1 as f32 * sy,
            self.width() as f32 * sx,
            self.height() as f32 * sy,
        )
    }
}

impl Key {
    /// The key's rectangle in the vendor image (from `KB.ini`), for drawing a
    /// keyboard, hit-testing clicks, or measuring key widths.
    pub const fn rect(self) -> Rect {
        match self {
            Key::Esc          => Rect::new( 18,  20,  51,  53),
            Key::F1           => Rect::new( 89,  20, 122,  53),
            Key::F2           => Rect::new(123,  20, 156,  53),
            Key::F3           => Rect::new(157,  20, 190,  53),
            Key::F4           => Rect::new(191,  20, 224,  53),
            Key::F5           => Rect::new(244,  20, 277,  53),
            Key::F6           => Rect::new(278,  20, 311,  53),
            Key::F7           => Rect::new(313,  20, 346,  53),
            Key::F8           => Rect::new(348,  20, 381,  53),
            Key::F9           => Rect::new(401,  20, 434,  53),
            Key::F10          => Rect::new(435,  20, 468,  53),
            Key::F11          => Rect::new(469,  20, 502,  53),
            Key::F12          => Rect::new(503,  20, 536,  53),
            Key::PrtSc        => Rect::new(548,  20, 581,  53),
            Key::ScrLock      => Rect::new(582,  20, 615,  53),
            Key::Pause        => Rect::new(617,  20, 650,  53),
            Key::Grave        => Rect::new( 18,  67,  51, 100),
            Key::Digit1       => Rect::new( 53,  67,  86, 100),
            Key::Digit2       => Rect::new( 88,  67, 121, 100),
            Key::Digit3       => Rect::new(122,  67, 155, 100),
            Key::Digit4       => Rect::new(156,  67, 189, 100),
            Key::Digit5       => Rect::new(190,  67, 223, 100),
            Key::Digit6       => Rect::new(225,  67, 258, 100),
            Key::Digit7       => Rect::new(259,  67, 292, 100),
            Key::Digit8       => Rect::new(294,  67, 327, 100),
            Key::Digit9       => Rect::new(329,  67, 362, 100),
            Key::Digit0       => Rect::new(363,  67, 396, 100),
            Key::Minus        => Rect::new(398,  67, 431, 100),
            Key::Equal        => Rect::new(433,  67, 466, 100),
            Key::Backspace    => Rect::new(468,  67, 537, 100),
            Key::Insert       => Rect::new(548,  67, 581, 100),
            Key::Home         => Rect::new(582,  67, 615, 100),
            Key::PgUp         => Rect::new(617,  67, 650, 100),
            Key::Tab          => Rect::new( 18, 102,  68, 135),
            Key::Q            => Rect::new( 71, 102, 104, 135),
            Key::W            => Rect::new(105, 102, 138, 135),
            Key::E            => Rect::new(139, 102, 172, 135),
            Key::R            => Rect::new(174, 102, 207, 135),
            Key::T            => Rect::new(209, 102, 242, 135),
            Key::Y            => Rect::new(244, 102, 277, 135),
            Key::U            => Rect::new(278, 102, 311, 135),
            Key::I            => Rect::new(313, 102, 346, 135),
            Key::O            => Rect::new(348, 102, 381, 135),
            Key::P            => Rect::new(383, 102, 416, 135),
            Key::BracketLeft  => Rect::new(418, 102, 451, 135),
            Key::BracketRight => Rect::new(453, 102, 486, 135),
            Key::Backslash    => Rect::new(488, 102, 536, 135),
            Key::Delete       => Rect::new(548, 102, 581, 135),
            Key::End          => Rect::new(582, 102, 615, 135),
            Key::PgDn         => Rect::new(617, 102, 650, 135),
            Key::CapsLock     => Rect::new( 18, 137,  77, 170),
            Key::A            => Rect::new( 79, 136, 112, 169),
            Key::S            => Rect::new(114, 136, 147, 169),
            Key::D            => Rect::new(149, 136, 182, 169),
            Key::F            => Rect::new(183, 136, 216, 169),
            Key::G            => Rect::new(217, 136, 250, 169),
            Key::H            => Rect::new(252, 136, 285, 169),
            Key::J            => Rect::new(287, 136, 320, 169),
            Key::K            => Rect::new(322, 136, 355, 169),
            Key::L            => Rect::new(356, 136, 389, 169),
            Key::Semicolon    => Rect::new(391, 136, 424, 169),
            Key::Quote        => Rect::new(426, 136, 459, 169),
            Key::Enter        => Rect::new(461, 137, 537, 170),
            Key::LShift       => Rect::new( 18, 172,  95, 205),
            Key::Z            => Rect::new( 97, 171, 130, 204),
            Key::X            => Rect::new(131, 171, 164, 204),
            Key::C            => Rect::new(166, 171, 199, 204),
            Key::V            => Rect::new(201, 171, 234, 204),
            Key::B            => Rect::new(235, 171, 268, 204),
            Key::N            => Rect::new(270, 171, 303, 204),
            Key::M            => Rect::new(305, 171, 338, 204),
            Key::Comma        => Rect::new(340, 171, 373, 204),
            Key::Period       => Rect::new(374, 171, 407, 204),
            Key::Slash        => Rect::new(409, 171, 442, 204),
            Key::RShift       => Rect::new(443, 171, 537, 204),
            Key::Up           => Rect::new(583, 172, 616, 205),
            Key::LCtrl        => Rect::new( 18, 207,  60, 240),
            Key::LWin         => Rect::new( 62, 207, 104, 240),
            Key::LAlt         => Rect::new(105, 207, 147, 240),
            Key::Space        => Rect::new(149, 206, 363, 239),
            Key::RAlt         => Rect::new(365, 206, 407, 239),
            Key::Fn           => Rect::new(408, 206, 450, 239),
            Key::App          => Rect::new(452, 206, 494, 239),
            Key::RCtrl        => Rect::new(495, 206, 537, 239),
            Key::Left         => Rect::new(548, 206, 581, 239),
            Key::Down         => Rect::new(583, 206, 616, 239),
            Key::Right        => Rect::new(617, 206, 650, 239),
        }
    }

    /// The key whose rectangle contains `(x, y)` in image pixels, if any.
    /// See also [`KeyGroup::at`].
    pub fn at(x: u16, y: u16) -> Option<Key> {
        Key::ALL.into_iter().find(|k| k.rect().contains(x, y))
    }

    /// Keys sharing this key's row, left to right; [`Key::left`] and
    /// [`Key::right`] step along it.
    fn row_keys(self) -> impl Iterator<Item = Key> {
        LAYOUT[self.row()].iter().flat_map(|c| c.iter().copied())
    }

    /// The key immediately to the left in the same physical row, if any.
    pub fn left(self) -> Option<Key> {
        self.row_keys().filter(|k| k.rect().x2 <= self.rect().x1).max_by_key(|k| k.rect().x2)
    }

    /// The key immediately to the right in the same physical row, if any.
    pub fn right(self) -> Option<Key> {
        self.row_keys().filter(|k| k.rect().x1 >= self.rect().x2).min_by_key(|k| k.rect().x1)
    }
}

impl KeyGroup {
    /// Bounding rectangle of the group's keys (areas and clusters alike), in
    /// the same pixel space as [`Key::rect`]. Gaps between clusters inside an
    /// area are included, so e.g. [`KeyGroup::FKey`] spans Esc to F12.
    pub const fn rect(self) -> Rect {
        let clusters = self.layout();
        let mut r = clusters[0][0].rect();
        let mut c = 0;
        while c < clusters.len() {
            let mut k = 0;
            while k < clusters[c].len() {
                r = r.union(clusters[c][k].rect());
                k += 1;
            }
            c += 1;
        }
        r
    }

    /// The area whose rectangle contains `(x, y)` in image pixels, if any.
    /// Use [`Key::at`] for the exact key, or check [`KeyGroup::CLUSTERS`].
    pub fn at(x: u16, y: u16) -> Option<KeyGroup> {
        KeyGroup::AREAS.into_iter().find(|g| g.rect().contains(x, y))
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

    #[test]
    fn rects_are_valid_and_do_not_overlap() {
        for k in Key::ALL {
            let r = k.rect();
            assert!(r.x1 < r.x2 && r.y1 < r.y2, "{k:?}");
            assert!(r.x2 <= IMAGE_WIDTH && r.y2 <= IMAGE_HEIGHT, "{k:?}");
            let (cx, cy) = r.center();
            assert_eq!(Key::at(cx as u16, cy as u16), Some(k), "{k:?} centre hit-test");
        }
        for (i, a) in Key::ALL.iter().enumerate() {
            for b in &Key::ALL[i + 1..] {
                let (ra, rb) = (a.rect(), b.rect());
                let overlap = ra.x1 < rb.x2 && rb.x1 < ra.x2 && ra.y1 < rb.y2 && rb.y1 < ra.y2;
                assert!(!overlap, "{a:?} overlaps {b:?}");
            }
        }
    }

    #[test]
    fn geometry_matches_layout_rows_and_order() {
        for (r, row) in LAYOUT.iter().enumerate() {
            let keys: Vec<Key> = row.iter().flat_map(|c| c.iter().copied()).collect();
            // Left to right inside each row, and clusters separated by a gap.
            for w in keys.windows(2) {
                assert!(w[0].rect().x2 <= w[1].rect().x1, "row {r}: {:?} then {:?}", w[0], w[1]);
            }
            for c in row.windows(2) {
                let gap = c[1][0].rect().x1 - c[0].last().unwrap().rect().x2;
                assert!(gap > 0, "row {r}: clusters touch");
            }
        }
    }

    #[test]
    fn group_rects_bound_their_keys() {
        let all = KeyGroup::AREAS.into_iter().chain(KeyGroup::CLUSTERS);
        for g in all {
            let r = g.rect();
            for k in g.keys() {
                let kr = k.rect();
                assert!(r.x1 <= kr.x1 && r.y1 <= kr.y1 && kr.x2 <= r.x2 && kr.y2 <= r.y2, "{g:?} misses {k:?}");
            }
            // Tight: each edge is touched by some key.
            assert!(g.keys().any(|k| k.rect().x1 == r.x1), "{g:?} left");
            assert!(g.keys().any(|k| k.rect().x2 == r.x2), "{g:?} right");
            assert!(g.keys().any(|k| k.rect().y1 == r.y1), "{g:?} top");
            assert!(g.keys().any(|k| k.rect().y2 == r.y2), "{g:?} bottom");
        }
        // A single-key cluster is exactly its key.
        assert_eq!(KeyGroup::Esc.rect(), Key::Esc.rect());
        assert_eq!(KeyGroup::ArrowUp.rect(), Key::Up.rect());
        // Clusters in the same row don't overlap.
        for row in LAYOUT {
            for pair in row.windows(2) {
                let a = KeyGroup::CLUSTERS.into_iter().find(|c| c.layout()[0] == pair[0]).unwrap().rect();
                let b = KeyGroup::CLUSTERS.into_iter().find(|c| c.layout()[0] == pair[1]).unwrap().rect();
                assert!(a.x2 < b.x1);
            }
        }
        assert_eq!(KeyGroup::FKey.rect(), Key::Esc.rect().union(Key::F12.rect()));
        assert_eq!(KeyGroup::at(30, 40), Some(KeyGroup::FKey));
        let (cx, cy) = Key::Space.rect().center();
        assert_eq!(KeyGroup::at(cx as u16, cy as u16), Some(KeyGroup::Main));
    }

    #[test]
    fn known_sizes_and_neighbours() {
        assert_eq!(Key::Esc.rect(), Rect::new(18, 20, 51, 53));
        assert_eq!(Key::Q.rect().width(), 33);
        assert!(Key::Space.rect().width() > 5 * Key::Q.rect().width());
        assert!(Key::Backspace.rect().width() > Key::Digit0.rect().width());
        assert_eq!(Key::Esc.left(), None);
        assert_eq!(Key::Esc.right(), Some(Key::F1));
        assert_eq!(Key::F4.right(), Some(Key::F5));
        assert_eq!(Key::A.left(), Some(Key::CapsLock));
        assert_eq!(Key::Enter.right(), None);
        assert_eq!(Key::Down.left(), Some(Key::Left));
        assert_eq!(Key::at(0, 0), None);
        let (x, y, w, h) = Key::Esc.rect().scaled(IMAGE_WIDTH as f32 * 2.0, IMAGE_HEIGHT as f32 * 2.0);
        assert_eq!((x, y, w, h), (36.0, 40.0, 66.0, 66.0));
    }
}
