# NDE Disk Cleaner

A cross-platform disk-space analyzer — deep clone of **Disk Space Analyzer: Inspector**
built on **Rust + Tauri 2 + Svelte 5**. Native on macOS, Windows, and Linux.
No WSL, no Electron.

## Architecture

```text
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

## Releasing

Releases are built and published automatically by
[.github/workflows/release.yml](.github/workflows/release.yml) for macOS
(Apple Silicon + Intel), Windows, and Linux.

### 1. Bump versions

Keep [package.json](package.json), [src-tauri/tauri.conf.json](src-tauri/tauri.conf.json),
and [src-tauri/Cargo.toml](src-tauri/Cargo.toml) in sync.

```sh
# edit "version" in all three files, then:
git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml
git commit -m "chore: release v0.1.0"
git push
```

### 2. Tag and push

The workflow triggers on any tag matching `v*`.

```sh
git tag v0.1.0
git push origin v0.1.0
```

### 3. Publish the release

The workflow creates a **draft** release with all OS bundles attached:

| Platform          | Artifacts                   |
| ----------------- | --------------------------- |
| macOS (arm64/x64) | `.dmg`, `.app.tar.gz`       |
| Windows           | `.msi`, `.exe` (NSIS)       |
| Linux             | `.AppImage`, `.deb`, `.rpm` |

Once the matrix finishes (~15–25 min), open the repo's **Releases** page,
review the draft, add notes, and click **Publish**.

### Manual trigger

You can also kick a build without tagging via **Actions → Release →
Run workflow**, supplying a tag like `v0.1.0-rc1`.

### Re-running a failed build

If one OS job fails, re-run just that job from the Actions UI — the other
artifacts stay attached to the same draft release.

### Notes

- Builds are **unsigned**. On first launch users will hit SmartScreen (Windows) and must explicitly allow the app. On **macOS**, because the app is unsigned and downloaded from the internet, Gatekeeper will flag it with a quarantine attribute. On Apple Silicon (M1/M2/M3), macOS will frequently say the app is "damaged and can't be opened".
  **To fix the macOS "damaged" error:**
  1. Drag `NDE Disk Cleaner.app` from the `.dmg` into your `/Applications` folder.
  2. Open the `Terminal` app and run exactly:
     ```sh
     xattr -cr "/Applications/NDE Disk Cleaner.app"
     ```
  3. You can now open it normally.
  
  *(Note: The tiny file size of < 2MB is expected! This app uses your Mac's built-in WebKit engine natively via Tauri, avoiding the massive bloat of Electron.)*
- To redo a tag, delete both the remote tag and the draft release first:
  `git push --delete origin v0.1.0 && git tag -d v0.1.0`.

## License

MIT
