//! Build a macro bank offline; hardware writes require --upload.
//! cargo run --example macro                 # print encoded size, no hardware
//! cargo run --example macro -- --upload      # replace shared bank, no binding
use attack_shark_x87::{
    macros::{Macro, MacroAction, MacroBank, MacroEvent},
    Keyboard,
};
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bank = MacroBank {
        macros: vec![Macro {
            name: "Type A".into(),
            events: vec![
                MacroEvent { action: MacroAction::Key { usage: 4, down: true }, delay_ms: 0 },
                MacroEvent { action: MacroAction::Key { usage: 4, down: false }, delay_ms: 50 },
            ],
        }],
    };
    let bytes = bank.encode()?;
    println!("Encoded {} macros into {} bytes", bank.macros.len(), bytes.len());
    if std::env::args().any(|a| a == "--upload") {
        eprintln!("Replacing shared macro bank: existing slot contents may change on all layers.");
        let kb = Keyboard::open()?;
        kb.save_macro_bank(&bank, &mut |p| {
            let filled = p.percent as usize / 4;
            print!("\rSaving macros [{}{}] {:>3}%", "#".repeat(filled), " ".repeat(25 - filled), p.percent);
            std::io::stdout().flush().ok();
        })?;
        println!("\nSaved. No keys assigned.");
    } else {
        println!("No hardware accessed. Back up before explicitly uploading.");
    }
    Ok(())
}
