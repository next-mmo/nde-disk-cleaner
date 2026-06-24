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
          <svg viewBox="0 0 24 24" width="11" height="11" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
            <rect x="5" y="11" width="14" height="9" rx="2"/>
            <path d="M8 11V7a4 4 0 0 1 8 0v4"/>
          </svg>
          Protected — {protectedReason}
        </div>
      {/if}
    </div>
    <div class="actions">
      <button type="button" class="ghost" onclick={() => onopen(node)}>Open</button>
      <button type="button" class="ghost" onclick={() => onreveal(node)}>Reveal</button>
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
    border-top: 1px solid var(--border-soft);
    background: var(--glass-toolbar);
    -webkit-backdrop-filter: blur(30px) saturate(180%);
    backdrop-filter: blur(30px) saturate(180%);
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
    letter-spacing: -0.005em;
  }
  .meta {
    display: flex;
    gap: 10px;
    font-size: 11px;
    color: var(--fg-dim);
    margin-top: 3px;
    align-items: center;
    flex-wrap: wrap;
  }
  .pill {
    background: var(--glass-card);
    border: 1px solid var(--border-soft);
    border-radius: 999px;
    padding: 1px 8px;
    text-transform: capitalize;
    letter-spacing: 0.01em;
    font-size: 10px;
    font-weight: 500;
    color: var(--fg-dim);
  }
  .path {
    font-family: ui-monospace, "SF Mono", Menlo, monospace;
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
    border-color: rgba(255, 69, 58, 0.35);
  }
  .danger:hover:not(:disabled) {
    background: rgba(255, 69, 58, 0.15);
    border-color: var(--accent-hot);
  }
  .danger:disabled {
    color: var(--fg-muted);
    cursor: not-allowed;
    opacity: 0.5;
  }
  .danger.permanent {
    background: linear-gradient(180deg, rgba(255, 69, 58, 0.22), rgba(255, 69, 58, 0.12));
    border-color: rgba(255, 69, 58, 0.55);
    font-weight: 600;
    animation: pulse-danger 1.6s ease-in-out infinite;
  }
  .danger.permanent:hover:not(:disabled) {
    background: linear-gradient(180deg, rgba(255, 69, 58, 0.32), rgba(255, 69, 58, 0.18));
    border-color: var(--accent-hot);
  }
  @keyframes pulse-danger {
    0%, 100% { box-shadow: 0 0 0 0 rgba(255, 69, 58, 0); }
    50% { box-shadow: 0 0 10px 1px rgba(255, 69, 58, 0.35); }
  }
  .protected {
    margin-top: 4px;
    font-size: 10px;
    color: var(--accent-hot);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }
</style>