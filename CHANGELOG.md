# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.2]

### Added
- **In-app updates.** HIDra checks for a new release on startup and on demand,
  and installs it itself: through `tauri-plugin-updater` on Windows, macOS and
  the Linux AppImage, and by handing the signed package to `dpkg`/`rpm` via
  `pkexec` on a `.deb` or `.rpm` install. Every artifact — including the Linux
  packages, which now ship a `.sig` — is verified against the project's release
  key before anything is installed, and a build HIDra did not package is only
  ever told about the release, never modified.
- The sidebar shows the running version and a "Procurar atualizações" button.

## [0.1.1]

### Fixed
- Black rectangles painted over the mouse lighting tab on Linux. WebKitGTK puts
  filtered elements on their own compositing layer and blits it to the wrong
  place on some GPU setups (reported on a hybrid Intel + NVIDIA laptop under
  Wayland, WebKitGTK 2.52). The mouse preview's LED glow is now drawn with
  gradients and stacked strokes, so the app uses no CSS or SVG filters at all.

### Added
- A troubleshooting section in both READMEs: rendering artifacts, wireless
  writes that do not stick, and device permissions.

## [0.1.0] — first public release

### Added
- Redragon Cobra Pro / M711-PRO (`25A7:FA07` receiver, `25A7:FA08` cable):
  lighting, 5 DPI stages with per-stage colour, button remapping, polling rate,
  and read-back of the device's stored state.
- Redragon Brahma Pro / K586RGB-PRO (`0C45:5004`): 18 lighting effects plus
  per-key colours, brightness/speed/direction/full-RGB, 3 onboard profiles, key
  remapping and factory keymap restore.
- Data-driven device table (`src-tauri/src/devices.rs`) so a compatible model can
  be added without touching protocol code.
- Byte-level protocol documentation for both devices in `PROTOCOL.md`.
- Linux, Windows and macOS packages built by CI.

[Unreleased]: https://github.com/luizjr/HIDra/compare/v0.1.2...HEAD
[0.1.2]: https://github.com/luizjr/HIDra/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/luizjr/HIDra/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/luizjr/HIDra/releases/tag/v0.1.0
