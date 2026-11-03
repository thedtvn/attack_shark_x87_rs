//! Node native module for the `attack_shark_x87` crate.
//!
//! This is the low-level layer; the typed TypeScript API (enums, classes) lives
//! in `ts/` and calls into it. Enums cross the boundary as their Rust variant
//! names (`Key::variant_name()`), plain objects carry key actions and macros.

use attack_shark_x87 as x87;
use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;
use std::sync::{Arc, Mutex};
use x87::macros::{Macro, MacroAction, MacroBank, MacroEvent, MacroRepeat};
use x87::{ControllerAction, Effect, EffectSettings, Key, KeyAction, KeyGroup, Rgb};

// ---- errors -----------------------------------------------------------------

/// Errors become JS `Error`s whose message starts with `[Code]`; the TS layer
/// turns them into typed error classes.
fn err(e: x87::Error) -> Error {
    let code = match &e {
        x87::Error::NotFound => "NotFound",
        x87::Error::Asleep => "Asleep",
        x87::Error::Timeout => "Timeout",
        x87::Error::InvalidArgument(_) => "InvalidArgument",
        x87::Error::Disconnected => "Disconnected",
        _ => "Device",
    };
    Error::new(Status::GenericFailure, format!("[{code}] {e}"))
}

fn invalid(msg: impl std::fmt::Display) -> Error {
    Error::new(Status::InvalidArg, format!("[InvalidArgument] {msg}"))
}

fn lookup<T>(name: &str, kind: &str, f: fn(&str) -> Option<T>) -> Result<T> {
    f(name).ok_or_else(|| invalid(format!("unknown {kind}: {name:?}")))
}
fn key(name: &str) -> Result<Key> {
    lookup(name, "key", Key::from_variant_name)
}
fn effect(name: &str) -> Result<Effect> {
    lookup(name, "effect", Effect::from_variant_name)
}
fn controller(name: &str) -> Result<ControllerAction> {
    lookup(name, "controller action", ControllerAction::from_variant_name)
}

fn byte(v: u32, what: &str) -> Result<u8> {
    u8::try_from(v).map_err(|_| invalid(format!("{what} must be 0..255, got {v}")))
}

// ---- static tables -----------------------------------------------------------

#[napi(object)]
pub struct NativeRect {
    pub x1: u16,
    pub y1: u16,
    pub x2: u16,
    pub y2: u16,
}

fn rect(r: x87::layout::Rect) -> NativeRect {
    NativeRect { x1: r.x1, y1: r.y1, x2: r.x2, y2: r.y2 }
}

#[napi(object)]
pub struct NativeKeyInfo {
    pub name: String,
    pub label: String,
    pub index: u8,
    pub rect: NativeRect,
    pub group: String,
    pub cluster: String,
    pub row: u32,
    pub left: Option<String>,
    pub right: Option<String>,
}

/// Every key in keyboard order.
#[napi]
pub fn key_table() -> Vec<NativeKeyInfo> {
    Key::ALL
        .into_iter()
        .map(|k| NativeKeyInfo {
            name: k.variant_name().into(),
            label: k.name().into(),
            index: k.index(),
            rect: rect(k.rect()),
            group: k.group().variant_name().into(),
            cluster: k.cluster().variant_name().into(),
            row: k.row() as u32,
            left: k.left().map(|n| n.variant_name().into()),
            right: k.right().map(|n| n.variant_name().into()),
        })
        .collect()
}

#[napi(object)]
pub struct NativeGroupInfo {
    pub name: String,
    pub is_area: bool,
    /// Clusters of key variant names.
    pub layout: Vec<Vec<String>>,
    pub rect: NativeRect,
}

#[napi]
pub fn group_table() -> Vec<NativeGroupInfo> {
    KeyGroup::ALL
        .into_iter()
        .map(|g| NativeGroupInfo {
            name: g.variant_name().into(),
            is_area: g.is_area(),
            layout: g.layout().iter().map(|c| c.iter().map(|k| k.variant_name().into()).collect()).collect(),
            rect: rect(g.rect()),
        })
        .collect()
}

