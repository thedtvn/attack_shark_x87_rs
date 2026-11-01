//! Paint WASD red, everything else dim blue, and switch to the per-key mode.
use attack_shark_x87::{layout, Effect, EffectSettings, Keyboard, PerKeyColors, Rgb};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let kb = Keyboard::open()?;
    let mut c = PerKeyColors::default();
    c.fill(Rgb(0, 0, 60));
    for k in ["W", "A", "S", "D"] {
        c.set(layout::key_index(k).unwrap(), Rgb(255, 0, 0));
    }
    kb.set_per_key_colors(0, &c)?;
    kb.set_effect(Effect::Custom, EffectSettings { brightness: 4, speed: 0, color: 0 })?;
    Ok(())
}
