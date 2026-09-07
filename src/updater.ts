// In-app updates.
//
// `tauri-plugin-updater` does the work: it reads the signed `latest.json` on
// the newest release, verifies the artifact against the key in tauri.conf.json,
// and installs it the way the format requires — in place for an AppImage or a
// Windows install, through dpkg/rpm under a system authorisation prompt for a
// Linux package. The backend only tells us whether this copy is one it can
// install into at all, so a source build gets a download link instead.

import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { getVersion } from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";
import { isTauri } from "./backend";

const RELEASES = "https://github.com/luizjr/HIDra/releases";

export interface UpdateSupport {
  /** Package format this copy came from, or null when HIDra did not package it. */
  bundle: string | null;
  /** Whether HIDra can install an update itself on this system. */
  can_install: boolean;
  /** Release asset for this system, with `{version}` still to fill in. */
  asset_pattern: string;
}

export interface AvailableUpdate {
  version: string;
  notes?: string;
  /** The plugin handle, only present when the plugin can do the install. */
  handle?: Update;
}

const UNKNOWN: UpdateSupport = {
  bundle: null,
  can_install: false,
  asset_pattern: "",
};

export async function currentVersion(): Promise<string> {
  if (!isTauri()) return "dev";
  try {
    return await getVersion();
  } catch {
    return "dev";
  }
}

export async function updateSupport(): Promise<UpdateSupport> {
  if (!isTauri()) return UNKNOWN;
  try {
    return await invoke<UpdateSupport>("update_support");
  } catch {
    return UNKNOWN;
  }
}

/** The newest release, or null when this is already it. */
export async function checkForUpdate(): Promise<AvailableUpdate | null> {
  if (!isTauri()) return null;
  const update = await check();
  if (!update) return null;
  return { version: update.version, notes: update.body, handle: update };
}

export function releaseNotesUrl(version: string): string {
  return `${RELEASES}/tag/v${version}`;
}

export function downloadUrl(support: UpdateSupport, version: string): string {
  if (!support.asset_pattern) return `${RELEASES}/latest`;
  const asset = support.asset_pattern.replace("{version}", version);
  return `${RELEASES}/download/v${version}/${asset}`;
}

/**
 * Install an update and relaunch. `onProgress` receives 0..1 while downloading,
 * or null when the server sends no size to measure against.
 */
export async function installUpdate(
  update: AvailableUpdate,
  onProgress: (fraction: number | null) => void,
): Promise<void> {
  if (!update.handle) throw new Error("atualização indisponível neste formato");

  let total = 0;
  let received = 0;
  await update.handle.downloadAndInstall((event) => {
    if (event.event === "Started") {
      total = event.data.contentLength ?? 0;
      onProgress(total ? 0 : null);
    } else if (event.event === "Progress") {
      received += event.data.chunkLength;
      onProgress(total ? Math.min(1, received / total) : null);
    } else if (event.event === "Finished") {
      onProgress(1);
    }
  });
  await relaunch();
}
