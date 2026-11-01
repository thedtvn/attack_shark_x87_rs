//! Compile-only check that the README usage example builds.
#![allow(dead_code)]
use attack_shark_x87::{layout, Effect, EffectSettings, Frame, KeyAction, Keyboard, PerKeyColors, Rgb};

fn readme_example() -> Result<(), attack_shark_x87::Error> {
    let kb = Keyboard::open()?;
    kb.set_effect(Effect::Respire, EffectSettings { brightness: 4, speed: 2, color: 0 })?;
    kb.set_effect_color(Effect::Respire, 0, Rgb(255, 80, 0))?;
    let mut c = PerKeyColors::default();
    c.fill(Rgb(0, 0, 60));
    for k in ["W", "A", "S", "D"] {
        c.set(layout::key_index(k).unwrap(), Rgb(255, 0, 0));
    }
    kb.set_per_key_colors(0, &c)?;
    kb.set_effect(Effect::Custom, EffectSettings { brightness: 4, speed: 0, color: 0 })?;
    kb.remap_key(0, "CapsLock", KeyAction::Key { modifiers: 0, usage: 0x29, usage2: 0 })?;
    kb.set_debounce(2)?;
    kb.set_tap(None)?;
    kb.set_sleep_timeout(Some(300))?;
    let mut f = Frame::new();
    f.set_key("Space", Rgb(0, 255, 0))?;
    kb.send_frame(&f)?;
    kb.stop_realtime()?;
    Ok(())
}

#[test]
fn compiles() {}
