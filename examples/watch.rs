//! Print connection state and battery, then connection events.
//!
//!   cargo run --example watch -- [seconds]
use attack_shark_x87::{watch, Event, Keyboard, Watcher};
use std::time::Duration;

fn main() -> Result<(), attack_shark_x87::Error> {
    println!("status: {:?}", watch::status()?);
    match Keyboard::open() {
        Ok(kb) => println!(
            "open {:?}: linked={:?} battery={:?}",
            kb.connection(),
            kb.is_linked(),
            kb.battery()
        ),
        Err(e) => println!("open: {e}"),
    }
    let secs = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(30);
    let start = std::time::Instant::now();
    let w = Watcher::start(move |e: Event| println!("{:>6}ms {e:?}", start.elapsed().as_millis()))?;
    std::thread::sleep(Duration::from_secs(secs));
    w.stop();
    Ok(())
}
