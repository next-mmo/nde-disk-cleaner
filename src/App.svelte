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
    permanentDelete,
    isPathProtected,
    type FileNode,
  } from "./lib/ipc";
  import {
    state as appState,
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
  import Settings from "./lib/Settings.svelte";

  let unlistenProgress: (() => void) | null = null;
  let elapsedTimer: number | null = null;
  let elapsedMs = $state(0);
  let hovering = $state<FileNode | null>(null);
  let activeScanPath = $state<string | null>(null);
  let protectedReason = $state<string | null>(null);
  let settingsOpen = $state(false);
  let allowPermanentDelete = $state(false);

  // Display node = hovered preview, else currently-zoomed node.
  let displayedDetail = $derived(hovering ?? appState.selected ?? appState.zoomed);

  $effect(() => {
    const path = displayedDetail?.path;
    if (!path) {
      protectedReason = null;
      return;
    }
    let cancelled = false;
    isPathProtected(path)
      .then((r) => {
        if (!cancelled) protectedReason = r;
      })
      .catch(() => {
        if (!cancelled) protectedReason = null;
      });
    return () => {
      cancelled = true;
    };
  });

  onMount(async () => {
    unlistenProgress = await onScanProgress((p) => {
      appState.progress = p;
    });
    appState.volumes = await listVolumes();

    window.addEventListener("keydown", onKeydown);
  });

  onDestroy(() => {
    unlistenProgress?.();
    if (elapsedTimer) clearInterval(elapsedTimer);
    window.removeEventListener("keydown", onKeydown);
  });

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      if (appState.status === "scanning") cancelScan();
      else if (appState.zoomPath.length > 1) zoomTo(appState.zoomPath.length - 2);
    }
    if (e.key === "Backspace" && appState.zoomPath.length > 1) {
      e.preventDefault();
      zoomTo(appState.zoomPath.length - 2);
    }
  }

  function doSelect(path: string) {
    activeScanPath = path;
    resetScan();
  }

  async function doScan(path: string) {
    doSelect(path);
    appState.status = "scanning";
    appState.scanStartedAt = Date.now();
    elapsedMs = 0;
    elapsedTimer = window.setInterval(() => {
      elapsedMs = Date.now() - appState.scanStartedAt;
    }, 200);

    try {
      const tree = await startScan(path, appState.showHidden);
      appState.tree = tree;
      appState.zoomPath = [tree];
      appState.selected = tree;
      appState.status = "done";
      appState.scanFinishedAt = Date.now();
    } catch (err: any) {
      const msg = String(err);
      if (msg === "cancelled") {
        appState.status = "cancelled";
      } else {
        appState.status = "error";
        appState.error = msg;
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
    if (typeof picked === "string") doSelect(picked);
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
    if (allowPermanentDelete) {
      const ok = confirm(
        `⚠️ PERMANENTLY DELETE?\n\nThis CANNOT be undone!\n\n${node.path}\n\n${
          node.is_dir ? "This will recursively delete the entire folder and all its contents." : ""
        }`,
      );
      if (!ok) return;
      try {
        await permanentDelete(node.path);
        if (activeScanPath) await doScan(activeScanPath);
      } catch (e: any) {
        alert("Permanent delete failed: " + e);
      }
    } else {
      const ok = confirm(
        `Move to Trash?\n\n${node.path}\n\n${
          node.is_dir ? "This will move the entire folder." : ""
        }`,
      );
      if (!ok) return;
      try {
        await trashPath(node.path);
        if (activeScanPath) await doScan(activeScanPath);
      } catch (e: any) {
        alert("Failed: " + e);
      }
    }
  }

  function scanDuration(): string {
    const ms = appState.scanFinishedAt - appState.scanStartedAt;
    if (ms < 1000) return `${ms}ms`;
    return `${(ms / 1000).toFixed(1)}s`;
  }
</script>

<div class="app">
  <Sidebar
    volumes={appState.volumes}
    activePath={activeScanPath}
    onselect={doSelect}
    onbrowse={browseFolder}
  />

  <main class="main">
    <header class="titlebar">
      <div class="brand">
        <div class="logo">◉</div>
        <div>
          <div class="title">NDE Disk Cleaner</div>
          <div class="sub">
            {#if appState.status === "done" && appState.tree}
              Scanned in {scanDuration()} ·
              {appState.tree.file_count.toLocaleString()} files ·
              {appState.tree.dir_count.toLocaleString()} folders
            {:else if appState.status === "idle"}
              Pick a volume or folder to begin
            {:else if appState.status === "error"}
              <span class="err">Error: {appState.error}</span>
            {:else if appState.status === "cancelled"}
              Cancelled
            {/if}
          </div>
        </div>
      </div>
      <div class="controls">
        <label class="toggle">
          <input
            type="checkbox"
            bind:checked={appState.showHidden}
          />
          <span>Hidden files</span>
        </label>
        {#if activeScanPath && appState.status !== "scanning" && appState.status !== "idle"}
          <button onclick={() => doScan(activeScanPath!)}>Rescan</button>
        {/if}
        <button class="gear-btn" onclick={() => (settingsOpen = true)} title="Settings" aria-label="Open Settings">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path d="M6.5.5h3l.4 2 1.3.6 1.7-1.1 2.1 2.1-1.1 1.7.6 1.3 2 .4v3l-2 .4-0.6 1.3 1.1 1.7-2.1 2.1-1.7-1.1-1.3.6-.4 2h-3l-.4-2-1.3-.6-1.7 1.1L1 13.4l1.1-1.7-.6-1.3-2-.4v-3l2-.4.6-1.3L1 3.6 3.1 1.5l1.7 1.1 1.3-.6.4-2z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round"/>
            <circle cx="8" cy="8" r="2" stroke="currentColor" stroke-width="1.2"/>
          </svg>
        </button>
      </div>
    </header>

    {#if appState.status === "scanning"}
      <ScanProgress
        progress={appState.progress}
        {elapsedMs}
        oncancel={cancelScan}
      />
    {/if}

    {#if appState.zoomPath.length > 0}
      <Breadcrumb path={appState.zoomPath} onnavigate={zoomTo} />
    {/if}

    <div class="body">
      {#if appState.zoomed}
        <div class="chart">
          <Sunburst
            root={appState.zoomed}
            onhover={(n) => (hovering = n)}
            onselect={(n) => (appState.selected = n)}
          />
        </div>
        <FileList
          node={appState.zoomed}
          selected={appState.selected}
          onselect={(n) => (appState.selected = n)}
          onopen={openNode}
        />
      {:else if appState.status === "idle"}
        <div class="placeholder">
          <div class="placeholder-icon">◉</div>
          <div class="placeholder-title">
            {#if activeScanPath}
              Ready to scan
            {:else}
              No scan yet
            {/if}
          </div>
          <div class="placeholder-sub" style="margin-top: 10px;">
            {#if activeScanPath}
              <button class="primary" onclick={() => doScan(activeScanPath!)} style="padding: 10px 24px; font-size: 14px;">
                Start Scan
              </button>
              <div style="margin-top: 12px; font-family: monospace; font-size: 11px;">{activeScanPath}</div>
            {:else}
              Select a volume on the left, or
              <button class="inline" onclick={browseFolder}>
                pick a folder
              </button>
              .
            {/if}
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
      protectedReason={protectedReason}
      {allowPermanentDelete}
      onreveal={revealNode}
      onopen={openNode}
      ontrash={trashNode}
    />
  </main>
</div>

<Settings
  open={settingsOpen}
  {allowPermanentDelete}
  onclose={() => (settingsOpen = false)}
  ontogglePermanent={(v) => (allowPermanentDelete = v)}
/>

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
  .gear-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    padding: 0;
    border-radius: 8px;
    color: var(--fg-dim);
    background: transparent;
    border: 1px solid transparent;
    cursor: pointer;
    transition: background 80ms, color 80ms, border-color 80ms, transform 200ms;
  }
  .gear-btn:hover {
    background: var(--bg-panel);
    color: var(--fg);
    border-color: var(--border);
    transform: rotate(30deg);
  }
</style>
