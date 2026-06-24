<script lang="ts">
  import type { ScanProgress } from "./ipc";
  import { formatBytes, formatCount } from "./format";

  interface Props {
    progress: ScanProgress | null;
    elapsedMs: number;
    oncancel: () => void;
  }
  let { progress, elapsedMs, oncancel }: Props = $props();

  let elapsed = $derived(formatElapsed(elapsedMs));

  function formatElapsed(ms: number): string {
    const s = Math.floor(ms / 1000);
    const m = Math.floor(s / 60);
    const rs = s % 60;
    if (m === 0) return `${s}s`;
    return `${m}m ${rs}s`;
  }
</script>

<div class="progress">
  <div class="badge">
    <div class="pulse">
      <div class="dot"></div>
      <div class="dot"></div>
      <div class="dot"></div>
    </div>
    <span class="badge-label">Scanning</span>
  </div>
  <div class="stats">
    <div class="row">
      <span class="k">Files</span>
      <span class="v">{formatCount(progress?.scanned_files ?? 0)}</span>
      <span class="k">Size</span>
      <span class="v">{formatBytes(progress?.scanned_bytes ?? 0)}</span>
      <span class="k">Elapsed</span>
      <span class="v">{elapsed}</span>
    </div>
    {#if progress?.current_path}
      <div class="current" title={progress.current_path}>
        {progress.current_path}
      </div>
    {/if}
  </div>
  <button onclick={oncancel}>Cancel</button>
</div>

<style>
  .progress {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 9px 16px;
    border-bottom: 1px solid var(--border-soft);
    background: var(--glass-toolbar);
    -webkit-backdrop-filter: blur(30px) saturate(180%);
    backdrop-filter: blur(30px) saturate(180%);
  }
  .badge {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 4px 10px 4px 8px;
    background: var(--accent-soft);
    border: 1px solid rgba(10, 132, 255, 0.35);
    border-radius: 999px;
    color: var(--accent);
    font-weight: 600;
    font-size: 11px;
    letter-spacing: 0.01em;
  }
  .badge-label {
    text-transform: uppercase;
    letter-spacing: 0.06em;
    font-size: 10px;
  }
  .pulse {
    display: flex;
    gap: 3px;
  }
  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--accent);
    box-shadow: 0 0 6px var(--accent-glow);
    animation: bounce 900ms infinite ease-in-out;
  }
  .dot:nth-child(2) {
    animation-delay: 150ms;
  }
  .dot:nth-child(3) {
    animation-delay: 300ms;
  }
  @keyframes bounce {
    0%, 80%, 100% { transform: scale(0.6); opacity: 0.4; }
    40% { transform: scale(1); opacity: 1; }
  }
  .stats {
    flex: 1;
    min-width: 0;
  }
  .row {
    display: flex;
    gap: 6px;
    align-items: baseline;
    font-size: 12px;
  }
  .k {
    color: var(--fg-muted);
    margin-left: 10px;
    text-transform: uppercase;
    font-size: 10px;
    letter-spacing: 0.05em;
    font-weight: 500;
  }
  .k:first-child {
    margin-left: 0;
  }
  .v {
    color: var(--fg);
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    font-size: 12px;
  }
  .current {
    font-family: ui-monospace, "SF Mono", Menlo, monospace;
    font-size: 11px;
    color: var(--fg-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-top: 3px;
  }
</style>