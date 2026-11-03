/**
 * Typed Node.js API for the ATTACK SHARK X87 keyboard (USB and 2.4G).
 *
 * ```ts
 * import { Keyboard, Effect, Key, KeyAction } from 'attack-shark-x87';
 *
 * const kb = Keyboard.open();
 * await kb.setEffect(Effect.FIXED_ON, { brightness: 4 });
 * await kb.remapKey(0, Key.CAPS_LOCK, KeyAction.key(0x29)); // Esc
 * ```
 */
import { EventEmitter } from 'node:events';
import type * as N from '../native/native';
import { ControllerAction, Effect, Key, KeyGroup } from './enums';

export { ControllerAction, Effect, Key, KeyGroup };

// The compiled Rust module (lib/ask87_ts/src), typed by its generated native.d.ts.
// eslint-disable-next-line @typescript-eslint/no-require-imports
const native = require('../native/ask87.node') as typeof N;

export const VERSION: string = native.version();

// ---- errors -----------------------------------------------------------------

/** Base error for ATTACK SHARK X87 operations. */
export class X87Error extends Error {
  override name = 'X87Error';
}
/** No keyboard (USB or 2.4G receiver) was found. */
export class NotFoundError extends X87Error {
  override name = 'NotFoundError';
}
/** An argument was rejected before talking to the keyboard. */
export class InvalidArgumentError extends X87Error {
  override name = 'InvalidArgumentError';
}
/** The 2.4G keyboard is asleep or off; press any key to wake it and retry. */
export class AsleepError extends X87Error {
  override name = 'AsleepError';
}
/** The keyboard returned an unexpected response, timed out or disconnected. */
export class DeviceError extends X87Error {
  override name = 'DeviceError';
}

/** Map native `[Code] message` errors to the classes above. */
function wrapError(e: unknown): Error {
  if (!(e instanceof Error)) return new X87Error(String(e));
  const m = /^\[(\w+)\] ([\s\S]*)$/.exec(e.message);
  if (!m) return e;
  const [, code, msg] = m;
  const Cls =
    code === 'NotFound'
      ? NotFoundError
      : code === 'InvalidArgument'
        ? InvalidArgumentError
        : code === 'Asleep'
          ? AsleepError
          : DeviceError;
  return new Cls(msg, { cause: e });
}

function call<T>(f: () => T): T {
  try {
    return f();
  } catch (e) {
    throw wrapError(e);
  }
}

async function callAsync<T>(f: () => Promise<T>): Promise<T> {
  try {
    return await call(f);
  } catch (e) {
    throw e instanceof X87Error ? e : wrapError(e);
  }
}

// ---- layout -------------------------------------------------------------------

/** A rectangle in vendor keyboard-image pixels (x2/y2 exclusive). */
export class Rect {
  constructor(
    readonly x1: number,
    readonly y1: number,
    readonly x2: number,
    readonly y2: number,
  ) {}
  get width(): number {
    return this.x2 - this.x1;
  }
  get height(): number {
    return this.y2 - this.y1;
  }
  contains(x: number, y: number): boolean {
    return this.x1 <= x && x < this.x2 && this.y1 <= y && y < this.y2;
  }
}

const rect = (r: N.NativeRect): Rect => new Rect(r.x1, r.y1, r.x2, r.y2);

const IMAGE = native.imageSize();
export const IMAGE_WIDTH: number = IMAGE.width;
export const IMAGE_HEIGHT: number = IMAGE.height;

const keyInfo = new Map(native.keyTable().map((k) => [k.name as Key, k]));
const groupInfo = new Map(native.groupTable().map((g) => [g.name as KeyGroup, g]));
const controllers = new Map(native.controllerTable().map((c) => [c.name as ControllerAction, c]));
const effectIds = new Map(native.effectTable().map((e) => [e.name as Effect, e.id]));

function info<K, V>(map: Map<K, V>, k: K, what: string): V {
  const v = map.get(k);
  if (v === undefined) throw new InvalidArgumentError(`unknown ${what}: ${String(k)}`);
  return v;
}

