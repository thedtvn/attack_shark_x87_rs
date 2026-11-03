//! Python bindings (`ask87._native`) for the `attack_shark_x87` crate.
//!
//! The Python-facing API (enums, dataclass-like types and type stubs) lives in
//! `python/ask87/`. This module exposes the device and conversions; enums cross
//! the boundary as their Rust variant names (`Key::variant_name()`), so names
//! are never duplicated here.

use attack_shark_x87 as x87;
use pyo3::exceptions::{PyConnectionError, PyTimeoutError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::sync::Mutex;
use x87::macros::{Macro, MacroAction, MacroBank, MacroEvent, MacroRepeat};
use x87::{ControllerAction, Effect, EffectSettings, Key, KeyAction, KeyGroup, Rgb};

// ---- errors -----------------------------------------------------------------

pyo3::create_exception!(_native, X87Error, pyo3::exceptions::PyException, "Base error for ATTACK SHARK X87 operations.");
pyo3::create_exception!(_native, NotFoundError, X87Error, "No ATTACK SHARK X87 keyboard (USB or 2.4G receiver) was found.");
pyo3::create_exception!(_native, AsleepError, X87Error, "The 2.4G keyboard is asleep or off; press any key to wake it and retry.");
pyo3::create_exception!(_native, DeviceError, X87Error, "The keyboard returned an unexpected response or a HID error occurred.");

fn err(e: x87::Error) -> PyErr {
    match e {
        x87::Error::NotFound => NotFoundError::new_err(e.to_string()),
        x87::Error::Asleep => AsleepError::new_err(e.to_string()),
        x87::Error::Timeout => PyTimeoutError::new_err(e.to_string()),
        x87::Error::InvalidArgument(m) => PyValueError::new_err(m),
        x87::Error::Disconnected => PyConnectionError::new_err(e.to_string()),
        other => DeviceError::new_err(other.to_string()),
    }
}

type PyRes<T> = PyResult<T>;

fn enum_from<T>(name: &str, kind: &str, lookup: fn(&str) -> Option<T>) -> PyRes<T> {
    lookup(name).ok_or_else(|| PyValueError::new_err(format!("unknown {kind}: {name:?}")))
}

fn key(name: &str) -> PyRes<Key> {
    enum_from(name, "key", Key::from_variant_name)
}
fn effect(name: &str) -> PyRes<Effect> {
    enum_from(name, "effect", Effect::from_variant_name)
}
fn group(name: &str) -> PyRes<KeyGroup> {
    enum_from(name, "key group", KeyGroup::from_variant_name)
}
fn controller(name: &str) -> PyRes<ControllerAction> {
    enum_from(name, "controller action", ControllerAction::from_variant_name)
}

// ---- static tables (names and layout come from the Rust enums) ---------------

/// `[(variant, label, index, (x1, y1, x2, y2)), ...]` in keyboard order.
#[pyfunction]
fn key_table() -> Vec<(&'static str, &'static str, u8, (u16, u16, u16, u16))> {
    Key::ALL
        .into_iter()
        .map(|k| {
            let r = k.rect();
            (k.variant_name(), k.name(), k.index(), (r.x1, r.y1, r.x2, r.y2))
        })
        .collect()
}

/// `[(variant, is_area, [[key, ...], ...], (x1, y1, x2, y2)), ...]`.
#[pyfunction]
fn group_table() -> Vec<(&'static str, bool, Vec<Vec<&'static str>>, (u16, u16, u16, u16))> {
    KeyGroup::ALL
        .into_iter()
        .map(|g| {
            let r = g.rect();
            let clusters = g.layout().iter().map(|c| c.iter().map(|k| k.variant_name()).collect()).collect();
            (g.variant_name(), g.is_area(), clusters, (r.x1, r.y1, r.x2, r.y2))
        })
        .collect()
}

/// Physical layout as cluster variant names: keyboard -> row -> cluster.
#[pyfunction]
fn layout_table() -> Vec<Vec<&'static str>> {
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
                })
                .collect()
        })
        .collect()
}

/// `[(variant, hardware id), ...]`.
#[pyfunction]
fn effect_table() -> Vec<(&'static str, u8)> {
    Effect::ALL.into_iter().map(|e| (e.variant_name(), e as u8)).collect()
}

