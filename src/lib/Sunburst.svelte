<script lang="ts">
  import type { FileNode } from "./ipc";
  import { formatBytes, kindColor, kindOf } from "./format";
  import { zoomInto } from "./stores.svelte";

  interface Props {
    root: FileNode;
    size?: number;
    maxRings?: number;
    minSliceAngle?: number;
    onhover?: (node: FileNode | null) => void;
    onselect?: (node: FileNode) => void;
  }

  let {
    root,
    size = 640,
    maxRings = 6,
    minSliceAngle = 0.008, // ~0.5° — slivers below this are merged into "Other"
    onhover,
    onselect,
  }: Props = $props();

  const TAU = Math.PI * 2;

  interface Arc {
    node: FileNode;
    depth: number;
    a0: number;
    a1: number;
    r0: number;
    r1: number;
    path: string;
    color: string;
  }

  let hovered = $state<Arc | null>(null);

  const center = $derived(size / 2);
  const ringWidth = $derived(Math.max(28, center / (maxRings + 1.2)));

  const arcs = $derived.by<Arc[]>(() => {
    const out: Arc[] = [];
    if (!root || root.size === 0) return out;

    const walk = (
      node: FileNode,
      depth: number,
      a0: number,
      a1: number,
    ) => {
      if (depth > maxRings) return;
      if (a1 - a0 < minSliceAngle) return;

      const r0 = depth === 0 ? 0 : ringWidth * (depth - 0.4);
      const r1 = ringWidth * (depth + 0.6);
      const color =
        depth === 0
          ? "var(--bg-panel)"
          : kindColor(kindOf(node));

      out.push({
        node,
        depth,
        a0,
        a1,
        r0,
        r1,
        path: arcPath(center, center, r0, r1, a0, a1),
        color,
      });

      if (!node.is_dir || !node.children || node.children.length === 0) return;

      // Layout children proportional to size.
      const total = node.size || 1;
      let cursor = a0;
      const span = a1 - a0;
      for (const child of node.children) {
        const frac = child.size / total;
        const childA1 = cursor + span * frac;
        walk(child, depth + 1, cursor, childA1);
        cursor = childA1;
      }
    };

    walk(root, 0, 0, TAU);
    return out;
  });

  /** SVG path for an annular sector. */
  function arcPath(
    cx: number,
    cy: number,
    r0: number,
    r1: number,
    a0: number,
    a1: number,
  ): string {
    // Rotate so 0 is at 12 o'clock.
    const offset = -Math.PI / 2;
    const sa = a0 + offset;
    const ea = a1 + offset;

    if (r0 === 0) {
      // Pie slice.
      const x0 = cx + r1 * Math.cos(sa);
      const y0 = cy + r1 * Math.sin(sa);
      const x1 = cx + r1 * Math.cos(ea);
      const y1 = cy + r1 * Math.sin(ea);
      const large = ea - sa > Math.PI ? 1 : 0;
      if (ea - sa >= TAU - 1e-6) {
        return `M ${cx - r1} ${cy} A ${r1} ${r1} 0 1 1 ${cx + r1} ${cy} A ${r1} ${r1} 0 1 1 ${cx - r1} ${cy} Z`;
      }
      return `M ${cx} ${cy} L ${x0} ${y0} A ${r1} ${r1} 0 ${large} 1 ${x1} ${y1} Z`;
    }

    const xOuter0 = cx + r1 * Math.cos(sa);
    const yOuter0 = cy + r1 * Math.sin(sa);
    const xOuter1 = cx + r1 * Math.cos(ea);
    const yOuter1 = cy + r1 * Math.sin(ea);
    const xInner1 = cx + r0 * Math.cos(ea);
    const yInner1 = cy + r0 * Math.sin(ea);
    const xInner0 = cx + r0 * Math.cos(sa);
    const yInner0 = cy + r0 * Math.sin(sa);
    const large = ea - sa > Math.PI ? 1 : 0;

    return [
      `M ${xOuter0} ${yOuter0}`,
      `A ${r1} ${r1} 0 ${large} 1 ${xOuter1} ${yOuter1}`,
      `L ${xInner1} ${yInner1}`,
      `A ${r0} ${r0} 0 ${large} 0 ${xInner0} ${yInner0}`,
      "Z",
    ].join(" ");
  }

  function handleClick(arc: Arc) {
    if (arc.depth === 0) return;
    onselect?.(arc.node);
    if (arc.node.is_dir) zoomInto(arc.node);
  }

  function handleEnter(arc: Arc) {
    hovered = arc;
    onhover?.(arc.node);
  }

  function handleLeave() {
    hovered = null;
    onhover?.(null);
  }

  // Readable label: only show on big center or big first-ring slices.
  function labelFor(arc: Arc): string | null {
    if (arc.depth === 0) {
      return arc.node.name;
    }
    if (arc.depth === 1 && arc.a1 - arc.a0 > 0.25) {
      return arc.node.name;
    }
    return null;
  }

  function labelTransform(arc: Arc): string {
    const mid = (arc.a0 + arc.a1) / 2 - Math.PI / 2;
    const r = arc.depth === 0 ? 0 : (arc.r0 + arc.r1) / 2;
    const x = center + r * Math.cos(mid);
    const y = center + r * Math.sin(mid);
    if (arc.depth === 0) return `translate(${x} ${y})`;
    // Rotate so text follows the ring, flip on bottom half.
    let deg = ((mid + Math.PI / 2) * 180) / Math.PI;
    if (deg > 90 && deg < 270) deg -= 180;
    return `translate(${x} ${y}) rotate(${deg})`;
  }
