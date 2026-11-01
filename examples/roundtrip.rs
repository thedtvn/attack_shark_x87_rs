//! Safe write test: read the LED config, write it back unchanged, read again.
use attack_shark_x87::Keyboard;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let kb = Keyboard::open()?;
    let before = kb.led_config()?;
    kb.set_led_config(&before)?;
    let after = kb.led_config()?;
    println!("connection: {:?}", kb.connection());
    println!("write ok, unchanged: {}", before == after);
    Ok(())
}
