# Contributing to HIDra

Thanks for being here. HIDra exists because vendor tools are Windows-only and
abandoned — every device someone adds makes that a little less true.

## Ways to help, roughly by value

1. **Add a device.** See [docs/ADDING-DEVICES.md](docs/ADDING-DEVICES.md). If it
   speaks a protocol we already implement, it is one line.
2. **Capture a protocol** for a device we cannot talk to yet, and write it down
   in `PROTOCOL.md` — even without code, that is a real contribution.
3. **Test the Windows and macOS builds.** They are produced by CI and barely
   tested. Telling us what breaks there is genuinely useful.
4. **Fix bugs, improve the UI, translate.**

No contributor licence agreement, no ceremony. Be decent to each other.

## Getting set up

```sh
npm install
npm run tauri dev
```

Requirements and system packages are in the [README](README.md#building-from-source).

Rust build artifacts are large (several GB). If your disk is tight, point cargo
somewhere else — `export CARGO_TARGET_DIR=/tmp/hidra-target`, or a local
`src-tauri/.cargo/config.toml` with `[build] target-dir = "…"` (git-ignored).

The UI can run without any hardware: `src/backend.ts` falls back to a mock
backend with plausible fake data when the Tauri API is absent, so
`npm run dev` in a browser is enough for pure UI work.

## Before opening a pull request

```sh
npm run lint                                  # oxlint
npm run build                                 # tsc + vite
cargo fmt --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
```

CI runs the same four on Linux, Windows and macOS.

## House style

- **The device table is data.** Anything specific to one model belongs in
  `src-tauri/src/devices.rs`, not in `hid.rs`, `mouse.rs` or `keyboard.rs`.
- **Write down protocol facts.** A new byte, offset or quirk goes in
  `PROTOCOL.md` in the same PR. Undocumented magic numbers are how these
  projects die.
- **Say what you tested.** "Applied brightness 3 on a K586RGB-PRO on Fedora 41,
  read back correctly" is worth more than any amount of review.
- Comments explain *why*, not *what*. Match the surrounding code.

## Safety

HIDra only ever writes to a device's configuration registers — the same ones the
vendor tool writes. It does not flash firmware and cannot brick a device. If you
are adding a protocol, keep it that way: no bootloader commands, no firmware
writes.

## Licence

By contributing you agree that your work is licensed under
[GPL-3.0-or-later](LICENSE), like the rest of the project.
