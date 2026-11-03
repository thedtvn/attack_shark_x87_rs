//! The macro-generated variant lists keep the order and values the library
//! relied on before (Effect slots, controller codes) and round-trip names.
use attack_shark_x87::{Connection, ControllerAction, Effect, Key, KeyGroup};

#[test]
fn effect_slots_and_ids_unchanged() {
    let ids: Vec<u8> = Effect::ALL.iter().map(|&e| e as u8).collect();
    assert_eq!(ids, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 21]);
    assert_eq!(Effect::Custom.slot(), 18);
    assert_eq!(Effect::from_id(21), Some(Effect::Custom));
}

#[test]
fn controller_order_and_codes_unchanged() {
    let words: Vec<u32> = ControllerAction::ALL.iter().map(|a| a.encode()).collect();
    assert_eq!(
        words,
        [
            0x05000007, 0x06000007, 0x07000007, 0x08000007, 0x11000007, 0x01000007, 0x04000007,
            0x00000008, 0x00000208, 0x0a000007, 0x0b000007, 0x00010308, 0x00020308, 0x00020408,
            0x00010408, 0x01000008, 0x01000208,
        ]
    );
}

#[test]
fn every_enum_round_trips_its_variant_names() {
    for v in Effect::ALL {
        assert_eq!(Effect::from_variant_name(v.variant_name()), Some(v));
    }
    for v in ControllerAction::ALL {
        assert_eq!(ControllerAction::from_variant_name(v.variant_name()), Some(v));
    }
    for v in KeyGroup::ALL {
        assert_eq!(KeyGroup::from_variant_name(v.variant_name()), Some(v));
    }
    for v in Key::ALL {
        assert_eq!(Key::from_variant_name(v.variant_name()), Some(v));
    }
    for v in Connection::ALL {
        assert_eq!(Connection::from_variant_name(v.variant_name()), Some(v));
    }
    assert_eq!(ControllerAction::Receiver24G.variant_name(), "Receiver24G");
    assert_eq!(Key::Digit1.variant_name(), "Digit1");
    assert_eq!(Key::Digit1.name(), "1");
    assert_eq!(KeyGroup::COUNT, 18);
    assert_eq!(&KeyGroup::ALL[..4], &KeyGroup::AREAS);
    assert_eq!(&KeyGroup::ALL[4..], &KeyGroup::CLUSTERS);
}
