<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import { openPath, revealItemInDir } from "@tauri-apps/plugin-opener";
  import {
    cancelScan,
    listVolumes,
    onScanProgress,
    startScan,
    trashPath,
    type FileNode,
  } from "./lib/ipc";
  import {
    state,
    resetScan,
    zoomTo,
    zoomInto,
  } from "./lib/stores.svelte";
  import Sidebar from "./lib/Sidebar.svelte";
  import Sunburst from "./lib/Sunburst.svelte";
  import FileList from "./lib/FileList.svelte";
  import Breadcrumb from "./lib/Breadcrumb.svelte";
  import ScanProgress from "./lib/ScanProgress.svelte";
  import FileDetails from "./lib/FileDetails.svelte";

  let unlistenProgress: (() => void) | null = null;
  let elapsedTimer: number | null = null;
  let elapsedMs = $state(0);
  let hovering = $state<FileNode | null>(null);
  let activeScanPath = $state<string | null>(null);

  // Display node = hovered preview, else currently-zoomed node.
  let displayedDetail = $derived(hovering ?? state.selected ?? state.zoomed);

  onMount(async () => {
    unlistenProgress = await onScanProgress((p) => {
      state.progress = p;
    });
    state.volumes = await listVolumes();

    window.addEventListener("keydown", onKeydown);
  });

  onDestroy(() => {
    unlistenProgress?.();
    if (elapsedTimer) clearInterval(elapsedTimer);
    window.removeEventListener("keydown", onKeydown);
  });

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      if (state.status === "scanning") cancelScan();
      else if (state.zoomPath.length > 1) zoomTo(state.zoomPath.length - 2);
    }
    if (e.key === "Backspace" && state.zoomPath.length > 1) {
      e.preventDefault();
      zoomTo(state.zoomPath.length - 2);
    }
  }

  async function doScan(path: string) {
    resetScan();
    activeScanPath = path;
    state.status = "scanning";
    state.scanStartedAt = Date.now();
    elapsedMs = 0;
    elapsedTimer = window.setInterval(() => {
      elapsedMs = Date.now() - state.scanStartedAt;
    }, 200);

    try {
      const tree = await startScan(path, state.showHidden);
      state.tree = tree;
      state.zoomPath = [tree];
      state.selected = tree;
      state.status = "done";
      state.scanFinishedAt = Date.now();
    } catch (err: any) {
      const msg = String(err);
      if (msg === "cancelled") {
        state.status = "cancelled";
      } else {
        state.status = "error";
        state.error = msg;
      }
    } finally {
      if (elapsedTimer) {
        clearInterval(elapsedTimer);
        elapsedTimer = null;
      }
    }
  }

  async function browseFolder() {
    const picked = await openDialog({ directory: true, multiple: false });
    if (typeof picked === "string") await doScan(picked);
  }

  async function revealNode(node: FileNode) {
    try {
      await revealItemInDir(node.path);
    } catch (e) {
      console.error("reveal failed", e);
    }
  }

  async function openNode(node: FileNode) {
    try {
      if (node.is_dir) {
        zoomInto(node);
      } else {
        await openPath(node.path);
      }
    } catch (e) {
      console.error("open failed", e);
    }
  }

  async function trashNode(node: FileNode) {
    const ok = confirm(
      `Move to Trash?\n\n${node.path}\n\n${
        node.is_dir ? "This will move the entire folder." : ""
      }`,
    );
    if (!ok) return;
    try {
      await trashPath(node.path);
      // Re-scan current root to refresh the tree.
      if (activeScanPath) await doScan(activeScanPath);
    } catch (e: any) {
      alert("Failed: " + e);
    }
  }

  function scanDuration(): string {
    const ms = state.scanFinishedAt - state.scanStartedAt;
    if (ms < 1000) return `${ms}ms`;
    return `${(ms / 1000).toFixed(1)}s`;
  }
</script>

