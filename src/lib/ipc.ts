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
