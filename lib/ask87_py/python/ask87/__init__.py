"""Typed Python API for the ATTACK SHARK X87 keyboard (USB and 2.4G).

from ask87 import Keyboard, Effect, Key, KeyAction

with Keyboard.open() as kb:
    kb.set_effect(Effect.FIXED_ON, brightness=4)
    kb.remap_key(0, Key.CAPS_LOCK, KeyAction.key(usage=0x29))  # Esc

or with asyncio:

    kb = await AsyncKeyboard.open()
    await kb.set_effect(Effect.FIXED_ON, brightness=4)
    kb.close()
"""

from __future__ import annotations

import asyncio
from dataclasses import dataclass, field
from typing import Callable, Literal, Optional, Union

from . import _native
from ._enums import ControllerAction, Effect, Key, KeyGroup, KeyLike, KeySet, Rect
from ._native import AsleepError, DeviceError, NotFoundError, X87Error

__all__ = [
    "IMAGE_HEIGHT",
    "IMAGE_WIDTH",
    "LAYOUT",
    "AsleepError",
    "AsyncKeyboard",
    "AsyncWatcher",
    "Battery",
    "BatteryEvent",
    "ConnectionStatus",
    "ControllerAction",
    "DeviceError",
    "Effect",
    "Event",
    "Key",
    "KeyAction",
    "KeyGroup",
    "KeyLike",
    "KeySet",
    "Keyboard",
    "Lighting",
    "Macro",
    "MacroEvent",
    "NotFoundError",
    "Rect",
    "Rgb",
    "UsbEvent",
    "Watcher",
    "WirelessEvent",
    "X87Error",
    "__version__",
    "connection_status",
    "decode_action",
    "encode_action",
    "validate_macros",
]

__version__: str = _native.version()

Rgb = tuple[int, int, int]
"""An (r, g, b) colour, each 0..255."""

IMAGE_WIDTH, IMAGE_HEIGHT = _native.image_size()


LAYOUT: tuple[tuple[KeyGroup, ...], ...] = tuple(tuple(KeyGroup(c) for c in row) for row in _native.layout_table())
"""Physical layout: keyboard -> row -> cluster (each a :class:`KeyGroup`)."""

# ---- key actions ------------------------------------------------------------

ActionKind = Literal["none", "key", "consumer", "mouse", "fn", "fn2", "macro", "controller", "raw"]
MacroRepeat = Union[int, Literal["until_released", "until_any_key"]]


@dataclass(frozen=True)
class KeyAction:
    """What a key does. Build with the classmethods, e.g. ``KeyAction.key(0x04)``.

    ``modifiers`` is the HID modifier mask (bit 0 LCtrl .. bit 7 RGui); ``usage``
    is a HID keyboard usage (0x04 = A) or, for ``consumer``, a consumer usage.
    """

    kind: ActionKind
    modifiers: int = 0
    usage: int = 0
    usage2: int = 0
    button: int = 0
    slot: int = 0
    repeat: MacroRepeat = 1
    controller: Optional[ControllerAction] = None
    value: int = 0

    @classmethod
    def none(cls) -> KeyAction:
        return cls("none")

    @classmethod
    def key(cls, usage: int, modifiers: int = 0, usage2: int = 0) -> KeyAction:
        return cls("key", modifiers=modifiers, usage=usage, usage2=usage2)

    @classmethod
    def consumer(cls, usage: int) -> KeyAction:
        return cls("consumer", usage=usage)

    @classmethod
    def mouse(cls, button: int) -> KeyAction:
        return cls("mouse", button=button)

    @classmethod
    def fn(cls) -> KeyAction:
        return cls("fn")

    @classmethod
    def fn2(cls) -> KeyAction:
        return cls("fn2")

    @classmethod
    def macro(cls, slot: int, repeat: MacroRepeat = 1) -> KeyAction:
        return cls("macro", slot=slot, repeat=repeat)

    @classmethod
    def control(cls, action: ControllerAction) -> KeyAction:
        return cls("controller", controller=action)

    @classmethod
    def raw(cls, value: int) -> KeyAction:
        return cls("raw", value=value)

    def _to_native(self) -> dict[str, object]:
        k = self.kind
        if k == "key":
            return {"kind": k, "modifiers": self.modifiers, "usage": self.usage, "usage2": self.usage2}
        if k == "consumer":
            return {"kind": k, "usage": self.usage}
        if k == "mouse":
            return {"kind": k, "button": self.button}
        if k == "macro":
            if isinstance(self.repeat, int):
                return {"kind": k, "slot": self.slot, "repeat": "times", "count": self.repeat}
            return {"kind": k, "slot": self.slot, "repeat": self.repeat}
        if k == "controller":
            if self.controller is None:
                raise ValueError("controller action needs `controller`")
            return {"kind": k, "action": ControllerAction(self.controller).value}
        if k == "raw":
            return {"kind": k, "value": self.value}
        return {"kind": k}

    @classmethod
    def _from_native(cls, d: dict[str, object]) -> KeyAction:
        def i(name: str) -> int:
            v = d[name]
            assert isinstance(v, int)
            return v

        k = d["kind"]
        if k == "key":
            return cls.key(i("usage"), i("modifiers"), i("usage2"))
        if k == "consumer":
            return cls.consumer(i("usage"))
        if k == "mouse":
            return cls.mouse(i("button"))
        if k == "macro":
            r = d["repeat"]
            repeat: MacroRepeat = (
                i("count") if r == "times" else ("until_released" if r == "until_released" else "until_any_key")
            )
            return cls.macro(i("slot"), repeat)
        if k == "controller":
            return cls.control(ControllerAction(str(d["action"])))
        if k == "raw":
            return cls.raw(i("value"))
        if k == "fn":
            return cls.fn()
        if k == "fn2":
            return cls.fn2()
        return cls.none()


