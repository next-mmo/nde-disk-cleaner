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

  function glyphFor(kind: string): string {
    const map: Record<string, string> = {
      folder: "📁",
      image: "🖼",
      video: "🎬",
      audio: "🎵",
      archive: "🗜",
      code: "〈〉",
      document: "📄",
      app: "⚙",
      other: "·",
    };
    return map[kind] ?? "·";
  }
</script>

<div class="list scroll">
  {#if !node}
    <div class="empty">No folder selected</div>
  {:else if items.length === 0}
    <div class="empty">Empty folder</div>
  {:else}
    <div class="head">
      <span class="col-name">Name</span>
      <span class="col-size">Size</span>
    </div>
    <div class="rows">
      {#each items as item (item.path)}
        {@const kind = kindOf(item)}
        <button
          type="button"
          class="row"
          class:active={selected?.path === item.path}
          onclick={() => onselect(item)}
          ondblclick={() => onopen(item)}
        >
          <span
            class="kind-glyph"
            style:color={kindColor(kind)}
          >
            {#if item.is_dir}
              <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor" aria-hidden="true">
                <path d="M3 7.5A2.5 2.5 0 0 1 5.5 5h3.4l1.7 1.8h7.9A2.5 2.5 0 0 1 21 9.3v7.2A2.5 2.5 0 0 1 18.5 19h-13A2.5 2.5 0 0 1 3 16.5v-9z" opacity="0.85"/>
              </svg>
            {:else}
              <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linejoin="round">
                <path d="M6 3h8l4 4v12a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2z"/>
                <path d="M14 3v4h4"/>
              </svg>
            {/if}
          </span>
          <span class="name" title={item.path}>{item.name}</span>
          <span class="size">{formatBytes(item.size)}</span>
          {#if item.is_dir}
            <span class="sub">
              {formatCount(item.file_count)} files · {formatCount(item.dir_count)} folders
            </span>
          {:else}
            <span class="sub">{kind}</span>
          {/if}
          <div class="bar" style:width="{pct(item)}%"></div>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .list {
    width: 340px;
    border-left: 1px solid var(--border-soft);
    background: var(--glass-sidebar);
    -webkit-backdrop-filter: blur(40px) saturate(180%);
    backdrop-filter: blur(40px) saturate(180%);
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
  .head {
    display: flex;
    justify-content: space-between;
    padding: 9px 14px;
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--fg-muted);
    border-bottom: 1px solid var(--border-soft);
    background: var(--glass-toolbar);
    -webkit-backdrop-filter: blur(20px) saturate(180%);
    backdrop-filter: blur(20px) saturate(180%);
    position: sticky;
    top: 0;
    z-index: 1;
  }
  .col-name { flex: 1; }
  .col-size { width: 80px; text-align: right; }

  .rows {
    display: flex;
    flex-direction: column;
    padding: 4px 0;
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
    grid-template-columns: 22px 1fr auto;
    grid-template-rows: auto auto;
    align-items: center;
    column-gap: 10px;
    row-gap: 1px;
    padding: 7px 14px;
    background: transparent;
    border: none;
    border-radius: 0;
    text-align: left;
    cursor: pointer;
    box-shadow: none;
    -webkit-backdrop-filter: none;
    backdrop-filter: none;
    transition: background 80ms ease;
  }
  .row:hover {
    background: var(--glass-hover);
    box-shadow: none;
  }
  .row.active {
    background: var(--accent-soft);
    box-shadow: none;
  }
  .row.active .name {
    color: var(--fg);
  }
  .kind-glyph {
    grid-row: 1 / span 2;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    border-radius: 5px;
  }
  .name {
    grid-row: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 13px;
    font-weight: 500;
  }
  .size {
    grid-row: 1;
    font-variant-numeric: tabular-nums;
    font-weight: 500;
    color: var(--fg-dim);
    font-size: 12px;
    text-align: right;
    width: 80px;
  }
  .sub {
    grid-row: 2;
    grid-column: 2 / span 2;
    font-size: 10px;
    color: var(--fg-muted);
    text-transform: capitalize;
    letter-spacing: 0.02em;
  }
  .bar {
    position: absolute;
    bottom: 1px;
    left: 14px;
    right: 14px;
    height: 1.5px;
    background: linear-gradient(90deg, #5ac8fa, #0a84ff, #bf5af2);
    opacity: 0.55;
    transition: width 160ms ease;
    border-radius: 999px;
  }
</style>