/// Physical layout as cluster names: keyboard -> row -> cluster.
#[napi]
pub fn layout_table() -> Vec<Vec<String>> {
    x87::LAYOUT
        .iter()
        .map(|row| {
            row.iter()
                .map(|c| {
                    KeyGroup::CLUSTERS
                        .into_iter()
                        .find(|g| g.layout()[0] == *c)
                        .expect("LAYOUT cluster")
                        .variant_name()
                        .into()
                })
                .collect()
        })
        .collect()
}

#[napi(object)]
pub struct NativeEffectInfo {
    pub name: String,
    pub id: u8,
}

#[napi]
pub fn effect_table() -> Vec<NativeEffectInfo> {
    Effect::ALL.into_iter().map(|e| NativeEffectInfo { name: e.variant_name().into(), id: e as u8 }).collect()
}

#[napi(object)]
pub struct NativeControllerInfo {
    pub name: String,
    pub label: String,
    pub factory_key: String,
    pub sensitive: bool,
}

#[napi]
pub fn controller_table() -> Vec<NativeControllerInfo> {
    ControllerAction::ALL
        .into_iter()
        .map(|a| NativeControllerInfo {
            name: a.variant_name().into(),
            label: a.name().into(),
            factory_key: a.factory_key().variant_name().into(),
            sensitive: a.is_sensitive(),
        })
        .collect()
}

#[napi(object)]
pub struct NativeImageSize {
    pub width: u16,
    pub height: u16,
}

#[napi]
pub fn image_size() -> NativeImageSize {
    NativeImageSize { width: x87::layout::IMAGE_WIDTH, height: x87::layout::IMAGE_HEIGHT }
}

/// Key under a point of the vendor keyboard image.
#[napi]
pub fn key_at(x: f64, y: f64) -> Option<String> {
    if !(0.0..=u16::MAX as f64).contains(&x) || !(0.0..=u16::MAX as f64).contains(&y) {
        return None;
    }
    Key::at(x as u16, y as u16).map(|k| k.variant_name().into())
}

/// Key group (area) under a point of the vendor keyboard image.
#[napi]
pub fn group_at(x: f64, y: f64) -> Option<String> {
    if !(0.0..=u16::MAX as f64).contains(&x) || !(0.0..=u16::MAX as f64).contains(&y) {
        return None;
    }
    KeyGroup::at(x as u16, y as u16).map(|g| g.variant_name().into())
}

#[napi]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

// ---- key actions ---------------------------------------------------------------

/// Flat key action; which fields are set depends on `kind`
/// (none | key | consumer | mouse | fn | fn2 | macro | controller | raw).
#[napi(object)]
#[derive(Default)]
pub struct NativeKeyAction {
    pub kind: String,
    pub modifiers: Option<u32>,
    pub usage: Option<u32>,
    pub usage2: Option<u32>,
    pub button: Option<u32>,
    pub slot: Option<u32>,
    /// times | until_released | until_any_key
    pub repeat: Option<String>,
    pub count: Option<u32>,
    pub action: Option<String>,
    pub value: Option<u32>,
}

fn action_to_js(a: KeyAction) -> NativeKeyAction {
    let kind = |k: &str| NativeKeyAction { kind: k.into(), ..Default::default() };
    match a {
        KeyAction::None => kind("none"),
        KeyAction::Key { modifiers, usage, usage2 } => NativeKeyAction {
            modifiers: Some(modifiers.into()),
            usage: Some(usage.into()),
            usage2: Some(usage2.into()),
            ..kind("key")
        },
        KeyAction::Consumer(u) => NativeKeyAction { usage: Some(u.into()), ..kind("consumer") },
        KeyAction::Mouse(b) => NativeKeyAction { button: Some(b.into()), ..kind("mouse") },
        KeyAction::Fn => kind("fn"),
        KeyAction::Fn2 => kind("fn2"),
        KeyAction::Macro { slot, repeat } => {
            let (repeat, count) = match repeat {
                MacroRepeat::Times(n) => ("times", Some(n.into())),
                MacroRepeat::UntilReleased => ("until_released", None),
                MacroRepeat::UntilAnyKey => ("until_any_key", None),
            };
            NativeKeyAction { slot: Some(slot.into()), repeat: Some(repeat.into()), count, ..kind("macro") }
        }
        KeyAction::Controller(c) => NativeKeyAction { action: Some(c.variant_name().into()), ..kind("controller") },
        KeyAction::Raw(v) => NativeKeyAction { value: Some(v), ..kind("raw") },
    }
}

