# Adding a device

There are two cases, and the first one is the easy one.

1. **Your device speaks a protocol HIDra already implements.** Rebadged
   peripherals are everywhere: the same OEM board ships under several brands
   with only the USB product id changed. Adding it is one entry in
   `src-tauri/src/devices.rs`.
2. **Your device speaks something new.** Then you need a capture, and there is a
   recipe for that at the bottom.

You do not need to know Rust for case 1. You need `lsusb` and ten minutes.

---

## 1. Identify the device

Plug it in and find its USB ids:

```sh
lsusb                       # Linux: "ID 25a7:fa08 ..." — vendor:product
```

On Windows: Device Manager → the device → Details → *Hardware Ids*
(`HID\VID_25A7&PID_FA08`). On macOS: System Information → USB.

## 2. Find the configuration endpoint

Configuration does not travel on the interface that types letters or moves the
pointer — it travels on a **vendor-defined HID collection**, usually on USB
interface 1, on a usage page in the `0xFF00`–`0xFFFF` range.

On Linux, HIDra's own report-descriptor dump is the quickest way to see them:

```sh
# one directory per hidraw node; check the ones for your device's ids
for n in /sys/class/hidraw/hidraw*; do
  echo "== $n"; cat $n/device/uevent | grep HID_
done
# then dump the descriptor of the candidate node
sudo usbhid-dump -d 25a7:fa08          # or:
xxd /sys/class/hidraw/hidrawN/device/report_descriptor
```

You are looking for a top-level collection whose usage page is `0xFFxx`. Write
down two things:

- `config_usage_page` — the collection you *write* configuration to.
- `reply_usage_page` — the collection the device *answers* on. For most devices
  it is the same one; the Compx mice are an exception (write on `0xFF02`, reply
  on `0xFF01`), which is why the table has both fields.

This distinction only matters on Windows, where every top-level collection is a
separate handle. Getting it wrong there means writes go to the mouse-movement
collection and are silently dropped.

## 3. Add the entry

In [`src-tauri/src/devices.rs`](../src-tauri/src/devices.rs), copy an existing
entry of the same kind and change the ids:

```rust
Device {
    vendor: "Redragon",
    model: "Cobra Pro (M711-PRO)",
    vid: 0x25a7,
    pid: 0xfa08,
    kind: Kind::Mouse,
    protocol: Protocol::Compx17,   // an existing protocol
    interface: 1,
    config_usage_page: 0xff02,
    reply_usage_page: 0xff01,
    connection: Connection::Wired, // or Receiver, for a 2.4 GHz dongle
    support: Support::Reported,    // Verified once you have run it for real
},
```

Set `support: Support::Reported` if you are guessing from a sibling model —
HIDra then shows a small warning badge next to the device so users know the
entry is unproven. Change it to `Verified` after you have applied settings and
watched them take effect.

Also add the USB ids to `packaging/70-hidra.rules` so Linux users get non-root
access, and a row to the table in both READMEs.

## 4. Test it

```sh
npm run tauri dev
```

The sidebar should show the device as connected with its model name. Read a
setting first (that proves the endpoint is right), then apply a harmless change
— brightness, or one DPI stage — and confirm the hardware reacts.

If reads come back empty, the endpoint is probably wrong: try another vendor
collection. If reads work but writes do nothing on a wireless mouse, move the
mouse while applying — the 2.4 GHz radio sleeps when idle.

Open a pull request with the entry, the ids, and one line saying what you tested
on which OS.

---

## Adding a new protocol

If nothing in `Protocol` fits, the device needs its packets captured. What
worked for the keyboard in this repo, on Linux without a second machine:

1. Run the vendor's Windows tool under Wine.
2. Wine's `winebus` refuses to hand a keyboard/mouse `hidraw` node to an app, so
   mirror the real device onto a `/dev/uhid` node and let the tool talk to the
   mirror. The proxy logs every packet in both directions.
3. Drive the tool's GUI with `xdotool` on a headless `Xvfb` display, changing one
   setting at a time, so each captured packet has a known meaning.

A USB analyser (Wireshark + `usbmon` on Linux, USBPcap on Windows) works too and
is simpler if you have a spare Windows machine.

Then:

- add a `Protocol` variant in `devices.rs`,
- implement it in its own module (`mouse.rs` and `keyboard.rs` are the models to
  follow — build packet, checksum, apply, read back),
- open the device with `hid::open_speaking(api, kind, Protocol::Yours)` so the
  dispatch stays honest,
- and write down what you learned in `PROTOCOL.md`. The notes matter as much as
  the code: they are what let the next person fix this without a capture rig.
