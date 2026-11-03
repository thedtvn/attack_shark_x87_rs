//! Realtime frame demo: a red/blue band sweeping across the board for 8 s.
//!   cargo run --example realtime
use std::time::{Duration, Instant};

use attack_shark_x87::{realtime::quantize, Frame, Key, Keyboard, Rgb};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let kb = Keyboard::open()?;
    println!("connection: {:?}", kb.connection());

    // Key x positions from KB.ini order: column index in the row ≈ led / 6.
    let x = |led: u8| (led / 6) as f32 / 17.0;

    let t0 = Instant::now();
    while t0.elapsed() < Duration::from_secs(8) {
        let pos = (t0.elapsed().as_secs_f32() * 0.5).fract();
        let mut f = Frame::new();
        for led in Key::ALL.map(Key::index) {
            let d = (x(led) - pos).abs();
            let v = (1.0 - d * 4.0).max(0.0);
            if v > 0.0 {
                let c = Rgb((255.0 * v) as u8, 0, (255.0 * (1.0 - v)) as u8);
                f.set(led, quantize(c, 3));
            }
        }
        kb.send_frame_lossy(&f)?;
        std::thread::sleep(Duration::from_millis(30));
    }
    kb.stop_realtime()?;
    Ok(())
}
