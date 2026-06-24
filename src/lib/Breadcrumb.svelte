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
  {#each path as node, i (node.path)}
    {#if i > 0}
      <span class="sep" aria-hidden="true">›</span>
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
    padding: 8px 16px;
    border-bottom: 1px solid var(--border-soft);
    background: var(--glass-toolbar);
    -webkit-backdrop-filter: blur(30px) saturate(180%);
    backdrop-filter: blur(30px) saturate(180%);
    min-height: 38px;
  }
  .crumb {
    display: inline-flex;
    align-items: baseline;
    gap: 6px;
    background: transparent;
    border: 1px solid transparent;
    padding: 3px 8px;
    border-radius: 6px;
    color: var(--fg-dim);
    box-shadow: none;
    -webkit-backdrop-filter: none;
    backdrop-filter: none;
    transition: background 100ms ease, color 100ms ease, border-color 100ms ease;
    font-size: 12px;
  }
  .crumb:hover {
    background: var(--glass-hover);
    color: var(--fg);
    border-color: var(--border-soft);
    box-shadow: none;
  }
  .crumb.last {
    color: var(--fg);
    font-weight: 500;
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
    font-size: 13px;
    opacity: 0.5;
  }
</style>