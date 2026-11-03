//! Read-only `KeyboardAsync` check. Works with any executor; this one uses a
//! tiny `block_on` so the library needs no async runtime dependency.
//! Several reads are started at once; they run one at a time on the worker.
use attack_shark_x87::KeyboardAsync;
use std::future::Future;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
use std::time::Instant;

struct ThreadWaker(std::thread::Thread);
impl Wake for ThreadWaker {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
}

fn block_on<F: Future>(fut: F) -> F::Output {
    let mut fut = std::pin::pin!(fut);
    let waker = Waker::from(Arc::new(ThreadWaker(std::thread::current())));
    let mut cx = Context::from_waker(&waker);
    loop {
        if let Poll::Ready(v) = fut.as_mut().poll(&mut cx) {
            return v;
        }
        std::thread::park();
    }
}

fn main() -> Result<(), attack_shark_x87::Error> {
    block_on(async {
        let start = Instant::now();
        let kb = KeyboardAsync::open().await?;
        println!("opened {:?} in {:?}", kb.connection(), start.elapsed());

        // Start three reads before awaiting any of them; the handle is
        // cloneable and they are queued in order on the worker thread.
        let (a, b, c) = (kb.clone(), kb.clone(), kb.clone());
        let led = a.led_config();
        let bank = b.macro_bank();
        let layer = c.key_matrix(1);

        let t = Instant::now();
        let bank = bank.await?;
        let layer = layer.await?;
        let led = led.await?;
        println!(
            "led valid: {}, macros: {}, FN1 entries: {} in {:?}",
            led.is_valid(),
            bank.macros.len(),
            layer.0.iter().filter(|&&w| w != 0).count(),
            t.elapsed()
        );
        println!("sleep: {:?} s", kb.sleep_timeout().await?);
        Ok(())
    })
}