/// `[(variant, label, factory key, sensitive), ...]`.
#[pyfunction]
fn controller_table() -> Vec<(&'static str, &'static str, &'static str, bool)> {
    ControllerAction::ALL
        .into_iter()
        .map(|a| (a.variant_name(), a.name(), a.factory_key().variant_name(), a.is_sensitive()))
        .collect()
}

#[pyfunction]
fn image_size() -> (u16, u16) {
    (x87::layout::IMAGE_WIDTH, x87::layout::IMAGE_HEIGHT)
}

// ---- key actions: Rust <-> plain dicts (wrapped by the Python layer) ---------

fn action_to_py(py: Python<'_>, a: KeyAction) -> PyRes<Py<PyDict>> {
    let d = PyDict::new(py);
    match a {
        KeyAction::None => d.set_item("kind", "none")?,
        KeyAction::Key { modifiers, usage, usage2 } => {
            d.set_item("kind", "key")?;
            d.set_item("modifiers", modifiers)?;
            d.set_item("usage", usage)?;
            d.set_item("usage2", usage2)?;
        }
        KeyAction::Consumer(u) => {
            d.set_item("kind", "consumer")?;
            d.set_item("usage", u)?;
        }
        KeyAction::Mouse(b) => {
            d.set_item("kind", "mouse")?;
            d.set_item("button", b)?;
        }
        KeyAction::Fn => d.set_item("kind", "fn")?,
        KeyAction::Fn2 => d.set_item("kind", "fn2")?,
        KeyAction::Macro { slot, repeat } => {
            d.set_item("kind", "macro")?;
            d.set_item("slot", slot)?;
            match repeat {
                MacroRepeat::Times(n) => {
                    d.set_item("repeat", "times")?;
                    d.set_item("count", n)?;
                }
                MacroRepeat::UntilReleased => d.set_item("repeat", "until_released")?,
                MacroRepeat::UntilAnyKey => d.set_item("repeat", "until_any_key")?,
            }
        }
        KeyAction::Controller(c) => {
            d.set_item("kind", "controller")?;
            d.set_item("action", c.variant_name())?;
        }
        KeyAction::Raw(v) => {
            d.set_item("kind", "raw")?;
            d.set_item("value", v)?;
        }
    }
    Ok(d.unbind())
}

/// Read a required field of a key-action dict.
macro_rules! get {
    ($d:expr, $k:literal) => {
        $d.get_item($k)?
            .ok_or_else(|| PyValueError::new_err(concat!("key action is missing \"", $k, "\"")))?
            .extract()?
    };
}

fn action_from_py(d: &Bound<'_, PyDict>) -> PyRes<KeyAction> {
    let kind: String = get!(d, "kind");
    let a = match kind.as_str() {
        "none" => KeyAction::None,
        "key" => KeyAction::Key { modifiers: get!(d, "modifiers"), usage: get!(d, "usage"), usage2: get!(d, "usage2") },
        "consumer" => KeyAction::Consumer(get!(d, "usage")),
        "mouse" => KeyAction::Mouse(get!(d, "button")),
        "fn" => KeyAction::Fn,
        "fn2" => KeyAction::Fn2,
        "macro" => {
            let repeat: String = get!(d, "repeat");
            let repeat = match repeat.as_str() {
                "times" => MacroRepeat::Times(get!(d, "count")),
                "until_released" => MacroRepeat::UntilReleased,
                "until_any_key" => MacroRepeat::UntilAnyKey,
                other => return Err(PyValueError::new_err(format!("unknown macro repeat {other:?}"))),
            };
            KeyAction::Macro { slot: get!(d, "slot"), repeat }
        }
        "controller" => {
            let name: String = get!(d, "action");
            KeyAction::Controller(controller(&name)?)
        }
        "raw" => KeyAction::Raw(get!(d, "value")),
        other => return Err(PyValueError::new_err(format!("unknown key action kind {other:?}"))),
    };
    a.try_encode().map_err(err)?;
    Ok(a)
}
/// Encode a key-action dict to its 32-bit matrix word (validates it).
#[pyfunction]
fn encode_action(action: &Bound<'_, PyDict>) -> PyRes<u32> {
    Ok(action_from_py(action)?.encode())
}

/// Decode a 32-bit matrix word to a key-action dict.
#[pyfunction]
fn decode_action(py: Python<'_>, word: u32) -> PyRes<Py<PyDict>> {
    action_to_py(py, KeyAction::decode(word))
}

