//! Userspace driver for the **ATTACK SHARK X87** keyboard.
//!
//! Reverse engineered from the vendor tool `OemDrv.exe` (class `CDevG5KB`).
//! See `PROTOCOL.md` for the full write-up.
//!
//! ```no_run
//! use attack_shark_x87::{Keyboard, Effect, EffectSettings, Rgb};
//! let kb = Keyboard::open()?;
//! kb.set_effect(Effect::Respire, EffectSettings { brightness: 4, speed: 2, color: 0 })?;
//! # Ok::<(), attack_shark_x87::Error>(())
//! ```

pub mod asynchronous;
pub mod controller;
pub mod macros;
pub mod keymap;
pub mod layout;
pub mod led;
pub mod progress;
pub mod protocol;
pub mod realtime;
pub mod wireless;
mod wireless_read;

use std::thread::sleep;
use std::time::Duration;

use hidapi::{HidApi, HidDevice};

pub use asynchronous::KeyboardAsync;
pub use controller::ControllerAction;
pub use progress::SaveProgress;
pub use macros::{Macro, MacroAction, MacroBank, MacroEvent, MacroRepeat};
pub use keymap::{KeyAction, KeyMatrix};
pub use led::{Effect, EffectSettings, LedConfig, PerKeyColors, RgbTable};
pub use protocol::Command;
pub use realtime::Frame;

use protocol::*;
use wireless::{cmd as wcmd, Wireless};

pub const VID_WIRED: u16 = 0x258A;
pub const PID_WIRED: u16 = 0x010C;
pub const VID_WIRELESS: u16 = 0x3554;
pub const PID_WIRELESS: u16 = 0xFA09;

/// Vendor interface used for configuration over USB.
pub const USAGE_PAGE: u16 = 0xFF00;
pub const USAGE: u16 = 0x0001;

