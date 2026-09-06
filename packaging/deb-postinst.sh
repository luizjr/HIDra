#!/bin/sh
# Reload udev so the rule shipped with the package takes effect without a reboot.
# Devices already plugged in are re-triggered; anything else needs a replug.
set -e
if command -v udevadm >/dev/null 2>&1; then
    udevadm control --reload-rules || true
    udevadm trigger --subsystem-match=hidraw --action=change || true
fi
exit 0
