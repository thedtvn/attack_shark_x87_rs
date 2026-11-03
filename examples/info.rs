use attack_shark_x87::{Key, Keyboard};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let kb = Keyboard::open()?;
    println!("connection: {:?}", kb.connection());
    println!("firmware:   {:#06x}", kb.firmware_version()?);
    match kb.password() {
        Ok(p) => println!("password:   {p:02x?}"),
        Err(e) => println!("password:   ({e})"),
    }

    let cfg = kb.led_config()?;
    println!("led config: {cfg:?}");
    for (i, row) in cfg.0.chunks(16).enumerate() {
        println!("  {:02x}: {:02x?}", i * 16, row);
    }

    let m = kb.key_matrix(0)?;
    for key in Key::ALL.into_iter().take(10) {
        println!("  {:>6} -> {:?}", key.name(), m.get(key.index()));
    }
    Ok(())
}