fn action_from_js(a: &NativeKeyAction) -> Result<KeyAction> {
    let need = |v: Option<u32>, f: &str| v.ok_or_else(|| invalid(format!("{} action is missing {f:?}", a.kind)));
    let b = |v: Option<u32>, f: &str| byte(need(v, f)?, f);
    let action = match a.kind.as_str() {
        "none" => KeyAction::None,
        "key" => KeyAction::Key {
            modifiers: b(a.modifiers.or(Some(0)), "modifiers")?,
            usage: b(a.usage, "usage")?,
            usage2: b(a.usage2.or(Some(0)), "usage2")?,
        },
        "consumer" => {
            let u = need(a.usage, "usage")?;
            KeyAction::Consumer(u16::try_from(u).map_err(|_| invalid("consumer usage must be 0..65535"))?)
        }
        "mouse" => KeyAction::Mouse(b(a.button, "button")?),
        "fn" => KeyAction::Fn,
        "fn2" => KeyAction::Fn2,
        "macro" => {
            let repeat = match a.repeat.as_deref().unwrap_or("times") {
                "times" => MacroRepeat::Times(b(a.count.or(Some(1)), "count")?),
                "until_released" => MacroRepeat::UntilReleased,
                "until_any_key" => MacroRepeat::UntilAnyKey,
                other => return Err(invalid(format!("unknown macro repeat {other:?}"))),
            };
            KeyAction::Macro { slot: b(a.slot, "slot")?, repeat }
        }
        "controller" => {
            let name = a.action.as_deref().ok_or_else(|| invalid("controller action is missing \"action\""))?;
            KeyAction::Controller(controller(name)?)
        }
        "raw" => KeyAction::Raw(need(a.value, "value")?),
        other => return Err(invalid(format!("unknown key action kind {other:?}"))),
    };
    action.try_encode().map_err(err)?;
    Ok(action)
}

/// Encode a key action to its 32-bit key-matrix word (validates it).
#[napi]
pub fn encode_action(action: NativeKeyAction) -> Result<u32> {
    Ok(action_from_js(&action)?.encode())
}

/// Decode a 32-bit key-matrix word.
#[napi]
pub fn decode_action(word: u32) -> NativeKeyAction {
    action_to_js(KeyAction::decode(word))
}

// ---- macros --------------------------------------------------------------------

/// `kind` is key | mouse_button | move_x | move_y | wheel.
#[napi(object)]
pub struct NativeMacroEvent {
    pub kind: String,
    pub value: i32,
    pub down: bool,
    pub delay_ms: u32,
}

#[napi(object)]
pub struct NativeMacro {
    pub name: String,
    pub events: Vec<NativeMacroEvent>,
}

fn event_from_js(e: &NativeMacroEvent) -> Result<MacroEvent> {
    let range = |lo: i32, hi: i32| {
        if (lo..=hi).contains(&e.value) {
            Ok(e.value)
        } else {
            Err(invalid(format!("{} value must be {lo}..{hi}, got {}", e.kind, e.value)))
        }
    };
    let action = match e.kind.as_str() {
        "key" => MacroAction::Key { usage: range(0, 255)? as u8, down: e.down },
        "mouse_button" => MacroAction::MouseButton { mask: range(0, 255)? as u8, down: e.down },
        "move_x" => MacroAction::MoveX(range(-128, 127)? as i8),
        "move_y" => MacroAction::MoveY(range(-128, 127)? as i8),
        "wheel" => MacroAction::Wheel(range(-128, 127)? as i8),
        other => return Err(invalid(format!("unknown macro event kind {other:?}"))),
    };
    let ev = MacroEvent { action, delay_ms: e.delay_ms };
    ev.encode().map_err(err)?;
    Ok(ev)
}

fn event_to_js(e: MacroEvent) -> NativeMacroEvent {
    let (kind, value, down) = match e.action {
        MacroAction::Key { usage, down } => ("key", usage as i32, down),
        MacroAction::MouseButton { mask, down } => ("mouse_button", mask as i32, down),
        MacroAction::MoveX(v) => ("move_x", v as i32, false),
        MacroAction::MoveY(v) => ("move_y", v as i32, false),
        MacroAction::Wheel(v) => ("wheel", v as i32, false),
    };
    NativeMacroEvent { kind: kind.into(), value, down, delay_ms: e.delay_ms }
}

