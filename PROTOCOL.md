# ATTACK SHARK X87 — HID protocol

Reverse engineered from `OemDrv.exe` (vendor tool, x86 MFC app) with Ghidra.
The keyboard is driven by class `CDevG5KB` (vtable `0x006128fc`).
These functions are renamed in the Ghidra project `ATTACK SHARK X87`:

| address    | name                       |
|------------|----------------------------|
| 0x0040b660 | EnumerateHidDevices        |
| 0x0048c630 | G5KB_FindHidDevice         |
| 0x0048c9e0 | G5KB_AccessData            |
| 0x0048da50 | G5KB_GetPassword           |
| 0x0048cc20 | G5KB_SetKeyMatrix          |
| 0x0048cc60 | G5KB_GetKeyMatrix          |
| 0x0048ce70 | G5KB_SetMacro              |
| 0x0048cec0 | G5KB_GetMacro              |
| 0x0048d0c0 | G5KB_SetLedConfig          |
| 0x0048d110 | G5KB_GetLedConfig          |
| 0x0048d310 | G5KB_SetGameColorGroup     |
| 0x0048d370 | G5KB_GetGameColorGroup     |
| 0x0048d760 | G5KB_SetLedRgbTab          |
| 0x0048d7b0 | G5KB_GetLedRgbTab          |
| 0x0048d9b0 | G5KB_ResetDevice           |
| 0x0048dc80 | G5KB_ConvertKeyCode        |
| 0x0048ec70 | G5KB_BuildLedConfigBlock   |
| 0x0048e730 | G5KB_FillKeyMatrix         |
| 0x0048eff0 | G5KB_ApplySetting          |

## Device

From `Dev/kb/KB.ini`:

* Wired: `VID 0x258A PID 0x010C`
* 2.4G dongle: `VID 0x3554 PID 0xFA09`
* Expected identification (`Psd`): `03 00 00 00 01 A7`

`FindHidDevice` opens the collection with **usage page 0xFF00, usage 1,
feature report length 0x208**. Everything goes through
`HidD_SetFeature` / `HidD_GetFeature`; there are no interrupt transfers.

Over the 2.4G dongle a different transport is used (class `CDev3632`),
described in the "2.4G dongle" section below. The data blocks are the same.

## Packet

520 bytes:

```
[0]   report id   = 0x06  (device type 0x18; other G5KB models use 0x09)
[1]   command     (| 0x80 = read)
[2]   sub index   (layer / colour group)
[3]   0
[4]   total chunks
[5]   chunk index
[6:8] chunk payload length, LE u16 (<= 512)
[8:]  payload
```

* **Write**: one SetFeature per 512-byte chunk, 20 ms apart, up to 3 retries 70 ms apart.
* **Read**: SetFeature with `cmd|0x80` (empty payload, length filled in), wait
  20 ms, then GetFeature. The reply has the same header and the data at `[8:]`.

## Commands

| cmd  | name            | sub         | size            |
|------|-----------------|-------------|-----------------|
| 0x02 | Password (read) | 1           | 6               |
| 0x03 | Key matrix      | layer 0..3  | MatrixLen*4=512 |
| 0x04 | LED config      | 0           | 128             |
| 0x05 | Macros          | 0           | variable        |
| 0x06 | Per-key colours | colour group| MatrixLen*3=384 |
| 0x08 | Music realtime  | 0           | n               |
| 0x0A | LED RGB table   | 0           | 0x1A4 (0x200 with CRC) |
| 0x0B | Screen param    | 0           | n               |
| 0x0D | GIF param       | 0           | 5               |
| 0x0E | Realtime 2      | 0           | n               |
| 0x11 | Reset           | 0           | 1               |

## LED config (cmd 0x04, 128 bytes)

Read first, change, write back (the driver retries if the marker is missing).

| off          | meaning                                   |
|--------------|-------------------------------------------|
| 0x09         | 1 = per-key ("Gaoshou") group active      |
| 0x0A         | effect id (`LedOptN` column 1)            |
| 0x0B         | colour group start                        |
| 0x38,0x39    | `FF FF`                                   |
| 0x3A + 2i    | brightness of effect slot i (0..4)        |
| 0x3B + 2i    | `speed<<4 | colour` (colour 0..6, 7 = random) |
| 0x7E,0x7F    | `5A A5` marker                            |

19 slots, slot `i` = `LedOpt{i+1}`. Effect ids: 1 Fixed, 2 Respire,
3 Rainbow, 4 Flash away, 5 Raindrops, 6 Rainbow wheel, 7 Ripples, 8 Stars,
9 Shadow, 10 Snake, 11 Neon, 12 Reaction, 13 Sine wave, 14 Scanning,
15 Windmill, 16 Waterfall, 17 Blossoming, 18 Storm, 21 Custom/per-key.

## RGB table (cmd 0x0A)

