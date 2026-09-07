# Adding a device

HIDra started with two Redragon peripherals because those were the ones on the
author's desk — nothing in it is tied to that brand. The device table is a plain
list of USB ids and protocol names, so a Logitech, Razer, Corsair, Motospeed,
Havit, Fantech or no-name mouse belongs there exactly as much as a Redragon one.
The goal is a compatibility library that only ever grows.

There are two cases:

1. **Your device speaks a protocol HIDra already implements.** Very common:
   peripherals are often the same OEM board sold under several brands, with only
   the USB product id changed. This is a one-line change and needs no Rust.
2. **Your device speaks something new.** Then it needs a protocol capture, and
   there is a recipe for that at the end.

---

## The short version

```sh
git clone https://github.com/luizjr/HIDra && cd HIDra
python3 scripts/probe-device.py          # tells you the numbers, prints an entry
# paste the entry into src-tauri/src/devices.rs, pick a protocol
python3 scripts/sync-devices.py          # updates udev rules, READMEs, website
npm install && npm run tauri dev         # read a setting, change a setting
```

Then open a pull request saying what you tested and on which OS. That is the
whole flow; the rest of this page explains each step.

---

## 1. Identify the device

Configuration does not travel on the collection that types letters or moves the
pointer — it travels on a **vendor-defined collection** (a HID usage page in the
`0xFF00`–`0xFFFF` range), usually on USB interface 1.

On Linux, the probe script finds it for you:

```sh
python3 scripts/probe-device.py
```

It prints every USB HID interface that has a vendor collection, the reports each
one carries, and its guess at which collection you write to and which one the
device answers on:

```
Compx · 2.4G Wireless Receiver
  25a7:fa07  interface 1  /dev/hidraw2
    usage page 0xff01 usage 0x00  0x09 I16B  <-- replies arrive here
    usage page 0xff02 usage 0x02  0x08 F16B  <-- config: write here
  → config_usage_page: 0xff02   reply_usage_page: 0xff01
```

The guess comes from report sizes: the configuration channel is normally the
roomiest Output or Feature report on the interface. It is a guess — if writes do
nothing later, come back and try another vendor collection.

`--issue` prints the same thing wrapped for pasting into a GitHub issue, and
`--json` is there for scripts and agents.

**On Windows**: Device Manager → the device → Details → *Hardware Ids* gives
`HID\VID_25A7&PID_FA08`. For the collections, [USBView] or [Wireshark with
USBPcap] shows the report descriptor.

**On macOS**: System Information → USB gives the ids; the report descriptor
needs a tool such as [hidapitester] or `ioreg -l -w0 | grep -i HIDReport`.

Both are more work than the Linux script, so if you have a Linux machine handy,
use it for this step even if you mainly run something else.

[USBView]: https://learn.microsoft.com/en-us/windows-hardware/drivers/debugger/usbview
[Wireshark with USBPcap]: https://desowin.org/usbpcap/
[hidapitester]: https://github.com/todbot/hidapitester

## 2. Pick a protocol

`Protocol` in `src-tauri/src/devices.rs` lists what HIDra can already speak:

| Protocol | Shape | Known in |
| --- | --- | --- |
| `Compx17` | 17-byte feature reports on id `0x08`, every packet summing to `0x55` | Compx/Areson mice (M711-PRO family) |
| `Sonix64` | 64-byte reports on id `0x04`, 16-bit sum checksum | SONiX SN32F keyboards (K586RGB-PRO family) |

Signs your device speaks one of them:

- the vendor's Windows tool looks and behaves like the one for a device we
  already support (same layout, same effect names, same profile count);
- the report sizes the probe script printed match (`16B` feature reports point
  at `Compx17`, `63B`/`126B` reports on one vendor collection at `Sonix64`);
- the controller is the same — a teardown photo or an FCC listing naming a SONiX
  SN32F or a Compx chip is strong evidence.

If you are unsure, the honest way to find out is to try. Setting
`support: Support::Reported` marks your entry as unproven, and HIDra shows a
warning badge next to the device, so a wrong guess is visible rather than
silently misleading.

## 3. Add the entry

Paste what the probe script printed into `SUPPORTED` in
[`src-tauri/src/devices.rs`](../src-tauri/src/devices.rs) and fill in what it
could not know:

```rust
Device {
    vendor: "Redragon",                  // brand on the box
    model: "Cobra Pro (M711-PRO)",       // marketing name + model code
    vid: 0x25a7,
    pid: 0xfa08,
    kind: Kind::Mouse,                   // decides which UI panel it gets
    protocol: Protocol::Compx17,
    interface: 1,                        // USB interface of the config endpoint
    config_usage_page: 0xff02,           // collection you write to
    reply_usage_page: 0xff01,            // collection it answers on
    connection: Connection::Wired,       // or Receiver, for a 2.4 GHz dongle
    support: Support::Reported,          // Verified once it works on your desk
},
```

