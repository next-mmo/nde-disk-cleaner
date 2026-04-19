<script lang="ts">
  import type { Volume } from "./ipc";
  import { formatBytes } from "./format";

  interface Props {
    volumes: Volume[];
    activePath: string | null;
    onscan: (path: string) => void;
    onbrowse: () => void;
  }

  let { volumes, activePath, onscan, onbrowse }: Props = $props();

  function usagePercent(v: Volume): number {
    if (v.total_bytes === 0) return 0;
    return Math.min(100, (v.used_bytes / v.total_bytes) * 100);
  }
</script>

<aside class="sidebar scroll">
  <div class="head">
    <span class="label">VOLUMES</span>
  </div>

  {#each volumes as vol (vol.mount_point)}
    <button
      type="button"
      class="vol"
      class:active={activePath === vol.mount_point}
      onclick={() => onscan(vol.mount_point)}
    >
      <div class="vol-top">
        <span class="name">
          {vol.name || vol.mount_point}
          {#if vol.is_removable}<span class="tag">USB</span>{/if}
        </span>
        <span class="fs">{vol.file_system}</span>
      </div>
      <div class="bar">
        <div class="fill" style:width="{usagePercent(vol)}%"></div>
      </div>
      <div class="meta">
        <span>{formatBytes(vol.used_bytes)} used</span>
        <span class="sep">·</span>
        <span>{formatBytes(vol.available_bytes)} free</span>
      </div>
      <div class="mount">{vol.mount_point}</div>
    </button>
  {/each}

  <div class="head spaced">
    <span class="label">CUSTOM</span>
  </div>
  <button type="button" class="browse" onclick={onbrowse}>
    Choose folder…
  </button>
</aside>

<style>
  .sidebar {
    width: 260px;
    border-right: 1px solid var(--border);
    background: var(--bg-elev);
    padding: 12px 10px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    flex-shrink: 0;
  }
  .head {
    padding: 4px 6px 2px;
  }
  .head.spaced {
    margin-top: 12px;
  }
  .label {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.1em;
    color: var(--fg-muted);
  }
  .vol {
    text-align: left;
    display: flex;
    flex-direction: column;
    gap: 5px;
    padding: 10px 12px;
    background: transparent;
    border: 1px solid transparent;
    border-radius: 8px;
    width: 100%;
  }
  .vol:hover {
    background: var(--bg-panel);
    border-color: var(--border);
  }
  .vol.active {
    background: var(--bg-panel);
    border-color: #3a4055;
  }
  .vol-top {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 8px;
  }
  .name {
    font-weight: 600;
    font-size: 13px;
  }
  .tag {
    font-size: 9px;
    font-weight: 700;
    background: var(--accent);
    color: #0a0e1a;
    padding: 1px 4px;
    border-radius: 3px;
    margin-left: 4px;
    vertical-align: middle;
  }
  .fs {
    font-size: 10px;
    color: var(--fg-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .bar {
    height: 4px;
    background: #0c0e18;
    border-radius: 999px;
    overflow: hidden;
  }
  .fill {
    height: 100%;
    background: linear-gradient(90deg, var(--accent), #b191ff);
    transition: width 200ms;
  }
  .meta {
    display: flex;
    gap: 4px;
    font-size: 11px;
    color: var(--fg-dim);
    font-variant-numeric: tabular-nums;
  }
  .sep {
    color: var(--fg-muted);
  }
  .mount {
    font-size: 10px;
    color: var(--fg-muted);
    font-family: ui-monospace, Menlo, monospace;
    word-break: break-all;
  }
  .browse {
    margin: 0 4px;
    text-align: left;
    padding: 10px 12px;
  }
</style>
