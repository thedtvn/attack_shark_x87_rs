//! List every HID interface of the X87 (wired and dongle) with usage pages.
use hidapi::HidApi;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = HidApi::new()?;
    for d in api.device_list() {
        let (v, p) = (d.vendor_id(), d.product_id());
        if (v == 0x258A && p == 0x010C) || v == 0x3554 {
            println!(
                "{v:04x}:{p:04x} if={} page={:#06x} usage={:#06x} {:?} {}",
                d.interface_number(),
                d.usage_page(),
                d.usage(),
                d.product_string(),
                d.path().to_string_lossy()
            );
        }
    }
    Ok(())
}