/// Key matrix entries per layer (`MatrixLen`).
pub const MATRIX_LEN: usize = 0x80;
pub const LAYERS: u8 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rgb(pub u8, pub u8, pub u8);

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("hid error: {0}")]
    Hid(#[from] hidapi::HidError),
    #[error("ATTACK SHARK X87 not found (USB 258A:010C or 2.4G dongle 3554:FA09)")]
    NotFound,
    #[error("device returned an unexpected response")]
    BadResponse,
    #[error("timed out waiting for the keyboard (is it switched on / in 2.4G mode?)")]
    Timeout,
    #[error("keyboard reported a checksum error")]
    Checksum,
    #[error("not supported over this connection")]
    Unsupported,
    #[error("invalid argument: {0}")]
    InvalidArgument(&'static str),
    /// The [`KeyboardAsync`] worker thread has stopped.
    #[error("keyboard worker thread stopped")]
    Disconnected,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Connection {
    Wired,
    Wireless,
}

/// A logical data block, mapped to the right command on each transport.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Block {
    KeyMatrix,
    LedConfig,
    Macro,
    PerKeyColors,
    RgbTable,
    ScreenParam,
}

impl Block {
    fn wired(self) -> u8 {
        (match self {
            Block::KeyMatrix => Command::KeyMatrix,
            Block::LedConfig => Command::LedConfig,
            Block::Macro => Command::Macro,
            Block::PerKeyColors => Command::PerKeyColors,
            Block::RgbTable => Command::LedRgbTable,
            Block::ScreenParam => Command::ScreenParam,
        }) as u8
    }

    /// (set, get) on the 2.4G dongle.
    fn wireless(self) -> (u8, Option<u8>) {
        match self {
            Block::KeyMatrix => (wcmd::SET_KEY_MATRIX, Some(wcmd::GET_KEY_MATRIX)),
            Block::LedConfig => (wcmd::SET_LED, Some(wcmd::GET_LED)),
            Block::Macro => (wcmd::SET_MACRO, Some(wcmd::GET_MACRO)),
            Block::PerKeyColors => (wcmd::SET_PER_KEY, Some(wcmd::GET_PER_KEY)),
            Block::RgbTable => (wcmd::SET_RGB_TABLE, Some(wcmd::GET_RGB_TABLE)),
            Block::ScreenParam => (wcmd::SET_SCREEN, None),
        }
    }
}

enum Transport {
    Wired { dev: HidDevice, report_id: u8, delay: Duration },
    Wireless(Wireless),
}

pub struct Keyboard {
    t: Transport,
}

impl Keyboard {
    /// Open the first X87 found (USB cable preferred, then the 2.4G dongle).
    pub fn open() -> Result<Self> {
        let api = HidApi::new()?;
        Self::open_with(&api)
    }

    pub fn open_with(api: &HidApi) -> Result<Self> {
        match Self::open_wired(api) {
            Err(Error::NotFound) => Self::open_wireless(api),
            result => result,
        }
    }

    /// USB cable: collection `FF00:0001` with a 0x208 byte feature report
    /// (`CDevG5KB::FindHIDDevice`, `0x0048c630`).
    ///
    /// The keyboard exposes three `FF00:0001` collections (4-byte input,
    /// 6-byte feature, 520-byte feature); only the 520-byte one accepts the
    /// protocol, so each candidate is probed with a LED-config read.
    pub fn open_wired(api: &HidApi) -> Result<Self> {
        let mut last = Error::NotFound;
        for info in api.device_list().filter(|d| {
            d.vendor_id() == VID_WIRED
                && d.product_id() == PID_WIRED
                && d.usage_page() == USAGE_PAGE
                && d.usage() == USAGE
        }) {
            let dev = match info.open_device(api) {
                Ok(d) => d,
                Err(e) => {
                    last = e.into();
                    continue;
                }
            };
            let kb = Keyboard {
                t: Transport::Wired { dev, report_id: REPORT_ID_X87, delay: Duration::from_millis(20) },
            };
            match kb.led_config() {
                Ok(_) => return Ok(kb),
                Err(e) => last = e,
            }
        }
        Err(last)
    }

    /// 2.4G dongle: collection `FF02:0002`, 20 byte reports (`CDev3632`).
    pub fn open_wireless(api: &HidApi) -> Result<Self> {
        let info = api
            .device_list()
            .find(|d| {
                d.vendor_id() == VID_WIRELESS
                    && d.product_id() == PID_WIRELESS
                    && d.usage_page() == wireless::USAGE_PAGE
                    && d.usage() == wireless::USAGE
            })
            .ok_or(Error::NotFound)?;
        Ok(Keyboard {
            t: Transport::Wireless(Wireless {
                dev: info.open_device(api)?,
                delay: Duration::from_millis(15),
                timeout_ms: 2000,
            }),
        })
    }

    pub fn connection(&self) -> Connection {
        match self.t {
            Transport::Wired { .. } => Connection::Wired,
            Transport::Wireless(_) => Connection::Wireless,
        }
    }

    pub fn hid(&self) -> &HidDevice {
        match &self.t {
            Transport::Wired { dev, .. } => dev,
            Transport::Wireless(w) => &w.dev,
        }
    }

    fn send_feature(dev: &HidDevice, pkt: &[u8; REPORT_LEN]) -> Result<()> {
        // The driver retries 3 times with a 70 ms pause.
        let mut last = None;
        for _ in 0..3 {
            match dev.send_feature_report(pkt) {
                Ok(()) => return Ok(()),
                Err(e) => {
                    last = Some(e);
                    sleep(Duration::from_millis(70));
                }
            }
        }
        Err(last.unwrap().into())
    }

    /// Read the 4096-byte bank in eight 512-byte units (USB chunks or 2.4G
    /// pages), calling `on_page(done_of_8)` after each.
    fn read_macro_pages(&self, on_page: &mut dyn FnMut(f32)) -> Result<Vec<u8>> {
        let mut bank = Vec::with_capacity(macros::BANK_SIZE);
        match &self.t {
            Transport::Wireless(w) => {
                for page in 0..8u8 {
                    bank.extend(w.read(wcmd::GET_MACRO, page, 512)?);
                    on_page((page + 1) as f32 / 8.0);
                }
            }
            Transport::Wired { dev, report_id, delay } => {
                let cmd = Block::Macro.wired() | READ_FLAG;
                for i in 0..8u8 {
                    sleep(*delay);
                    let pkt = build_packet(*report_id, cmd, 0, 8, i, CHUNK_LEN as u16, &[]);
                    Self::send_feature(dev, &pkt)?;
                    sleep(*delay);
                    let mut buf = [0u8; REPORT_LEN];
                    buf[0] = *report_id;
                    let got = dev.get_feature_report(&mut buf)?;
                    bank.extend_from_slice(parse_reply(&buf[..got], &pkt)?);
                    on_page((i + 1) as f32 / 8.0);
                }
            }
        }
        Ok(bank)
    }

    /// Write the 4096-byte bank in eight 512-byte units with progress.
    fn write_macro_pages(&self, data: &[u8], on_page: &mut dyn FnMut(f32)) -> Result<()> {
        if data.len() != macros::BANK_SIZE { return Err(Error::InvalidArgument("macro bank must be 4096 bytes")); }
        match &self.t {
            Transport::Wireless(w) => {
                for (page, bytes) in data.chunks(512).enumerate() {
                    w.write(wcmd::SET_MACRO, page as u8, bytes, 6)?;
                    on_page((page + 1) as f32 / 8.0);
                    sleep(Duration::from_millis(100));
                }
            }
            Transport::Wired { dev, report_id, delay } => {
                let cmd = Block::Macro.wired();
                for (i, chunk) in data.chunks(CHUNK_LEN).enumerate() {
                    sleep(*delay);
                    let pkt = build_packet(*report_id, cmd, 0, 8, i as u8, chunk.len() as u16, chunk);
                    Self::send_feature(dev, &pkt)?;
                    on_page((i + 1) as f32 / 8.0);
                }
            }
        }
        Ok(())
    }

    /// Raw transport write. `cmd` is the transport specific command number
    /// (see [`protocol::Command`] for USB and [`wireless::cmd`] for 2.4G).
    pub fn write_raw(&self, cmd: u8, sub: u8, data: &[u8]) -> Result<()> {
        if data.is_empty() {
            return Err(Error::InvalidArgument("empty payload"));
        }
        match &self.t {
            Transport::Wired { dev, report_id, delay } => {
                let n = chunk_count(data.len());
                for (i, chunk) in data.chunks(CHUNK_LEN).enumerate() {
                    sleep(*delay);
                    let pkt =
                        build_packet(*report_id, cmd, sub, n as u8, i as u8, chunk.len() as u16, chunk);
                    Self::send_feature(dev, &pkt)?;
                }
                Ok(())
            }
            Transport::Wireless(w) => w.write(cmd, sub, data, 5),
        }
    }

    /// Raw transport read of `len` bytes. On USB `cmd` is the base command
    /// (0x80 is added); on 2.4G it is the full read command (0x41..).
    pub fn read_raw(&self, cmd: u8, sub: u8, len: usize) -> Result<Vec<u8>> {
        match &self.t {
            Transport::Wired { dev, report_id, delay } => {
                let n = chunk_count(len);
                let mut out = Vec::with_capacity(len);
                for i in 0..n {
                    let size = (len - out.len()).min(CHUNK_LEN);
                    sleep(*delay);
                    let pkt = build_packet(
                        *report_id,
                        cmd | READ_FLAG,
                        sub,
                        n as u8,
                        i as u8,
                        size as u16,
                        &[],
                    );
                    Self::send_feature(dev, &pkt)?;
                    sleep(*delay);
                    let mut buf = [0u8; REPORT_LEN];
                    buf[0] = *report_id;
                    let got = dev.get_feature_report(&mut buf)?;
                    out.extend_from_slice(parse_reply(&buf[..got], &pkt)?);
                }
                Ok(out)
            }
            Transport::Wireless(w) => w.read(cmd, sub, len),
        }
    }

    /// Read a data block over whichever transport is in use.
    pub fn read_block(&self, block: Block, sub: u8, len: usize) -> Result<Vec<u8>> {
        if block == Block::Macro {
            if sub != 0 || len != macros::BANK_SIZE { return Err(Error::InvalidArgument("macro read requires full 4096-byte bank")); }
            return self.read_macro_pages(&mut |_| {});
        }
        match &self.t {
            Transport::Wired { .. } => self.read_raw(block.wired(), sub, len),
            Transport::Wireless(_) => {
                let get = block.wireless().1.ok_or(Error::Unsupported)?;
                self.read_raw(get, sub, len)
            }
        }
    }

    /// Write a data block over whichever transport is in use.
    pub fn write_block(&self, block: Block, sub: u8, data: &[u8]) -> Result<()> {
        if block == Block::Macro {
            if sub != 0 { return Err(Error::InvalidArgument("macro bank sub-index must be zero")); }
            return self.write_macro_pages(data, &mut |_| {});
        }
        let cmd = match &self.t {
            Transport::Wired { .. } => block.wired(),
            Transport::Wireless(_) => block.wireless().0,
        };
        self.write_raw(cmd, sub, data)
    }

    // ---- info -------------------------------------------------------------

    /// `bcdDevice` of the HID interface (keyboard firmware on USB, dongle
    /// firmware on 2.4G).
    pub fn firmware_version(&self) -> Result<u16> {
        Ok(self.hid().get_device_info()?.release_number())
    }

    /// 6 byte identification code (USB only). The vendor tool compares it to
    /// `Psd=3,0,0,0,1,a7` from `KB.ini`.
    pub fn password(&self) -> Result<[u8; 6]> {
        if self.connection() != Connection::Wired {
            return Err(Error::Unsupported);
        }
        let v = self.read_raw(Command::Password as u8, 1, 6)?;
        Ok(v.try_into().unwrap())
    }

    // ---- lighting ---------------------------------------------------------

    pub fn led_config(&self) -> Result<LedConfig> {
        let v = self.read_block(Block::LedConfig, 0, led::LED_CONFIG_LEN)?;
        let cfg = LedConfig(v.try_into().map_err(|_| Error::BadResponse)?);
        if !cfg.is_valid() { return Err(Error::BadResponse); }
        Ok(cfg)
    }

    pub fn set_led_config(&self, cfg: &LedConfig) -> Result<()> {
        let mut cfg = cfg.clone();
        cfg.normalize();
        self.write_block(Block::LedConfig, 0, &cfg.0)
    }

    /// Read-modify-write: switch effect and set its brightness/speed/colour.
    pub fn set_effect(&self, effect: Effect, settings: EffectSettings) -> Result<()> {
        let mut cfg = self.led_config()?;
        if !cfg.is_valid() {
            // Same as the driver: retry once.
            sleep(Duration::from_millis(100));
            cfg = self.led_config()?;
        }
        cfg.set_effect(effect);
        cfg.set_settings(effect, settings);
        self.set_led_config(&cfg)
    }

    /// Turn the backlight off (brightness 0 of the active effect).
    pub fn lights_off(&self) -> Result<()> {
        let mut cfg = self.led_config()?;
        let e = cfg.effect().unwrap_or(Effect::FixedOn);
        let mut s = cfg.settings(e);
        s.brightness = 0;
        cfg.set_settings(e, s);
        self.set_led_config(&cfg)
    }

    pub fn rgb_table(&self) -> Result<RgbTable> {
        let v = self.read_block(Block::RgbTable, 0, led::RGB_TABLE_LEN)?;
        Ok(RgbTable(v.try_into().unwrap()))
    }

    pub fn set_rgb_table(&self, t: &RgbTable) -> Result<()> {
        let mut t = t.clone();
        t.normalize();
        self.write_block(Block::RgbTable, 0, &t.0)
    }

    /// Set palette colour `index` (0..7) of an effect. Select it with
    /// `EffectSettings { color: index, .. }`.
    pub fn set_effect_color(&self, effect: Effect, index: usize, c: Rgb) -> Result<()> {
        if index >= led::PALETTE_COLORS {
            return Err(Error::InvalidArgument("palette index must be < 7"));
        }
        let mut t = self.rgb_table()?;
        t.set_color(effect, index, c);
        self.set_rgb_table(&t)
    }

    pub fn per_key_colors(&self, group: u8) -> Result<PerKeyColors> {
        let v = self.read_block(Block::PerKeyColors, group, led::PER_KEY_LEN)?;
        Ok(PerKeyColors(v.try_into().unwrap()))
    }

    /// Upload per-key colours to `group` (X87 has 1 group, index 0).
    /// Activate with `set_effect(Effect::Custom, ..)`.
    pub fn set_per_key_colors(&self, group: u8, colors: &PerKeyColors) -> Result<()> {
        self.write_block(Block::PerKeyColors, group, &colors.0)
    }

    // ---- keys -------------------------------------------------------------

    pub fn key_matrix(&self, layer: u8) -> Result<KeyMatrix> {
        if layer >= LAYERS {
            return Err(Error::InvalidArgument("layer must be < 4"));
        }
        let v = self.read_block(Block::KeyMatrix, layer, MATRIX_LEN * 4)?;
        Ok(KeyMatrix::from_bytes(&v))
    }

    pub fn set_key_matrix(&self, layer: u8, m: &KeyMatrix) -> Result<()> {
        if m.0.len() != MATRIX_LEN { return Err(Error::InvalidArgument("key matrix must contain 128 entries")); }
        if layer >= LAYERS {
            return Err(Error::InvalidArgument("layer must be < 4"));
        }
        let mut b = m.to_bytes();
        b.resize(MATRIX_LEN * 4, 0);
        self.write_block(Block::KeyMatrix, layer, &b)
    }

    /// Remap one key. `key` is a name from [`layout::KEYS`].
    pub fn remap_key(&self, layer: u8, key: &str, action: KeyAction) -> Result<()> {
        action.try_encode()?;
        let idx = layout::key_index(key).ok_or(Error::InvalidArgument("unknown key name"))?;
        let mut m = self.key_matrix(layer)?;
        m.set(idx, action);
        self.set_key_matrix(layer, &m)
    }

    // ---- onboard macros (experimental) ------------------------------------

    /// Read USB bank or eight tagged 512-byte receiver pages (experimental).
    pub fn macro_bank(&self) -> Result<macros::MacroBank> {
        macros::MacroBank::decode(&self.read_block(Block::Macro, 0, macros::BANK_SIZE)?)
    }

    /// Write the whole shared bank, as the vendor app does
    /// (`G5KB_ApplySetting` → `W3632_WriteMacroData`): encode and write it in
    /// eight 512-byte units. Every packet is acknowledged and retried by the
    /// transport, so there is no extra read-back, and nothing is read first
    /// because the bank is overwritten. An empty `bank` clears it.
    ///
    /// Key bindings are not checked or changed. Slot numbers are shared by all
    /// layers: a key bound to slot `n` plays whatever is now in slot `n`, and a
    /// key bound past the end of the bank does nothing. Not atomic: a failed
    /// write may leave the bank partially written; saving again fixes it.
    ///
    /// `on_progress` (optional, pass `None` to skip) receives a
    /// non-decreasing percentage ending at 100 on success. Over 2.4G this
    /// takes about 10 s.
    pub fn save_macro_bank(
        &self,
        bank: &macros::MacroBank,
        on_progress: Option<&mut dyn FnMut(SaveProgress)>,
    ) -> Result<()> {
        let bytes = bank.encode()?;
        let mut r = progress::Reporter::new(on_progress);
        r.report(0.0);
        self.write_macro_pages(&bytes, &mut |f| r.report(f))?;
        r.report(1.0);
        Ok(())
    }

    /// Write an empty bank (same as `save_macro_bank` with no macros).
    pub fn clear_macro_bank(&self) -> Result<()> {
        self.save_macro_bank(&macros::MacroBank::default(), None)
    }

    // ---- device options ---------------------------------------------------

    fn update_config(&self, f: impl FnOnce(&mut LedConfig)) -> Result<()> {
        let mut cfg = self.led_config()?;
        if !cfg.is_valid() {
            sleep(Duration::from_millis(100));
            cfg = self.led_config()?;
        }
        f(&mut cfg);
        self.set_led_config(&cfg)
    }

    /// Debounce stage (1-based, app range 1..=5).
    pub fn debounce(&self) -> Result<u8> {
        Ok(self.led_config()?.debounce_stage())
    }

    pub fn set_debounce(&self, stage: u8) -> Result<()> {
        self.update_config(|c| c.set_debounce_stage(stage))
    }

    /// Tap sensitivity, `None` when Tap is off.
    pub fn tap(&self) -> Result<Option<u8>> {
        Ok(self.led_config()?.tap())
    }

    pub fn set_tap(&self, sensitivity: Option<u8>) -> Result<()> {
        self.update_config(|c| c.set_tap(sensitivity))
    }

    /// Wireless sleep timeout in seconds, `None` = never.
    pub fn sleep_timeout(&self) -> Result<Option<u32>> {
        Ok(self.led_config()?.sleep_secs())
    }

    pub fn set_sleep_timeout(&self, secs: Option<u32>) -> Result<()> {
        self.update_config(|c| c.set_sleep_secs(secs))
    }

    // ---- realtime / music -------------------------------------------------

    /// Show one realtime frame (see [`realtime`]). It overrides the stored
    /// effect while frames keep coming; call this in a loop (20-60 ms) and
    /// finish with [`Keyboard::stop_realtime`].
    pub fn send_frame(&self, frame: &Frame) -> Result<()> {
        self.send_frame_raw(realtime::SUB_FRAME, &frame.encode()?)
    }

    /// Like [`Keyboard::send_frame`] but drops colour groups that don't fit
    /// in 200 bytes instead of failing.
    pub fn send_frame_lossy(&self, frame: &Frame) -> Result<()> {
        self.send_frame_raw(realtime::SUB_FRAME, &frame.encode_lossy())
    }

    /// End realtime mode; the keyboard goes back to its stored effect.
    pub fn stop_realtime(&self) -> Result<()> {
        self.send_frame_raw(realtime::SUB_END, &[0, 0, 0])
    }

    /// Send an already encoded realtime payload (max 200 bytes).
    pub fn send_frame_raw(&self, sub: u8, data: &[u8]) -> Result<()> {
        if data.len() > realtime::MAX_FRAME_LEN {
            return Err(Error::InvalidArgument("realtime frame > 200 bytes"));
        }
        match &self.t {
            Transport::Wired { dev, report_id, .. } => {
                // G5KB_SetMusicRealData: AccessData(write, cmd 8, sub 0, delay 7 ms).
                sleep(Duration::from_millis(7));
                let pkt = build_packet(
                    *report_id,
                    Command::MusicRealtime as u8,
                    0,
                    1,
                    0,
                    data.len() as u16,
                    data,
                );
                Self::send_feature(dev, &pkt)
            }
            Transport::Wireless(w) => w.stream(wcmd::REALTIME, sub, data, Duration::from_millis(10)),
        }
    }

    // ---- misc -------------------------------------------------------------

    /// Factory reset (USB cmd 0x11, 2.4G cmd 0x06).
    pub fn factory_reset(&self) -> Result<()> {
        match self.connection() {
            Connection::Wired => self.write_raw(Command::Reset as u8, 0, &[0]),
            Connection::Wireless => self.write_raw(wcmd::RESET, 0, &[0]),
        }
    }
}