def encode_action(action: KeyAction) -> int:
    """Encode a key action to its 32-bit key-matrix word (validates it)."""
    return _native.encode_action(action._to_native())


def decode_action(word: int) -> KeyAction:
    """Decode a 32-bit key-matrix word."""
    return KeyAction._from_native(_native.decode_action(word))


# ---- macros -----------------------------------------------------------------

MacroEventKind = Literal["key", "mouse_button", "move_x", "move_y", "wheel"]


@dataclass(frozen=True)
class MacroEvent:
    """One macro step. ``value`` is a HID usage (key), button mask
    (mouse_button) or a signed -128..127 delta (move_x/move_y/wheel).
    ``down`` only matters for key and mouse_button. ``delay_ms`` is the wait
    after this event."""

    kind: MacroEventKind
    value: int
    down: bool = False
    delay_ms: int = 0

    @classmethod
    def key_down(cls, usage: int, delay_ms: int = 0) -> MacroEvent:
        return cls("key", usage, True, delay_ms)

    @classmethod
    def key_up(cls, usage: int, delay_ms: int = 0) -> MacroEvent:
        return cls("key", usage, False, delay_ms)


@dataclass(frozen=True)
class Macro:
    """A named macro (name: 1..30 UTF-16 units; 1..1023 events)."""

    name: str
    events: list[MacroEvent] = field(default_factory=list)


def _macros_to_native(macros: list[Macro]) -> list[tuple[str, list[tuple[str, int, bool, int]]]]:
    return [(m.name, [(e.kind, e.value, e.down, e.delay_ms) for e in m.events]) for m in macros]


def _macros_from_native(raw: list[tuple[str, list[tuple[str, int, bool, int]]]]) -> list[Macro]:
    kinds: dict[str, MacroEventKind] = {
        "key": "key",
        "mouse_button": "mouse_button",
        "move_x": "move_x",
        "move_y": "move_y",
        "wheel": "wheel",
    }
    return [Macro(name, [MacroEvent(kinds[k], v, d, ms) for k, v, d, ms in evs]) for name, evs in raw]


def validate_macros(macros: list[Macro]) -> int:
    """Validate a macro bank offline; returns its encoded size (of 4096 bytes)."""
    return _native.validate_macros(_macros_to_native(macros))


# ---- device -----------------------------------------------------------------


@dataclass(frozen=True)
class Lighting:
    effect: Optional[Effect]
    brightness: int
    speed: int
    color: int


