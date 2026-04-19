<script lang="ts">
  import type { FileNode } from "./ipc";
  import { formatBytes } from "./format";

  interface Props {
    path: FileNode[];
    onnavigate: (index: number) => void;
  }
  let { path, onnavigate }: Props = $props();
</script>

<nav class="crumbs" aria-label="Folder path">
  {#each path as node, i (node.id)}
    {#if i > 0}
      <span class="sep">›</span>
    {/if}
    <button
      type="button"
      class="crumb"
      class:last={i === path.length - 1}
      onclick={() => onnavigate(i)}
    >
      <span class="name">{node.name || node.path}</span>
      <span class="size">{formatBytes(node.size)}</span>
    </button>
  {/each}
</nav>

<style>
  .crumbs {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-wrap: wrap;
    padding: 10px 16px;
    border-bottom: 1px solid var(--border);
    background: var(--bg-elev);
    min-height: 42px;
  }
  .crumb {
    display: inline-flex;
    align-items: baseline;
    gap: 6px;
    background: transparent;
    border: none;
    padding: 4px 8px;
    border-radius: 5px;
    color: var(--fg-dim);
  }
  .crumb:hover {
    background: var(--bg-panel);
    color: var(--fg);
  }
  .crumb.last {
    color: var(--fg);
    font-weight: 600;
  }
  .name {
    font-size: 13px;
  }
  .size {
    font-size: 11px;
    color: var(--fg-muted);
    font-variant-numeric: tabular-nums;
  }
  .sep {
    color: var(--fg-muted);
    font-size: 14px;
  }
</style>
