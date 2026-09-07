# Contributing to HIDra

Thanks for being here. HIDra exists because vendor tools are Windows-only and
abandoned — every device someone adds makes that a little less true.

## Ways to help, roughly by value

1. **Add a device.** See [docs/ADDING-DEVICES.md](docs/ADDING-DEVICES.md)
   ([em português](docs/ADDING-DEVICES.pt-BR.md)). If it speaks a protocol we
   already implement, it is one entry in a table — `scripts/probe-device.py`
   works out the numbers for you and prints the entry. HIDra is not a Redragon
   project: any brand belongs in that table.
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
python3 scripts/sync-devices.py --check       # generated files match devices.rs
```

CI runs the same on Linux, Windows and macOS.

## Working with an AI assistant

[`CLAUDE.md`](CLAUDE.md) is the instruction file for AI coding assistants —
architecture, invariants and the device-adding recipe — and
[`AGENTS.md`](AGENTS.md) points other tools at it. Claude Code users get a
ready-made `/add-device` command that runs the whole flow.

Agent-written pull requests are welcome on the same terms as any other: the
hardware test in the description has to come from a person who owns the device.

## House style

- **The device table is data.** Anything specific to one model belongs in
  `src-tauri/src/devices.rs`, not in `hid.rs`, `mouse.rs` or `keyboard.rs`.
- **Write down protocol facts.** A new byte, offset or quirk goes in
  `PROTOCOL.md` in the same PR. Undocumented magic numbers are how these
  projects die.
- **Say what you tested.** "Applied brightness 3 on a K586RGB-PRO on Fedora 41,
  read back correctly" is worth more than any amount of review.
- Comments explain *why*, not *what*. Match the surrounding code.

## Cutting a release (maintainers)

```sh
sh scripts/bump-version.sh 0.2.0     # package.json + tauri.conf.json + Cargo.toml
# write the CHANGELOG.md entry for 0.2.0
git commit -am "Release 0.2.0" && git push
git tag v0.2.0 && git push origin v0.2.0
```

Pushing the tag is the whole release: CI builds Linux, Windows and macOS,
uploads every installer, and publishes the release. Nothing is done by hand, and
re-running a tag replaces its release instead of duplicating assets. Pushing to
`main` separately refreshes the rolling `nightly` pre-release.

Repository settings that cannot live in the repo (description, topics, enabled
features) are in `scripts/setup-repo.sh`.

## Safety

HIDra only ever writes to a device's configuration registers — the same ones the
vendor tool writes. It does not flash firmware and cannot brick a device. If you
are adding a protocol, keep it that way: no bootloader commands, no firmware
writes.

## Licence

By contributing you agree that your work is licensed under
[GPL-3.0-or-later](LICENSE), like the rest of the project.
