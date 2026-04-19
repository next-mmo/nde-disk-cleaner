<script lang="ts">
  import type { FileNode } from "./ipc";
  import { formatBytes, formatCount, formatDate, kindOf } from "./format";

  interface Props {
    node: FileNode | null;
    protectedReason: string | null;
    allowPermanentDelete: boolean;
    onreveal: (node: FileNode) => void;
    ontrash: (node: FileNode) => void;
    onopen: (node: FileNode) => void;
  }
  let { node, protectedReason, allowPermanentDelete, onreveal, ontrash, onopen }: Props = $props();
</script>

<div class="bar">
  {#if !node}
    <span class="empty">Hover a slice or pick a file</span>
  {:else}
    <div class="info">
      <div class="name" title={node.path}>{node.name}</div>
      <div class="meta">
        <span class="pill">{kindOf(node)}</span>
        <span>{formatBytes(node.size)}</span>
        {#if node.is_dir}
          <span>{formatCount(node.file_count)} files</span>
          <span>{formatCount(node.dir_count)} folders</span>
        {/if}
        {#if node.modified}
          <span>modified {formatDate(node.modified)}</span>
        {/if}
      </div>
      <div class="path" title={node.path}>{node.path}</div>
      {#if protectedReason}
        <div class="protected" title={protectedReason}>
          🔒 Protected — {protectedReason}
        </div>
      {/if}
    </div>
    <div class="actions">
      <button type="button" onclick={() => onopen(node)}>Open</button>
      <button type="button" onclick={() => onreveal(node)}>Reveal</button>
      <button
        type="button"
        class="danger"
        class:permanent={allowPermanentDelete}
        disabled={!!protectedReason}
        title={protectedReason ?? (allowPermanentDelete ? "Permanently delete this item (cannot be undone!)" : "Move this item to the Trash")}
        onclick={() => ontrash(node)}
      >
        {allowPermanentDelete ? "⚠ Delete Forever" : "Move to Trash"}
      </button>
    </div>
  {/if}
</div>

<style>
  .bar {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 10px 16px;
    border-top: 1px solid var(--border);
    background: var(--bg-elev);
    min-height: 64px;
  }
  .empty {
    color: var(--fg-muted);
    font-size: 12px;
  }
  .info {
    flex: 1;
    min-width: 0;
  }
  .name {
    font-weight: 600;
    font-size: 13px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .meta {
    display: flex;
    gap: 10px;
    font-size: 11px;
    color: var(--fg-dim);
    margin-top: 3px;
    align-items: center;
  }
  .pill {
    background: var(--bg-panel);
    border: 1px solid var(--border);
    border-radius: 3px;
    padding: 1px 6px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    font-size: 9px;
    font-weight: 700;
    color: var(--fg-dim);
  }
  .path {
    font-family: ui-monospace, Menlo, monospace;
    font-size: 10px;
    color: var(--fg-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    margin-top: 2px;
  }
  .actions {
    display: flex;
    gap: 6px;
    flex-shrink: 0;
  }
  .danger {
    color: var(--accent-hot);
  }
  .danger:hover:not(:disabled) {
    border-color: var(--accent-hot);
  }
  .danger:disabled {
    color: var(--fg-muted);
    cursor: not-allowed;
    opacity: 0.55;
  }
  .danger.permanent {
    background: rgba(255, 80, 80, 0.15);
    border-color: rgba(255, 80, 80, 0.4);
    font-weight: 600;
    animation: pulse-danger 1.5s ease-in-out infinite;
  }
  .danger.permanent:hover:not(:disabled) {
    background: rgba(255, 80, 80, 0.25);
    border-color: var(--accent-hot);
  }
  @keyframes pulse-danger {
    0%, 100% { box-shadow: 0 0 0 0 rgba(255, 80, 80, 0); }
    50% { box-shadow: 0 0 8px 2px rgba(255, 80, 80, 0.15); }
  }
  .protected {
    margin-top: 4px;
    font-size: 10px;
    color: var(--accent-hot);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
