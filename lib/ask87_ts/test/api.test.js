// Offline tests (no keyboard needed): `npm test`.
const test = require('node:test');
const assert = require('node:assert/strict');
const x = require('../../..');
const native = require('../native/ask87.node');

test('enums match the Rust ones', () => {
  assert.deepEqual(Object.values(x.Key), native.keyTable().map((k) => k.name));
  assert.deepEqual(Object.values(x.KeyGroup), native.groupTable().map((g) => g.name));
  assert.deepEqual(Object.values(x.Effect), native.effectTable().map((e) => e.name));
  assert.deepEqual(Object.values(x.ControllerAction), native.controllerTable().map((c) => c.name));
});

test('layout', () => {
  assert.deepEqual(x.LAYOUT.flat().flatMap((g) => x.KeyGroups.keys(g)), Object.values(x.Key));
  assert.equal(x.Keys.group(x.Key.CAPS_LOCK), x.KeyGroup.MAIN);
  assert.equal(x.Keys.left(x.Key.S), x.Key.A);
  const r = x.Keys.rect(x.Key.F6);
  assert.equal(x.Keys.at(r.x1 + 1, r.y1 + 1), x.Key.F6);
  assert.equal(x.controllerInfo(x.ControllerAction.FACTORY_RESET).factoryKey, x.Key.SPACE);
});

test('key actions round-trip', () => {
  for (const a of [
    x.KeyAction.none(),
    x.KeyAction.key(0x04, 0x02),
    x.KeyAction.consumer(0xe9),
    x.KeyAction.mouse(1),
    x.KeyAction.fn(),
    x.KeyAction.macro(3, 'untilReleased'),
    x.KeyAction.macro(0, 5),
    x.KeyAction.control(x.ControllerAction.BLUETOOTH_1),
  ]) {
    assert.deepEqual(x.decodeAction(x.encodeAction(a)), a);
  }
});

test('validation errors are typed', () => {
  assert.throws(() => x.encodeAction(x.KeyAction.macro(99)), x.InvalidArgumentError);
  assert.ok(x.validateMacros([{ name: 'hi', events: [x.MacroEvent.keyDown(4, 10), x.MacroEvent.keyUp(4)] }]) > 0);
  assert.throws(() => x.validateMacros([{ name: '', events: [] }]), x.InvalidArgumentError);
});

test('async open rejects with a typed error', async () => {
  await assert.rejects(x.Keyboard.openAsync('bogus'), x.InvalidArgumentError);
});

test('watcher api surface', () => {
  assert.equal(typeof x.Watcher, 'function');
  assert.equal(typeof x.connectionStatus, 'function');
  assert.equal(typeof x.Keyboard.prototype.battery, 'function');
  assert.equal(typeof x.Keyboard.prototype.isLinked, 'function');
});

test('keyboard.watch exists', () => {
  assert.equal(typeof x.Keyboard.prototype.watch, 'function');
});

test('UPPER_SNAKE enum members and keys() combining', () => {
  assert.equal(x.Key.CAPS_LOCK, 'CapsLock');
  assert.equal(x.KeyGroup.F1_F4, 'F1F4');
  const row = x.keys(x.KeyGroup.NUMBER_ROW, x.KeyGroup.INSERT_HOME_PGUP);
  assert.deepEqual(row.slice(-3), [x.Key.INSERT, x.Key.HOME, x.Key.PGUP]);
  assert.deepEqual(x.keys(x.Key.ESC, x.KeyGroup.F1_F4), ['Esc', 'F1', 'F2', 'F3', 'F4']);
  assert.deepEqual(x.keys(x.KeyGroup.ESC, x.Key.ESC), ['Esc']);
  assert.deepEqual(x.keys(...x.LAYOUT.flat()), Object.values(x.Key));
  assert.ok(x.keysRect(row).width > 0);
});