/** Metadata for a {@link Key}. */
export const Keys = {
  /** Every key, in keyboard order. */
  all: (): Key[] => [...keyInfo.keys()],
  /** Printed legend, e.g. "Caps Lock". */
  label: (k: Key): string => info(keyInfo, k, 'key').label,
  /** Hardware LED / key-matrix index. */
  index: (k: Key): number => info(keyInfo, k, 'key').index,
  rect: (k: Key): Rect => rect(info(keyInfo, k, 'key').rect),
  /** The area (FKey, Main, Navigation, Arrows) containing the key. */
  group: (k: Key): KeyGroup => info(keyInfo, k, 'key').group as KeyGroup,
  cluster: (k: Key): KeyGroup => info(keyInfo, k, 'key').cluster as KeyGroup,
  /** Row 0..5, top to bottom. */
  row: (k: Key): number => info(keyInfo, k, 'key').row,
  left: (k: Key): Key | null => (info(keyInfo, k, 'key').left ?? null) as Key | null,
  right: (k: Key): Key | null => (info(keyInfo, k, 'key').right ?? null) as Key | null,
  /** The key under a point of the vendor keyboard image. */
  at: (x: number, y: number): Key | null => (native.keyAt(x, y) ?? null) as Key | null,
} as const;

/** Metadata for a {@link KeyGroup}. */
export const KeyGroups = {
  all: (): KeyGroup[] => [...groupInfo.keys()],
  /** The four areas, top-left to bottom-right. */
  areas: (): KeyGroup[] => [...groupInfo.values()].filter((g) => g.isArea).map((g) => g.name as KeyGroup),
  /** The physical clusters, in keyboard order. */
  clusters: (): KeyGroup[] => LAYOUT.flat(),
  isArea: (g: KeyGroup): boolean => info(groupInfo, g, 'key group').isArea,
  /** Clusters of adjacent keys (an area has several, a cluster one). */
  layout: (g: KeyGroup): Key[][] => info(groupInfo, g, 'key group').layout.map((c) => c.map((k) => k as Key)),
  keys: (g: KeyGroup): Key[] => KeyGroups.layout(g).flat(),
  rect: (g: KeyGroup): Rect => rect(info(groupInfo, g, 'key group').rect),
  /** The area under a point of the vendor keyboard image. */
  at: (x: number, y: number): KeyGroup | null => (native.groupAt(x, y) ?? null) as KeyGroup | null,
} as const;

const isKey = (x: unknown): x is Key => typeof x === 'string' && keyInfo.has(x as Key);

/**
 * Combine keys and groups into one ordered key list (duplicates dropped);
 * the TypeScript counterpart of Python's `KeyGroup.A + KeyGroup.B`.
 *
 * ```ts
 * const row = keys(KeyGroup.NUMBER_ROW, KeyGroup.INSERT_HOME_PGUP);
 * const wasd = keys(Key.W, Key.A, Key.S, Key.D);
 * ```
 */
export function keys(...parts: readonly (Key | KeyGroup | Iterable<Key>)[]): Key[] {
  const out = new Set<Key>();
  for (const p of parts) {
    if (isKey(p)) out.add(p);
    else if (typeof p === 'string') for (const k of KeyGroups.keys(p as KeyGroup)) out.add(k);
    else for (const k of p) out.add(k);
  }
  return [...out];
}

/** Bounding box of a set of keys. */
export function keysRect(ks: Iterable<Key>): Rect {
  const rs = [...ks].map((k) => Keys.rect(k));
  if (!rs.length) throw new InvalidArgumentError('no keys');
  return new Rect(
    Math.min(...rs.map((r) => r.x1)),
    Math.min(...rs.map((r) => r.y1)),
    Math.max(...rs.map((r) => r.x2)),
    Math.max(...rs.map((r) => r.y2)),
  );
}

/** Physical layout: keyboard -> row -> cluster. */
export const LAYOUT: readonly (readonly KeyGroup[])[] = native
  .layoutTable()
  .map((row) => row.map((c) => c as KeyGroup));

/** Hardware id of an {@link Effect}. */
export const effectId = (e: Effect): number => info(effectIds, e, 'effect');

export interface ControllerInfo {
  readonly label: string;
  /** The key that triggers this control with FN1 by default. */
  readonly factoryKey: Key;
  /** Triggering it can disconnect the keyboard or erase settings. */
  readonly sensitive: boolean;
}

export function controllerInfo(a: ControllerAction): ControllerInfo {
  const c = info(controllers, a, 'controller action');
  return { label: c.label, factoryKey: c.factoryKey as Key, sensitive: c.sensitive };
}

// ---- key actions ----------------------------------------------------------------

/** How a macro bound to a key plays back: a repeat count (1..255) or a mode. */
export type MacroRepeat = number | 'untilReleased' | 'untilAnyKey';

