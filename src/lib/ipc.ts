import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface FileNode {
  name: string;
  path: string;
  size: number;
  is_dir: boolean;
  file_count: number;
  dir_count: number;
  children?: FileNode[];
  modified?: number;
  extension?: string;
  depth: number;
}

export interface ScanProgress {
  scanned_files: number;
  scanned_bytes: number;
  current_path: string;
}

export interface Volume {
  name: string;
  mount_point: string;
  total_bytes: number;
  available_bytes: number;
  used_bytes: number;
  file_system: string;
  is_removable: boolean;
}

export function listVolumes(): Promise<Volume[]> {
  // Browser-preview fallback: return sample disks so the UI is inspectable
  // without launching the full Tauri runtime.
  if (typeof window === "undefined" || !("__TAURI_INTERNALS__" in window)) {
    const TB = 1024 ** 4;
    return Promise.resolve([
      {
        name: "Macintosh HD",
        mount_point: "/",
        total_bytes: 2 * TB,
        available_bytes: 812 * TB / 1024,
        used_bytes: 2 * TB - 812 * TB / 1024,
        file_system: "apfs",
        is_removable: false,
      },
      {
        name: "Data",
        mount_point: "/System/Volumes/Data",
        total_bytes: 2 * TB,
        available_bytes: 812 * TB / 1024,
        used_bytes: 2 * TB - 812 * TB / 1024,
        file_system: "apfs",
        is_removable: false,
      },
      {
        name: "NDE USB",
        mount_point: "/Volumes/NDE USB",
        total_bytes: 64 * 1024 ** 3,
        available_bytes: 21 * 1024 ** 3,
        used_bytes: 43 * 1024 ** 3,
        file_system: "exfat",
        is_removable: true,
      },
    ]);
  }
  return invoke("list_volumes");
}

export function startScan(
  path: string,
  showHidden: boolean,
  maxDepth = 8,
): Promise<FileNode> {
  return invoke("start_scan", { path, showHidden, maxDepth });
}

export function cancelScan(): Promise<void> {
  return invoke("cancel_scan");
}

export function trashPath(path: string): Promise<void> {
  return invoke("trash_path", { path });
}

export function isPathProtected(path: string): Promise<string | null> {
  return invoke("is_path_protected", { path });
}

export function homeDir(): Promise<string | null> {
  return invoke("home_dir");
}

export function onScanProgress(
  cb: (p: ScanProgress) => void,
): Promise<UnlistenFn> {
  if (typeof window === "undefined" || !("__TAURI_INTERNALS__" in window)) {
    return Promise.resolve(() => {});
  }
  return listen<ScanProgress>("scan:progress", (e) => cb(e.payload));
}

export function permanentDelete(path: string): Promise<void> {
  return invoke("permanent_delete", { path });
}

export interface UpdateInfo {
  current_version: string;
  latest_version: string;
  has_update: boolean;
  release_url: string;
  release_notes: string;
}

export function checkForUpdates(): Promise<UpdateInfo> {
  return invoke("check_for_updates");
}
