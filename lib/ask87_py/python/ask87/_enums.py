"""Enums mirroring the Rust ones. Members are the Rust variant names;
metadata comes from the native module. tests/test_enums.py checks they stay in
sync with `_native`."""

from __future__ import annotations

from collections.abc import Iterable
from dataclasses import dataclass
from enum import Enum
from functools import cache
from typing import Optional, Union

from . import _native


@dataclass(frozen=True)
class Rect:
    """A rectangle in vendor keyboard-image pixels (x2/y2 exclusive)."""

    x1: int
    y1: int
    x2: int
    y2: int

    def contains(self, x: int, y: int) -> bool:
        return self.x1 <= x < self.x2 and self.y1 <= y < self.y2


@cache
def _keys() -> dict[str, tuple[str, int, Rect]]:
    return {v: (label, i, Rect(*r)) for v, label, i, r in _native.key_table()}


@cache
def _groups() -> dict[str, tuple[bool, tuple[tuple[str, ...], ...], Rect]]:
    return {v: (a, tuple(tuple(c) for c in cs), Rect(*r)) for v, a, cs, r in _native.group_table()}


class KeySet(tuple["Key", ...]):
    """An ordered set of keys, built with ``+`` from keys and groups.

        row = KeyGroup.NUMBER_ROW + KeyGroup.INSERT_HOME_PGUP
        wasd = Key.W + Key.A + Key.S + Key.D

    Duplicates are dropped (first occurrence wins). Usable anywhere a key
    collection is accepted, e.g. ``{k: color for k in row}``.
    """

    def __new__(cls, keys: Iterable[Key] = ()) -> KeySet:
        return super().__new__(cls, dict.fromkeys(keys))

    def __add__(self, other: KeyLike) -> KeySet:  # type: ignore[override]
        return KeySet((*self, *_keys_of(other)))

    def __radd__(self, other: KeyLike) -> KeySet:
        return KeySet((*_keys_of(other), *self))

    def __sub__(self, other: KeyLike) -> KeySet:
        drop = set(_keys_of(other))
        return KeySet(k for k in self if k not in drop)

    @property
    def rect(self) -> Rect:
        """Bounding box of all keys."""
        rs = [k.rect for k in self]
        if not rs:
            raise ValueError("empty KeySet has no rect")
        return Rect(min(r.x1 for r in rs), min(r.y1 for r in rs), max(r.x2 for r in rs), max(r.y2 for r in rs))

    def __repr__(self) -> str:
        return f"KeySet({', '.join(k.name for k in self)})"


KeyLike = Union["Key", "KeyGroup", KeySet, Iterable["Key"]]


def _keys_of(x: KeyLike) -> tuple[Key, ...]:
    if isinstance(x, Key):
        return (x,)
    if isinstance(x, KeyGroup):
        return x.keys
    return tuple(x)


