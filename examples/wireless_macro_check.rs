//! Read-only check: reads the full macro bank (USB or 2.4G) and prints it.
use attack_shark_x87::Keyboard;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let kb = Keyboard::open()?;
    println!("connection: {:?}", kb.connection());
    let bank = kb.macro_bank()?;
    println!("{} macros", bank.macros.len());
    for (i, m) in bank.macros.iter().enumerate() {
        println!("slot {i}: {} ({} events)", m.name, m.events.len());
    }
    let cfg = kb.led_config()?;
    println!("sleep: {:?} s", cfg.sleep_secs());
    Ok(())
}
