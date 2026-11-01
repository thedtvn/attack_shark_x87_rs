//! cargo run --example effect -- respire 4 2 0
use attack_shark_x87::{Effect, EffectSettings, Keyboard};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let name = args.first().map(String::as_str).unwrap_or("fixed");
    let effect = match name {
        "fixed" => Effect::FixedOn,
        "respire" => Effect::Respire,
        "rainbow" => Effect::Rainbow,
        "wheel" => Effect::RainbowWheel,
        "ripple" => Effect::RipplesShining,
        "stars" => Effect::StarsTwinkle,
        "snake" => Effect::RetroSnake,
        "neon" => Effect::NeonStream,
        "reaction" => Effect::Reaction,
        "wave" => Effect::SineWave,
        "custom" => Effect::Custom,
        _ => return Err(format!("unknown effect {name}").into()),
    };
    let n = |i: usize, d: u8| args.get(i).and_then(|s| s.parse().ok()).unwrap_or(d);
    let s = EffectSettings { brightness: n(1, 4), speed: n(2, 2), color: n(3, 0) };

    let kb = Keyboard::open()?;
    kb.set_effect(effect, s)?;
    println!("set {effect:?} {s:?}");
    Ok(())
}
