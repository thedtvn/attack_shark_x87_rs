//! Lighting configuration.
//!
//! Layout of the 128 byte `LedConfig` block (built by `0x0048ec70`):
//!
//! | offset          | meaning                                                     |
//! |-----------------|-------------------------------------------------------------|
//! | 0x03            | debounce, stage - 1 (app: 0..=4; firmware accepts more)     |
//! | 0x09            | 1 when the per-key ("self-define") colour group is active   |
//! | 0x0A            | active effect (hardware id, see [`Effect`])                 |
//! | 0x0B            | colour-group start (`StartGp` + profile)                    |
//! | 0x16            | Tap: 0 = off, otherwise sensitivity 1..=127                 |
//! | 0x18            | sleep in 30 s units, 0 = never (wireless only)              |
//! | 0x38, 0x39      | always `0xFF`                                               |
//! | 0x3A + 2*i      | brightness for effect slot `i` (0..=4)                      |
//! | 0x3B + 2*i      | `speed << 4 | colour` for effect slot `i`                   |
//! | 0x7E, 0x7F      | validity marker `0x5A 0xA5`                                 |
//!
//! There are 19 effect slots; slot `i` belongs to `LedOpt{i+1}` in `KB.ini`.
//! `colour` is an index (0..=6) into that effect's 7-colour palette (the
//! `LedRgbTable`); 7 means random/rainbow.

use crate::Rgb;

pub const LED_CONFIG_LEN: usize = 0x80;
pub const LED_MARKER: [u8; 2] = [0x5A, 0xA5];
pub const EFFECT_SLOTS: usize = 19;
pub const PALETTE_COLORS: usize = 7;

pub const OFF_DEBOUNCE: usize = 0x03;
pub const OFF_TAP: usize = 0x16;
pub const OFF_SLEEP: usize = 0x18;
pub const OFF_PERKEY_ACTIVE: usize = 0x09;

/// Sleep choices offered by the vendor app, in seconds (`SleepUI` default).
pub const SLEEP_PRESETS: [u32; 10] = [30, 60, 90, 120, 180, 240, 300, 600, 900, 1200];
/// Debounce stages shown by the vendor app (`ShowDebounce`). Higher raw values
/// work too; they add input delay.
pub const DEBOUNCE_APP_MAX: u8 = 5;
/// Highest tap sensitivity the app's slider offers.
pub const TAP_MAX: u8 = 127;
pub const OFF_EFFECT: usize = 0x0A;
pub const OFF_SLOTS: usize = 0x3A;
pub const OFF_MARKER: usize = 0x7E;

/// Max brightness / speed level.
pub const MAX_LEVEL: u8 = 4;
/// Colour index meaning "random / colourful".
pub const COLOR_RANDOM: u8 = 7;

/// Lighting effects. The value is the hardware id stored at config byte 0x0A
/// (first column of `LedOptN` in `KB.ini`). Names come from `Text/en/text.xml`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Effect {
    FixedOn = 1,
    Respire = 2,
    Rainbow = 3,
    FlashAway = 4,
    Raindrops = 5,
    RainbowWheel = 6,
    RipplesShining = 7,
    StarsTwinkle = 8,
    ShadowDisappear = 9,
    RetroSnake = 10,
    NeonStream = 11,
    Reaction = 12,
    SineWave = 13,
    RetinueScanning = 14,
    RotatingWindmill = 15,
    ColorfulWaterfall = 16,
    Blossoming = 17,
    RotatingStorm = 18,
    /// Per-key colours ("Self-define"), see [`crate::Keyboard::set_per_key_colors`].
    Custom = 21,
}

impl Effect {
    pub const ALL: [Effect; 19] = [
        Effect::FixedOn,
        Effect::Respire,
        Effect::Rainbow,
        Effect::FlashAway,
        Effect::Raindrops,
        Effect::RainbowWheel,
        Effect::RipplesShining,
        Effect::StarsTwinkle,
        Effect::ShadowDisappear,
        Effect::RetroSnake,
        Effect::NeonStream,
        Effect::Reaction,
        Effect::SineWave,
        Effect::RetinueScanning,
        Effect::RotatingWindmill,
        Effect::ColorfulWaterfall,
        Effect::Blossoming,
        Effect::RotatingStorm,
        Effect::Custom,
    ];

    pub fn from_id(id: u8) -> Option<Effect> {
        Self::ALL.iter().copied().find(|e| *e as u8 == id)
    }

    /// Slot in the per-effect tables (`LedOptN` → `N-1`).
    pub fn slot(self) -> usize {
        Self::ALL.iter().position(|e| *e == self).unwrap()
    }
}

/// Settings stored per effect slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectSettings {
    /// 0..=4
    pub brightness: u8,
    /// 0..=4
    pub speed: u8,
    /// 0..=6 palette index, 7 = random.
    pub color: u8,
}

/// The 128 byte lighting block, kept raw so unknown bytes round-trip.
#[derive(Clone, PartialEq, Eq)]
pub struct LedConfig(pub [u8; LED_CONFIG_LEN]);

impl std::fmt::Debug for LedConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LedConfig")
            .field("effect", &self.effect_id())
            .field("per_key_active", &self.0[OFF_PERKEY_ACTIVE])
            .field("debounce_stage", &self.debounce_stage())
            .field("tap", &self.tap())
            .field("sleep_secs", &self.sleep_secs())
            .field("valid", &self.is_valid())
            .field("current", &self.effect().map(|e| self.settings(e)))
            .finish()
    }
}

impl LedConfig {
    pub fn is_valid(&self) -> bool {
        self.0[OFF_MARKER..OFF_MARKER + 2] == LED_MARKER
    }

