//! Write one byte of the LED/config block and read it back.
//!   cargo run --example setbyte -- 0x03 6
use attack_shark_x87::Keyboard;

fn parse(s: &str) -> u8 {
    if let Some(h) = s.strip_prefix("0x") {
        u8::from_str_radix(h, 16).unwrap()
    } else {
        s.parse().unwrap()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let (off, val) = (parse(&a[0]) as usize, parse(&a[1]));
    let kb = Keyboard::open()?;
    let mut cfg = kb.led_config()?;
    let before = cfg.0[off];
    cfg.0[off] = val;
    kb.set_led_config(&cfg)?;
    let after = kb.led_config()?.0[off];
    println!("0x{off:02x}: {before} -> wrote {val} -> reads back {after}");
    Ok(())
}