// ---- macros: Rust <-> plain tuples -------------------------------------------

/// (kind, value, down, delay_ms); kind is key | mouse_button | move_x | move_y | wheel.
type PyEvent = (String, i32, bool, u32);
type PyMacro = (String, Vec<PyEvent>);

fn event_from_py((kind, value, down, delay_ms): PyEvent) -> PyRes<MacroEvent> {
    let v8 = |lo: i32, hi: i32| {
        if (lo..=hi).contains(&value) {
            Ok(value)
        } else {
            Err(PyValueError::new_err(format!("{kind} value must be {lo}..{hi}, got {value}")))
        }
    };
    let action = match kind.as_str() {
        "key" => MacroAction::Key { usage: v8(0, 255)? as u8, down },
        "mouse_button" => MacroAction::MouseButton { mask: v8(0, 255)? as u8, down },
        "move_x" => MacroAction::MoveX(v8(-128, 127)? as i8),
        "move_y" => MacroAction::MoveY(v8(-128, 127)? as i8),
        "wheel" => MacroAction::Wheel(v8(-128, 127)? as i8),
        other => return Err(PyValueError::new_err(format!("unknown macro event kind {other:?}"))),
    };
    let e = MacroEvent { action, delay_ms };
    e.encode().map_err(err)?;
    Ok(e)
}

fn event_to_py(e: MacroEvent) -> PyEvent {
    let (kind, value, down) = match e.action {
        MacroAction::Key { usage, down } => ("key", usage as i32, down),
        MacroAction::MouseButton { mask, down } => ("mouse_button", mask as i32, down),
        MacroAction::MoveX(v) => ("move_x", v as i32, false),
        MacroAction::MoveY(v) => ("move_y", v as i32, false),
        MacroAction::Wheel(v) => ("wheel", v as i32, false),
    };
    (kind.to_string(), value, down, e.delay_ms)
}

fn bank_from_py(macros: Vec<PyMacro>) -> PyRes<MacroBank> {
    let bank = MacroBank {
        macros: macros
            .into_iter()
            .map(|(name, events)| {
                Ok(Macro { name, events: events.into_iter().map(event_from_py).collect::<PyRes<_>>()? })
            })
            .collect::<PyRes<_>>()?,
    };
    bank.encode().map_err(err)?;
    Ok(bank)
}

fn bank_to_py(bank: MacroBank) -> Vec<PyMacro> {
    bank.macros
        .into_iter()
        .map(|m| (m.name, m.events.into_iter().map(event_to_py).collect()))
        .collect()
}

/// Validate a macro bank without a keyboard; returns its encoded size in bytes
/// out of the 4096-byte bank (directory and records).
#[pyfunction]
fn validate_macros(macros: Vec<PyMacro>) -> PyRes<usize> {
    let bank = bank_from_py(macros)?;
    let used = bank.macros.len() * 4 + bank.macros.iter().map(|m| m.encode().map(|b| b.len())).sum::<x87::Result<usize>>().map_err(err)?;
    Ok(used)
}

// ---- device -----------------------------------------------------------------

/// An open keyboard. HID I/O releases the GIL; calls are serialized.
#[pyclass(module = "ask87._native", name = "NativeKeyboard")]
struct NativeKeyboard {
    /// None once closed.
    kb: Mutex<Option<x87::Keyboard>>,
    connection: &'static str,
}

impl NativeKeyboard {
    fn with<T: Send>(&self, py: Python<'_>, f: impl FnOnce(&x87::Keyboard) -> x87::Result<T> + Send) -> PyRes<T> {
        py.detach(|| {
            let kb = self.kb.lock().map_err(|_| x87::Error::Disconnected)?;
            f(kb.as_ref().ok_or(x87::Error::InvalidArgument("keyboard is closed"))?)
        })
        .map_err(err)
    }
}

fn rgb((r, g, b): (u8, u8, u8)) -> Rgb {
    Rgb(r, g, b)
}

