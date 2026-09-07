---
description: Add a mouse or keyboard to HIDra's supported-device table, end to end
argument-hint: "[device name, or vid:pid, or nothing to probe what is plugged in]"
---

Add a device to HIDra's compatibility table: $ARGUMENTS

Follow this exactly. Read `CLAUDE.md` first if it is not already in context.

## 1. Find the device

If the user is on Linux and the device is plugged in:

```sh
python3 scripts/probe-device.py
```

That prints, per USB interface: vid, pid, interface number, every HID collection
with its report ids and sizes, a guess at `config_usage_page` /
`reply_usage_page`, and a pasteable `Device { … }` entry for anything not
already supported.

If the probe finds nothing, try `--all` (the device may expose no vendor
collection on the interfaces Linux enumerated) and check the device is not
behind a KVM. If the user is on Windows or macOS, do not guess the numbers — ask
them for the ids (Device Manager → Details → Hardware Ids, or System Information
→ USB) and for a report descriptor dump; `docs/ADDING-DEVICES.md` lists the
tools. Never invent a usage page.

## 2. Choose a protocol

`Protocol` in `src-tauri/src/devices.rs` is the list of dialects HIDra speaks.
Match on evidence, in this order:

- report shape from the probe output — 16-byte feature reports look like
  `Compx17`; a single vendor collection with ~63-byte input and output reports
  looks like `Sonix64`;
- the same controller or the same vendor tool as a device already in the table;
- the user telling you the vendor app behaves like one we support.

If nothing matches, stop and tell the user plainly: this device needs a protocol
capture, which is a much bigger job, and point them at "Adding a new protocol"
in `docs/ADDING-DEVICES.md`. Do not force an entry into an existing protocol to
make the table longer — a device that accepts packets of the wrong dialect is
worse than an unsupported one.

## 3. Add the entry

Edit `SUPPORTED` in `src-tauri/src/devices.rs`. Keep entries grouped by kind,
mice first, in the order the file already uses. One entry per USB product id: a
device with a different id over cable and over its receiver gets two, with
`connection: Connection::Wired` and `Connection::Receiver`.

Set `support: Support::Reported`. It becomes `Verified` only after the user
reports the hardware reacting — see step 5.

## 4. Regenerate and check

```sh
python3 scripts/sync-devices.py
npm run build
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path src-tauri/Cargo.toml --all
```

Never hand-edit `packaging/70-hidra.rules`, the README device tables or the
table in `site/index.html` — the script owns them.

## 5. Hand the test to the user

You cannot see a keyboard light up. Ask them to run the app and report back,
with these exact steps:

```sh
sudo sh packaging/install-udev.sh   # Linux, once, then replug the device
npm run tauri dev
```

1. Does the sidebar show the device as connected, with the right model name?
2. Does a panel fill in with the device's real values? (that proves reads work)
3. Change one small thing — brightness, or one DPI stage — and apply. Does the
   hardware react?
4. Reopen the app: did the change stick?

If reads fail, the endpoint is likely wrong: go back to the probe output and try
another vendor collection. If reads work but writes do nothing on a wireless
mouse, tell them to move the mouse while applying — the 2.4 GHz radio sleeps
when idle.

Wait for their answer. Do not write "verified" anywhere on your own authority.

## 6. Finish

Once they confirm: set `support: Support::Verified`, re-run
`python3 scripts/sync-devices.py`, and commit. The message must say which device
and which OS the user tested on, in their words, e.g.
"Verified on a K586RGB-PRO on Fedora 41: brightness 3 applied and read back."

If they want a pull request, use `gh pr create` from a branch — never push
directly to `main` for someone else's device.
