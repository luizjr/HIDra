#!/bin/sh
# Bump the version in the three files that carry it, so they can never drift:
#   package.json  ·  src-tauri/tauri.conf.json  ·  src-tauri/Cargo.toml
#
#   sh scripts/bump-version.sh 0.2.0
#
# Then update CHANGELOG.md, commit, and tag — see CONTRIBUTING.md.
set -e

V="$1"
case "$V" in
    [0-9]*.[0-9]*.[0-9]*) ;;
    *) echo "usage: sh scripts/bump-version.sh <major.minor.patch>" >&2; exit 1 ;;
esac

cd "$(dirname "$0")/.."

python3 - "$V" <<'PY'
import json, re, sys

version = sys.argv[1]

for path, key in (("package.json", "version"), ("src-tauri/tauri.conf.json", "version")):
    with open(path) as f:
        data = json.load(f)
    data[key] = version
    with open(path, "w") as f:
        json.dump(data, f, indent=2)
        f.write("\n")

path = "src-tauri/Cargo.toml"
text = open(path).read()
text = re.sub(r'^version = "[^"]+"', f'version = "{version}"', text, count=1, flags=re.M)
open(path, "w").write(text)
PY

# Keep Cargo.lock in step with the crate version.
cargo update --manifest-path src-tauri/Cargo.toml --package hidra --precise "$V" >/dev/null 2>&1 || \
    cargo check --manifest-path src-tauri/Cargo.toml --quiet >/dev/null 2>&1 || true

grep -H '"version"' package.json src-tauri/tauri.conf.json
grep -H '^version' src-tauri/Cargo.toml
echo "Now: update CHANGELOG.md, commit, then 'git tag v$V && git push origin v$V'."
