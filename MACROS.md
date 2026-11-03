# X87 macro reverse-engineering notes

Status: static analysis of `OemDrv.exe` through Ghidra MCP. The library and web
codecs and upload APIs implement this specification, with offline regression
tests.

## The two screens

The macro editor manages named sequences on the PC. The key-assignment screen
selects one of those sequences and attaches playback options to a key/layer.
The editor's tree/folders are not the firmware's macro table: when applying a
profile, the app collects referenced macros, assigns compact slots and uploads
the resulting table plus records. Playback itself is on the keyboard.

## Sources checked in Ghidra

| Address | Purpose |
|---|---|
| `0x0048e360` / continuation `0x0048e3a8` | Convert PC macro to firmware record |
| `0x0048e46d` | Pack event type, edge, delay and value |
| `0x0048dc80` | Convert macro assignment to hardware key-map entry |
| `0x0048e730` | Collect macros, allocate slots, build offset/length table |
| `0x0048eb10`–`0x0048eb90` | Relocate record offsets and copy table/data |
| `0x00424b60` / `0x00424d04`–`0x00424d73` | Read repeat radio buttons and count |
| `0x00402870` | Reconstruct editor timeline from PC macro events |
| `0x0048ce70`, `0x0048cec0` | USB macro write/read wrappers |
| `0x00487c40` | Wireless macro upload, page by page |
| `0x00487d20` | Wireless macro read wrapper |
| `0x00410c50` | `MacroBufferSize` configuration, default 4096 bytes |

The decompiler incorrectly separates the prologue at `0x0048e360` from the
body at `0x0048e3a8`; the assembly confirms they are one encoder. Some custom
register calling conventions also produce misleading C argument lists.

## Named macro record

```text
u8 name_byte_length              // UTF-16LE byte length, not character count
u8 name_utf16le[name_byte_length] // no trailing NUL in this record
Event events[]                  // each event is exactly 4 bytes
```

The app copies at most 30 UTF-16 code units (60 bytes) for the name. A record's
length in the directory includes the name-length byte, name and event bytes.
There is no separate event count inside the firmware record; its enclosing
directory length determines the end.

Example: name `A`, press/release USB keyboard usage `0x04`:

```text
02 41 00                 // name length 2; "A"
80 00 00 04              // keyboard down, zero interval
00 00 32 04              // keyboard up, interval 0x32
```

The encoder copies the PC event interval directly to the 20-bit field. The UI
works in millisecond intervals; physical timing still needs verification.

## Event encoding

```text
byte 0: [edge bit 7] [type bits 6..4] [delay bits 19..16]
byte 1: delay bits 15..8
byte 2: delay bits 7..0
byte 3: event value
```

- The app caps the interval at `0xFFFFF` (1,048,575).
- For types 0, 1 and 2, bit 7 is set on **press** and clear on release.
- Movement and wheel events (3, 4 and 5) do not set an edge bit.
- Byte 3 is an HID usage, mouse-button mask, or signed movement/wheel byte.

| Type | Meaning | Value |
|---|---|---|
| 0 | Ordinary keyboard key | USB HID keyboard usage (not Windows VK) |
| 1 | Modifier key | HID usage `E0`..`E7` |
| 2 | Mouse button | bit mask `01`, `02`, `04`, `08`, `10` |
| 3 | Mouse X movement | low byte of signed displacement |
| 4 | Mouse Y movement | low byte of signed displacement |
| 5 | Mouse wheel | signed amount |

The app maps left/right modifiers explicitly. For the keyboard device type
it swaps the PC-side mouse back/forward values before packing (VK `F3`/`F4`).
Do not reuse the library's normal key-remap mouse codes here: macro button
values are masks, whereas ordinary remaps use a different layout.

The input-side editor stores 12-byte actions: `u16 vk`, `u16 edge_flags`,
`u32 interval`, `u32 parameter`. The firmware uses the compact 4-byte encoding
above, so a vendor `.mcr` export must not be uploaded directly as macro data.
The exporter size helper returns `0x68 + action_count * 12`; additional
export-format details have not been fully decoded.

## Key assignment and repeat modes