/** What a key does. Build with the {@link KeyAction} helpers. */
export type KeyAction =
  | { readonly kind: 'none' }
  /** `modifiers`: HID mask (bit 0 LCtrl .. bit 7 RGui); `usage`: HID usage (0x04 = A). */
  | { readonly kind: 'key'; readonly modifiers: number; readonly usage: number; readonly usage2: number }
  /** Consumer-control usage (media keys), e.g. 0xE9 volume up. */
  | { readonly kind: 'consumer'; readonly usage: number }
  /** Mouse button 1..5. */
  | { readonly kind: 'mouse'; readonly button: number }
  | { readonly kind: 'fn' }
  | { readonly kind: 'fn2' }
  /** Onboard macro slot (0-based). */
  | { readonly kind: 'macro'; readonly slot: number; readonly repeat: MacroRepeat }
  | { readonly kind: 'controller'; readonly action: ControllerAction }
  | { readonly kind: 'raw'; readonly value: number };

export const KeyAction = {
  none: (): KeyAction => ({ kind: 'none' }),
  key: (usage: number, modifiers = 0, usage2 = 0): KeyAction => ({ kind: 'key', modifiers, usage, usage2 }),
  consumer: (usage: number): KeyAction => ({ kind: 'consumer', usage }),
  mouse: (button: number): KeyAction => ({ kind: 'mouse', button }),
  fn: (): KeyAction => ({ kind: 'fn' }),
  fn2: (): KeyAction => ({ kind: 'fn2' }),
  macro: (slot: number, repeat: MacroRepeat = 1): KeyAction => ({ kind: 'macro', slot, repeat }),
  control: (action: ControllerAction): KeyAction => ({ kind: 'controller', action }),
  raw: (value: number): KeyAction => ({ kind: 'raw', value }),
} as const;

const REPEAT_TO_NATIVE = { untilReleased: 'until_released', untilAnyKey: 'until_any_key' } as const;

function actionToNative(a: KeyAction): N.NativeKeyAction {
  switch (a.kind) {
    case 'key':
      return { kind: a.kind, modifiers: a.modifiers, usage: a.usage, usage2: a.usage2 };
    case 'consumer':
      return { kind: a.kind, usage: a.usage };
    case 'mouse':
      return { kind: a.kind, button: a.button };
    case 'macro':
      return typeof a.repeat === 'number'
        ? { kind: a.kind, slot: a.slot, repeat: 'times', count: a.repeat }
        : { kind: a.kind, slot: a.slot, repeat: REPEAT_TO_NATIVE[a.repeat] };
    case 'controller':
      return { kind: a.kind, action: a.action };
    case 'raw':
      return { kind: a.kind, value: a.value };
    default:
      return { kind: a.kind };
  }
}

function actionFromNative(a: N.NativeKeyAction): KeyAction {
  switch (a.kind) {
    case 'key':
      return KeyAction.key(a.usage ?? 0, a.modifiers ?? 0, a.usage2 ?? 0);
    case 'consumer':
      return KeyAction.consumer(a.usage ?? 0);
    case 'mouse':
      return KeyAction.mouse(a.button ?? 0);
    case 'fn':
      return KeyAction.fn();
    case 'fn2':
      return KeyAction.fn2();
    case 'macro': {
      const repeat: MacroRepeat =
        a.repeat === 'until_released' ? 'untilReleased' : a.repeat === 'until_any_key' ? 'untilAnyKey' : (a.count ?? 1);
      return KeyAction.macro(a.slot ?? 0, repeat);
    }
    case 'controller':
      return KeyAction.control(a.action as ControllerAction);
    case 'raw':
      return KeyAction.raw(a.value ?? 0);
    default:
      return KeyAction.none();
  }
}

/** Encode a key action to its 32-bit key-matrix word (validates it). */
export const encodeAction = (a: KeyAction): number => call(() => native.encodeAction(actionToNative(a)));

/** Decode a 32-bit key-matrix word. */
export const decodeAction = (word: number): KeyAction => actionFromNative(native.decodeAction(word));

// ---- macros -----------------------------------------------------------------------

/**
 * One macro step. `value` is a HID usage (`key`), a button mask
 * (`mouseButton`) or a signed -128..127 delta (`moveX`/`moveY`/`wheel`).
 * `down` only matters for `key` and `mouseButton`. `delayMs` is the wait
 * after this event.
 */
export interface MacroEvent {
  readonly kind: 'key' | 'mouseButton' | 'moveX' | 'moveY' | 'wheel';
  readonly value: number;
  readonly down: boolean;
  readonly delayMs: number;
}