</script>

<div class="wrap" style:--size="{size}px">
  <svg
    viewBox="0 0 {size} {size}"
    width={size}
    height={size}
    role="img"
    aria-label="Disk usage sunburst"
  >
    <defs>
      <filter id="slice-shadow" x="-10%" y="-10%" width="120%" height="120%">
        <feGaussianBlur in="SourceAlpha" stdDeviation="1.5" />
        <feOffset dx="0" dy="1" result="off" />
        <feComponentTransfer>
          <feFuncA type="linear" slope="0.4" />
        </feComponentTransfer>
        <feMerge>
          <feMergeNode />
          <feMergeNode in="SourceGraphic" />
        </feMerge>
      </filter>
    </defs>

    {#each arcs as arc (arc.node.id)}
      <path
        d={arc.path}
        fill={arc.color}
        stroke="var(--bg)"
        stroke-width="1.5"
        class="slice"
        class:hot={hovered === arc}
        class:root={arc.depth === 0}
        onmouseenter={() => handleEnter(arc)}
        onmouseleave={handleLeave}
        onclick={() => handleClick(arc)}
        role="button"
        tabindex={arc.depth === 0 ? -1 : 0}
        aria-label="{arc.node.name}, {formatBytes(arc.node.size)}"
      >
        <title>{arc.node.path}\n{formatBytes(arc.node.size)}</title>
      </path>
    {/each}

    {#each arcs as arc (arc.node.id + ":label")}
      {@const label = labelFor(arc)}
      {#if label}
        <text
          transform={labelTransform(arc)}
          text-anchor="middle"
          dominant-baseline="middle"
          class="label"
          class:center={arc.depth === 0}
          pointer-events="none"
        >
          {#if arc.depth === 0}
            <tspan x="0" dy="-0.6em">{label}</tspan>
            <tspan x="0" dy="1.4em" class="sub">
              {formatBytes(arc.node.size)}
            </tspan>
          {:else}
            {label.length > 18 ? label.slice(0, 17) + "…" : label}
          {/if}
        </text>
      {/if}
    {/each}
  </svg>

  {#if hovered && hovered.depth !== 0}
    <div class="tip">
      <div class="tip-name">{hovered.node.name}</div>
      <div class="tip-size">{formatBytes(hovered.node.size)}</div>
      <div class="tip-path">{hovered.node.path}</div>
    </div>
  {/if}
</div>

<style>
  .wrap {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
  }
  svg {
    max-width: 100%;
    max-height: 100%;
  }
  .slice {
    cursor: pointer;
    transition: opacity 80ms, filter 80ms;
  }
  .slice.root {
    cursor: default;
  }
  .slice:hover:not(.root) {
    filter: brightness(1.15);
  }
  .slice.hot {
    filter: brightness(1.2);
  }
  .label {
    font-size: 11px;
    fill: #0c0f16;
    font-weight: 600;
    paint-order: stroke;
    stroke: rgba(255, 255, 255, 0.5);
    stroke-width: 0.5px;
  }
  .label.center {
    font-size: 15px;
    fill: var(--fg);
    stroke: none;
    font-weight: 600;
  }
  .label.center .sub {
    font-size: 12px;
    fill: var(--fg-dim);
    font-weight: 500;
  }
  .tip {
    position: absolute;
    top: 12px;
    left: 12px;
    background: rgba(10, 12, 20, 0.92);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 8px 12px;
    max-width: 340px;
    pointer-events: none;
    backdrop-filter: blur(8px);
  }
  .tip-name {
    font-weight: 600;
  }
  .tip-size {
    color: var(--accent);
    font-variant-numeric: tabular-nums;
    margin-top: 2px;
  }
  .tip-path {
    color: var(--fg-muted);
    font-size: 11px;
    margin-top: 4px;
    word-break: break-all;
  }
</style>
