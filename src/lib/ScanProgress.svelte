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
  <div class="pulse">
    <div class="dot"></div>
    <div class="dot"></div>
    <div class="dot"></div>
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
    padding: 10px 16px;
    border-bottom: 1px solid var(--border);
    background: linear-gradient(
      90deg,
      rgba(122, 162, 255, 0.08),
      transparent 40%
    );
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
  }
  .k:first-child {
    margin-left: 0;
  }
  .v {
    color: var(--fg);
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }
  .current {
    font-family: ui-monospace, Menlo, monospace;
    font-size: 11px;
    color: var(--fg-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-top: 3px;
  }
</style>