#[pymethods]
impl NativeKeyboard {
    /// Open the keyboard; `mode` is "auto" (USB first), "usb" or "wireless".
    #[staticmethod]
    #[pyo3(signature = (mode = "auto"))]
    fn open(py: Python<'_>, mode: &str) -> PyRes<Self> {
        let mode = mode.to_string();
        let kb = py
            .detach(move || -> x87::Result<x87::Keyboard> {
                let api = hidapi::HidApi::new()?;
                match mode.as_str() {
                    "auto" => x87::Keyboard::open_with(&api),
                    "usb" => x87::Keyboard::open_wired(&api),
                    "wireless" => x87::Keyboard::open_wireless(&api),
                    _ => Err(x87::Error::InvalidArgument("mode must be auto, usb or wireless")),
                }
            })
            .map_err(err)?;
        let connection = match kb.connection() {
            x87::Connection::Wired => "usb",
            x87::Connection::Wireless => "wireless",
        };
        Ok(Self { kb: Mutex::new(Some(kb)), connection })
    }

    /// Release the device. Waits for a running call; later calls raise ValueError.
    fn close(&self, py: Python<'_>) {
        py.detach(|| {
            if let Ok(mut kb) = self.kb.lock() {
                kb.take();
            }
        })
    }

    #[getter]
    fn closed(&self) -> bool {
        self.kb.lock().map(|k| k.is_none()).unwrap_or(true)
    }

