#!/bin/sh
# One-off GitHub repository setup: description, homepage, topics and the
# features the docs assume exist. Everything here is a repository *setting*,
# not a file, so it cannot live in the repo itself.
#
#   gh auth login          # once, if you have not
#   sh scripts/setup-repo.sh
#
# Safe to re-run: every call overwrites with the same values.
set -e

REPO="${REPO:-luizjr/HIDra}"

command -v gh >/dev/null 2>&1 || {
    echo "gh (GitHub CLI) is required: https://cli.github.com" >&2
    exit 1
}

DESCRIPTION="Open-source configurator for gaming mice and keyboards on Linux, Windows and macOS — RGB lighting, DPI, polling and key remapping over raw HID, no vendor software"

echo "Setting description, homepage and features on $REPO…"
gh repo edit "$REPO" \
    --description "$DESCRIPTION" \
    --homepage "https://luizjr.github.io/HIDra/" \
    --enable-issues \
    --enable-discussions \
    --enable-wiki=false \
    --enable-projects=false

echo "Setting topics…"
gh repo edit "$REPO" \
    --add-topic hid \
    --add-topic hidapi \
    --add-topic rgb \
    --add-topic gaming-mouse \
    --add-topic gaming-keyboard \
    --add-topic peripherals \
    --add-topic tauri \
    --add-topic rust \
    --add-topic react \
    --add-topic linux \
    --add-topic reverse-engineering \
    --add-topic redragon

# Labels the issue templates refer to; GitHub only ships its own defaults.
echo "Ensuring labels…"
gh label create device -R "$REPO" --color 1d76db \
    --description "Support for a device HIDra does not know yet" 2>/dev/null || true
gh label create protocol -R "$REPO" --color 5319e7 \
    --description "USB/HID protocol capture or documentation" 2>/dev/null || true

echo "Done. Check it: gh repo view $REPO --web"
