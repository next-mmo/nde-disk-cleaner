<script lang="ts">
  import type { Volume } from "./ipc";
  import { formatBytes } from "./format";

  interface Props {
    volumes: Volume[];
    activePath: string | null;
    onselect: (path: string) => void;
    onbrowse: () => void;
  }

  let { volumes, activePath, onselect, onbrowse }: Props = $props();

  function usagePercent(v: Volume): number {
    if (v.total_bytes === 0) return 0;
    return Math.min(100, (v.used_bytes / v.total_bytes) * 100);
  }

  function volIcon(v: Volume): string {
    if (v.is_removable) return "externaldrive";
    if (v.name.toLowerCase().includes("system")) return "internal";
    return "internal";
  }
</script>

<aside class="sidebar scroll">
  <div class="head">
    <span class="label">Volumes</span>
  </div>

  {#each volumes as vol (vol.mount_point)}
    <button
      type="button"
      class="vol"
      class:active={activePath === vol.mount_point}
      onclick={() => onselect(vol.mount_point)}
    >
      <div class="vol-icon" class:removable={vol.is_removable}>
        <svg viewBox="0 0 24 24" width="22" height="22" fill="none">
          {#if vol.is_removable}
            <rect x="3" y="6" width="13" height="12" rx="2.5" stroke="currentColor" stroke-width="1.5" fill="rgba(10,132,255,0.12)"/>
            <rect x="16" y="9.5" width="5" height="5" rx="1" stroke="currentColor" stroke-width="1.5" fill="rgba(10,132,255,0.18)"/>
          {:else}
            <rect x="3" y="6" width="18" height="12" rx="2.5" stroke="currentColor" stroke-width="1.5" fill="rgba(255,255,255,0.04)"/>
            <circle cx="17.5" cy="12" r="0.9" fill="currentColor"/>
          {/if}
        </svg>
      </div>
      <div class="vol-body">
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
      </div>
    </button>
  {/each}

  <div class="head spaced">
    <span class="label">Locations</span>
  </div>
  <button type="button" class="browse" onclick={onbrowse}>
    <svg viewBox="0 0 24 24" width="16" height="16" fill="none">
      <path d="M3 7.5A2.5 2.5 0 0 1 5.5 5h3.4l1.7 1.8h7.9A2.5 2.5 0 0 1 21 9.3v7.2A2.5 2.5 0 0 1 18.5 19h-13A2.5 2.5 0 0 1 3 16.5v-9z" stroke="currentColor" stroke-width="1.4" fill="rgba(10,132,255,0.10)"/>
    </svg>
    <span>Choose folder…</span>
  </button>
</aside>

<style>
  .sidebar {
    width: 260px;
    border-right: 1px solid var(--border-soft);
    background: var(--glass-sidebar);
    -webkit-backdrop-filter: blur(40px) saturate(180%);
    backdrop-filter: blur(40px) saturate(180%);
    padding: 14px 12px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    flex-shrink: 0;
    overflow-y: auto;
  }
  .head {
    padding: 6px 10px 4px;
  }
  .head.spaced {
    margin-top: 14px;
  }
  .label {
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--fg-muted);
  }

  .vol {
    text-align: left;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 9px 10px;
    background: transparent;
    border: 1px solid transparent;
    border-radius: 9px;
    width: 100%;
    box-shadow: none;
    -webkit-backdrop-filter: none;
    backdrop-filter: none;
    transition: background 120ms ease, border-color 120ms ease;
  }
  .vol:hover {
    background: var(--glass-hover);
    border-color: var(--border-soft);
    box-shadow: none;
  }
  .vol.active {
    background: var(--accent-soft);
    border-color: rgba(10, 132, 255, 0.45);
    box-shadow:
      0 0 0 1px rgba(10, 132, 255, 0.25) inset,
      0 0 12px rgba(10, 132, 255, 0.18);
  }
  .vol-icon {
    width: 32px;
    height: 32px;
    border-radius: 7px;
    background: var(--glass-card);
    border: 1px solid var(--border-soft);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--fg-dim);
    flex-shrink: 0;
  }
  .vol-icon.removable {
    color: var(--accent);
  }
  .vol-body {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .vol-top {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 8px;
  }
  .name {
    font-weight: 500;
    font-size: 13px;
    color: var(--fg);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  .tag {
    font-size: 9px;
    font-weight: 700;
    background: linear-gradient(180deg, #5ac8fa, #0a84ff);
    color: #fff;
    padding: 1px 5px;
    border-radius: 4px;
    letter-spacing: 0.04em;
  }
  .fs {
    font-size: 10px;
    color: var(--fg-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    font-variant-numeric: tabular-nums;
  }
  .bar {
    height: 4px;
    background: rgba(255, 255, 255, 0.08);
    border-radius: 999px;
    overflow: hidden;
  }
  .fill {
    height: 100%;
    background: linear-gradient(90deg, #5ac8fa 0%, #0a84ff 60%, #bf5af2 100%);
    transition: width 200ms ease;
    box-shadow: 0 0 8px rgba(10, 132, 255, 0.5);
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

  .browse {
    margin: 0;
    text-align: left;
    padding: 8px 10px;
    display: inline-flex;
    align-items: center;
    gap: 8px;
    color: var(--fg-dim);
    background: transparent;
    border: 1px solid transparent;
    box-shadow: none;
    -webkit-backdrop-filter: none;
    backdrop-filter: none;
  }
  .browse:hover {
    background: var(--glass-hover);
    color: var(--fg);
    border-color: var(--border-soft);
    box-shadow: none;
  }
</style>