    #[getter]
    fn connection(&self) -> &'static str {
        self.connection
    }

    fn firmware_version(&self, py: Python<'_>) -> PyRes<u16> {
        self.with(py, |k| k.firmware_version())
    }

    /// (percent, charging), or None over the USB cable / when the 2.4G link is down.
    fn battery(&self, py: Python<'_>) -> PyRes<Option<(u8, bool)>> {
        Ok(self.with(py, |k| k.battery())?.map(|b| (b.percent, b.charging)))
    }

    fn is_linked(&self, py: Python<'_>) -> PyRes<bool> {
        self.with(py, |k| k.is_linked())
    }

    // lighting
    fn set_effect(&self, py: Python<'_>, name: &str, brightness: u8, speed: u8, color: u8) -> PyRes<()> {
        let e = effect(name)?;
        self.with(py, move |k| k.set_effect(e, EffectSettings { brightness, speed, color }))
    }

    /// (effect variant or None, brightness, speed, color, debounce, tap, sleep_secs)
    fn lighting(&self, py: Python<'_>) -> PyRes<(Option<&'static str>, u8, u8, u8)> {
        let cfg = self.with(py, |k| k.led_config())?;
        let e = cfg.effect();
        let s = e.map(|e| cfg.settings(e)).unwrap_or(EffectSettings { brightness: 0, speed: 0, color: 0 });
        Ok((e.map(|e| e.variant_name()), s.brightness, s.speed, s.color))
    }

    fn lights_off(&self, py: Python<'_>) -> PyRes<()> {
        self.with(py, |k| k.lights_off())
    }

    fn set_effect_color(&self, py: Python<'_>, name: &str, index: usize, color: (u8, u8, u8)) -> PyRes<()> {
        let e = effect(name)?;
        self.with(py, move |k| k.set_effect_color(e, index, rgb(color)))
    }

    fn effect_palette(&self, py: Python<'_>, name: &str) -> PyRes<Vec<(u8, u8, u8)>> {
        let e = effect(name)?;
        let t = self.with(py, |k| k.rgb_table())?;
        Ok((0..x87::led::PALETTE_COLORS).map(|i| t.color(e, i)).map(|c| (c.0, c.1, c.2)).collect())
    }

    /// Per-key colours as {key variant: (r, g, b)} for every key.
    fn per_key_colors(&self, py: Python<'_>) -> PyRes<Vec<(&'static str, (u8, u8, u8))>> {
        let c = self.with(py, |k| k.per_key_colors(0))?;
        Ok(Key::ALL.into_iter().map(|k| (k.variant_name(), { let v = c.get_key(k); (v.0, v.1, v.2) })).collect())
    }

    fn set_per_key_colors(&self, py: Python<'_>, colors: Vec<(String, (u8, u8, u8))>) -> PyRes<()> {
        let mut c = x87::PerKeyColors::default();
        for (name, color) in colors {
            c.set_key(key(&name)?, rgb(color));
        }
        self.with(py, move |k| k.set_per_key_colors(0, &c))
    }

    // keys
    fn key_action(&self, py: Python<'_>, layer: u8, name: &str) -> PyRes<Py<PyDict>> {
        let k = key(name)?;
        let m = self.with(py, move |kb| kb.key_matrix(layer))?;
        action_to_py(py, m.get(k.index()))
    }

    /// Every key's action on a layer: [(key variant, action dict), ...].
    fn layer(&self, py: Python<'_>, layer: u8) -> PyRes<Vec<(&'static str, Py<PyDict>)>> {
        let m = self.with(py, move |kb| kb.key_matrix(layer))?;
        Key::ALL.into_iter().map(|k| Ok((k.variant_name(), action_to_py(py, m.get(k.index()))?))).collect()
    }

    fn remap_key(&self, py: Python<'_>, layer: u8, name: &str, action: &Bound<'_, PyDict>) -> PyRes<()> {
        let k = key(name)?;
        let a = action_from_py(action)?;
        self.with(py, move |kb| kb.remap_key(layer, k, a))
    }

    // macros
    fn macros(&self, py: Python<'_>) -> PyRes<Vec<PyMacro>> {
        Ok(bank_to_py(self.with(py, |k| k.macro_bank())?))
    }

    #[pyo3(signature = (macros, on_progress = None))]
    fn save_macros(&self, py: Python<'_>, macros: Vec<PyMacro>, on_progress: Option<Py<PyAny>>) -> PyRes<()> {
        let bank = bank_from_py(macros)?;
        self.with(py, move |k| match on_progress {
            None => k.save_macro_bank(&bank, None),
            Some(cb) => {
                // Progress is reported from this thread; take the GIL briefly per update.
                let mut call = |p: x87::SaveProgress| {
                    Python::attach(|py| {
                        if let Err(e) = cb.call1(py, (p.percent,)) {
                            e.print(py);
                        }
                    })
                };
                k.save_macro_bank(&bank, Some(&mut call))
            }
        })
    }

    fn clear_macros(&self, py: Python<'_>) -> PyRes<()> {
        self.with(py, |k| k.clear_macro_bank())
    }

    // options
    fn debounce(&self, py: Python<'_>) -> PyRes<u8> {
        self.with(py, |k| k.debounce())
    }
    fn set_debounce(&self, py: Python<'_>, stage: u8) -> PyRes<()> {
        self.with(py, move |k| k.set_debounce(stage))
    }
    fn tap(&self, py: Python<'_>) -> PyRes<Option<u8>> {
        self.with(py, |k| k.tap())
    }
    #[pyo3(signature = (sensitivity))]
    fn set_tap(&self, py: Python<'_>, sensitivity: Option<u8>) -> PyRes<()> {
        self.with(py, move |k| k.set_tap(sensitivity))
    }
    fn sleep_timeout(&self, py: Python<'_>) -> PyRes<Option<u32>> {
        self.with(py, |k| k.sleep_timeout())
    }
    #[pyo3(signature = (secs))]
    fn set_sleep_timeout(&self, py: Python<'_>, secs: Option<u32>) -> PyRes<()> {
        self.with(py, move |k| k.set_sleep_timeout(secs))
    }

    // realtime
    #[pyo3(signature = (colors, lossy = false))]
    fn send_frame(&self, py: Python<'_>, colors: Vec<(String, (u8, u8, u8))>, lossy: bool) -> PyRes<()> {
        let mut f = x87::Frame::new();
        for (name, color) in colors {
            f.set_key(key(&name)?, rgb(color));
        }
        self.with(py, move |k| if lossy { k.send_frame_lossy(&f) } else { k.send_frame(&f) })
    }
    fn stop_realtime(&self, py: Python<'_>) -> PyRes<()> {
        self.with(py, |k| k.stop_realtime())
    }

    fn factory_reset(&self, py: Python<'_>) -> PyRes<()> {
        self.with(py, |k| k.factory_reset())
    }
}

// ---- connection events ---------------------------------------------------------

