"""Enums mirroring the Rust ones. Members are the Rust variant names;
metadata comes from the native module. tests/test_enums.py checks they stay in
sync with `_native`."""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from functools import cache
from typing import Optional

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


class Key(Enum):
    """A physical key on the X87."""

    Esc = "Esc"
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
    PrtSc = "PrtSc"
    ScrLock = "ScrLock"
    Pause = "Pause"
    Grave = "Grave"
    Digit1 = "Digit1"
    Digit2 = "Digit2"
    Digit3 = "Digit3"
    Digit4 = "Digit4"
    Digit5 = "Digit5"
    Digit6 = "Digit6"
    Digit7 = "Digit7"
    Digit8 = "Digit8"
    Digit9 = "Digit9"
    Digit0 = "Digit0"
    Minus = "Minus"
    Equal = "Equal"
    Backspace = "Backspace"
    Insert = "Insert"
    Home = "Home"
    PgUp = "PgUp"
    Tab = "Tab"
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
    BracketLeft = "BracketLeft"
    BracketRight = "BracketRight"
    Backslash = "Backslash"
    Delete = "Delete"
    End = "End"
    PgDn = "PgDn"
    CapsLock = "CapsLock"
    A = "A"
    S = "S"
    D = "D"
    F = "F"
    G = "G"
    H = "H"
    J = "J"
    K = "K"
    L = "L"
    Semicolon = "Semicolon"
    Quote = "Quote"
    Enter = "Enter"
    LShift = "LShift"
    Z = "Z"
    X = "X"
    C = "C"
    V = "V"
    B = "B"
    N = "N"
    M = "M"
    Comma = "Comma"
    Period = "Period"
    Slash = "Slash"
    RShift = "RShift"
    Up = "Up"
    LCtrl = "LCtrl"
    LWin = "LWin"
    LAlt = "LAlt"
    Space = "Space"
    RAlt = "RAlt"
    Fn = "Fn"
    App = "App"
    RCtrl = "RCtrl"
    Left = "Left"
    Down = "Down"
    Right = "Right"

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

    FKey = "FKey"
    Main = "Main"
    Navigation = "Navigation"
    Arrows = "Arrows"
    Esc = "Esc"
    F1F4 = "F1F4"
    F5F8 = "F5F8"
    F9F12 = "F9F12"
    PrintScrollPause = "PrintScrollPause"
    NumberRow = "NumberRow"
    InsertHomePgUp = "InsertHomePgUp"
    TopRow = "TopRow"
    DeleteEndPgDn = "DeleteEndPgDn"
    HomeRow = "HomeRow"
    ShiftRow = "ShiftRow"
    ArrowUp = "ArrowUp"
    BottomRow = "BottomRow"
    ArrowLeftDownRight = "ArrowLeftDownRight"

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

    FixedOn = "FixedOn"
    Respire = "Respire"
    Rainbow = "Rainbow"
    FlashAway = "FlashAway"
    Raindrops = "Raindrops"
    RainbowWheel = "RainbowWheel"
    RipplesShining = "RipplesShining"
    StarsTwinkle = "StarsTwinkle"
    ShadowDisappear = "ShadowDisappear"
    RetroSnake = "RetroSnake"
    NeonStream = "NeonStream"
    Reaction = "Reaction"
    SineWave = "SineWave"
    RetinueScanning = "RetinueScanning"
    RotatingWindmill = "RotatingWindmill"
    ColorfulWaterfall = "ColorfulWaterfall"
    Blossoming = "Blossoming"
    RotatingStorm = "RotatingStorm"
    Custom = "Custom"

    @property
    def id(self) -> int:
        """Hardware effect id."""
        return dict(_native.effect_table())[self.value]


class ControllerAction(Enum):
    """Built-in keyboard control (Bluetooth/receiver, lighting, reset...)."""

    Bluetooth1 = "Bluetooth1"
    Bluetooth2 = "Bluetooth2"
    Bluetooth3 = "Bluetooth3"
    Receiver24G = "Receiver24G"
    BatteryLevel = "BatteryLevel"
    WindowsKeyLock = "WindowsKeyLock"
    FactoryReset = "FactoryReset"
    BacklightEffect = "BacklightEffect"
    BacklightColor = "BacklightColor"
    LightingPowerSaving = "LightingPowerSaving"
    KeyboardLighting = "KeyboardLighting"
    BacklightBrightnessUp = "BacklightBrightnessUp"
    BacklightBrightnessDown = "BacklightBrightnessDown"
    BacklightSpeedDown = "BacklightSpeedDown"
    BacklightSpeedUp = "BacklightSpeedUp"
    LightBarMode = "LightBarMode"
    LightBarColor = "LightBarColor"

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
