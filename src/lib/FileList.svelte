<script lang="ts">
  import type { FileNode } from "./ipc";
  import { formatBytes, formatCount, kindColor, kindOf } from "./format";

  interface Props {
    node: FileNode | null;
    selected: FileNode | null;
    onselect: (node: FileNode) => void;
    onopen: (node: FileNode) => void;
  }
  let { node, selected, onselect, onopen }: Props = $props();

  // Show top 200 children by size.
  let items = $derived<FileNode[]>(
    node?.children ? node.children.slice(0, 200) : [],
  );

  let maxSize = $derived(items[0]?.size ?? 1);

  function pct(n: FileNode): number {
    return (n.size / maxSize) * 100;
  }
</script>

<div class="list scroll">
  {#if !node}
    <div class="empty">No folder selected</div>
  {:else if items.length === 0}
    <div class="empty">Empty folder</div>
  {:else}
    <div class="head">
      <span>Name</span>
      <span class="size">Size</span>
    </div>
    {#each items as item (item.id)}
      <button
        type="button"
        class="row"
        class:active={selected?.id === item.id}
        onclick={() => onselect(item)}
        ondblclick={() => onopen(item)}
      >
        <span
          class="swatch"
          style:background={kindColor(kindOf(item))}
        ></span>
        <span class="icon">{item.is_dir ? "▸" : " "}</span>
        <span class="name" title={item.path}>{item.name}</span>
        <span class="size">{formatBytes(item.size)}</span>
        <div class="bar" style:width="{pct(item)}%"></div>
        {#if item.is_dir}
          <span class="sub">
            {formatCount(item.file_count)} files
          </span>
        {/if}
      </button>
    {/each}
  {/if}
</div>

<style>
  .list {
    width: 340px;
    border-left: 1px solid var(--border);
    background: var(--bg-elev);
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
  }
  .head {
    display: flex;
    justify-content: space-between;
    padding: 8px 14px;
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.1em;
    color: var(--fg-muted);
    border-bottom: 1px solid var(--border);
    position: sticky;
    top: 0;
    background: var(--bg-elev);
  }
  .empty {
    padding: 40px 20px;
    text-align: center;
    color: var(--fg-muted);
    font-size: 12px;
  }
  .row {
    position: relative;
    display: grid;
    grid-template-columns: 4px 14px 1fr auto;
    grid-template-rows: auto auto;
    align-items: center;
    gap: 4px 8px;
    padding: 8px 14px;
    background: transparent;
    border: none;
    border-radius: 0;
    border-bottom: 1px solid rgba(38, 42, 56, 0.5);
    text-align: left;
    cursor: pointer;
  }
  .row:hover {
    background: var(--bg-panel);
  }
  .row.active {
    background: rgba(122, 162, 255, 0.12);
  }
  .swatch {
    grid-row: 1 / span 2;
    width: 4px;
    height: 24px;
    border-radius: 2px;
  }
  .icon {
    grid-row: 1;
    color: var(--fg-muted);
    font-size: 10px;
  }
  .name {
    grid-row: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .size {
    grid-row: 1;
    font-variant-numeric: tabular-nums;
    font-weight: 600;
    color: var(--fg-dim);
  }
  .sub {
    grid-row: 2;
    grid-column: 3 / span 2;
    font-size: 10px;
    color: var(--fg-muted);
  }
  .bar {
    position: absolute;
    bottom: 0;
    left: 0;
    height: 2px;
    background: var(--accent);
    opacity: 0.4;
    transition: width 160ms;
  }
</style>
