# Agent instructions

This repository's instructions for AI assistants live in
**[CLAUDE.md](CLAUDE.md)** — architecture, invariants, commands and the recipe
for the most common contribution (adding a device). Read that file first,
whatever agent you are.

The three things most likely to bite you if you skip it:

- `src-tauri/src/devices.rs` is the only place model-specific data belongs;
- `packaging/70-hidra.rules`, the README device tables and the website table are
  **generated** from it by `scripts/sync-devices.py` (CI fails on drift);
- no CSS or SVG filters anywhere in the UI — they trigger a WebKitGTK
  compositing bug that paints black rectangles over the app.

Humans adding a device should read [docs/ADDING-DEVICES.md](docs/ADDING-DEVICES.md).