fn bank_from_js(macros: &[NativeMacro]) -> Result<MacroBank> {
    let bank = MacroBank {
        macros: macros
            .iter()
            .map(|m| {
                Ok(Macro { name: m.name.clone(), events: m.events.iter().map(event_from_js).collect::<Result<_>>()? })
            })
            .collect::<Result<_>>()?,
    };
    bank.encode().map_err(err)?;
    Ok(bank)
}

fn bank_to_js(bank: MacroBank) -> Vec<NativeMacro> {
    bank.macros
        .into_iter()
        .map(|m| NativeMacro { name: m.name, events: m.events.into_iter().map(event_to_js).collect() })
        .collect()
}

/// Validate a macro bank offline; returns its encoded size out of 4096 bytes.
#[napi]
pub fn validate_macros(macros: Vec<NativeMacro>) -> Result<u32> {
    let bank = bank_from_js(&macros)?;
    let records: usize = bank.macros.iter().map(|m| m.encode().map(|b| b.len())).sum::<x87::Result<_>>().map_err(err)?;
    Ok((bank.macros.len() * 4 + records) as u32)
}

// ---- device --------------------------------------------------------------------

/// None once closed.
type Shared = Arc<Mutex<Option<x87::Keyboard>>>;

/// A blocking HID job run on the libuv thread pool.
pub struct Job<T> {
    kb: Shared,
    f: Option<Box<dyn FnOnce(&x87::Keyboard) -> Result<T> + Send>>,
}

impl<T: ToNapiValue + TypeName + Send + 'static> Task for Job<T> {
    type Output = T;
    type JsValue = T;

    fn compute(&mut self) -> Result<T> {
        let f = self.f.take().expect("job runs once");
        let kb = self.kb.lock().map_err(|_| err(x87::Error::Disconnected))?;
        f(kb.as_ref().ok_or_else(|| invalid("keyboard is closed"))?)
    }

    fn resolve(&mut self, _env: Env, output: T) -> Result<T> {
        Ok(output)
    }
}

#[napi(object)]
pub struct NativeLighting {
    pub effect: Option<String>,
    pub brightness: u8,
    pub speed: u8,
    pub color: u8,
}

#[napi(object)]
pub struct NativeKeyColor {
    pub key: String,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[napi(object)]
pub struct NativeKeyBinding {
    pub key: String,
    pub action: NativeKeyAction,
}

#[napi(object)]
pub struct NativeColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

fn colors_from_js(colors: &[NativeKeyColor]) -> Result<Vec<(Key, Rgb)>> {
    colors.iter().map(|c| Ok((key(&c.key)?, Rgb(c.r, c.g, c.b)))).collect()
}

/// An open keyboard. Every I/O method returns a Promise and runs off the JS
/// thread; calls are serialized.
#[napi]
pub struct NativeKeyboard {
    kb: Shared,
    connection: &'static str,
}

impl NativeKeyboard {
    fn job<T>(&self, f: impl FnOnce(&x87::Keyboard) -> x87::Result<T> + Send + 'static) -> AsyncTask<Job<T>>
    where
        T: ToNapiValue + TypeName + Send + 'static,
    {
        AsyncTask::new(Job { kb: self.kb.clone(), f: Some(Box::new(move |k| f(k).map_err(err))) })
    }
}

fn open_blocking(mode: &str) -> Result<NativeKeyboard> {
    let api = hidapi::HidApi::new().map_err(|e| err(e.into()))?;
    let kb = match mode {
        "auto" => x87::Keyboard::open_with(&api),
        "usb" => x87::Keyboard::open_wired(&api),
        "wireless" => x87::Keyboard::open_wireless(&api),
        _ => return Err(invalid("mode must be auto, usb or wireless")),
    }
    .map_err(err)?;
    let connection = match kb.connection() {
        x87::Connection::Wired => "usb",
        x87::Connection::Wireless => "wireless",
    };
    Ok(NativeKeyboard { kb: Arc::new(Mutex::new(Some(kb))), connection })
}

/// Opens the keyboard on the libuv thread pool.
pub struct OpenTask(String);

impl Task for OpenTask {
    type Output = NativeKeyboard;
    type JsValue = NativeKeyboard;

