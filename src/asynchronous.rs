//! Async-friendly interface: [`KeyboardAsync`].
//!
//! `Keyboard` is `Send` but not `Sync`, and its calls block (a macro save over
//! 2.4G takes about 10 s). `KeyboardAsync` moves the `Keyboard` onto one
//! dedicated worker thread. Each method sends a job to it and returns a future,
//! so async tasks never block executor threads.
//!
//! - Runtime-agnostic: no tokio/async-std dependency. Futures are woken from
//!   the worker thread, so they work in any executor (or a simple `block_on`).
//! - `Clone + Send + Sync`: clones share the same worker and keyboard.
//! - Calls run one at a time, in the order they are made. The keyboard can't
//!   process concurrent requests anyway (the 2.4G receiver aborts a stream when
//!   a new request arrives).
//! - The worker stops when the last clone is dropped. A future resolves to
//!   [`Error::Disconnected`] if the worker is gone.
//!
//! ```no_run
//! # async fn example() -> Result<(), attack_shark_x87::Error> {
//! use attack_shark_x87::{KeyboardAsync, Effect, EffectSettings};
//! let kb = KeyboardAsync::open().await?;
//! kb.set_effect(Effect::Respire, EffectSettings { brightness: 4, speed: 2, color: 0 }).await?;
//! let bank = kb.macro_bank().await?;
//! println!("{} macros", bank.macros.len());
//! // Anything else in the blocking API:
//! let fw = kb.with(|k| k.firmware_version()).await?;
//! # let _ = fw; Ok(()) }
//! ```

use std::future::Future;
use std::pin::Pin;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

use crate::{
    macros, Connection, Effect, EffectSettings, Error, Frame, Key, KeyAction, KeyMatrix, Keyboard,
    LedConfig, PerKeyColors, Result, Rgb, RgbTable, SaveProgress,
};

type Job = Box<dyn FnOnce(&Keyboard) + Send>;

/// Async handle to a keyboard owned by a worker thread. See the module docs.
#[derive(Clone)]
pub struct KeyboardAsync {
    jobs: Arc<Mutex<mpsc::Sender<Job>>>,
    connection: Connection,
}

impl std::fmt::Debug for KeyboardAsync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyboardAsync").field("connection", &self.connection).finish()
    }
}

impl KeyboardAsync {
    /// Open the keyboard (USB preferred, else 2.4G) on a new worker thread.
    pub async fn open() -> Result<Self> {
        Self::spawn(Keyboard::open).await
    }

    /// Wrap an already open keyboard.
    pub fn from_keyboard(kb: Keyboard) -> Self {
        let connection = kb.connection();
        let (tx, rx) = mpsc::channel::<Job>();
        std::thread::Builder::new()
            .name("attack-shark-x87".into())
            .spawn(move || {
                // Ends when every sender (every clone) is dropped.
                for job in rx {
                    job(&kb);
                }
            })
            .expect("failed to spawn keyboard worker thread");
        Self { jobs: Arc::new(Mutex::new(tx)), connection }
    }

    /// Open on the worker thread with a custom opener (e.g. `open_wired`).
    pub async fn spawn<F>(open: F) -> Result<Self>
    where
        F: FnOnce() -> Result<Keyboard> + Send + 'static,
    {
        let (fut, done) = oneshot();
        std::thread::Builder::new()
            .name("attack-shark-x87-open".into())
            .spawn(move || done(open()))
            .map_err(|_| Error::Disconnected)?;
        Ok(Self::from_keyboard(fut.await??))
    }

    /// USB cable or 2.4G receiver.
    pub fn connection(&self) -> Connection {
        self.connection
    }

    /// Run any blocking `Keyboard` method on the worker thread.
    pub fn with<T, F>(&self, f: F) -> impl Future<Output = Result<T>> + Send + 'static
    where
        T: Send + 'static,
        F: FnOnce(&Keyboard) -> Result<T> + Send + 'static,
    {
        let (fut, done) = oneshot();
        let sent = self
            .jobs
            .lock()
            .map_err(|_| ())
            .and_then(|tx| tx.send(Box::new(move |kb: &Keyboard| done(f(kb)))).map_err(|_| ()));
        async move {
            sent.map_err(|_| Error::Disconnected)?;
            fut.await?
        }
    }

    // ---- info -------------------------------------------------------------

