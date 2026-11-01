//! Dump the 128-byte LED/config block as hex (and diff against a saved file).
//!   cargo run --example dumpcfg -- save a.bin
//!   cargo run --example dumpcfg -- diff a.bin
use attack_shark_x87::Keyboard;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let kb = Keyboard::open()?;
    let cur = kb.led_config()?.0;
    match (args.first().map(String::as_str), args.get(1)) {
        (Some("save"), Some(path)) => {
            std::fs::write(path, cur)?;
            println!("saved {path}");
        }
        (Some("diff"), Some(path)) => {
            let old = std::fs::read(path)?;
            let mut any = false;
            for i in 0..cur.len() {
                if old[i] != cur[i] {
                    println!("0x{i:02x}: {:#04x} -> {:#04x}  ({} -> {})", old[i], cur[i], old[i], cur[i]);
                    any = true;
                }
            }
            if !any {
                println!("no change");
            }
        }
        _ => {
            for (i, row) in cur.chunks(16).enumerate() {
                println!("{:02x}: {:02x?}", i * 16, row);
            }
        }
    }
    Ok(())
}
