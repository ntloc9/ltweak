# ltweak

Mouse and keyboard enhancements for macOS, living in your menu bar.

## Features

- **Ctrl + Scroll → Zoom** — hold `Control` and scroll to zoom in/out (sends `Cmd +` / `Cmd -`) in any app.
- **Finder Side-Button Navigation** — use your mouse's back/forward buttons to go back/forward between folders in Finder.
- **Menu bar icon** — left-click to toggle the app on/off or quit.

The menu bar icon's **Disable App** item pauses every feature and lets your mouse behave normally.

## Requirements

- macOS 13 or later (built and tested on macOS 15)
- [Node.js](https://nodejs.org/) 18+
- [Rust](https://www.rust-lang.org/tools/install) (via `rustup`)
- Xcode Command Line Tools: `xcode-select --install`

## Install

Download the latest `.dmg` from [Releases](https://github.com/ntloc9/ltweak/releases), drag `ltweak.app` into `/Applications`.

### First run

1. Launch **ltweak** — it appears only in the menu bar (no Dock icon).
2. Grant **Accessibility** permission: `System Settings → Privacy & Security → Accessibility`, enable **ltweak**. The features only work with this permission.

## Build from source

```bash
npm install
npm run tauri build
```

Artifacts are written to:

- `src-tauri/target/release/bundle/macos/ltweak.app` — runnable app
- `src-tauri/target/release/bundle/dmg/ltweak_0.1.0_aarch64.dmg` — installer

## Develop

```bash
npm install
npm run tauri dev
```

This starts Vite (`localhost:1420`) and launches the app with hot reload. Live Rust logs (`[zoom]`, `[finder]`) print to the terminal that ran the command.

### Project layout

```
src/                     React frontend (settings UI)
src-tauri/src/
  lib.rs                 App entrypoint
  cg_tap.rs              Shared CoreGraphics event-tap helpers
  tray.rs                Menu bar icon + menu
  features/
    ctrl_scroll_zoom.rs  Ctrl+Scroll → zoom
    finder_sidebutton.rs Mouse back/forward in Finder
```

Adding a new feature? Create a module under `src-tauri/src/features/`, give it a `start()`, and call it from `features::start()` in `src-tauri/src/features/mod.rs`.

## License

Copyright © 2026 Loc Nguyen. All rights reserved.