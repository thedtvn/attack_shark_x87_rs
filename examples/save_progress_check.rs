//! Hardware check of `save_macro_bank`. Requires `--confirm`.
//! Writes a one-macro test bank, checks it reads back, then restores the bank
//! that was there before. Only the macro bank is written.
use attack_shark_x87::{
    macros::{Macro, MacroAction, MacroBank, MacroEvent},
    Keyboard,
};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if !std::env::args().any(|a| a == "--confirm") {
        println!("Writes to the keyboard and restores the previous bank. Pass --confirm.");
        return Ok(());
    }
    let kb = Keyboard::open()?;
    let original = kb.macro_bank()?;
    let test = MacroBank {
        macros: vec![Macro {
            name: "Progress".into(),
            events: vec![
                MacroEvent { action: MacroAction::Key { usage: 4, down: true }, delay_ms: 0 },
                MacroEvent { action: MacroAction::Key { usage: 4, down: false }, delay_ms: 20 },
            ],
        }],
    };
    let save = |bank: &MacroBank| -> Result<(), attack_shark_x87::Error> {
        let start = Instant::now();
        let mut seen = Vec::new();
        kb.save_macro_bank(bank, &mut |p| seen.push(p.percent))?;
        assert_eq!(seen.first(), Some(&0));
        assert_eq!(seen.last(), Some(&100));
        assert!(seen.windows(2).all(|w| w[0] < w[1]), "progress went backwards");
        println!("{:?}: saved in {:?}, {} progress updates", kb.connection(), start.elapsed(), seen.len());
        Ok(())
    };
    save(&test)?;
    assert_eq!(kb.macro_bank()?, test, "test bank did not read back");
    save(&original)?;
    assert_eq!(kb.macro_bank()?, original, "original bank not restored");
    println!("PASS: test bank written and read back; original bank ({} macros) restored", original.macros.len());
    Ok(())
}