    fn compute(&mut self) -> Result<NativeKeyboard> {
        open_blocking(&self.0)
    }

    fn resolve(&mut self, _env: Env, output: NativeKeyboard) -> Result<NativeKeyboard> {
        Ok(output)
    }
}

#[napi]
impl NativeKeyboard {
    /// Open the keyboard; `mode` is "auto" (USB first), "usb" or "wireless".
    #[napi(factory)]
    pub fn open(mode: Option<String>) -> Result<Self> {
        open_blocking(mode.as_deref().unwrap_or("auto"))
    }

    /// Like `open`, but runs off the JS thread.
    #[napi(ts_return_type = "Promise<NativeKeyboard>")]
    pub fn open_async(mode: Option<String>) -> AsyncTask<OpenTask> {
        AsyncTask::new(OpenTask(mode.unwrap_or_else(|| "auto".into())))
    }

    /// Release the device. Waits for a running call; later calls reject.
    #[napi]
    pub fn close(&self) {
        if let Ok(mut kb) = self.kb.lock() {
            kb.take();
        }
    }

    #[napi(getter)]
    pub fn closed(&self) -> bool {
        self.kb.lock().map(|k| k.is_none()).unwrap_or(true)
    }

    #[napi(getter)]
    pub fn connection(&self) -> &'static str {
        self.connection
    }

    #[napi(ts_return_type = "Promise<number>")]
    pub fn firmware_version(&self) -> AsyncTask<Job<u32>> {
        self.job(|k| k.firmware_version().map(u32::from))
    }

    /// Null over the USB cable or when the 2.4G link is down.
    #[napi(ts_return_type = "Promise<NativeBattery | null>")]
    pub fn battery(&self) -> AsyncTask<Job<Option<NativeBattery>>> {
        self.job(|k| Ok(k.battery()?.map(|b| NativeBattery { percent: b.percent, charging: b.charging })))
    }

    #[napi(ts_return_type = "Promise<boolean>")]
    pub fn is_linked(&self) -> AsyncTask<Job<bool>> {
        self.job(|k| k.is_linked())
    }

    // lighting
    #[napi(ts_return_type = "Promise<NativeLighting>")]
    pub fn lighting(&self) -> AsyncTask<Job<NativeLighting>> {
        self.job(|k| {
            let cfg = k.led_config()?;
            let e = cfg.effect();
            let s = e.map(|e| cfg.settings(e)).unwrap_or(EffectSettings { brightness: 0, speed: 0, color: 0 });
            Ok(NativeLighting {
                effect: e.map(|e| e.variant_name().into()),
                brightness: s.brightness,
                speed: s.speed,
                color: s.color,
            })
        })
    }

    #[napi(ts_return_type = "Promise<void>")]
    pub fn set_effect(&self, name: String, brightness: u8, speed: u8, color: u8) -> Result<AsyncTask<Job<()>>> {
        let e = effect(&name)?;
        Ok(self.job(move |k| k.set_effect(e, EffectSettings { brightness, speed, color })))
    }

    #[napi(ts_return_type = "Promise<void>")]
    pub fn lights_off(&self) -> AsyncTask<Job<()>> {
        self.job(|k| k.lights_off())
    }

    #[napi(ts_return_type = "Promise<NativeColor[]>")]
    pub fn effect_palette(&self, name: String) -> Result<AsyncTask<Job<Vec<NativeColor>>>> {
        let e = effect(&name)?;
        Ok(self.job(move |k| {
            let t = k.rgb_table()?;
            Ok((0..x87::led::PALETTE_COLORS)
                .map(|i| t.color(e, i))
                .map(|c| NativeColor { r: c.0, g: c.1, b: c.2 })
                .collect())
        }))
    }

    #[napi(ts_return_type = "Promise<void>")]
    pub fn set_effect_color(&self, name: String, index: u32, color: NativeColor) -> Result<AsyncTask<Job<()>>> {
        let e = effect(&name)?;
        Ok(self.job(move |k| k.set_effect_color(e, index as usize, Rgb(color.r, color.g, color.b))))
    }

