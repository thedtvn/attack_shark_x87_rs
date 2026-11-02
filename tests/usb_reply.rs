use attack_shark_x87::{protocol::*, led::LedConfig};
#[test]
fn short_usb_reply_has_full_header() {
    let req = build_packet(6, 0x84, 0, 1, 0, 128, &[]);
    let mut reply = req[..136].to_vec();
    reply[8+10] = 11;
    reply[8+0x18] = 4;
    reply[8+126] = 0x5a; reply[8+127] = 0xa5;
    let bytes = parse_reply(&reply, &req).unwrap();
    let cfg = LedConfig(bytes.try_into().unwrap());
    assert!(cfg.is_valid());
    assert_eq!(cfg.effect_id(), 11);
    assert_eq!(cfg.sleep_secs(), Some(120));
    assert!(parse_reply(&reply[..135], &req).is_err());
    reply[1] = 0x83;
    assert!(parse_reply(&reply, &req).is_err());
}
