//! Connection events: USB cable, 2.4G receiver and the wireless link.
//!
//! A [`Watcher`] runs a background thread that
//!
//! - polls the HID device list (every [`POLL`]) for the wired keyboard
//!   (`258A:010C`) and the 2.4G receiver (`3554:FA09`), and
//! - while the receiver is plugged in, listens to its vendor collection for
//!   link reports, which the receiver pushes on its own when the keyboard
//!   connects or drops (switch off, sleep, mode change), the same reports
//!   `CDev3632`'s service thread handles in OemDrv.exe (`m_bOnline`):
//!
//! | report                        | meaning                         |
//! |-------------------------------|---------------------------------|
//! | `13 0A 01 00 04 02 xx ..`     | link: `xx` = 1 online, 0 offline |
//! | `13 0A 01 00 04 05 pp ss`     | battery `pp` %, `ss` 0x10 = charging |
//!
//! When the receiver appears, the link state is queried once with wireless
//! command `0x07` (`13 07 01 00 01 ..` → byte 5 = online), as the driver's
//! `Watcher_Thread_3632` does.
//!
//! The watcher uses its own HID handles, so it runs next to an open
//! [`crate::Keyboard`] (each handle receives its own copy of input reports).
//!
//! ```no_run
//! use attack_shark_x87::watch::{Event, Watcher};
//! let w = Watcher::start(|e| match e {
//!     Event::Usb { connected } => println!("USB cable: {connected}"),
//!     Event::Wireless { connected, receiver } => println!("2.4G link: {connected} (receiver plugged in: {receiver})"),
//!     Event::Battery(b) => println!("battery {}% charging={}", b.percent, b.charging),
//! })?;
//! // ...
//! w.stop();
//! # Ok::<(), attack_shark_x87::Error>(())
//! ```

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use hidapi::{HidApi, HidDevice};

use crate::wireless::{self, notify, parse_notify, query_battery, query_link, Battery, PACKET_LEN};
use crate::{Result, PID_WIRED, PID_WIRELESS, USAGE, USAGE_PAGE, VID_WIRED, VID_WIRELESS};

/// How often the device list is checked for plugged / unplugged devices.
pub const POLL: Duration = Duration::from_millis(500);

/// A change in how the keyboard is connected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// The keyboard was plugged in / unplugged over the USB cable.
    Usb { connected: bool },
    /// The 2.4G link went up or down. `receiver` says whether the USB
    /// receiver is plugged in (`connected` is always `false` when it isn't).
    Wireless { connected: bool, receiver: bool },
    /// Battery reported by the receiver over 2.4G (on link-up, when it changes,
    /// and once when the watcher starts).
    Battery(Battery),
}

/// Current connection state, see [`status`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Status {
    /// The keyboard is plugged in over USB.
    pub usb: bool,
    /// The 2.4G receiver is plugged in.
    pub receiver: bool,
    /// The keyboard is linked to the receiver.
    pub wireless: bool,
}

fn is_wired(d: &hidapi::DeviceInfo) -> bool {
    d.vendor_id() == VID_WIRED && d.product_id() == PID_WIRED && d.usage_page() == USAGE_PAGE && d.usage() == USAGE
}

fn is_receiver(d: &hidapi::DeviceInfo) -> bool {
    d.vendor_id() == VID_WIRELESS
        && d.product_id() == PID_WIRELESS
        && d.usage_page() == wireless::USAGE_PAGE
        && d.usage() == wireless::USAGE
}

/// One-off snapshot of the current connection state.
pub fn status() -> Result<Status> {
    let api = HidApi::new()?;
    let usb = api.device_list().any(is_wired);
    let receiver = api.device_list().find(|d| is_receiver(d));
    let wireless = match receiver {
        Some(info) => query_link(&info.open_device(&api)?).unwrap_or(false),
        None => false,
    };
    Ok(Status { usb, receiver: receiver.is_some(), wireless })
}

/// Background connection watcher; stops on [`Watcher::stop`] or drop.
pub struct Watcher {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl Watcher {
    /// Start watching. `on_event` runs on the watcher thread; it first gets
    /// the current state (`Usb` and `Wireless` events), then every change.
    pub fn start(on_event: impl FnMut(Event) + Send + 'static) -> Result<Self> {
        let api = HidApi::new()?;
        let stop = Arc::new(AtomicBool::new(false));
        let flag = stop.clone();
        let thread = std::thread::Builder::new()
            .name("x87-watch".into())
            .spawn(move || run(api, &flag, on_event))
            .map_err(|_| crate::Error::Disconnected)?;
        Ok(Watcher { stop, thread: Some(thread) })
    }

    /// Stop the thread and wait for it (at most about one poll interval).
    pub fn stop(mut self) {
        self.shutdown();
    }

    fn shutdown(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

impl Drop for Watcher {
    fn drop(&mut self) {
        self.shutdown();
    }
}

struct State<F: FnMut(Event)> {
    emit: F,
    usb: Option<bool>,
    wireless: Option<(bool, bool)>,
}

impl<F: FnMut(Event)> State<F> {
    fn usb(&mut self, connected: bool) {
        if self.usb != Some(connected) {
            self.usb = Some(connected);
            (self.emit)(Event::Usb { connected });
        }
    }

    fn wireless(&mut self, connected: bool, receiver: bool) {
        if self.wireless != Some((connected, receiver)) {
            self.wireless = Some((connected, receiver));
            (self.emit)(Event::Wireless { connected, receiver });
        }
    }
}

fn run(mut api: HidApi, stop: &AtomicBool, emit: impl FnMut(Event)) {
    let mut s = State { emit, usb: None, wireless: None };
    let mut receiver: Option<HidDevice> = None;
    let mut last_poll: Option<Instant> = None;
    let mut buf = [0u8; PACKET_LEN];

    while !stop.load(Ordering::Relaxed) {
        if last_poll.is_none_or(|t| t.elapsed() >= POLL) {
            last_poll = Some(Instant::now());
            if api.refresh_devices().is_ok() {
                s.usb(api.device_list().any(is_wired));
                let info = api.device_list().find(|d| is_receiver(d));
                match (info, &receiver) {
                    (Some(info), None) => match info.open_device(&api) {
                        Ok(dev) => {
                            let linked = query_link(&dev).unwrap_or(false);
                            s.wireless(linked, true);
                            if let Ok(Some(b)) = query_battery(&dev) {
                                (s.emit)(Event::Battery(b));
                            }
                            receiver = Some(dev);
                        }
                        Err(_) => s.wireless(false, true),
                    },
                    (None, _) => {
                        receiver = None;
                        s.wireless(false, false);
                    }
                    (Some(_), Some(_)) => {}
                }
            }
        }

        let Some(dev) = &receiver else {
            std::thread::sleep(Duration::from_millis(50));
            continue;
        };
        match dev.read_timeout(&mut buf, 100) {
            Ok(PACKET_LEN) => match parse_notify(&buf) {
                Some((notify::LINK, v, _)) => s.wireless(v != 0, true),
                Some((notify::BATTERY, pct, state)) => (s.emit)(Event::Battery(Battery::from_report(pct, state))),
                _ => {}
            },
            Ok(_) => {}
            Err(_) => {
                // Unplugged: the next poll reports it.
                receiver = None;
                last_poll = None;
            }
        }
    }
}