class Keyboard:
    """An open X87. Calls block on HID I/O (the GIL is released meanwhile)."""

    def __init__(self, native: _native.NativeKeyboard) -> None:
        """Use :meth:`Keyboard.open`."""
        self._native = native
        self._watchers: list[Watcher] = []

    @classmethod
    def open(cls, mode: Literal["auto", "usb", "wireless"] = "auto") -> Keyboard:
        """Open the keyboard: USB first, then the 2.4G receiver (``auto``)."""
        return cls(_native.NativeKeyboard.open(mode))

    def __enter__(self) -> Keyboard:
        return self

    def __exit__(self, *exc: object) -> None:
        self.close()

    def close(self) -> None:
        """Release the device and stop watchers started with :meth:`watch`
        (also done when garbage collected). Safe to call twice; later calls
        raise ValueError."""
        for w in self._watchers:
            w.stop()
        self._watchers.clear()
        self._native.close()

    def watch(self, on_event: Callable[[Event], object]) -> Watcher:
        """Get connection events (USB cable, 2.4G link / sleep, battery) on a
        background thread until :meth:`close` or ``Watcher.stop()``.

            kb.watch(lambda e: print(e))
        """
        w = Watcher(on_event)
        self._watchers.append(w)
        return w

    @property
    def closed(self) -> bool:
        return self._native.closed

    @property
    def connection(self) -> Literal["usb", "wireless"]:
        return "usb" if self._native.connection == "usb" else "wireless"

    def firmware_version(self) -> int:
        return self._native.firmware_version()

    def battery(self) -> Optional[Battery]:
        """Battery level over 2.4G. `None` means the keyboard is connected by USB
        cable (it reports no battery there) or the 2.4G link is down."""
        b = self._native.battery()
        return None if b is None else Battery(b[0], b[1])

    def is_linked(self) -> bool:
        """`True` over USB; over 2.4G whether the keyboard is linked (not asleep / off)."""
        return self._native.is_linked()

    # lighting
    def lighting(self) -> Lighting:
        e, b, s, c = self._native.lighting()
        return Lighting(None if e is None else Effect(e), b, s, c)

    def set_effect(self, effect: Effect, brightness: int = 4, speed: int = 2, color: int = 7) -> None:
        """Brightness/speed 0..4; color is a palette index 0..6 or 7 = random."""
        self._native.set_effect(Effect(effect).value, brightness, speed, color)

    def lights_off(self) -> None:
        self._native.lights_off()

    def effect_palette(self, effect: Effect) -> list[Rgb]:
        return self._native.effect_palette(Effect(effect).value)

    def set_effect_color(self, effect: Effect, index: int, color: Rgb) -> None:
        self._native.set_effect_color(Effect(effect).value, index, color)

    def per_key_colors(self) -> dict[Key, Rgb]:
        return {Key(k): c for k, c in self._native.per_key_colors()}

    def set_per_key_colors(self, colors: dict[Key, Rgb]) -> None:
        """Unlisted keys are set to black."""
        self._native.set_per_key_colors([(Key(k).value, c) for k, c in colors.items()])

    # keys
    def key_action(self, layer: int, key: Key) -> KeyAction:
        return KeyAction._from_native(self._native.key_action(layer, Key(key).value))

    def layer(self, layer: int) -> dict[Key, KeyAction]:
        """Every key's action on a layer (0 = base, 1 = FN, 2 = FN2)."""
        return {Key(k): KeyAction._from_native(a) for k, a in self._native.layer(layer)}

    def remap_key(self, layer: int, key: Key, action: KeyAction) -> None:
        self._native.remap_key(layer, Key(key).value, action._to_native())

    # macros
    def get_macros(self) -> list[Macro]:
        return _macros_from_native(self._native.macros())

    def save_macros(self, macros: list[Macro], on_progress: Optional[Callable[[int], object]] = None) -> None:
        """Write the whole macro bank in one go; ``on_progress`` gets 0..100."""
        self._native.save_macros(_macros_to_native(macros), on_progress)

    def clear_macros(self) -> None:
        self._native.clear_macros()

    # options
    def debounce(self) -> int:
        return self._native.debounce()

    def set_debounce(self, stage: int) -> None:
        self._native.set_debounce(stage)

    def tap(self) -> Optional[int]:
        return self._native.tap()

    def set_tap(self, sensitivity: Optional[int]) -> None:
        self._native.set_tap(sensitivity)

    def sleep_timeout(self) -> Optional[int]:
        """Seconds before sleep, or None when disabled."""
        return self._native.sleep_timeout()

    def set_sleep_timeout(self, secs: Optional[int]) -> None:
        self._native.set_sleep_timeout(secs)

    # realtime
    def send_frame(self, colors: dict[Key, Rgb], lossy: bool = False) -> None:
        """Show a realtime frame (unlisted keys black). ``lossy`` merges near colours."""
        self._native.send_frame([(Key(k).value, c) for k, c in colors.items()], lossy)

    def stop_realtime(self) -> None:
        self._native.stop_realtime()

    def factory_reset(self) -> None:
        """Destructive: restores factory settings."""
        self._native.factory_reset()


