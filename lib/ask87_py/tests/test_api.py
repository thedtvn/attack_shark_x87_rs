"""Offline tests (no keyboard needed): `uv run pytest`."""

from ask87 import (
    LAYOUT,
    ControllerAction,
    Effect,
    Key,
    KeyAction,
    KeyGroup,
    Macro,
    MacroEvent,
    decode_action,
    encode_action,
    validate_macros,
)
from ask87 import _native as native


def test_enums_match_rust() -> None:
    assert [k.value for k in Key] == [t[0] for t in native.key_table()]
    assert [g.value for g in KeyGroup] == [t[0] for t in native.group_table()]
    assert [e.value for e in Effect] == [t[0] for t in native.effect_table()]
    assert [c.value for c in ControllerAction] == [t[0] for t in native.controller_table()]


def test_layout() -> None:
    assert [k for row in LAYOUT for g in row for k in g.keys] == list(Key)
    assert all(g.is_area for g in KeyGroup.areas())
    assert Key.CAPS_LOCK.group is KeyGroup.MAIN
    assert Key.S.left is Key.A
    assert Key.at(Key.F6.rect.x1 + 1, Key.F6.rect.y1 + 1) is Key.F6


def test_actions_round_trip() -> None:
    for a in [
        KeyAction.none(),
        KeyAction.key(0x04, modifiers=0x02),
        KeyAction.consumer(0xE9),
        KeyAction.mouse(1),
        KeyAction.fn(),
        KeyAction.macro(3, "until_released"),
        KeyAction.macro(0, 5),
        KeyAction.control(ControllerAction.BLUETOOTH_1),
    ]:
        assert decode_action(encode_action(a)) == a


def test_macros_validate() -> None:
    assert validate_macros([Macro("hi", [MacroEvent.key_down(4, 10), MacroEvent.key_up(4)])]) > 0


def test_async_api_is_awaitable() -> None:
    import asyncio
    import inspect

    import pytest

    from ask87 import AsyncKeyboard, Keyboard

    public = [n for n in vars(Keyboard) if not n.startswith("_") and callable(getattr(Keyboard, n))]
    for name in public:
        if name in ("close", "open", "closed", "watch"):
            continue
        assert inspect.iscoroutinefunction(getattr(AsyncKeyboard, name)), name

    with pytest.raises(ValueError):
        asyncio.run(AsyncKeyboard.open("bogus"))  # ty: ignore[invalid-argument-type]


def test_close_is_sync() -> None:
    import inspect

    from ask87 import AsyncKeyboard

    assert not inspect.iscoroutinefunction(AsyncKeyboard.close)


def test_connection_events_types() -> None:
    from ask87 import Battery, BatteryEvent, UsbEvent, WirelessEvent, _event

    assert _event(("usb", 1, None, None)) == UsbEvent(True)
    assert _event(("wireless", 0, True, None)) == WirelessEvent(False, True)
    assert _event(("battery", 93, False, None)) == BatteryEvent(Battery(93, False))


def test_keyboard_watch_methods() -> None:
    import inspect

    from ask87 import AsyncKeyboard, Keyboard

    assert callable(Keyboard.watch)
    assert not inspect.iscoroutinefunction(AsyncKeyboard.watch)


def test_upper_snake_members_and_keyset() -> None:
    from ask87 import Key, KeyGroup, KeySet

    assert Key.CAPS_LOCK.value == "CapsLock"
    assert KeyGroup.F1_F4.value == "F1F4"
    row: KeySet = KeyGroup.NUMBER_ROW + KeyGroup.INSERT_HOME_PGUP
    assert row[-3:] == (Key.INSERT, Key.HOME, Key.PGUP)
    assert Key.W + Key.A == (Key.W, Key.A)
    assert Key.ESC + KeyGroup.F1_F4 == (Key.ESC, Key.F1, Key.F2, Key.F3, Key.F4)
    assert (KeyGroup.F1_F4 - Key.F2) == (Key.F1, Key.F3, Key.F4)
    assert KeyGroup.ESC + Key.ESC == (Key.ESC,)
    rows = [g for row in __import__("ask87").LAYOUT for g in row]
    total = KeySet()
    for g in rows:
        total = total + g
    assert list(total) == list(Key)