export const MacroEvent = {
  keyDown: (usage: number, delayMs = 0): MacroEvent => ({ kind: 'key', value: usage, down: true, delayMs }),
  keyUp: (usage: number, delayMs = 0): MacroEvent => ({ kind: 'key', value: usage, down: false, delayMs }),
} as const;

/** A named macro (name: 1..30 UTF-16 units; 1..1023 events). */
export interface Macro {
  readonly name: string;
  readonly events: readonly MacroEvent[];
}

const EVENT_KINDS = {
  key: 'key',
  mouseButton: 'mouse_button',
  moveX: 'move_x',
  moveY: 'move_y',
  wheel: 'wheel',
} as const satisfies Record<MacroEvent['kind'], string>;

const EVENT_KINDS_BACK = Object.fromEntries(Object.entries(EVENT_KINDS).map(([k, v]) => [v, k])) as Record<
  string,
  MacroEvent['kind']
>;

const macrosToNative = (macros: readonly Macro[]): N.NativeMacro[] =>
  macros.map((m) => ({
    name: m.name,
    events: m.events.map((e) => ({ kind: EVENT_KINDS[e.kind], value: e.value, down: e.down, delayMs: e.delayMs })),
  }));

const macrosFromNative = (macros: N.NativeMacro[]): Macro[] =>
  macros.map((m) => ({
    name: m.name,
    events: m.events.map((e) => ({
      kind: EVENT_KINDS_BACK[e.kind] ?? 'key',
      value: e.value,
      down: e.down,
      delayMs: e.delayMs,
    })),
  }));

/** Validate a macro bank offline; returns its encoded size (of 4096 bytes). */
export const validateMacros = (macros: readonly Macro[]): number =>
  call(() => native.validateMacros(macrosToNative(macros)));

// ---- device -------------------------------------------------------------------------

/** An (r, g, b) colour, each 0..255. */
export type Rgb = readonly [r: number, g: number, b: number];

export interface Lighting {
  readonly effect: Effect | null;
  readonly brightness: number;
  readonly speed: number;
  readonly color: number;
}

export interface EffectOptions {
  /** 0..4, default 4. */
  readonly brightness?: number;
  /** 0..4, default 2. */
  readonly speed?: number;
  /** Palette index 0..6, or 7 = random (default). */
  readonly color?: number;
}

const toKeyColors = (colors: ReadonlyMap<Key, Rgb> | Partial<Record<Key, Rgb>>): N.NativeKeyColor[] => {
  const entries: [string, Rgb | undefined][] = colors instanceof Map ? [...colors] : Object.entries(colors);
  return entries.flatMap(([key, c]) => (c ? [{ key, r: c[0], g: c[1], b: c[2] }] : []));
};

/** An open X87. I/O methods return Promises and run off the JS thread. */
export class Keyboard {
  readonly #native: N.NativeKeyboard;
  readonly #watchers = new Set<Watcher>();

  private constructor(n: N.NativeKeyboard) {
    this.#native = n;
  }

  /** Open the keyboard: USB first, then the 2.4G receiver (`auto`). Blocks briefly. */
  static open(mode: 'auto' | 'usb' | 'wireless' = 'auto'): Keyboard {
    return new Keyboard(call(() => native.NativeKeyboard.open(mode)));
  }

  /** Like {@link Keyboard.open}, without blocking the event loop. */
  static async openAsync(mode: 'auto' | 'usb' | 'wireless' = 'auto'): Promise<Keyboard> {
    return new Keyboard(await callAsync(() => native.NativeKeyboard.openAsync(mode)));
  }