class AsyncKeyboard:
    """asyncio version of :class:`Keyboard`.

    Each call runs the blocking HID I/O in a worker thread (``asyncio.to_thread``);
    the native layer releases the GIL, so the event loop keeps running. Calls on
    one keyboard are serialized.

        kb = await AsyncKeyboard.open()
        await kb.set_effect(Effect.FIXED_ON)
        kb.close()  # or: async with await AsyncKeyboard.open() as kb: ...
    """

    def __init__(self, keyboard: Keyboard) -> None:
        """Use :meth:`AsyncKeyboard.open`, or wrap an open :class:`Keyboard`."""
        self.sync = keyboard
        self._watchers: list[AsyncWatcher] = []

    @classmethod
    async def open(cls, mode: Literal["auto", "usb", "wireless"] = "auto") -> AsyncKeyboard:
        """Open the keyboard: USB first, then the 2.4G receiver (``auto``)."""
        return cls(await asyncio.to_thread(Keyboard.open, mode))

    async def __aenter__(self) -> AsyncKeyboard:
        return self

    async def __aexit__(self, *exc: object) -> None:
        self.close()

    def close(self) -> None:
        """Release the device and stop watchers from :meth:`watch`. Not async:
        it only drops the handle (waiting for a call still running in a worker
        thread). Safe to call twice."""
        for w in self._watchers:
            w.stop()
        self._watchers.clear()
        self.sync.close()

    def watch(self) -> AsyncWatcher:
        """Connection events as an async iterator, stopped by :meth:`close`.

        async for event in kb.watch():
            print(event)
        """
        w = AsyncWatcher()
        self._watchers.append(w)
        return w

    @property
    def closed(self) -> bool:
        return self.sync.closed

    @property
    def connection(self) -> Literal["usb", "wireless"]:
        return self.sync.connection

    async def firmware_version(self) -> int:
        return await asyncio.to_thread(self.sync.firmware_version)

    async def battery(self) -> Optional[Battery]:
        """See :meth:`Keyboard.battery`."""
        return await asyncio.to_thread(self.sync.battery)

    async def is_linked(self) -> bool:
        return await asyncio.to_thread(self.sync.is_linked)

    # lighting
    async def lighting(self) -> Lighting:
        return await asyncio.to_thread(self.sync.lighting)

    async def set_effect(self, effect: Effect, brightness: int = 4, speed: int = 2, color: int = 7) -> None:
        """Brightness/speed 0..4; color is a palette index 0..6 or 7 = random."""
        await asyncio.to_thread(self.sync.set_effect, effect, brightness, speed, color)

    async def lights_off(self) -> None:
        await asyncio.to_thread(self.sync.lights_off)

    async def effect_palette(self, effect: Effect) -> list[Rgb]:
        return await asyncio.to_thread(self.sync.effect_palette, effect)

    async def set_effect_color(self, effect: Effect, index: int, color: Rgb) -> None:
        await asyncio.to_thread(self.sync.set_effect_color, effect, index, color)

    async def per_key_colors(self) -> dict[Key, Rgb]:
        return await asyncio.to_thread(self.sync.per_key_colors)

    async def set_per_key_colors(self, colors: dict[Key, Rgb]) -> None:
        """Unlisted keys are set to black."""
        await asyncio.to_thread(self.sync.set_per_key_colors, colors)

    # keys
    async def key_action(self, layer: int, key: Key) -> KeyAction:
        return await asyncio.to_thread(self.sync.key_action, layer, key)

    async def layer(self, layer: int) -> dict[Key, KeyAction]:
        """Every key's action on a layer (0 = base, 1 = FN, 2 = FN2)."""
        return await asyncio.to_thread(self.sync.layer, layer)

    async def remap_key(self, layer: int, key: Key, action: KeyAction) -> None:
        await asyncio.to_thread(self.sync.remap_key, layer, key, action)

    # macros
    async def get_macros(self) -> list[Macro]:
        return await asyncio.to_thread(self.sync.get_macros)

    async def save_macros(self, macros: list[Macro], on_progress: Optional[Callable[[int], object]] = None) -> None:
        """Write the whole macro bank in one go; ``on_progress`` gets 0..100 and
        is called on the event loop thread."""
        if on_progress is None:
            await asyncio.to_thread(self.sync.save_macros, macros)
            return
        loop = asyncio.get_running_loop()
        cb = on_progress

        def progress(p: int) -> None:
            loop.call_soon_threadsafe(cb, p)

        await asyncio.to_thread(self.sync.save_macros, macros, progress)

    async def clear_macros(self) -> None:
        await asyncio.to_thread(self.sync.clear_macros)

    # options
    async def debounce(self) -> int:
        return await asyncio.to_thread(self.sync.debounce)

    async def set_debounce(self, stage: int) -> None:
        await asyncio.to_thread(self.sync.set_debounce, stage)

    async def tap(self) -> Optional[int]:
        return await asyncio.to_thread(self.sync.tap)

    async def set_tap(self, sensitivity: Optional[int]) -> None:
        await asyncio.to_thread(self.sync.set_tap, sensitivity)

    async def sleep_timeout(self) -> Optional[int]:
        """Seconds before sleep, or None when disabled."""
        return await asyncio.to_thread(self.sync.sleep_timeout)

    async def set_sleep_timeout(self, secs: Optional[int]) -> None:
        await asyncio.to_thread(self.sync.set_sleep_timeout, secs)

    # realtime
    async def send_frame(self, colors: dict[Key, Rgb], lossy: bool = False) -> None:
        """Show a realtime frame (unlisted keys black). ``lossy`` merges near colours."""
        await asyncio.to_thread(self.sync.send_frame, colors, lossy)

    async def stop_realtime(self) -> None:
        await asyncio.to_thread(self.sync.stop_realtime)

    async def factory_reset(self) -> None:
        """Destructive: restores factory settings."""
        await asyncio.to_thread(self.sync.factory_reset)


