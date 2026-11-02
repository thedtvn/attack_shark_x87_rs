//! Offline binding examples. This does not open a keyboard or write settings.
use attack_shark_x87::{ControllerAction, KeyAction};
fn main() {
    for action in [
        KeyAction::Fn,
        KeyAction::Fn2,
        KeyAction::Controller(ControllerAction::Receiver24G),
        KeyAction::Controller(ControllerAction::LightBarMode),
        attack_shark_x87::controller::APPLICATION_SWITCH,
    ] {
        println!("{action:?}: {:02x?}", action.encode().to_le_bytes());
    }
    println!("No hardware accessed. To assign deliberately, use Keyboard::remap_key.");
}
