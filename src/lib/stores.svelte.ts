import type { FileNode, ScanProgress, Volume } from "./ipc";

/** Cached scan results keyed by root scan path. */
const scanCache = new Map<string, {
  tree: FileNode;
  zoomPath: FileNode[];
  selected: FileNode | null;
  scanStartedAt: number;
  scanFinishedAt: number;
}>();

export type ScanStatus = "idle" | "scanning" | "done" | "error" | "cancelled";

function createState() {
  let status = $state<ScanStatus>("idle");
  let error = $state<string | null>(null);
  let progress = $state<ScanProgress | null>(null);
  let tree = $state<FileNode | null>(null);
  /** Path from root to the currently-zoomed folder (inclusive of root). */
  let zoomPath = $state<FileNode[]>([]);
  let selected = $state<FileNode | null>(null);
  let volumes = $state<Volume[]>([]);
  let showHidden = $state(false);
  let scanStartedAt = $state<number>(0);
  let scanFinishedAt = $state<number>(0);

  return {
    get status() { return status; },
    set status(v) { status = v; },
    get error() { return error; },
    set error(v) { error = v; },
    get progress() { return progress; },
    set progress(v) { progress = v; },
    get tree() { return tree; },
    set tree(v) { tree = v; },
    get zoomPath() { return zoomPath; },
    set zoomPath(v) { zoomPath = v; },
    get zoomed() { return zoomPath.at(-1) ?? null; },
    get selected() { return selected; },
    set selected(v) { selected = v; },
    get volumes() { return volumes; },
    set volumes(v) { volumes = v; },
    get showHidden() { return showHidden; },
    set showHidden(v) { showHidden = v; },
    get scanStartedAt() { return scanStartedAt; },
    set scanStartedAt(v) { scanStartedAt = v; },
    get scanFinishedAt() { return scanFinishedAt; },
    set scanFinishedAt(v) { scanFinishedAt = v; },
  };
}

export const state = createState();

/** Navigate into a child folder (zoom the sunburst). */
export function zoomInto(node: FileNode) {
  if (!node.is_dir) return;
  state.zoomPath = [...state.zoomPath, node];
  state.selected = node;
}

/** Jump to a specific depth on the breadcrumb. */
export function zoomTo(index: number) {
  state.zoomPath = state.zoomPath.slice(0, index + 1);
  state.selected = state.zoomPath.at(-1) ?? null;
}

export function resetScan() {
  state.status = "idle";
  state.error = null;
  state.progress = null;
  state.tree = null;
  state.zoomPath = [];
  state.selected = null;
}

/** Save the current scan result into cache (keyed by root path). */
export function cacheScan(rootPath: string) {
  if (!state.tree) return;
  scanCache.set(rootPath, {
    tree: state.tree,
    zoomPath: [...state.zoomPath],
    selected: state.selected,
    scanStartedAt: state.scanStartedAt,
    scanFinishedAt: state.scanFinishedAt,
  });
}

/** Restore a cached scan if one exists for `rootPath`. Returns true on hit. */
export function restoreCachedScan(rootPath: string): boolean {
  const cached = scanCache.get(rootPath);
  if (!cached) return false;
  state.tree = cached.tree;
  state.zoomPath = cached.zoomPath;
  state.selected = cached.selected;
  state.scanStartedAt = cached.scanStartedAt;
  state.scanFinishedAt = cached.scanFinishedAt;
  state.status = "done";
  state.error = null;
  state.progress = null;
  return true;
}

/** Invalidate (remove) cache for a specific root path. */
export function invalidateCache(rootPath: string) {
  scanCache.delete(rootPath);
}

/**
 * Remove a node from the in-memory tree by its path.
 * Walks up the tree recalculating sizes/counts so the sunburst stays
 * accurate without a full rescan.
 */
export function removeNodeByPath(targetPath: string) {
  if (!state.tree) return;

  // If the deleted node IS the tree root, just reset.
  if (state.tree.path === targetPath) {
    resetScan();
    return;
  }

  // DFS to find and remove the node, returning the removed node for
  // size/count adjustment.
  function remove(parent: FileNode): FileNode | null {
    if (!parent.children) return null;
    const idx = parent.children.findIndex((c) => c.path === targetPath);
    if (idx !== -1) {
      const [removed] = parent.children.splice(idx, 1);
      return removed!;
    }
    for (const child of parent.children) {
      const found = remove(child);
      if (found) return found;
    }
    return null;
  }

  const removed = remove(state.tree);
  if (!removed) return;

  // Walk from root down to every ancestor and subtract sizes/counts.
  function adjust(node: FileNode): boolean {
    if (node.path === targetPath) return true;
    if (!node.children) return false;
    for (const child of node.children) {
      if (adjust(child)) {
        node.size -= removed!.size;
        node.file_count -= removed!.is_dir ? removed!.file_count : 1;
        node.dir_count -= removed!.is_dir ? removed!.dir_count + 1 : 0;
        return true;
      }
    }
    return false;
  }
  adjust(state.tree);

  // If the selected node was the removed one, clear selection.
  if (state.selected?.path === targetPath) {
    state.selected = state.zoomPath.at(-1) ?? state.tree;
  }

  // If any zoom-path node was removed, trim it.
  const badIdx = state.zoomPath.findIndex((n) => n.path === targetPath);
  if (badIdx !== -1) {
    state.zoomPath = state.zoomPath.slice(0, badIdx);
    state.selected = state.zoomPath.at(-1) ?? state.tree;
  }

  // Trigger reactivity by reassigning tree reference.
  state.tree = { ...state.tree };
  // Also update the cache.
  const rootPath = state.zoomPath[0]?.path;
  if (rootPath) cacheScan(rootPath);
}
