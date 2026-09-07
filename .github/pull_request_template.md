## What this changes

<!-- One or two sentences. -->

## Tested on

<!-- Which device, which OS, and what you actually saw happen.
     e.g. "K586RGB-PRO on Ubuntu 24.04: set brightness 3, read back 3, keyboard
     visibly dimmed."

     For a new device this is the review: nobody else owns your hardware. If you
     added an entry without testing it, say so — it lands as Support::Reported
     rather than being turned away. If an AI assistant wrote the change, the
     test still has to come from a person with the device. -->

## Checklist

- [ ] `npm run lint` and `npm run build` pass
- [ ] `cargo fmt` and `cargo clippy -- -D warnings` pass
- [ ] `python3 scripts/sync-devices.py --check` passes
      (run `python3 scripts/sync-devices.py` if it does not — the udev rule, the
      README tables and the website table are generated from `devices.rs`)
- [ ] New protocol facts are written down in `PROTOCOL.md`
- [ ] `support:` reflects reality — `Verified` only if it worked on hardware