The resulting 4-byte hardware key-map entry is:

```text
03 mode repeat_count slot
```

These bytes are the little-endian representation of the map's `u32` value.
The firmware slot is zero-based. The app internally assigns buffer IDs starting
at 1, then writes `buffer_id - 1` into byte 3.

| PC mode field | Firmware mode byte | Repeat byte |
|---|---|---|
| 0 | `01` | requested count, at least 1 in the UI |
| 1 | `02` | `01` |
| 2 | `04` | `01` |

Counted playback is confirmed end-to-end from the count edit field to mode 0
and firmware byte `01`. The first radio control (`+0x1cf4` checked state) sets
PC mode 2; the second (`+0x1e80`) sets PC mode 1. Combined with their vertical
order and the supplied screenshots, the working interpretation is:

- **Cycle until the key released**: firmware mode `04`.
- **Cycle until any key pressed**: firmware mode `02`.
- **Cycle times**: firmware mode `01`, count in byte 2.

The byte count field does not justify assuming counts above 255 work.

## Macro directory and buffer

For `N` referenced macros, the buffer starts with `N` four-byte entries:

```text
u16le absolute_offset
u16le record_length
```

The first record starts immediately after the directory, at `N * 4`. Records
follow contiguously. The encoder first records relative offsets, then adds
`N * 4` to every offset. There is no count field inserted ahead of the table.

The first record's offset can therefore help infer the table size on read,
but this inference needs consistency/bounds checks and a real-device fixture.
The key maps also reveal which slots are referenced. Empty or corrupt buffers
must not be interpreted blindly.

`MacroBufferSize` defaults to `0x1000` (4096); the X87's `KB.ini` does not
override it. The vendor app's temporary staging arrays allow larger profiles
(up to `0x2800`), but that is **not** evidence that this keyboard has 10 KiB of
macro storage. The builder has temporary arrays for 100 entries; the actual
firmware slot limit is not established. No per-record checksum was found.
Transport packets have their normal framing/checksums.

## Upload/read transport

### USB cable

- Write macro buffer: base command `05`, sub `00`.
- Read macro buffer: command `85`, sub `00`.
- Normal 520-byte feature framing, data chunked into at most 512 bytes.

### 2.4G receiver

- Write: command `03`.
- Split the full buffer into **512-byte pages**; the sub-index identifies the
  page, and the chunk counter restarts inside each page.
- Each page uses the ordinary 14-byte wireless packets and acknowledgements.
- The app uses 15 ms packet spacing, retries, then waits 100 ms after each page.
- Read wrapper: `W3632_GetMacro` at `00487d20` calls command `43`, sub `00`.
- Receiver captures confirmed 512-byte replies for all eight subs (37 packets,
  final payload 8 bytes, sub echoed in the high nibble). Hardware test over
  2.4G: an empty bank read twice identically; a test macro (key down/up +
  move Y) was written, read back byte-identical, then the empty bank was
  restored and verified, with all four key layers unchanged. The Rust library
  also read the bank over 2.4G.
- Ordinary receiver captures returned 504-byte matrix and 490-byte RGB blocks,
  shorter than USB allocation. Readers require every packet and meaningful
  bytes, pad only unused tails, and retry incomplete/conflicting streams.

Library macro API: `macro_bank()` reads the bank, `save_macro_bank()` writes
it once like the vendor app (no layer check, no read-back), and
`clear_macro_bank()` writes an empty bank. Writes use the 512-byte page
wrapper and require a complete 4096-byte bank.

## Safe implementation/verification plan

1. Add pure record/directory/key-binding codecs with fixture tests.
2. Capture one tiny vendor-generated macro (e.g. `A down`, 50 ms, `A up`) and
   compare bytes; check each repeat mode separately.
3. Back up all layers and the entire macro buffer before any upload.
4. Upload only with explicit user approval: rebuilding the table can move slot
   numbers and invalidate references in **other layers**.
5. Read back, compare, then temporarily assign an unused key for a single
   counted playback. Restore its original mapping afterwards.
6. Add library and web support only after the codec and page handling agree
   with the fixture. The UI should distinguish local draft macros from macros
   actually uploaded to the keyboard.
