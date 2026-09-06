# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[Semantic Versioning](https://semver.org/).

## [Unreleased]

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

[Unreleased]: https://github.com/luizjr/HIDra/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/luizjr/HIDra/releases/tag/v0.1.0
