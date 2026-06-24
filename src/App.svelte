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
        <div class="logo">
          <svg viewBox="0 0 24 24" width="22" height="22" fill="none">
            <defs>
              <linearGradient id="logo-grad" x1="0" y1="0" x2="1" y2="1">
                <stop offset="0%" stop-color="#5ac8fa"/>
                <stop offset="55%" stop-color="#0a84ff"/>
                <stop offset="100%" stop-color="#bf5af2"/>
              </linearGradient>
            </defs>
            <path
              d="M12 2.5l8.5 4.9v9.2L12 21.5 3.5 16.6V7.4L12 2.5z"
              fill="url(#logo-grad)"
              opacity="0.95"
            />
            <path
              d="M12 7.2c-2.6 0-4.8 2.1-4.8 4.8s2.1 4.8 4.8 4.8 4.8-2.1 4.8-4.8"
              stroke="rgba(255,255,255,0.9)"
              stroke-width="1.4"
              stroke-linecap="round"
              fill="none"
            />
          </svg>
        </div>
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
          <span class="switch-tahoe">
            <input type="checkbox" bind:checked={appState.showHidden} />
            <span class="track"><span class="thumb"></span></span>
          </span>
          <span>Hidden files</span>
        </label>
        {#if activeScanPath && appState.status !== "scanning" && appState.status !== "idle"}
          <button onclick={() => doScan(activeScanPath!)}>Rescan</button>
        {/if}
        <button
          class="icon-btn"
          onclick={() => (settingsOpen = true)}
          title="Settings"
          aria-label="Open Settings"
        >
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
          <div class="placeholder-glyph">
            <svg viewBox="0 0 64 64" width="64" height="64" fill="none">
              <defs>
                <linearGradient id="hero-grad" x1="0" y1="0" x2="1" y2="1">
                  <stop offset="0%" stop-color="#5ac8fa"/>
                  <stop offset="50%" stop-color="#0a84ff"/>
                  <stop offset="100%" stop-color="#bf5af2"/>
                </linearGradient>
              </defs>
              <circle cx="32" cy="32" r="22" stroke="url(#hero-grad)" stroke-width="2.2" fill="rgba(10,132,255,0.08)"/>
              <path d="M22 32a10 10 0 1 0 20 0" stroke="url(#hero-grad)" stroke-width="2.2" stroke-linecap="round" fill="none"/>
              <circle cx="22" cy="32" r="2" fill="#5ac8fa"/>
            </svg>
          </div>
          <div class="placeholder-title">
            {#if activeScanPath}
              Ready to scan
            {:else}
              No scan yet
            {/if}
          </div>
          <div class="placeholder-sub">
            {#if activeScanPath}
              <button class="primary" onclick={() => doScan(activeScanPath!)}>
                Start Scan
              </button>
              <div class="path-line">{activeScanPath}</div>
            {:else}
              Select a volume on the left, or
              <button class="link" onclick={browseFolder}>
                pick a folder
              </button>.
            {/if}
          </div>
          <div class="hint">
            <kbd>Esc</kbd> cancel/back · <kbd>⌫</kbd> navigate up · double-click to open
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
    min-height: 0;
  }

  /* ── Titlebar (toolbar) ──────────────────────────────── */
  .titlebar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 10px 14px 10px 90px; /* leave room for traffic lights */
    background: var(--glass-toolbar);
    -webkit-backdrop-filter: blur(30px) saturate(180%);
    backdrop-filter: blur(30px) saturate(180%);
    border-bottom: 1px solid var(--border-soft);
    user-select: none;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
  }
  .logo {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: 8px;
    background: var(--glass-card);
    border: 1px solid var(--border);
    box-shadow: 0 1px 0 rgba(255, 255, 255, 0.05) inset;
  }
  .title {
    font-weight: 600;
    font-size: 13px;
    letter-spacing: -0.01em;
    color: var(--fg);
  }
  .sub {
    font-size: 11px;
    color: var(--fg-muted);
    margin-top: 1px;
    font-variant-numeric: tabular-nums;
  }
  .err {
    color: var(--accent-hot);
  }

  .controls {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .toggle {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    color: var(--fg-dim);
    cursor: pointer;
  }

  /* macOS Tahoe-style iOS toggle */
  .switch-tahoe {
    position: relative;
    display: inline-block;
    width: 38px;
    height: 22px;
    flex-shrink: 0;
  }
  .switch-tahoe input {
    opacity: 0;
    width: 0;
    height: 0;
    position: absolute;
  }
  .switch-tahoe .track {
    position: absolute;
    inset: 0;
    background: rgba(120, 120, 128, 0.36);
    border-radius: 999px;
    transition: background 200ms ease;
  }
  .switch-tahoe .thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: #fff;
    box-shadow:
      0 2px 4px rgba(0, 0, 0, 0.3),
      0 1px 1px rgba(0, 0, 0, 0.2),
      0 0 0 0.5px rgba(0, 0, 0, 0.05);
    transition: transform 220ms cubic-bezier(0.4, 0, 0.2, 1), background 200ms;
  }
  .switch-tahoe input:checked + .track {
    background: var(--accent-ok);
  }
  .switch-tahoe input:checked + .track .thumb {
    transform: translateX(16px);
  }

  .icon-btn {
    width: 30px;
    height: 30px;
    padding: 0;
    border-radius: 8px;
    color: var(--fg-dim);
    background: transparent;
    border: 1px solid transparent;
    box-shadow: none;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }
  .icon-btn:hover {
    background: var(--glass-hover);
    color: var(--fg);
    border-color: var(--border);
    box-shadow: none;
  }

  /* ── Body ───────────────────────────────────────────── */
  .body {
    flex: 1;
    display: flex;
    min-height: 0;
    min-width: 0;
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

  /* ── Placeholder ────────────────────────────────────── */
  .placeholder {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: var(--fg-dim);
    padding: 32px;
  }
  .placeholder-glyph {
    filter: drop-shadow(0 8px 24px rgba(10, 132, 255, 0.25));
  }
  .placeholder-title {
    font-size: 20px;
    font-weight: 600;
    color: var(--fg);
    letter-spacing: -0.02em;
    margin-top: 4px;
  }
  .placeholder-sub {
    color: var(--fg-dim);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
  }
  .placeholder-sub .primary {
    padding: 9px 22px;
    font-size: 13px;
  }
  .path-line {
    margin-top: 2px;
    font-family: ui-monospace, "SF Mono", Menlo, monospace;
    font-size: 11px;
    color: var(--fg-muted);
    padding: 4px 10px;
    background: var(--glass-input);
    border: 1px solid var(--border-soft);
    border-radius: 6px;
  }
  .link {
    background: transparent;
    border: none;
    box-shadow: none;
    color: var(--accent);
    padding: 0;
    cursor: pointer;
    text-decoration: none;
    border-radius: 0;
    font-weight: 500;
  }
  .link:hover {
    background: transparent;
    border: none;
    box-shadow: none;
    text-decoration: underline;
  }
  .hint {
    margin-top: 18px;
    color: var(--fg-muted);
    font-size: 11px;
    display: flex;
    gap: 8px;
    align-items: center;
    flex-wrap: wrap;
    justify-content: center;
  }
</style>
