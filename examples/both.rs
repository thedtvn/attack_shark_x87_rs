//! Read the config block over USB and over the 2.4G dongle and print both.
use attack_shark_x87::Keyboard;
use hidapi::HidApi;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = HidApi::new()?;
    for (name, kb) in [("usb", Keyboard::open_wired(&api)), ("2.4g", Keyboard::open_wireless(&api))] {
        match kb.and_then(|k| k.led_config()) {
            Ok(c) => {
                println!("{name}:");
                for (i, row) in c.0.chunks(16).enumerate() {
                    println!("  {:02x}: {:02x?}", i * 16, row);
                }
            }
            Err(e) => println!("{name}: {e}"),
        }
    }
    Ok(())
}
