# attack_shark_x87_rs

Rust library for configuring the **ATTACK SHARK X87** keyboard, over the USB
cable or the 2.4G receiver, without the vendor software.

The protocol was reverse engineered from the vendor tool `OemDrv.exe` with
Ghidra. It is documented in [PROTOCOL.md](PROTOCOL.md).

Browser version (WebHID + React): [attack_shark_x87_web](https://github.com/thedtvn/attack_shark_x87_web)

## Features

| Feature | USB | 2.4G |
|---|---|---|
| Lighting effects (19), brightness, speed, palette colour | ✓ | ✓ verified |
| Per-effect 7-colour palettes | ✓ | ✓ |
| Per-key RGB (custom mode) | ✓ | ✓ |
| Key remap, 4 layers (keys, modifiers, media, mouse, Fn, disable) | ✓ | ✓ verified (read) |
| Tap on/off + sensitivity | ✓ | ✓ verified |
| Sleep timeout | ✓ | ✓ verified |
| Debounce (including beyond the app's 5 stages) | ✓ | ✓ verified |
| Realtime frames ("music mode" transport) | untested | ✓ verified |
| Factory reset | ✓ | ✓ |

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