class Key(Enum):
    """A physical key on the X87."""

    ESC = "Esc"
    F1 = "F1"
    F2 = "F2"
    F3 = "F3"
    F4 = "F4"
    F5 = "F5"
    F6 = "F6"
    F7 = "F7"
    F8 = "F8"
    F9 = "F9"
    F10 = "F10"
    F11 = "F11"
    F12 = "F12"
    PRT_SC = "PrtSc"
    SCR_LOCK = "ScrLock"
    PAUSE = "Pause"
    GRAVE = "Grave"
    DIGIT_1 = "Digit1"
    DIGIT_2 = "Digit2"
    DIGIT_3 = "Digit3"
    DIGIT_4 = "Digit4"
    DIGIT_5 = "Digit5"
    DIGIT_6 = "Digit6"
    DIGIT_7 = "Digit7"
    DIGIT_8 = "Digit8"
    DIGIT_9 = "Digit9"
    DIGIT_0 = "Digit0"
    MINUS = "Minus"
    EQUAL = "Equal"
    BACKSPACE = "Backspace"
    INSERT = "Insert"
    HOME = "Home"
    PGUP = "PgUp"
    TAB = "Tab"
    Q = "Q"
    W = "W"
    E = "E"
    R = "R"
    T = "T"
    Y = "Y"
    U = "U"
    I = "I"
    O = "O"
    P = "P"
    BRACKET_LEFT = "BracketLeft"
    BRACKET_RIGHT = "BracketRight"
    BACKSLASH = "Backslash"
    DELETE = "Delete"
    END = "End"
    PGDN = "PgDn"
    CAPS_LOCK = "CapsLock"
    A = "A"
    S = "S"
    D = "D"
    F = "F"
    G = "G"
    H = "H"
    J = "J"
    K = "K"
    L = "L"
    SEMICOLON = "Semicolon"
    QUOTE = "Quote"
    ENTER = "Enter"
    L_SHIFT = "LShift"
    Z = "Z"
    X = "X"
    C = "C"
    V = "V"
    B = "B"
    N = "N"
    M = "M"
    COMMA = "Comma"
    PERIOD = "Period"
    SLASH = "Slash"
    R_SHIFT = "RShift"
    UP = "Up"
    L_CTRL = "LCtrl"
    L_WIN = "LWin"
    L_ALT = "LAlt"
    SPACE = "Space"
    R_ALT = "RAlt"
    FN = "Fn"
    APP = "App"
    R_CTRL = "RCtrl"
    LEFT = "Left"
    DOWN = "Down"
    RIGHT = "Right"

    def __add__(self, other: KeyLike) -> KeySet:
        """``Key.W + Key.A`` / ``Key.ESC + KeyGroup.F1_F4`` -> KeySet."""
        return KeySet((*_keys_of(self), *_keys_of(other)))

    def __radd__(self, other: KeyLike) -> KeySet:
        return KeySet((*_keys_of(other), *_keys_of(self)))

    def __sub__(self, other: KeyLike) -> KeySet:
        drop = set(_keys_of(other))
        return KeySet(k for k in _keys_of(self) if k not in drop)

    @property
    def label(self) -> str:
        """Printed legend, e.g. "Caps Lock"."""
        return _keys()[self.value][0]

    @property
    def index(self) -> int:
        """Hardware key index (key matrix / LED position)."""
        return _keys()[self.value][1]

    @property
    def rect(self) -> Rect:
        return _keys()[self.value][2]

    @property
    def group(self) -> KeyGroup:
        """The area (FKey, Main, Navigation, Arrows) containing this key."""
        return KeyGroup(_native.group_of(self.value)[0])

    @property
    def cluster(self) -> KeyGroup:
        return KeyGroup(_native.group_of(self.value)[1])

    @property
    def row(self) -> int:
        return _native.group_of(self.value)[2]

    @property
    def left(self) -> Optional[Key]:
        n = _native.key_neighbour(self.value, "left")
        return None if n is None else Key(n)

    @property
    def right(self) -> Optional[Key]:
        n = _native.key_neighbour(self.value, "right")
        return None if n is None else Key(n)

    @staticmethod
    def at(x: int, y: int) -> Optional[Key]:
        """The key under a point of the vendor keyboard image."""
        n = _native.key_at(x, y)
        return None if n is None else Key(n)