    #[napi(ts_return_type = "Promise<NativeKeyColor[]>")]
    pub fn per_key_colors(&self) -> AsyncTask<Job<Vec<NativeKeyColor>>> {
        self.job(|k| {
            let c = k.per_key_colors(0)?;
            Ok(Key::ALL
                .into_iter()
                .map(|key| {
                    let v = c.get_key(key);
                    NativeKeyColor { key: key.variant_name().into(), r: v.0, g: v.1, b: v.2 }
                })
                .collect())
        })
    }

    #[napi(ts_return_type = "Promise<void>")]
    pub fn set_per_key_colors(&self, colors: Vec<NativeKeyColor>) -> Result<AsyncTask<Job<()>>> {
        let mut c = x87::PerKeyColors::default();
        for (key, rgb) in colors_from_js(&colors)? {
            c.set_key(key, rgb);
        }
        Ok(self.job(move |k| k.set_per_key_colors(0, &c)))
    }

    // keys
    #[napi(ts_return_type = "Promise<NativeKeyBinding[]>")]
    pub fn layer(&self, layer: u8) -> AsyncTask<Job<Vec<NativeKeyBinding>>> {
        self.job(move |kb| {
            let m = kb.key_matrix(layer)?;
            Ok(Key::ALL
                .into_iter()
                .map(|k| NativeKeyBinding { key: k.variant_name().into(), action: action_to_js(m.get(k.index())) })
                .collect())
        })
    }

    #[napi(ts_return_type = "Promise<void>")]
    pub fn remap_key(&self, layer: u8, name: String, action: NativeKeyAction) -> Result<AsyncTask<Job<()>>> {
        let k = key(&name)?;
        let a = action_from_js(&action)?;
        Ok(self.job(move |kb| kb.remap_key(layer, k, a)))
    }

    // macros
    #[napi(ts_return_type = "Promise<NativeMacro[]>")]
    pub fn macros(&self) -> AsyncTask<Job<Vec<NativeMacro>>> {
        self.job(|k| k.macro_bank().map(bank_to_js))
    }

    /// Write the whole macro bank once; `onProgress` gets 0..100.
    #[napi(ts_return_type = "Promise<void>")]
    pub fn save_macros(
        &self,
        macros: Vec<NativeMacro>,
        on_progress: Option<ThreadsafeFunction<u32, (), u32, Status, false>>,
    ) -> Result<AsyncTask<Job<()>>> {
        let bank = bank_from_js(&macros)?;
        Ok(self.job(move |k| match on_progress {
            None => k.save_macro_bank(&bank, None),
            Some(cb) => {
                let mut call = |p: x87::SaveProgress| {
                    cb.call(p.percent.into(), ThreadsafeFunctionCallMode::NonBlocking);
                };
                k.save_macro_bank(&bank, Some(&mut call))
            }
        }))
    }

    #[napi(ts_return_type = "Promise<void>")]
    pub fn clear_macros(&self) -> AsyncTask<Job<()>> {
        self.job(|k| k.clear_macro_bank())
    }

    // options
    #[napi(ts_return_type = "Promise<number>")]
    pub fn debounce(&self) -> AsyncTask<Job<u8>> {
        self.job(|k| k.debounce())
    }
    #[napi(ts_return_type = "Promise<void>")]
    pub fn set_debounce(&self, stage: u8) -> AsyncTask<Job<()>> {
        self.job(move |k| k.set_debounce(stage))
    }
    #[napi(ts_return_type = "Promise<number | null>")]
    pub fn tap(&self) -> AsyncTask<Job<Option<u8>>> {
        self.job(|k| k.tap())
    }
    #[napi(ts_return_type = "Promise<void>")]
    pub fn set_tap(&self, sensitivity: Option<u8>) -> AsyncTask<Job<()>> {
        self.job(move |k| k.set_tap(sensitivity))
    }
    #[napi(ts_return_type = "Promise<number | null>")]
    pub fn sleep_timeout(&self) -> AsyncTask<Job<Option<u32>>> {
        self.job(|k| k.sleep_timeout())
    }
    #[napi(ts_return_type = "Promise<void>")]
    pub fn set_sleep_timeout(&self, secs: Option<u32>) -> AsyncTask<Job<()>> {
        self.job(move |k| k.set_sleep_timeout(secs))
    }

