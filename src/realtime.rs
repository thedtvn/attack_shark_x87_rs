//! Realtime lighting ("music mode").
//!
//! The keyboard has no audio-reactive firmware effect. The vendor app's Music
//! page captures system audio with WASAPI loopback
//! (`Audio_LoopbackCaptureThread`, 0x00483310), runs an FFT into 23 bands,
//! renders a frame on the PC (`Music_RenderFrame`, 0x00480150) and streams it
//! to the keyboard about every 20-35 ms from `Music_Thread` (0x00482b80).
//! While frames keep arriving they override the stored effect; keys that a
//! frame doesn't list stay dark.
//!
//! This module only covers the transport, so you can drive the keys from any
//! source (audio, game state, notifications…).
//!
//! # Wire format
//!
//! Compact colour groups, at most 200 bytes:
//!
//! ```text
//! repeat { R, G, B, count, led_index × count }
//! ```
//!
//! * 2.4G dongle: command 0x88 over the 20-byte transport, sub = 1, no acks,
//!   about 10 ms between packets. Verified on hardware.
//! * USB cable: command 0x08 feature report (`G5KB_SetMusicRealData`). Not yet
//!   tested on hardware.
//! * End streaming: `[0, 0, 0]` with sub = 2.
//!
//! The app also has a full 126×RGB frame variant (0x17A bytes) for another
//! device type; the X87 dongle ignores it.

use std::collections::BTreeMap;

use crate::{layout::LED_SLOTS, Error, Result, Rgb};

/// Largest frame the firmware accepts (the driver clamps to 200).
pub const MAX_FRAME_LEN: usize = 200;

/// Frame type sent in the sub-index field.
pub const SUB_FRAME: u8 = 1;
pub const SUB_END: u8 = 2;

/// One realtime frame: a colour per LED index. Unset LEDs are off.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Frame {
    leds: BTreeMap<u8, Rgb>,
}

impl Frame {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set one LED by index ([`crate::Key::index`]). Black removes it.
    pub fn set(&mut self, led: u8, c: Rgb) -> &mut Self {
        assert!((led as usize) < LED_SLOTS, "LED index out of range");
        if c == Rgb(0, 0, 0) {
            self.leds.remove(&led);
        } else {
            self.leds.insert(led, c);
        }
        self
    }

    /// Set one key. Black removes it.
    pub fn set_key(&mut self, key: crate::Key, c: Rgb) -> &mut Self {
        self.set(key.index(), c)
    }

    /// Light every key with one colour.
    pub fn fill(&mut self, c: Rgb) -> &mut Self {
        for key in crate::Key::ALL {
            self.set(key.index(), c);
        }
        self
    }

    pub fn clear(&mut self) -> &mut Self {
        self.leds.clear();
        self
    }

    pub fn get(&self, led: u8) -> Rgb {
        self.leds.get(&led).copied().unwrap_or_default()
    }

    pub fn len(&self) -> usize {
        self.leds.len()
    }

    pub fn is_empty(&self) -> bool {
        self.leds.is_empty()
    }

    /// Number of distinct colours. Each costs 4 bytes plus 1 per key, so use
    /// few colours (≈ ≤ 20 when lighting the whole board) to stay under 200.
    pub fn color_count(&self) -> usize {
        self.groups().len()
    }

    fn groups(&self) -> BTreeMap<[u8; 3], Vec<u8>> {
        let mut g: BTreeMap<[u8; 3], Vec<u8>> = BTreeMap::new();
        for (&led, &Rgb(r, gg, b)) in &self.leds {
            g.entry([r, gg, b]).or_default().push(led);
        }
        g
    }

    /// Encode to the compact wire format. Fails if it doesn't fit.
    pub fn encode(&self) -> Result<Vec<u8>> {
        let out = self.encode_lossy();
        let needed: usize = self.groups().values().map(|v| 4 + v.len()).sum();
        if needed > MAX_FRAME_LEN {
            return Err(Error::InvalidArgument(
                "frame too large (>200 bytes): use fewer distinct colours",
            ));
        }
        Ok(out)
    }

    /// Encode, dropping colour groups that don't fit in 200 bytes
    /// (largest groups first so the most keys stay lit).
    pub fn encode_lossy(&self) -> Vec<u8> {
        let mut groups: Vec<_> = self.groups().into_iter().collect();
        groups.sort_by_key(|(_, v)| std::cmp::Reverse(v.len()));
        let mut out = Vec::with_capacity(MAX_FRAME_LEN);
        for ([r, g, b], leds) in groups {
            let mut leds = leds.as_slice();
            let room = MAX_FRAME_LEN.saturating_sub(out.len());
            if room < 5 {
                break;
            }
            if 4 + leds.len() > room {
                leds = &leds[..room - 4];
            }
            out.extend_from_slice(&[r, g, b, leds.len() as u8]);
            out.extend_from_slice(leds);
        }
        if out.is_empty() {
            // An empty frame still has to blank the board.
            out.extend_from_slice(&[0, 0, 0, 0]);
        }
        out
    }
}

/// Quantise a colour to `levels` steps per channel. Helps keep the number of
/// distinct colours (and so the frame size) down.
pub fn quantize(c: Rgb, levels: u8) -> Rgb {
    let l = levels.max(1) as u16;
    let q = |v: u8| (((v as u16 * l + 127) / 255) * 255 / l) as u8;
    Rgb(q(c.0), q(c.1), q(c.2))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_groups() {
        let mut f = Frame::new();
        f.set(9, Rgb(255, 0, 0)).set(14, Rgb(255, 0, 0)).set(35, Rgb(0, 255, 0));
        let b = f.encode().unwrap();
        // red group (2 keys) first, then green
        assert_eq!(b, vec![255, 0, 0, 2, 9, 14, 0, 255, 0, 1, 35]);
    }

    #[test]
    fn empty_frame_blanks() {
        assert_eq!(Frame::new().encode().unwrap(), vec![0, 0, 0, 0]);
    }

    #[test]
    fn too_many_colours() {
        let mut f = Frame::new();
        for (n, key) in crate::Key::ALL.into_iter().enumerate() {
            f.set_key(key, Rgb(n as u8 + 1, 0, 0));
        }
        assert!(f.encode().is_err());
        assert!(f.encode_lossy().len() <= MAX_FRAME_LEN);
    }

    #[test]
    fn full_board_one_colour_fits() {
        let mut f = Frame::new();
        f.fill(Rgb(1, 2, 3));
        assert_eq!(f.encode().unwrap().len(), 4 + crate::Key::ALL.len());
    }

    #[test]
    fn quantize_levels() {
        assert_eq!(quantize(Rgb(0, 128, 255), 2), Rgb(0, 127, 255));
    }
}