<div class="app">
  <Sidebar
    volumes={state.volumes}
    activePath={activeScanPath}
    onscan={doScan}
    onbrowse={browseFolder}
  />

  <main class="main">
    <header class="titlebar">
      <div class="brand">
        <div class="logo">◉</div>
        <div>
          <div class="title">Disk Inspector</div>
          <div class="sub">
            {#if state.status === "done" && state.tree}
              Scanned in {scanDuration()} ·
              {state.tree.file_count.toLocaleString()} files ·
              {state.tree.dir_count.toLocaleString()} folders
            {:else if state.status === "idle"}
              Pick a volume or folder to begin
            {:else if state.status === "error"}
              <span class="err">Error: {state.error}</span>
            {:else if state.status === "cancelled"}
              Cancelled
            {/if}
          </div>
        </div>
      </div>
      <div class="controls">
        <label class="toggle">
          <input
            type="checkbox"
            bind:checked={state.showHidden}
          />
          <span>Hidden files</span>
        </label>
        {#if activeScanPath && state.status !== "scanning"}
          <button onclick={() => doScan(activeScanPath!)}>Rescan</button>
        {/if}
      </div>
    </header>

    {#if state.status === "scanning"}
      <ScanProgress
        progress={state.progress}
        {elapsedMs}
        oncancel={cancelScan}
      />
    {/if}

    {#if state.zoomPath.length > 0}
      <Breadcrumb path={state.zoomPath} onnavigate={zoomTo} />
    {/if}

    <div class="body">
      {#if state.zoomed}
        <div class="chart">
          <Sunburst
            root={state.zoomed}
            onhover={(n) => (hovering = n)}
            onselect={(n) => (state.selected = n)}
          />
        </div>
        <FileList
          node={state.zoomed}
          selected={state.selected}
          onselect={(n) => (state.selected = n)}
          onopen={openNode}
        />
      {:else if state.status === "idle"}
        <div class="placeholder">
          <div class="placeholder-icon">◉</div>
          <div class="placeholder-title">No scan yet</div>
          <div class="placeholder-sub">
            Select a volume on the left, or
            <button class="inline" onclick={browseFolder}>
              pick a folder
            </button>
            .
          </div>
          <div class="hint">
            <kbd>Esc</kbd> cancel/back · <kbd>⌫</kbd> navigate up · double-click
            to open
          </div>
        </div>
      {/if}
    </div>

    <FileDetails
      node={displayedDetail}
      onreveal={revealNode}
      onopen={openNode}
      ontrash={trashNode}
    />
  </main>
</div>

<style>
  .app {
    display: flex;
    height: 100%;
    width: 100%;
  }
  .main {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .titlebar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border);
    background: var(--bg-elev);
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .logo {
    font-size: 22px;
    color: var(--accent);
    line-height: 1;
  }
  .title {
    font-weight: 700;
    font-size: 14px;
  }
  .sub {
    font-size: 11px;
    color: var(--fg-muted);
  }
  .err {
    color: var(--accent-hot);
  }
  .controls {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .toggle {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--fg-dim);
    cursor: pointer;
  }
  .body {
    flex: 1;
    display: flex;
    min-height: 0;
  }
  .chart {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 24px;
    min-width: 0;
    min-height: 0;
  }
  .placeholder {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    color: var(--fg-dim);
  }
  .placeholder-icon {
    font-size: 48px;
    color: var(--bg-panel);
  }
  .placeholder-title {
    font-size: 18px;
    font-weight: 600;
    color: var(--fg);
  }
  .placeholder-sub {
    color: var(--fg-dim);
  }
  .inline {
    background: transparent;
    border: none;
    color: var(--accent);
    padding: 0;
    text-decoration: underline;
    cursor: pointer;
  }
  .hint {
    margin-top: 24px;
    color: var(--fg-muted);
    font-size: 11px;
    display: flex;
    gap: 6px;
    align-items: center;
  }
</style>