    // realtime
    #[napi(ts_return_type = "Promise<void>")]
    pub fn send_frame(&self, colors: Vec<NativeKeyColor>, lossy: Option<bool>) -> Result<AsyncTask<Job<()>>> {
        let mut f = x87::Frame::new();
        for (key, rgb) in colors_from_js(&colors)? {
            f.set_key(key, rgb);
        }
        let lossy = lossy.unwrap_or(false);
        Ok(self.job(move |k| if lossy { k.send_frame_lossy(&f) } else { k.send_frame(&f) }))
    }
    #[napi(ts_return_type = "Promise<void>")]
    pub fn stop_realtime(&self) -> AsyncTask<Job<()>> {
        self.job(|k| k.stop_realtime())
    }

    #[napi(ts_return_type = "Promise<void>")]
    pub fn factory_reset(&self) -> AsyncTask<Job<()>> {
        self.job(|k| k.factory_reset())
    }
}

// ---- connection events ---------------------------------------------------------

#[napi(object)]
pub struct NativeBattery {
    pub percent: u8,
    pub charging: bool,
}

/// `kind` is usb | wireless | battery.
#[napi(object)]
pub struct NativeConnEvent {
    pub kind: String,
    /// usb / wireless: link state.
    pub connected: Option<bool>,
    /// wireless: whether the receiver is plugged in.
    pub receiver: Option<bool>,
    pub battery: Option<NativeBattery>,
}

fn conn_event(e: x87::Event) -> NativeConnEvent {
    let base = |kind: &str| NativeConnEvent { kind: kind.into(), connected: None, receiver: None, battery: None };
    match e {
        x87::Event::Usb { connected } => NativeConnEvent { connected: Some(connected), ..base("usb") },
        x87::Event::Wireless { connected, receiver } => {
            NativeConnEvent { connected: Some(connected), receiver: Some(receiver), ..base("wireless") }
        }
        x87::Event::Battery(b) => {
            NativeConnEvent { battery: Some(NativeBattery { percent: b.percent, charging: b.charging }), ..base("battery") }
        }
    }
}

/// Background connection watcher; the callback runs on the JS thread.
#[napi]
pub struct NativeWatcher {
    inner: Mutex<Option<x87::Watcher>>,
}

#[napi]
impl NativeWatcher {
    #[napi(constructor)]
    pub fn new(callback: ThreadsafeFunction<NativeConnEvent, (), NativeConnEvent, Status, false, true>) -> Result<Self> {
        let w = x87::Watcher::start(move |e| {
            callback.call(conn_event(e), ThreadsafeFunctionCallMode::NonBlocking);
        })
        .map_err(err)?;
        Ok(Self { inner: Mutex::new(Some(w)) })
    }

    /// Stop and join the watcher thread (idempotent; takes up to ~0.5 s).
    #[napi]
    pub fn stop(&self) {
        if let Some(w) = self.inner.lock().ok().and_then(|mut g| g.take()) {
            w.stop();
        }
    }

    #[napi(getter)]
    pub fn running(&self) -> bool {
        self.inner.lock().map(|g| g.is_some()).unwrap_or(false)
    }
}

#[napi(object)]
pub struct NativeConnStatus {
    pub usb: bool,
    pub receiver: bool,
    pub wireless: bool,
}

pub struct StatusTask;

impl Task for StatusTask {
    type Output = NativeConnStatus;
    type JsValue = NativeConnStatus;

    fn compute(&mut self) -> Result<NativeConnStatus> {
        let s = x87::watch::status().map_err(err)?;
        Ok(NativeConnStatus { usb: s.usb, receiver: s.receiver, wireless: s.wireless })
    }

    fn resolve(&mut self, _env: Env, output: NativeConnStatus) -> Result<NativeConnStatus> {
        Ok(output)
    }
}

/// Current connection state, without opening the keyboard.
#[napi(ts_return_type = "Promise<NativeConnStatus>")]
pub fn connection_status() -> AsyncTask<StatusTask> {
    AsyncTask::new(StatusTask)
}