/// Event as a tuple: ("usb", connected, None, None) |
/// ("wireless", connected, receiver, None) | ("battery", percent, charging, None).
type PyEventTuple = (&'static str, i64, Option<bool>, Option<bool>);

fn conn_event_to_py(e: x87::Event) -> PyEventTuple {
    match e {
        x87::Event::Usb { connected } => ("usb", connected as i64, None, None),
        x87::Event::Wireless { connected, receiver } => ("wireless", connected as i64, Some(receiver), None),
        x87::Event::Battery(b) => ("battery", b.percent as i64, Some(b.charging), None),
    }
}

/// Background watcher; `callback(event_tuple)` runs on the watcher thread.
#[pyclass(module = "ask87._native", name = "NativeWatcher")]
struct NativeWatcher {
    inner: Mutex<Option<x87::Watcher>>,
}

#[pymethods]
impl NativeWatcher {
    #[new]
    fn new(py: Python<'_>, callback: Py<PyAny>) -> PyRes<Self> {
        let w = py
            .detach(move || {
                x87::Watcher::start(move |e| {
                    Python::attach(|py| {
                        if let Err(err) = callback.call1(py, (conn_event_to_py(e),)) {
                            err.print(py);
                        }
                    })
                })
            })
            .map_err(err)?;
        Ok(Self { inner: Mutex::new(Some(w)) })
    }

    /// Stop and join the watcher thread (idempotent).
    fn stop(&self, py: Python<'_>) {
        let w = self.inner.lock().ok().and_then(|mut g| g.take());
        if let Some(w) = w {
            py.detach(move || w.stop());
        }
    }

    #[getter]
    fn running(&self) -> bool {
        self.inner.lock().map(|g| g.is_some()).unwrap_or(false)
    }
}

/// (usb, receiver, wireless) right now.
#[pyfunction]
fn connection_status(py: Python<'_>) -> PyRes<(bool, bool, bool)> {
    let s = py.detach(x87::watch::status).map_err(err)?;
    Ok((s.usb, s.receiver, s.wireless))
}
/// Key under a point (vendor image pixels), as a variant name.
#[pyfunction]
fn key_at(x: i64, y: i64) -> Option<&'static str> {
    let (x, y) = (u16::try_from(x).ok()?, u16::try_from(y).ok()?);
    Key::at(x, y).map(|k| k.variant_name())
}

/// Key group (area) under a point, as a variant name.
#[pyfunction]
fn group_at(x: i64, y: i64) -> Option<&'static str> {
    let (x, y) = (u16::try_from(x).ok()?, u16::try_from(y).ok()?);
    KeyGroup::at(x, y).map(|g| g.variant_name())
}

/// Neighbour key in the same row: direction is "left" or "right".
#[pyfunction]
fn key_neighbour(name: &str, direction: &str) -> PyRes<Option<&'static str>> {
    let k = key(name)?;
    let n = match direction {
        "left" => k.left(),
        "right" => k.right(),
        _ => return Err(PyValueError::new_err("direction must be 'left' or 'right'")),
    };
    Ok(n.map(|k| k.variant_name()))
}

#[pyfunction]
fn group_of(name: &str) -> PyRes<(&'static str, &'static str, usize)> {
    let k = key(name)?;
    Ok((k.group().variant_name(), k.cluster().variant_name(), k.row()))
}

#[pyfunction]
fn group_rect(name: &str) -> PyRes<(u16, u16, u16, u16)> {
    let r = group(name)?.rect();
    Ok((r.x1, r.y1, r.x2, r.y2))
}

#[pyfunction]
fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = m.py();
    m.add("X87Error", py.get_type::<X87Error>())?;
    m.add("NotFoundError", py.get_type::<NotFoundError>())?;
    m.add("DeviceError", py.get_type::<DeviceError>())?;
    m.add("AsleepError", py.get_type::<AsleepError>())?;
    m.add_class::<NativeKeyboard>()?;
    m.add_class::<NativeWatcher>()?;
    for f in [
        wrap_pyfunction!(key_table, m)?,
        wrap_pyfunction!(group_table, m)?,
        wrap_pyfunction!(layout_table, m)?,
        wrap_pyfunction!(effect_table, m)?,
        wrap_pyfunction!(controller_table, m)?,
        wrap_pyfunction!(image_size, m)?,
        wrap_pyfunction!(encode_action, m)?,
        wrap_pyfunction!(decode_action, m)?,
        wrap_pyfunction!(validate_macros, m)?,
        wrap_pyfunction!(key_at, m)?,
        wrap_pyfunction!(group_at, m)?,
        wrap_pyfunction!(key_neighbour, m)?,
        wrap_pyfunction!(group_of, m)?,
        wrap_pyfunction!(group_rect, m)?,
        wrap_pyfunction!(version, m)?,
        wrap_pyfunction!(connection_status, m)?,
    ] {
        m.add_function(f)?;
    }
    Ok(())
}