20 rows × 7 colours × RGB (21 bytes/row); row `slot+1` = effect slot.
With `CRC=1` (the X87 setting) it is padded to 512 bytes with `5A A5` at 0x1FA.

## Per-key colours (cmd 0x06)

Three planes of 126 bytes (R, G, B; `GameRGBIndex=0,1,2`), indexed by the key
index from `KB.ini` (last column of `Kn=`).

## Key matrix (cmd 0x03)

128 × u32 LE per layer, indexed by the key index. Entry bytes `[b0 b1 b2 b3]`:

* `00 mods k2 k1` — keyboard key (HID usage k1, optional k2, modifier bitmask)
* `02 00 hi lo`   — consumer usage (e.g. `E9` = Vol+)
* `01 n 01 00`    — mouse button
* `0D 00 00 00`   — Fn / FN1 layer key (vendor virtual key FA)
* `0D 01 00 00`   — FN2 layer key (vendor virtual key FB); established by
  `G5KB_ConvertKeyCode` at `0x0048dc80`, not yet hardware-tested
* `05 …`          — macro

## 2.4G dongle (`CDev3632`, vtable `0x00610544`)

Dongle `3554:FA09`, collection **usage page 0xFF02, usage 2**, 20-byte
input/output reports (interrupt, `WriteFile`/`ReadFile`), report id `0x13`.
Verified working on hardware (read + write of the LED config).

Packet (`AccessData` 0x00487530, `SendCMD` 0x004878d0, checksum 0x00488120):

```
[0]  0x13
[1]  command
[2]  total packets           (reply: bit 7 = checksum error)
[3]  packet index
[4]  sub << 4 | data length  (<= 14)
[5..19] data (14 bytes)
[19] checksum = sum(bytes 0..19) & 0xFF
```

* **Write**: split into 14-byte packets, 15 ms apart. Each one is acked by an
  input report with the same command at `[1]` and the same index at `[3]`.
  `[0] == 0xFF` means failure. Reset (0x06) is not acked.
* **Read**: send `[13, cmd, 01, 00, sub<<4, 0…, csum]`; the dongle then streams
  packets `[_, cmd, total, index, len, data…]` until `index == total-1`.

| set  | get  | block            |
|------|------|------------------|
| 0x01 | 0x41 | key matrix (sub = layer) |
| 0x02 | 0x42 | per-key colours (sub = group) |
| 0x03 | 0x43 | macros (512-byte pages, sub = page) |
| 0x04 | 0x44 | LED config (128 bytes) |
| 0x06 |      | reset            |
| 0x09 | 0x49 | RGB table        |
| 0x0A |      | on-board mode    |
| 0x0B |      | screen param     |

There is no password command over 2.4G, and `bcdDevice` of the dongle
interface reports the firmware version (0x0102 = V1.0.2).

## Realtime / music mode

There is no audio-reactive firmware effect. The vendor app's Music page:

1. `Audio_LoopbackCaptureThread` (0x00483310): WASAPI loopback capture of the
   default output device, then FFT into 23 bands.
2. `Music_Thread` (0x00482b80): every 20 ms reads the bands; after ~30 min of
   silence it idles.
3. `Music_RenderFrame` (0x00480150): one of 10 visualiser styles ("Audio dance",
   "Dazzling", …) rendered on the PC, then sent to the keyboard.

While frames keep arriving they override the stored effect. Keys not listed
in a frame are dark. Stopping restores the stored effect.

Frame payload (max 200 bytes), groups of keys sharing a colour:

```
repeat { R, G, B, count, led_index × count }
```

| transport | command | sub | notes |
|-----------|---------|-----|-------|
| 2.4G      | 0x88    | 1 = frame, 2 = end | no ack; ~10 ms between 14-byte packets or the frame breaks up (verified) |
| USB       | 0x08    | 0   | feature report, 7 ms delay (not hardware-tested) |

End streaming with `[0,0,0]` and sub 2. A full 126×3 RGB variant (0x17A bytes)
exists in the code for other device types; the X87 dongle ignores it.

Rust API: `Keyboard::send_frame(&Frame)`, `send_frame_lossy`, `stop_realtime`,
`send_frame_raw`; see `examples/realtime.rs`.

## Device options (inside the LED config block, verified on hardware)

| offset | setting  | encoding |
|--------|----------|----------|
| 0x03   | Debounce | stage - 1. App offers stages 1..5 (`ShowDebounce`); the firmware stores and applies larger values (60 tested: keys become laggy) |
| 0x16   | Tap      | 0 = off, otherwise sensitivity 1..127 |
| 0x18   | Sleep    | timeout in 30 s units, 0 = never (2 min = 4, 5 min = 10). App presets: 30 s, 1, 1.5, 2, 3, 4, 5, 10, 15, 20 min |

Factory reset: USB cmd 0x11 / 2.4G cmd 0x06 with a 1-byte payload `00`.