    pub fn effect_id(&self) -> u8 {
        self.0[OFF_EFFECT]
    }

    pub fn effect(&self) -> Option<Effect> {
        Effect::from_id(self.effect_id())
    }

    pub fn set_effect(&mut self, effect: Effect) {
        self.0[OFF_EFFECT] = effect as u8;
        self.0[OFF_PERKEY_ACTIVE] = (effect == Effect::Custom) as u8;
    }

    pub fn settings(&self, effect: Effect) -> EffectSettings {
        let o = OFF_SLOTS + effect.slot() * 2;
        EffectSettings {
            brightness: self.0[o],
            speed: self.0[o + 1] >> 4,
            color: self.0[o + 1] & 0x0F,
        }
    }

    pub fn set_settings(&mut self, effect: Effect, s: EffectSettings) {
        let o = OFF_SLOTS + effect.slot() * 2;
        self.0[o] = s.brightness.min(MAX_LEVEL);
        self.0[o + 1] = (s.speed.min(MAX_LEVEL) << 4) | (s.color & 0x0F);
    }

    /// Debounce stage as shown by the app (1-based). Stored as stage - 1.
    pub fn debounce_stage(&self) -> u8 {
        self.0[OFF_DEBOUNCE] + 1
    }

    /// Set debounce stage (1-based). The app offers 1..=5; larger values are
    /// accepted by the firmware and increase key latency.
    pub fn set_debounce_stage(&mut self, stage: u8) {
        self.0[OFF_DEBOUNCE] = stage.max(1) - 1;
    }

    /// Tap sensitivity, `None` when Tap is off.
    pub fn tap(&self) -> Option<u8> {
        match self.0[OFF_TAP] {
            0 => None,
            v => Some(v),
        }
    }

    /// `None` turns Tap off; `Some(1..=127)` turns it on with that sensitivity.
    pub fn set_tap(&mut self, sensitivity: Option<u8>) {
        self.0[OFF_TAP] = sensitivity.map_or(0, |s| s.clamp(1, TAP_MAX));
    }

    /// Wireless sleep timeout in seconds, `None` = never sleep.
    pub fn sleep_secs(&self) -> Option<u32> {
        match self.0[OFF_SLEEP] {
            0 => None,
            v => Some(v as u32 * 30),
        }
    }

    /// Set the sleep timeout (rounded to 30 s, max 127.5 min). `None` disables.
    pub fn set_sleep_secs(&mut self, secs: Option<u32>) {
        self.0[OFF_SLEEP] = secs.map_or(0, |s| (s.div_ceil(30)).clamp(1, 255) as u8);
    }

    /// Ensure the fixed bytes the driver always writes are present.
    pub fn normalize(&mut self) {
        self.0[0x38] = 0xFF;
        self.0[0x39] = 0xFF;
        self.0[OFF_MARKER..OFF_MARKER + 2].copy_from_slice(&LED_MARKER);
    }
}

/// Size of the palette table payload.
pub const RGB_TABLE_DATA_LEN: usize = 0x1A4;
/// Size actually sent when `CRC=1` (X87): data padded to 512 with marker at 0x1FA.
pub const RGB_TABLE_LEN: usize = 0x200;

/// Per-effect palettes (cmd 0x0A). 20 entries of 7 RGB colours (21 bytes);
/// entry 0 is unused, entry `slot+1` belongs to effect slot `slot`.
#[derive(Clone, PartialEq, Eq)]
pub struct RgbTable(pub [u8; RGB_TABLE_LEN]);

impl RgbTable {
    fn off(effect: Effect, i: usize) -> usize {
        (effect.slot() + 1) * 21 + i * 3
    }

    pub fn color(&self, effect: Effect, i: usize) -> Rgb {
        let o = Self::off(effect, i);
        Rgb(self.0[o], self.0[o + 1], self.0[o + 2])
    }

    pub fn set_color(&mut self, effect: Effect, i: usize, c: Rgb) {
        assert!(i < PALETTE_COLORS);
        let o = Self::off(effect, i);
        self.0[o] = c.0;
        self.0[o + 1] = c.1;
        self.0[o + 2] = c.2;
    }

    pub fn normalize(&mut self) {
        self.0[0x1FA] = 0x5A;
        self.0[0x1FB] = 0xA5;
    }
}

/// Per-key colour planes for a colour group (cmd 0x06).
///
/// Three planes of [`crate::layout::LED_SLOTS`] bytes (R, G, B for the X87,
/// `GameRGBIndex=0,1,2`), sent as `MatrixLen * 3` = 384 bytes.
#[derive(Clone, PartialEq, Eq)]
pub struct PerKeyColors(pub [u8; PER_KEY_LEN]);

pub const PER_KEY_LEN: usize = 0x80 * 3;

impl Default for PerKeyColors {
    fn default() -> Self {
        PerKeyColors([0; PER_KEY_LEN])
    }
}

impl PerKeyColors {
    pub fn set(&mut self, led: u8, c: Rgb) {
        let i = led as usize;
        assert!(i < crate::layout::LED_SLOTS);
        let p = crate::layout::LED_SLOTS;
        self.0[i] = c.0;
        self.0[p + i] = c.1;
        self.0[2 * p + i] = c.2;
    }

    pub fn get(&self, led: u8) -> Rgb {
        let i = led as usize;
        let p = crate::layout::LED_SLOTS;
        Rgb(self.0[i], self.0[p + i], self.0[2 * p + i])
    }

    pub fn fill(&mut self, c: Rgb) {
        for &(_, i) in crate::layout::KEYS {
            self.set(i, c);
        }
    }
}
