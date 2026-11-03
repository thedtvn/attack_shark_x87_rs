//! Compile-only check that the README usage example builds.
#![allow(dead_code)]
use attack_shark_x87::{Effect, EffectSettings, Frame, Key, KeyAction, Keyboard, PerKeyColors, Rgb};

fn readme_example() -> Result<(), attack_shark_x87::Error> {
    let kb = Keyboard::open()?;
    kb.set_effect(Effect::Respire, EffectSettings { brightness: 4, speed: 2, color: 0 })?;
    kb.set_effect_color(Effect::Respire, 0, Rgb(255, 80, 0))?;
    let mut c = PerKeyColors::default();
    c.fill(Rgb(0, 0, 60));
    for k in [Key::W, Key::A, Key::S, Key::D] {
        c.set_key(k, Rgb(255, 0, 0));
    }
    kb.set_per_key_colors(0, &c)?;
    kb.set_effect(Effect::Custom, EffectSettings { brightness: 4, speed: 0, color: 0 })?;
    kb.remap_key(0, Key::CapsLock, KeyAction::Key { modifiers: 0, usage: 0x29, usage2: 0 })?;
    kb.set_debounce(2)?;
    kb.set_tap(None)?;
    kb.set_sleep_timeout(Some(300))?;
    let mut f = Frame::new();
    f.set_key(Key::Space, Rgb(0, 255, 0));
    kb.send_frame(&f)?;
    kb.stop_realtime()?;
    Ok(())
}

#[test]
fn compiles() {}

#[test]
fn keys_and_layout_example() {
    use attack_shark_x87::{KeyGroup, LAYOUT};
    assert_eq!(Key::CapsLock.index(), 3);
    assert_eq!(Key::Digit1.name(), "1");
    assert_eq!("space".parse::<Key>(), Ok(Key::Space));
    assert_eq!(LAYOUT[0][0], &[Key::Esc]);
    assert_eq!(LAYOUT[0][1], &[Key::F1, Key::F2, Key::F3, Key::F4]);
    assert_eq!(KeyGroup::FKey.layout()[0], &[Key::Esc]);
    assert_eq!(KeyGroup::Main.layout()[0][0], Key::Grave);
    assert_eq!(KeyGroup::Navigation.keys().count(), 9);
    assert_eq!(Key::Left.group(), KeyGroup::Arrows);
    assert_eq!(Key::Space.row(), 5);
}