    pub async fn firmware_version(&self) -> Result<u16> {
        self.with(|k| k.firmware_version()).await
    }

    pub async fn password(&self) -> Result<[u8; 6]> {
        self.with(|k| k.password()).await
    }

    // ---- lighting ---------------------------------------------------------

    pub async fn led_config(&self) -> Result<LedConfig> {
        self.with(|k| k.led_config()).await
    }

    pub async fn set_led_config(&self, cfg: LedConfig) -> Result<()> {
        self.with(move |k| k.set_led_config(&cfg)).await
    }

    pub async fn set_effect(&self, effect: Effect, settings: EffectSettings) -> Result<()> {
        self.with(move |k| k.set_effect(effect, settings)).await
    }

    pub async fn lights_off(&self) -> Result<()> {
        self.with(|k| k.lights_off()).await
    }

    pub async fn rgb_table(&self) -> Result<RgbTable> {
        self.with(|k| k.rgb_table()).await
    }

    pub async fn set_rgb_table(&self, table: RgbTable) -> Result<()> {
        self.with(move |k| k.set_rgb_table(&table)).await
    }

    pub async fn set_effect_color(&self, effect: Effect, index: usize, c: Rgb) -> Result<()> {
        self.with(move |k| k.set_effect_color(effect, index, c)).await
    }

    pub async fn per_key_colors(&self, group: u8) -> Result<PerKeyColors> {
        self.with(move |k| k.per_key_colors(group)).await
    }

    pub async fn set_per_key_colors(&self, group: u8, colors: PerKeyColors) -> Result<()> {
        self.with(move |k| k.set_per_key_colors(group, &colors)).await
    }

    // ---- keys -------------------------------------------------------------

    pub async fn key_matrix(&self, layer: u8) -> Result<KeyMatrix> {
        self.with(move |k| k.key_matrix(layer)).await
    }

    pub async fn set_key_matrix(&self, layer: u8, m: KeyMatrix) -> Result<()> {
        self.with(move |k| k.set_key_matrix(layer, &m)).await
    }

    pub async fn remap_key(&self, layer: u8, key: Key, action: KeyAction) -> Result<()> {
        self.with(move |k| k.remap_key(layer, key, action)).await
    }

    // ---- macros: get / save / clear ---------------------------------------

    pub async fn macro_bank(&self) -> Result<macros::MacroBank> {
        self.with(|k| k.macro_bank()).await
    }

    /// See [`Keyboard::save_macro_bank`]. `on_progress` runs on the worker
    /// thread; keep it quick (e.g. send to a channel or update an atomic).
    pub async fn save_macro_bank<P>(&self, bank: macros::MacroBank, on_progress: Option<P>) -> Result<()>
    where
        P: FnMut(SaveProgress) + Send + 'static,
    {
        self.with(move |k| match on_progress {
            Some(mut p) => k.save_macro_bank(&bank, Some(&mut p)),
            None => k.save_macro_bank(&bank, None),
        })
        .await
    }

    pub async fn clear_macro_bank(&self) -> Result<()> {
        self.with(|k| k.clear_macro_bank()).await
    }

    // ---- options ----------------------------------------------------------

    pub async fn debounce(&self) -> Result<u8> {
        self.with(|k| k.debounce()).await
    }

    pub async fn set_debounce(&self, stage: u8) -> Result<()> {
        self.with(move |k| k.set_debounce(stage)).await
    }

    pub async fn tap(&self) -> Result<Option<u8>> {
        self.with(|k| k.tap()).await
    }

    pub async fn set_tap(&self, sensitivity: Option<u8>) -> Result<()> {
        self.with(move |k| k.set_tap(sensitivity)).await
    }

    pub async fn sleep_timeout(&self) -> Result<Option<u32>> {
        self.with(|k| k.sleep_timeout()).await
    }

    pub async fn set_sleep_timeout(&self, secs: Option<u32>) -> Result<()> {
        self.with(move |k| k.set_sleep_timeout(secs)).await
    }

    // ---- realtime ---------------------------------------------------------

    pub async fn send_frame(&self, frame: Frame) -> Result<()> {
        self.with(move |k| k.send_frame(&frame)).await
    }

    pub async fn send_frame_lossy(&self, frame: Frame) -> Result<()> {
        self.with(move |k| k.send_frame_lossy(&frame)).await
    }

