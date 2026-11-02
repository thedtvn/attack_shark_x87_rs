//! Dump raw USB feature replies for the LED config read (debugging).
use hidapi::HidApi;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = HidApi::new()?;
    for d in api.device_list().filter(|d| d.vendor_id() == 0x258a && d.usage_page() == 0xff00) {
        println!("== {}", d.path().to_string_lossy());
        let dev = d.open_device(&api)?;
        let mut pkt = [0u8; 0x208];
        pkt[..8].copy_from_slice(&[6, 0x84, 0, 0, 1, 0, 0x80, 0]);
        if let Err(e) = dev.send_feature_report(&pkt) {
            println!("  send: {e}");
            continue;
        }
        let mut buf = [0u8; 0x208];
        let mut res = Err(hidapi::HidError::HidApiErrorEmpty);
        for _ in 0..3 {
            std::thread::sleep(std::time::Duration::from_millis(30));
            dev.send_feature_report(&pkt)?;
            std::thread::sleep(std::time::Duration::from_millis(30));
            buf = [0u8; 0x208];
            buf[0] = 6;
            res = dev.get_feature_report(&mut buf);
            if res.is_ok() {
                break;
            }
        }
        match res {
            Ok(n) => {
                println!("  got {n} bytes");
                for (i, row) in buf[..0x90].chunks(16).enumerate() {
                    println!("  {:03x}: {:02x?}", i * 16, row);
                }
            }
            Err(e) => println!("  get: {e}"),
        }
    }
    Ok(())
}
