use attack_shark_x87::{Frame, Key, Rgb};

#[test]
fn full_frame_is_led_indexed_rgb() {
    let mut f = Frame::new();
    f.set_key(Key::Esc, Rgb(1, 2, 3)).set_key(Key::A, Rgb(9, 8, 7));
    let b = f.encode_full();
    assert_eq!(b.len(), attack_shark_x87::realtime::FULL_FRAME_LEN);
    let at = |k: Key| &b[k.index() as usize * 3..k.index() as usize * 3 + 3];
    assert_eq!(at(Key::Esc), &[1, 2, 3]);
    assert_eq!(at(Key::A), &[9, 8, 7]);
    assert_eq!(at(Key::B), &[0, 0, 0]);
}