class KeyGroup(Enum):
    """A key area (FKey, Main, Navigation, Arrows) or a physical cluster."""

    F_KEY = "FKey"
    MAIN = "Main"
    NAVIGATION = "Navigation"
    ARROWS = "Arrows"
    ESC = "Esc"
    F1_F4 = "F1F4"
    F5_F8 = "F5F8"
    F9_F12 = "F9F12"
    PRINT_SCROLL_PAUSE = "PrintScrollPause"
    NUMBER_ROW = "NumberRow"
    INSERT_HOME_PGUP = "InsertHomePgUp"
    TOP_ROW = "TopRow"
    DELETE_END_PGDN = "DeleteEndPgDn"
    HOME_ROW = "HomeRow"
    SHIFT_ROW = "ShiftRow"
    ARROW_UP = "ArrowUp"
    BOTTOM_ROW = "BottomRow"
    ARROW_LEFT_DOWN_RIGHT = "ArrowLeftDownRight"

    def __add__(self, other: KeyLike) -> KeySet:
        """``Key.W + Key.A`` / ``Key.ESC + KeyGroup.F1_F4`` -> KeySet."""
        return KeySet((*_keys_of(self), *_keys_of(other)))

    def __radd__(self, other: KeyLike) -> KeySet:
        return KeySet((*_keys_of(other), *_keys_of(self)))

    def __sub__(self, other: KeyLike) -> KeySet:
        drop = set(_keys_of(other))
        return KeySet(k for k in _keys_of(self) if k not in drop)

    @property
    def is_area(self) -> bool:
        return _groups()[self.value][0]

    @property
    def layout(self) -> tuple[tuple[Key, ...], ...]:
        """Clusters of adjacent keys (an area has several, a cluster one)."""
        return tuple(tuple(Key(k) for k in c) for c in _groups()[self.value][1])

    @property
    def keys(self) -> tuple[Key, ...]:
        return tuple(k for c in self.layout for k in c)

    @property
    def rect(self) -> Rect:
        return _groups()[self.value][2]

    @staticmethod
    def areas() -> tuple[KeyGroup, ...]:
        return tuple(g for g in KeyGroup if g.is_area)

    @staticmethod
    def clusters() -> tuple[KeyGroup, ...]:
        return tuple(KeyGroup(c) for row in _native.layout_table() for c in row)

    @staticmethod
    def at(x: int, y: int) -> Optional[KeyGroup]:
        """The area under a point of the vendor keyboard image."""
        n = _native.group_at(x, y)
        return None if n is None else KeyGroup(n)


class Effect(Enum):
    """Built-in lighting effect."""

    FIXED_ON = "FixedOn"
    RESPIRE = "Respire"
    RAINBOW = "Rainbow"
    FLASH_AWAY = "FlashAway"
    RAINDROPS = "Raindrops"
    RAINBOW_WHEEL = "RainbowWheel"
    RIPPLES_SHINING = "RipplesShining"
    STARS_TWINKLE = "StarsTwinkle"
    SHADOW_DISAPPEAR = "ShadowDisappear"
    RETRO_SNAKE = "RetroSnake"
    NEON_STREAM = "NeonStream"
    REACTION = "Reaction"
    SINE_WAVE = "SineWave"
    RETINUE_SCANNING = "RetinueScanning"
    ROTATING_WINDMILL = "RotatingWindmill"
    COLORFUL_WATERFALL = "ColorfulWaterfall"
    BLOSSOMING = "Blossoming"
    ROTATING_STORM = "RotatingStorm"
    CUSTOM = "Custom"

    @property
    def id(self) -> int:
        """Hardware effect id."""
        return dict(_native.effect_table())[self.value]


class ControllerAction(Enum):
    """Built-in keyboard control (Bluetooth/receiver, lighting, reset...)."""

    BLUETOOTH_1 = "Bluetooth1"
    BLUETOOTH_2 = "Bluetooth2"
    BLUETOOTH_3 = "Bluetooth3"
    RECEIVER_24G = "Receiver24G"
    BATTERY_LEVEL = "BatteryLevel"
    WINDOWS_KEY_LOCK = "WindowsKeyLock"
    FACTORY_RESET = "FactoryReset"
    BACKLIGHT_EFFECT = "BacklightEffect"
    BACKLIGHT_COLOR = "BacklightColor"
    LIGHTING_POWER_SAVING = "LightingPowerSaving"
    KEYBOARD_LIGHTING = "KeyboardLighting"
    BACKLIGHT_BRIGHTNESS_UP = "BacklightBrightnessUp"
    BACKLIGHT_BRIGHTNESS_DOWN = "BacklightBrightnessDown"
    BACKLIGHT_SPEED_DOWN = "BacklightSpeedDown"
    BACKLIGHT_SPEED_UP = "BacklightSpeedUp"
    LIGHT_BAR_MODE = "LightBarMode"
    LIGHT_BAR_COLOR = "LightBarColor"

    @property
    def label(self) -> str:
        return _controllers()[self.value][0]

    @property
    def factory_key(self) -> Key:
        """The key that triggers this control with FN1 by default."""
        return Key(_controllers()[self.value][1])

    @property
    def sensitive(self) -> bool:
        """Triggering it can disconnect the keyboard or erase settings."""
        return _controllers()[self.value][2]


@cache
def _controllers() -> dict[str, tuple[str, str, bool]]:
    return {v: (label, k, s) for v, label, k, s in _native.controller_table()}
