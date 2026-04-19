import type { FileNode, ScanProgress, Volume } from "./ipc";

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
