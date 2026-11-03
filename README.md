# attack_shark_x87_rs

Rust library for configuring the **ATTACK SHARK X87** keyboard, over the USB
cable or the 2.4G receiver, without the vendor software.

The protocol was reverse engineered from the vendor tool `OemDrv.exe` with
Ghidra. It is documented in [PROTOCOL.md](PROTOCOL.md).
Macro event encoding, buffer layout and repeat-mode research are in
[MACROS.md](MACROS.md). An **experimental** macro codec and explicit replacement
API are available; macro playback has not been validated on hardware.

Browser version (WebHID + React): [attack_shark_x87_web](https://github.com/thedtvn/attack_shark_x87_web)

## Features

| Feature | USB | 2.4G |
|---|---|---|
| Lighting effects (19), brightness, speed, palette colour | ✓ | ✓ verified |
| Per-effect 7-colour palettes | ✓ | ✓ |
| Per-key RGB (custom mode) | ✓ | ✓ |
| Key remap, 4 layers (keys, modifiers, media, mouse, Fn/Fn2, disable) | ✓ | ✓ verified (read) |
| Tap on/off + sensitivity | ✓ | ✓ verified |
| Sleep timeout | ✓ | ✓ verified |
| Debounce (including beyond the app's 5 stages) | ✓ | ✓ verified |
| Realtime frames ("music mode" transport) | untested | ✓ verified |
| Factory reset | ✓ | ✓ |
| Experimental macro bank read/save (with progress) + repeat bindings | ✓ | ✓ paged read/write |

"Verified" means tested against a real keyboard. The USB path follows the same
decompiled code but has not been tested on hardware yet.

## Install

```toml
[dependencies]
attack_shark_x87 = { git = "https://github.com/thedtvn/attack_shark_x87_rs" }
```

Uses [`hidapi`](https://crates.io/crates/hidapi). On Linux you need a udev rule
for `258a:010c` and `3554:fa09`, or root access.

## Usage

```rust
use attack_shark_x87::{Effect, EffectSettings, Keyboard, KeyAction, Frame, Rgb, PerKeyColors, layout};

fn main() -> Result<(), attack_shark_x87::Error> {
    // USB cable first, falls back to the 2.4G receiver.
    let kb = Keyboard::open()?;

    // Lighting
    kb.set_effect(Effect::Respire, EffectSettings { brightness: 4, speed: 2, color: 0 })?;
    kb.set_effect_color(Effect::Respire, 0, Rgb(255, 80, 0))?;

    // Per-key colours
    let mut c = PerKeyColors::default();
    c.fill(Rgb(0, 0, 60));
    for k in ["W", "A", "S", "D"] {
        c.set(layout::key_index(k).unwrap(), Rgb(255, 0, 0));
    }
    kb.set_per_key_colors(0, &c)?;
    kb.set_effect(Effect::Custom, EffectSettings { brightness: 4, speed: 0, color: 0 })?;

    // Remap CapsLock to Esc on layer 1
    kb.remap_key(0, "CapsLock", KeyAction::Key { modifiers: 0, usage: 0x29, usage2: 0 })?;

    // Options
    kb.set_debounce(2)?;
    kb.set_tap(None)?;                  // Tap off
    kb.set_sleep_timeout(Some(300))?;   // 5 minutes

    // Realtime frame (what the vendor "music" mode streams)
    let mut f = Frame::new();
    f.set_key("Space", Rgb(0, 255, 0))?;
    kb.send_frame(&f)?;
    kb.stop_realtime()?;
    Ok(())
}
```

## Examples

```sh
cargo run --example info                   # connection, firmware, config, layer 1
cargo run --example effect -- respire 4 2 0
cargo run --example per_key                # WASD red
cargo run --example realtime               # 8 s sweep using realtime frames
cargo run --example roundtrip              # safe write test (writes config back unchanged)
cargo run --example list                   # list all HID interfaces
cargo run --example dumpcfg -- save a.bin  # snapshot / diff the 128-byte config
cargo run --example setbyte -- 0x03 2      # write one config byte
```

Close the vendor app before using the library; it holds the same HID interface.

## Protocol summary

- **USB**: `258A:010C`, collection `FF00:0001`, 520-byte feature reports
  (report id 6) `[id, cmd, sub, 0, chunks, index, lenLo, lenHi, data[512]]`.
  Reads use `cmd | 0x80`.
- **2.4G**: `3554:FA09`, collection `FF02:0002`, 20-byte reports (id 0x13)
  `[0x13, cmd, total, index, sub<<4|len, data[14], checksum]`, acked per packet.
- Both carry the same data blocks: a 128-byte config (effects, brightness,
  speed, Tap, Sleep, Debounce), RGB palette table, per-key planes and a
  128 × u32 key matrix per layer.

See [PROTOCOL.md](PROTOCOL.md) for every offset and command.

## Disclaimer

Not affiliated with ATTACK SHARK. Writing wrong data to a keyboard can leave
it misconfigured; a factory reset (`kb.factory_reset()`) restores defaults.
Use at your own risk.

## License

[MIT](LICENSE) © 2026 The DT

## Controller bindings

```rust,no_run
use attack_shark_x87::{Keyboard, KeyAction, ControllerAction};
# fn example() -> Result<(), attack_shark_x87::Error> {
let kb = Keyboard::open()?;
// Explicit assignments only. These examples are not executed automatically.
kb.remap_key(0, "App", KeyAction::Fn2)?;
kb.remap_key(1, "4", KeyAction::Controller(ControllerAction::Receiver24G))?;
kb.remap_key(1, "CapsLock", KeyAction::Controller(ControllerAction::LightBarMode))?;
# Ok(()) }
```

`ControllerAction::ALL` includes Bluetooth 1/2/3, receiver, battery display,
Windows-key lock, reset, main backlight and light-bar controls. `name()`,
`factory_key()` and `is_sensitive()` provide caller-facing metadata. Host
media controls remain `KeyAction::Consumer`; application/task switching and
input-method defaults are constants in `controller`. `factory_fn1_action()`
returns the vendor FN1 mapping. FN2/Tap are not assumed to share FN1 defaults.

FN encodes as `0D 00 00 00`; FN2 as `0D 01 00 00`. Unknown words stay `Raw`.
`KeyAction::try_encode()` validates macro slots/counts without panicking;
`remap_key` uses it before device I/O. Connection controls may disconnect the
keyboard, and reset may erase settings. Remapped hold behavior is experimental.
`cargo run --example controller` prints bytes offline without opening hardware.

## Experimental onboard macros

Macro playback is encoded from reverse-engineered vendor code and covered by
byte fixtures, **not hardware playback verification**. Until-released and
until-any-key mode semantics remain experimental.

```rust,no_run
use attack_shark_x87::{Keyboard, KeyAction};
use attack_shark_x87::macros::{Macro, MacroAction, MacroEvent, MacroBank, MacroRepeat};
# fn example() -> Result<(), attack_shark_x87::Error> {
let kb = Keyboard::open()?;
let bank = MacroBank { macros: vec![Macro {
    name: "Type A".into(),
    events: vec![
        MacroEvent { action: MacroAction::Key { usage: 4, down: true }, delay_ms: 0 },
        MacroEvent { action: MacroAction::Key { usage: 4, down: false }, delay_ms: 50 },
    ],
}] };
// Replaces the whole shared bank used by all layers (about 10 s over 2.4G).
kb.save_macro_bank(&bank, Some(&mut |p| println!("saving {}%", p.percent)))?;
// Or without progress: kb.save_macro_bank(&bank, None)?;
// Bind a key deliberately; saving the bank never changes key bindings.
kb.remap_key(0, "F12", KeyAction::Macro { slot: 0, repeat: MacroRepeat::Times(1) })?;
# Ok(()) }
```

- Names: up to 30 UTF-16 code units. Delays: 0..1,048,575 ms.
- Bank: 4096 bytes; 32 macros is a conservative client limit, not a measured
  firmware limit. Events must fit alongside names and the directory.
- `save_macro_bank` works like the vendor app (`G5KB_ApplySetting` →
  `W3632_WriteMacroData`): it writes the bank once in eight 512-byte units,
  reporting a non-decreasing percentage. Nothing is read first and nothing is
  read back: every packet is acknowledged and retried by the transport.
  `replace_macro_bank` is the same without progress and refuses an empty bank;
  `clear_macro_bank` writes an empty one.
- Slot numbers are shared across all four layers and key bindings are not
  checked. A key bound to slot `n` plays whatever is now in slot `n`; a key
  bound past the end of the bank does nothing.
- `macro_bank()` reads the bank (USB, or eight tagged 512-byte pages over
  2.4G via command `43`, vendor wrapper `00487d20`). A 2.4G write + read +
  restore was verified on hardware once; playback is not yet validated.
- `cargo run --example wireless_macro_check` reads the bank read-only and
  prints timings (2.4G: ~1.9 s for the bank, ~1.9 s for four layers).
- `cargo run --example save_progress_check -- --confirm` writes a test bank,
  reads it back and restores the original.
- The receiver cannot pipeline requests; reads are sequential. Dropped
  packets are kept across quick re-requests rather than restarting blindly.
- Saving is not atomic: interruption can leave a partial bank; save again.
- `cargo run --example macro` only builds/prints bytes offline. The
  `--upload` flag saves the bank with a progress bar; it never assigns a key.
  `--upload` flag explicitly enables the replacement; it never assigns a key.