A device that appears under different product ids over cable and over its own
receiver gets **one entry per id** — that is how HIDra can fan a write out to
whichever one is awake.

`config_usage_page` and `reply_usage_page` are the same value for most devices.
They differ when a device answers on another collection than the one it is
written to (the Compx mice do), and the distinction only matters on Windows,
where every top-level collection is a separate handle. Getting it wrong there
means writes go to the mouse-movement collection and are silently dropped.

## 4. Regenerate what depends on the table

```sh
python3 scripts/sync-devices.py
```

This rewrites the udev rule, the tables in both READMEs and the table on the
website from your entry. CI runs `--check` and fails if you forget, so there is
no way to leave them stale.

## 5. Test it on your hardware

```sh
sudo sh packaging/install-udev.sh   # Linux, once, then replug the device
npm install
npm run tauri dev
```

In order:

1. The sidebar should show the device as connected, with your model name.
2. **Read** something first — the lighting or DPI panel filling in with the
   device's real values proves the endpoint is right.
3. **Write** something small — brightness, or one DPI stage — and watch the
   hardware react.
4. Read it back and confirm the change stuck.

If reads come back empty, the endpoint is probably wrong: try another vendor
collection from the probe output. If reads work but writes do nothing on a
wireless mouse, move the mouse while applying — the 2.4 GHz radio sleeps when
idle, and the device ignores configuration while asleep.

When it all works, change `support` to `Support::Verified` and run
`python3 scripts/sync-devices.py` again.

## 6. Send the pull request

```sh
npm run lint && npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml --all
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
python3 scripts/sync-devices.py --check
```

In the description, say **what you tested, on which OS, and what you saw**. For
a new device that is the whole review: nobody else owns your hardware, so your
report is the evidence. "Set brightness 3 on Fedora 41, read back 3, keyboard
visibly dimmed" is worth more than any amount of code review.

A pull request that adds an entry as `Support::Reported` without testing is
still welcome — say so plainly, and it lands as unproven rather than being
turned away.

---

## Adding a new protocol

When nothing in `Protocol` fits, the device's packets have to be captured.

### Capture

The recipe that produced the keyboard protocol in this repository, on Linux with
no second machine:

1. Run the vendor's Windows tool under Wine.
2. Wine's `winebus` refuses to hand a keyboard or mouse `hidraw` node to an app,
   so mirror the real device onto a `/dev/uhid` node and let the tool talk to
   the mirror. The proxy logs every packet in both directions.
3. Drive the tool's GUI with `xdotool` on a headless `Xvfb` display, changing
   **one setting at a time**, so each captured packet has a known meaning.

With a spare Windows machine, [USBPcap] + Wireshark is simpler. On Linux,
`usbmon` (`modprobe usbmon`, then capture `usbmon` in Wireshark) records
everything the kernel sees, including traffic from a tool running in a VM with
the device passed through.

[USBPcap]: https://desowin.org/usbpcap/

### Implement

- Add a variant to `Protocol` in `devices.rs`, with a comment saying what the
  packets look like and which module implements it.
- Write the module next to `mouse.rs` and `keyboard.rs`, following their shape:
  build packet → checksum → apply → read back.
- Open the device with `hid::open_speaking(api, kind, Protocol::Yours)` so a
  device that speaks a different dialect can never be handed your packets.
- Add the feature strings for the new protocol to `FEATURES` in
  `scripts/sync-devices.py`, so the generated tables describe it.
- Write down every byte you learned in [`PROTOCOL.md`](../PROTOCOL.md). The
  notes matter as much as the code: they are what lets the next person fix this
  without rebuilding your capture rig.

### What HIDra will not do

Configuration registers only — the same ones the vendor tool writes. No
bootloader commands, no firmware writes, nothing that can brick a device. A pull
request that flashes firmware will be declined however well it works.

---

## Using an AI assistant

If you work with Claude Code or a similar agent, this repository ships
instructions written for it: [`CLAUDE.md`](../CLAUDE.md) at the root describes
the architecture and conventions, and `/add-device` is a ready-made command that
walks the whole flow above — probe, entry, regenerate, build, test plan, pull
request. Point your agent at the repository and ask it to add your device; it
has everything it needs. You still have to run the app and confirm the hardware
reacts, because no agent can see your keyboard light up.
