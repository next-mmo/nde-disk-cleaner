const UNITS = ["B", "KB", "MB", "GB", "TB", "PB"];

export function formatBytes(n: number, digits = 1): string {
  if (!Number.isFinite(n) || n < 0) return "—";
  if (n < 1024) return `${n} B`;
  let u = 0;
  let v = n;
  while (v >= 1024 && u < UNITS.length - 1) {
    v /= 1024;
    u++;
  }
  return `${v.toFixed(digits)} ${UNITS[u]}`;
}

export function formatCount(n: number): string {
  return new Intl.NumberFormat().format(n);
}

export function formatDate(unixSecs?: number): string {
  if (!unixSecs) return "—";
  const d = new Date(unixSecs * 1000);
  return d.toLocaleString();
}

/** Deterministic HSL from string — used for sunburst slice colors. */
export function colorFor(s: string, depth = 0): string {
  let h = 0;
  for (let i = 0; i < s.length; i++) {
    h = (h * 31 + s.charCodeAt(i)) >>> 0;
  }
  const hue = h % 360;
  // Outer rings are slightly lighter/desaturated to feel recessed.
  const sat = Math.max(45, 72 - depth * 4);
  const lum = Math.min(70, 52 + depth * 3);
  return `hsl(${hue} ${sat}% ${lum}%)`;
}

/** Color by file-kind bucket — macOS Tahoe palette. */
const KIND_COLORS: Record<string, string> = {
  image:    "hsl(200 95% 65%)",  /* system blue   */
  video:    "hsl(330 90% 65%)",  /* system pink   */
  audio:    "hsl(275 85% 70%)",  /* system purple */
  archive:  "hsl(28 95% 60%)",   /* system orange */
  code:     "hsl(140 70% 60%)",  /* system green  */
  document: "hsl(48 95% 62%)",   /* system yellow */
  app:      "hsl(0 90% 65%)",    /* system red    */
  folder:   "hsl(215 90% 68%)",  /* system blue (lighter) */
  other:    "hsl(220 12% 60%)",  /* graphite      */
};

const EXT_KIND: Record<string, string> = {
  jpg: "image", jpeg: "image", png: "image", gif: "image", webp: "image",
  heic: "image", tiff: "image", bmp: "image", svg: "image",
  mp4: "video", mov: "video", mkv: "video", webm: "video", avi: "video",
  mp3: "audio", flac: "audio", wav: "audio", ogg: "audio", m4a: "audio", aac: "audio",
  zip: "archive", tar: "archive", gz: "archive", rar: "archive", "7z": "archive",
  dmg: "archive", iso: "archive",
  ts: "code", tsx: "code", js: "code", jsx: "code", rs: "code", py: "code",
  go: "code", java: "code", c: "code", cpp: "code", h: "code", svelte: "code",
  html: "code", css: "code", json: "code", yaml: "code", yml: "code", toml: "code",
  pdf: "document", doc: "document", docx: "document", txt: "document",
  md: "document", xls: "document", xlsx: "document", ppt: "document", pptx: "document",
  app: "app", exe: "app", deb: "app", rpm: "app", appimage: "app",
};

export function kindOf(node: { is_dir: boolean; extension?: string }): string {
  if (node.is_dir) return "folder";
  const ext = node.extension?.toLowerCase();
  if (!ext) return "other";
  return EXT_KIND[ext] ?? "other";
}

export function kindColor(kind: string): string {
  return KIND_COLORS[kind] ?? KIND_COLORS.other;
}
