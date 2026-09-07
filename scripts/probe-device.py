#!/usr/bin/env python3
"""Print everything HIDra needs to know about a plugged-in HID device.

Configuration does not travel on the collection that types letters or moves the
pointer — it travels on a vendor-defined collection, and finding it is the one
genuinely fiddly step in adding a device. This script does that part for you: it
walks the HID devices Linux knows about, parses their report descriptors, and
prints a ready-to-paste entry for `src-tauri/src/devices.rs`.

    python3 scripts/probe-device.py              # candidates worth configuring
    python3 scripts/probe-device.py --all        # every HID device
    python3 scripts/probe-device.py --issue      # a block to paste into an issue
    python3 scripts/probe-device.py --json       # machine-readable, for agents

Linux only, because it reads /sys. On Windows and macOS, see
docs/ADDING-DEVICES.md for how to get the same numbers by hand.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
from pathlib import Path

HIDRAW_CLASS = Path("/sys/class/hidraw")
DEVICES_RS = Path(__file__).resolve().parent.parent / "src-tauri/src/devices.rs"

# Usage pages at or above this are vendor-defined: that is where configuration
# channels live.
VENDOR_PAGE_MIN = 0xFF00

GENERIC_DESKTOP = 0x01
USAGE_MOUSE = 0x02
USAGE_KEYBOARD = 0x06


# --------------------------------------------------------------------------
# HID report descriptor parsing
# --------------------------------------------------------------------------

def parse_descriptor(blob: bytes) -> list[dict]:
    """Top-level collections of a report descriptor.

    Returns one dict per collection: usage page, usage, the reports declared
    inside it and how big they are. Input reports are what a device answers on;
    Output and Feature reports are what it is configured through, and the
    configuration channel is normally the roomiest of them.
    """
    collections: list[dict] = []
    usage_page = 0
    usages: list[int] = []
    report_id = 0
    report_size = 0
    report_count = 0
    depth = 0
    current: dict | None = None
    i = 0

    def note(kind_name: str) -> None:
        """Record a main item against the collection it lives in."""
        if current is None:
            return
        current[kind_name] = True
        payload = (report_size * report_count + 7) // 8
        reports = current["reports"].setdefault(report_id, {"bytes": 0, "kinds": set()})
        reports["bytes"] += payload
        reports["kinds"].add(kind_name[0].upper())

    while i < len(blob):
        prefix = blob[i]
        size = prefix & 0x03
        size = 4 if size == 3 else size
        kind = (prefix >> 2) & 0x03  # 0 main, 1 global, 2 local
        tag = (prefix >> 4) & 0x0F
        data = int.from_bytes(blob[i + 1 : i + 1 + size], "little") if size else 0
        i += 1 + size

        if kind == 1:  # global
            if tag == 0x0:
                usage_page = data
            elif tag == 0x7:
                report_size = data
            elif tag == 0x8:
                report_id = data
            elif tag == 0x9:
                report_count = data
        elif kind == 2 and tag == 0x0:  # Usage
            usages.append(data)
        elif kind == 0:  # main
            if tag == 0xA:  # Collection
                if depth == 0:
                    current = {
                        "usage_page": usage_page,
                        "usage": usages[0] if usages else 0,
                        # Report ids are global state that survives across
                        # collections, so only ids actually used by a main item
                        # inside this collection are recorded.
                        "reports": {},
                        "input": False,
                        "output": False,
                        "feature": False,
                    }
                    collections.append(current)
                depth += 1
            elif tag == 0xC:  # End Collection
                depth -= 1
                if depth == 0:
                    current = None
            elif tag == 0x8:
                note("input")
            elif tag == 0x9:
                note("output")
            elif tag == 0xB:
                note("feature")
            usages = []

    return collections


# --------------------------------------------------------------------------
# Reading what Linux knows
# --------------------------------------------------------------------------

def read(path: Path) -> str:
    try:
        return path.read_text(errors="replace").strip()
    except OSError:
        return ""


def usb_strings(node: Path) -> tuple[str, str]:
    """Manufacturer and product strings from the USB device above `node`."""
    for parent in node.resolve().parents:
        if (parent / "idVendor").exists():
            return read(parent / "manufacturer"), read(parent / "product")
    return "", ""


def collect() -> list[dict]:
    """One record per (vid, pid, interface) that Linux exposes as hidraw."""
    if not HIDRAW_CLASS.is_dir():
        sys.exit("No /sys/class/hidraw — this script needs Linux. See docs/ADDING-DEVICES.md.")

    by_key: dict[tuple, dict] = {}
    for entry in sorted(HIDRAW_CLASS.iterdir()):
        dev = entry / "device"
        uevent = read(dev / "uevent")
        m = re.search(r"HID_ID=([0-9A-Fa-f]+):([0-9A-Fa-f]+):([0-9A-Fa-f]+)", uevent)
        if not m:
            continue
        bus = int(m.group(1), 16)  # 0x03 USB, 0x05 Bluetooth, 0x18 I2C
        vid, pid = int(m.group(2), 16), int(m.group(3), 16)
        name = re.search(r"HID_NAME=(.*)", uevent)
        interface = read(dev / ".." / "bInterfaceNumber")
        interface = int(interface, 16) if interface else -1

        try:
            blob = (dev / "report_descriptor").read_bytes()
        except OSError:
            continue

        vendor_str, product_str = usb_strings(dev)
        key = (vid, pid, interface)
        record = by_key.setdefault(
            key,
            {
                "node": f"/dev/{entry.name}",
                "bus": bus,
                "vid": vid,
                "pid": pid,
                "interface": interface,
                "hid_name": name.group(1).strip() if name else "",
                "usb_vendor": vendor_str,
                "usb_product": product_str,
                "collections": [],
            },
        )
        for c in parse_descriptor(blob):
            if c not in record["collections"]:
                record["collections"].append(c)

    return list(by_key.values())


def already_supported() -> set[tuple[int, int]]:
    """(vid, pid) pairs already in the device table."""
    if not DEVICES_RS.exists():
        return set()
    text = DEVICES_RS.read_text()
    pairs = re.findall(r"vid:\s*0x([0-9a-fA-F]+),\s*\n\s*pid:\s*0x([0-9a-fA-F]+)", text)
    return {(int(v, 16), int(p, 16)) for v, p in pairs}


# --------------------------------------------------------------------------
# Interpreting it
# --------------------------------------------------------------------------

def report_span(collection: dict, kinds: str) -> int:
    """Size of the biggest report of these kinds in a collection, in bytes."""
    sizes = [
        info["bytes"]
        for info in collection["reports"].values()
        if info["kinds"] & set(kinds)
    ]
    return max(sizes) if sizes else 0


def classify(record: dict, siblings: list[dict]) -> dict:
    """Work out the config endpoint and what kind of device this is."""
    vendor = [c for c in record["collections"] if c["usage_page"] >= VENDOR_PAGE_MIN]

    # Configuration is written through Output or Feature reports, and the
    # configuration channel is normally the roomiest one: a device that takes
    # 17- or 64-byte packets of settings next to a collection that carries a
    # 2-byte hotkey event is telling you which is which.
    writable = sorted(
        (c for c in vendor if c["output"] or c["feature"]),
        key=lambda c: report_span(c, "OF"),
        reverse=True,
    )
    config = writable[0] if writable else (vendor[0] if vendor else None)

    reply = None
    if config is not None:
        if config["input"]:
            reply = config
        else:
            # Some devices answer on a different collection than the one they
            # are written to (the Compx mice do). Prefer an input-only vendor
            # collection whose reports are about as big as the config ones.
            wanted = report_span(config, "OF")
            readers = sorted(
                (c for c in vendor if c is not config and c["input"]),
                key=lambda c: abs(report_span(c, "I") - wanted),
            )
            reply = readers[0] if readers else config

    # The kind comes from any interface of the same product: the boot mouse or
    # keyboard collection says what the user is holding.
    kind = None
    for sib in siblings:
        for c in sib["collections"]:
            if c["usage_page"] == GENERIC_DESKTOP and c["usage"] == USAGE_MOUSE:
                kind = "Mouse"
            elif c["usage_page"] == GENERIC_DESKTOP and c["usage"] == USAGE_KEYBOARD and kind is None:
                kind = "Keyboard"

    return {
        "vendor_collections": vendor,
        "config": config,
        "reply": reply,
        "kind": kind,
    }


def rust_entry(record: dict, verdict: dict) -> str:
    config, reply = verdict["config"], verdict["reply"]
    kind = verdict["kind"] or "Mouse /* or Keyboard */"
    vendor = record["usb_vendor"] or "TODO vendor"
    model = record["usb_product"] or record["hid_name"] or "TODO model"
    cfg_page = f'0x{config["usage_page"]:04x}' if config else "0x0000 /* TODO */"
    rep_page = f'0x{reply["usage_page"]:04x}' if reply else cfg_page
    return f"""    Device {{
        vendor: "{vendor}",
        model: "{model}",
        vid: 0x{record["vid"]:04x},
        pid: 0x{record["pid"]:04x},
        kind: Kind::{kind},
        // Which protocol does it speak? Try an existing one first; if nothing
        // fits, see "Adding a new protocol" in docs/ADDING-DEVICES.md.
        protocol: Protocol::TODO,
        interface: {record["interface"]},
        config_usage_page: {cfg_page},
        reply_usage_page: {rep_page},
        connection: Connection::Wired, // or Receiver, for a 2.4 GHz dongle
        support: Support::Reported,    // Verified once it works on your desk
    }},"""


def describe(record: dict, verdict: dict, supported: set) -> str:
    lines = []
    title = " · ".join(x for x in (record["usb_vendor"], record["usb_product"]) if x)
    mark = "  [already supported]" if (record["vid"], record["pid"]) in supported else ""
    lines.append(f'{title or record["hid_name"]}{mark}')
    lines.append(
        f'  {record["vid"]:04x}:{record["pid"]:04x}  interface {record["interface"]}  {record["node"]}'
    )
    for c in record["collections"]:
        reports = (
            ", ".join(
                f'0x{rid:02x} {"".join(sorted(info["kinds"]))}{info["bytes"]}B'
                for rid, info in sorted(c["reports"].items())
            )
            or "—"
        )
        tag = ""
        if c is verdict["config"]:
            tag = "  <-- config: write here"
        elif c is verdict["reply"]:
            tag = "  <-- replies arrive here"
        elif c["usage_page"] >= VENDOR_PAGE_MIN:
            tag = "  (vendor)"
        lines.append(
            f'    usage page 0x{c["usage_page"]:04x} usage 0x{c["usage"]:02x}  {reports}{tag}'
        )
    if verdict["config"]:
        lines.append(
            f'  → config_usage_page: 0x{verdict["config"]["usage_page"]:04x}   '
            f'reply_usage_page: 0x{verdict["reply"]["usage_page"]:04x}'
        )
        if len(verdict["vendor_collections"]) > 1:
            lines.append(
                "    (a guess from report sizes — if writes do nothing, try another vendor collection)"
            )
    else:
        lines.append("  → no vendor collection here; this interface is not the config channel")
    lines.append("    report flags: I input (device answers) · O output · F feature (you write)")
    return "\n".join(lines)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--all", action="store_true", help="include interfaces with no vendor collection")
    ap.add_argument("--json", action="store_true", help="machine-readable output")
    ap.add_argument("--issue", action="store_true", help="markdown block for a device request issue")
    args = ap.parse_args()

    records = collect()
    if not records:
        print("No HID devices found. Is the device plugged in?", file=sys.stderr)
        return 1

    supported = already_supported()
    by_product: dict[tuple, list[dict]] = {}
    for r in records:
        by_product.setdefault((r["vid"], r["pid"]), []).append(r)

    results = []
    for record in records:
        verdict = classify(record, by_product[(record["vid"], record["pid"])])
        if not args.all:
            # HIDra talks to USB devices; a Bluetooth or I2C node is noise here.
            if record["bus"] != 0x03 or not verdict["vendor_collections"]:
                continue
        results.append((record, verdict))

    if args.json:
        def plain(record: dict) -> dict:
            out = dict(record)
            out["collections"] = [
                {
                    **{k: v for k, v in c.items() if k != "reports"},
                    "reports": {
                        f"0x{rid:02x}": {"bytes": i["bytes"], "kinds": sorted(i["kinds"])}
                        for rid, i in c["reports"].items()
                    },
                }
                for c in record["collections"]
            ]
            return out

        print(json.dumps(
            [
                {
                    **plain(record),
                    "kind": verdict["kind"],
                    "config_usage_page": verdict["config"]["usage_page"] if verdict["config"] else None,
                    "reply_usage_page": verdict["reply"]["usage_page"] if verdict["reply"] else None,
                    "already_supported": (record["vid"], record["pid"]) in supported,
                }
                for record, verdict in results
            ],
            indent=2,
        ))
        return 0

    if not results:
        print("No vendor-defined collections found. Re-run with --all to see every HID interface,")
        print("and check that the device is not connected through a KVM or a USB hub that hides it.")
        return 1

    if args.issue:
        print("<details><summary>probe-device.py output</summary>\n\n```")
    for record, verdict in results:
        print(describe(record, verdict, supported))
        print()
    if args.issue:
        print("```\n\n</details>")
        return 0

    print("Suggested entries for src-tauri/src/devices.rs")
    print("(check every field against docs/ADDING-DEVICES.md before sending a pull request)\n")
    for record, verdict in results:
        if (record["vid"], record["pid"]) in supported:
            continue
        print(rust_entry(record, verdict))
    return 0


if __name__ == "__main__":
    sys.exit(main())