  /** Release the device and stop watchers from {@link Keyboard.watch}. Safe to call twice; later calls reject. */
  close(): void {
    for (const w of this.#watchers) w.stop();
    this.#watchers.clear();
    this.#native.close();
  }

  /**
   * Connection events (USB cable, 2.4G link / sleep, battery) until
   * {@link Keyboard.close} or `watcher.stop()`:
   *
   * ```ts
   * kb.watch().on('wireless', (e) => console.log(e.connected ? 'awake' : 'asleep'));
   * for await (const e of kb.watch()) console.log(e);
   * ```n   */
  watch(): Watcher {
    const w = new Watcher();
    this.#watchers.add(w);
    w.once('stop', () => this.#watchers.delete(w));
    return w;
  }

  get closed(): boolean {
    return this.#native.closed;
  }

  /** `using kb = Keyboard.open()` closes it at the end of the scope. */
  [Symbol.dispose](): void {
    this.close();
  }

  get connection(): 'usb' | 'wireless' {
    return this.#native.connection === 'usb' ? 'usb' : 'wireless';
  }

  firmwareVersion(): Promise<number> {
    return callAsync(() => this.#native.firmwareVersion());
  }

  /**
   * Battery level over 2.4G. `null` means the keyboard is connected by USB
   * cable (it reports no battery there) or the 2.4G link is down.
   */
  async battery(): Promise<Battery | null> {
    const b = await callAsync(() => this.#native.battery());
    return b ? { percent: b.percent, charging: b.charging } : null;
  }

  /** `true` over USB; over 2.4G whether the keyboard is linked (not asleep / off). */
  isLinked(): Promise<boolean> {
    return callAsync(() => this.#native.isLinked());
  }

  // lighting
  async lighting(): Promise<Lighting> {
    const l = await callAsync(() => this.#native.lighting());
    return { effect: (l.effect ?? null) as Effect | null, brightness: l.brightness, speed: l.speed, color: l.color };
  }

  setEffect(effect: Effect, { brightness = 4, speed = 2, color = 7 }: EffectOptions = {}): Promise<void> {
    return callAsync(() => this.#native.setEffect(effect, brightness, speed, color));
  }

  lightsOff(): Promise<void> {
    return callAsync(() => this.#native.lightsOff());
  }

  async effectPalette(effect: Effect): Promise<Rgb[]> {
    const p = await callAsync(() => this.#native.effectPalette(effect));
    return p.map((c) => [c.r, c.g, c.b] as const);
  }

  setEffectColor(effect: Effect, index: number, [r, g, b]: Rgb): Promise<void> {
    return callAsync(() => this.#native.setEffectColor(effect, index, { r, g, b }));
  }

  async perKeyColors(): Promise<Map<Key, Rgb>> {
    const c = await callAsync(() => this.#native.perKeyColors());
    return new Map(c.map((k) => [k.key as Key, [k.r, k.g, k.b] as const]));
  }

  /** Unlisted keys are set to black. */
  setPerKeyColors(colors: ReadonlyMap<Key, Rgb> | Partial<Record<Key, Rgb>>): Promise<void> {
    return callAsync(() => this.#native.setPerKeyColors(toKeyColors(colors)));
  }

  // keys
  /** Every key's action on a layer (0 = base, 1 = FN, 2 = FN2). */
  async layer(layer: number): Promise<Map<Key, KeyAction>> {
    const l = await callAsync(() => this.#native.layer(layer));
    return new Map(l.map((b) => [b.key as Key, actionFromNative(b.action)]));
  }

  async keyAction(layer: number, key: Key): Promise<KeyAction> {
    return (await this.layer(layer)).get(key) ?? KeyAction.none();
  }

  remapKey(layer: number, key: Key, action: KeyAction): Promise<void> {
    return callAsync(() => this.#native.remapKey(layer, key, actionToNative(action)));
  }

  // macros
  async getMacros(): Promise<Macro[]> {
    return macrosFromNative(await callAsync(() => this.#native.macros()));
  }

  /** Write the whole macro bank in one go; `onProgress` gets 0..100. */
  saveMacros(macros: readonly Macro[], onProgress?: (percent: number) => void): Promise<void> {
    return callAsync(() => this.#native.saveMacros(macrosToNative(macros), onProgress ?? null));
  }

  clearMacros(): Promise<void> {
    return callAsync(() => this.#native.clearMacros());
  }

  // options
  debounce(): Promise<number> {
    return callAsync(() => this.#native.debounce());
  }
  setDebounce(stage: number): Promise<void> {
    return callAsync(() => this.#native.setDebounce(stage));
  }
  tap(): Promise<number | null> {
    return callAsync(() => this.#native.tap());
  }
  setTap(sensitivity: number | null): Promise<void> {
    return callAsync(() => this.#native.setTap(sensitivity));
  }
  /** Seconds before sleep, or null when disabled. */
  sleepTimeout(): Promise<number | null> {
    return callAsync(() => this.#native.sleepTimeout());
  }
  setSleepTimeout(secs: number | null): Promise<void> {
    return callAsync(() => this.#native.setSleepTimeout(secs));
  }

  // realtime
  /** Show a realtime frame (unlisted keys black). `lossy` merges near colours. */
  sendFrame(colors: ReadonlyMap<Key, Rgb> | Partial<Record<Key, Rgb>>, lossy = false): Promise<void> {
    return callAsync(() => this.#native.sendFrame(toKeyColors(colors), lossy));
  }
  stopRealtime(): Promise<void> {
    return callAsync(() => this.#native.stopRealtime());
  }

  /** Destructive: restores factory settings. */
  factoryReset(): Promise<void> {
    return callAsync(() => this.#native.factoryReset());
  }
}

// ---- connection events ---------------------------------------------------------

export interface Battery {
  /** 0..100. */
  readonly percent: number;
  readonly charging: boolean;
}

/** The keyboard was plugged in / unplugged over the USB cable. */
export interface UsbEvent {
  readonly kind: 'usb';
  readonly connected: boolean;
}

/** The 2.4G link went up / down (keyboard woke, slept, switched off...). */
export interface WirelessEvent {
  readonly kind: 'wireless';
  readonly connected: boolean;
  /** Whether the USB receiver is plugged in. */
  readonly receiver: boolean;
}

/** Battery reported by the receiver over 2.4G. */
export interface BatteryEvent {
  readonly kind: 'battery';
  readonly battery: Battery;
}

export type ConnectionEvent = UsbEvent | WirelessEvent | BatteryEvent;

export interface ConnectionStatus {
  /** The keyboard is plugged in by cable. */
  readonly usb: boolean;
  /** The 2.4G receiver is plugged in. */
  readonly receiver: boolean;
  /** The keyboard is linked to the receiver. */
  readonly wireless: boolean;
}

/** Current connection state, without opening the keyboard. */
export const connectionStatus = (): Promise<ConnectionStatus> => callAsync(() => native.connectionStatus());

function toEvent(e: N.NativeConnEvent): ConnectionEvent {
  if (e.kind === 'usb') return { kind: 'usb', connected: e.connected ?? false };
  if (e.kind === 'wireless') return { kind: 'wireless', connected: e.connected ?? false, receiver: e.receiver ?? false };
  return { kind: 'battery', battery: { percent: e.battery?.percent ?? 0, charging: e.battery?.charging ?? false } };
}

interface WatcherEvents {
  event: [ConnectionEvent];
  usb: [UsbEvent];
  wireless: [WirelessEvent];
  battery: [BatteryEvent];
  /** The watcher was stopped. */
  stop: [];
}

/**
 * Watch USB cable / 2.4G receiver / wireless link changes and battery.
 *
 * ```ts
 * const w = new Watcher();
 * w.on('wireless', (e) => console.log(e.connected ? 'awake' : 'asleep', e.receiver));
 * w.on('battery', (e) => console.log(e.battery.percent));
 * // ...
 * w.stop();
 * ```
 *
 * Emits the current state first (a `usb` and a `wireless` event), then every
 * change. While running it keeps the Node process alive; call `stop()`. Also
 * an async iterator: `for await (const e of new Watcher()) ...`.
 */
export class Watcher extends EventEmitter<WatcherEvents> implements AsyncIterable<ConnectionEvent> {
  readonly #native: N.NativeWatcher;

  constructor() {
    super();
    this.#native = call(
      () =>
        new native.NativeWatcher((e: N.NativeConnEvent) => {
          const ev = toEvent(e);
          this.emit('event', ev);
          if (ev.kind === 'usb') this.emit('usb', ev);
          else if (ev.kind === 'wireless') this.emit('wireless', ev);
          else this.emit('battery', ev);
        }),
    );
  }

  /** Stop watching (safe to call twice). */
  stop(): void {
    if (this.#native.running) {
      this.#native.stop();
      this.emit('stop');
    }
  }

  get running(): boolean {
    return this.#native.running;
  }

  [Symbol.dispose](): void {
    this.stop();
  }

  async *[Symbol.asyncIterator](): AsyncIterator<ConnectionEvent> {
    const queue: ConnectionEvent[] = [];
    let wake: (() => void) | null = null;
    const onEvent = (e: ConnectionEvent): void => {
      queue.push(e);
      wake?.();
    };
    const onStop = (): void => wake?.();
    this.on('event', onEvent);
    this.on('stop', onStop);
    try {
      while (this.running || queue.length) {
        const e = queue.shift();
        if (e) yield e;
        else await new Promise<void>((r) => (wake = r));
        wake = null;
      }
    } finally {
      this.off('event', onEvent);
      this.off('stop', onStop);
    }
  }
}