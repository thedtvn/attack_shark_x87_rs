//! Read-only check: reads the full macro bank and key layers (USB or 2.4G),
//! printing contents and timings. Never writes to the keyboard.
use attack_shark_x87::Keyboard;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let kb = Keyboard::open()?;
    println!("connection: {:?}", kb.connection());

    let t = Instant::now();
    let bank = kb.macro_bank()?;
    println!("macro bank: {} macros in {:?}", bank.macros.len(), t.elapsed());
    for (i, m) in bank.macros.iter().enumerate() {
        println!("  slot {i}: {} ({} events)", m.name, m.events.len());
    }

    let t = Instant::now();
    for layer in 0..attack_shark_x87::LAYERS {
        kb.key_matrix(layer)?;
    }
    println!("4 key layers in {:?}", t.elapsed());

    println!("sleep: {:?} s", kb.led_config()?.sleep_secs());
    Ok(())
}
