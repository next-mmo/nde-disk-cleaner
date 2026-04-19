# Disk Inspector

A cross-platform disk-space analyzer — deep clone of **Disk Space Analyzer: Inspector**
(Nektony) built on **Rust + Tauri 2 + Svelte 5**. Native on macOS, Windows, and Linux.
No WSL, no Electron.

## Architecture

```
┌────────────── Svelte 5 frontend ──────────────┐
│  Sunburst (SVG)  ·  Sidebar  ·  FileList      │
│  Breadcrumb      ·  Details   ·  Progress     │
└──────────── @tauri-apps/api invoke/event ─────┘
                       ⇅
┌────────────── Rust backend ───────────────────┐
│  scanner.rs  jwalk parallel walk (all cores)  │
│  tree.rs     FileNode, ScanProgress, Volume   │
│  commands.rs list_volumes / start_scan /      │
│              cancel_scan / trash_path         │
└───────────────────────────────────────────────┘
```

## Prereqs

- Node 20+
- Rust stable (`rustup`)
- Tauri 2 system deps — see <https://v2.tauri.app/start/prerequisites/>
- macOS: Xcode CLT · Windows: MSVC + WebView2 · Linux: `webkit2gtk-4.1`, etc.

## Run

```sh
pnpm install
pnpm run tauri dev
```

## Build

```sh
pnpm run tauri build
```

Artifacts land in `src-tauri/target/release/bundle/`.

## Features

- Parallel directory walk with `jwalk` (all CPU cores)
- Real-time progress events (5 Hz, throttled)
- Cancellable scans (`Esc` or Cancel button)
- Interactive sunburst chart — click to zoom, hover for tooltip
- Breadcrumb back-navigation (`Backspace`)
- Biggest-children list sorted by size
- Reveal in Finder/Explorer/file-manager
- Move to Trash (cross-platform via `trash` crate)
- Hidden-files toggle
- Volume list with usage bars (via `sysinfo`)

## Keyboard

| Key          | Action                        |
| ------------ | ----------------------------- |
| `Esc`        | Cancel scan / navigate up     |
| `Backspace`  | Navigate up                   |
| double-click | Open file / drill into folder |

## Icons

Placeholder icons live in `src-tauri/icons/`. Replace with your branded set
before shipping. Tauri ships an `icon` CLI subcommand to generate all formats
from a single 1024×1024 PNG:

```sh
pnpm run tauri icon path/to/icon.png
```

## License

MIT — for the scaffold code. Name and trade dress belong to Nektony LLC;
this is an independently-written clone, not their code.