    pub async fn stop_realtime(&self) -> Result<()> {
        self.with(|k| k.stop_realtime()).await
    }

    // ---- misc -------------------------------------------------------------

    pub async fn factory_reset(&self) -> Result<()> {
        self.with(|k| k.factory_reset()).await
    }
}

// ---- minimal one-shot channel (no runtime dependency) ------------------------

struct Slot<T> {
    value: Option<T>,
    waker: Option<Waker>,
    closed: bool,
}

struct Receiver<T>(Arc<Mutex<Slot<T>>>);

/// Completes the paired receiver; dropping it without calling resolves the
/// receiver to `Error::Disconnected` (e.g. if the job panicked).
struct Sender<T>(Option<Arc<Mutex<Slot<T>>>>);

impl<T> Sender<T> {
    fn finish(&mut self, value: Option<T>) {
        if let Some(shared) = self.0.take() {
            let waker = {
                let mut slot = shared.lock().unwrap_or_else(|e| e.into_inner());
                slot.value = value;
                slot.closed = true;
                slot.waker.take()
            };
            if let Some(w) = waker {
                w.wake();
            }
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        self.finish(None);
    }
}

fn oneshot<T: Send + 'static>() -> (Receiver<T>, impl FnOnce(T) + Send + 'static) {
    let shared = Arc::new(Mutex::new(Slot { value: None, waker: None, closed: false }));
    let mut sender = Sender(Some(shared.clone()));
    (Receiver(shared), move |value| sender.finish(Some(value)))
}

impl<T> Future for Receiver<T> {
    type Output = Result<T>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut slot = self.0.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(v) = slot.value.take() {
            return Poll::Ready(Ok(v));
        }
        if slot.closed {
            return Poll::Ready(Err(Error::Disconnected));
        }
        slot.waker = Some(cx.waker().clone());
        Poll::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::task::Wake;

    struct ThreadWaker(std::thread::Thread);
    impl Wake for ThreadWaker {
        fn wake(self: Arc<Self>) {
            self.0.unpark();
        }
    }

    /// Minimal executor proving the futures need no async runtime.
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

    #[test]
    fn oneshot_delivers_from_another_thread() {
        let (rx, done) = oneshot::<u32>();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(20));
            done(7);
        });
        assert_eq!(block_on(rx).unwrap(), 7);
    }

    #[test]
    fn dropped_sender_resolves_to_disconnected() {
        let (rx, done) = oneshot::<u32>();
        std::thread::spawn(move || drop(done));
        assert!(matches!(block_on(rx), Err(Error::Disconnected)));
    }

    #[test]
    fn panicking_job_does_not_hang_the_caller() {
        let (rx, done) = oneshot::<u32>();
        let worker = std::thread::spawn(move || {
            let _done = done; // dropped during unwind
            panic!("job failed");
        });
        assert!(matches!(block_on(rx), Err(Error::Disconnected)));
        assert!(worker.join().is_err());
    }

    #[test]
    fn jobs_run_in_order_on_one_thread() {
        // Same pattern as KeyboardAsync, without hardware.
        let (tx, rx) = mpsc::channel::<Box<dyn FnOnce() + Send>>();
        let worker = std::thread::spawn(move || {
            for job in rx {
                job();
            }
        });
        let order = Arc::new(Mutex::new(Vec::new()));
        let counter = Arc::new(AtomicUsize::new(0));
        let mut waits = Vec::new();
        for i in 0..20 {
            let (fut, done) = oneshot::<usize>();
            let (order, counter) = (order.clone(), counter.clone());
            tx.send(Box::new(move || {
                order.lock().unwrap().push(i);
                done(counter.fetch_add(1, Ordering::SeqCst));
            }))
            .unwrap();
            waits.push(fut);
        }
        drop(tx);
        let results: Vec<usize> = waits.into_iter().map(|f| block_on(f).unwrap()).collect();
        worker.join().unwrap();
        assert_eq!(results, (0..20).collect::<Vec<_>>());
        assert_eq!(*order.lock().unwrap(), (0..20).collect::<Vec<_>>());
    }

    fn assert_send_sync<T: Send + Sync + Clone + 'static>() {}
    #[test]
    fn handle_is_shareable() {
        assert_send_sync::<KeyboardAsync>();
    }
}