# ---- connection events -------------------------------------------------------


@dataclass(frozen=True)
class Battery:
    percent: int
    """0..100."""
    charging: bool


@dataclass(frozen=True)
class UsbEvent:
    """The keyboard was plugged in / unplugged over the USB cable."""

    connected: bool


@dataclass(frozen=True)
class WirelessEvent:
    """The 2.4G link went up / down (keyboard woke, slept, switched off...).
    ``receiver`` says whether the USB receiver is plugged in."""

    connected: bool
    receiver: bool


@dataclass(frozen=True)
class BatteryEvent:
    """Battery reported by the receiver over 2.4G."""

    battery: Battery


Event = Union[UsbEvent, WirelessEvent, BatteryEvent]


def _event(raw: tuple[str, int, Optional[bool], Optional[bool]]) -> Event:
    kind, value, flag, _ = raw
    if kind == "usb":
        return UsbEvent(bool(value))
    if kind == "wireless":
        return WirelessEvent(bool(value), bool(flag))
    return BatteryEvent(Battery(value, bool(flag)))


@dataclass(frozen=True)
class ConnectionStatus:
    usb: bool
    """The keyboard is plugged in by cable."""
    receiver: bool
    """The 2.4G receiver is plugged in."""
    wireless: bool
    """The keyboard is linked to the receiver."""


def connection_status() -> ConnectionStatus:
    """Current connection state, without opening the keyboard."""
    return ConnectionStatus(*_native.connection_status())


class Watcher:
    """Watch USB cable / 2.4G receiver / wireless link changes and battery.

        w = Watcher(lambda e: print(e))
        ...
        w.stop()

    The callback runs on a background thread. It gets the current state first
    (a ``UsbEvent`` and a ``WirelessEvent``), then every change.
    """

    def __init__(self, on_event: Callable[[Event], object]) -> None:
        self._native = _native.NativeWatcher(lambda raw: on_event(_event(raw)))

    def stop(self) -> None:
        """Stop watching (safe to call twice)."""
        self._native.stop()

    @property
    def running(self) -> bool:
        return self._native.running

    def __enter__(self) -> Watcher:
        return self

    def __exit__(self, *exc: object) -> None:
        self.stop()


class AsyncWatcher:
    """asyncio version of :class:`Watcher`: iterate events with ``async for``.

    w = AsyncWatcher()
    async for event in w:
        print(event)
    # elsewhere: w.stop()
    """

    def __init__(self) -> None:
        loop = asyncio.get_running_loop()
        self._queue = asyncio.Queue[Optional[Event]]()
        queue = self._queue
        self._watcher = Watcher(lambda e: loop.call_soon_threadsafe(queue.put_nowait, e))
        self._loop = loop

    def stop(self) -> None:
        """Stop watching; a running ``async for`` ends."""
        if self._watcher.running:
            self._watcher.stop()
            self._loop.call_soon_threadsafe(self._queue.put_nowait, None)

    @property
    def running(self) -> bool:
        return self._watcher.running

    def __aiter__(self) -> AsyncWatcher:
        return self

    async def __anext__(self) -> Event:
        e = await self._queue.get()
        if e is None:
            raise StopAsyncIteration
        return e

    async def __aenter__(self) -> AsyncWatcher:
        return self

    async def __aexit__(self, *exc: object) -> None:
        self.stop()
