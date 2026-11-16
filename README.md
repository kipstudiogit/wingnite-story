# Wingnite Story

**Wingnite Story** (`v1.4.0001`) is a cross-platform narrative workbench and worldbuilding studio built with **Tauri v2**, **Rust**, **Vue 3**, **TypeScript**, and **SQLite WAL + FTS5** for Windows, macOS, Linux, and Android.

---

## Key Features

- **Multi-Book Isolated Workspaces**: Create, edit, switch, and delete isolated literary projects backed by SQLite WAL with FTS5 full-text indexing.
- **Rope-Powered Writing Studio**: Live `ropey::Rope` synchronization with split, editor, preview, and focus modes, formatting toolbar, interactive checklists, and `[[WikiLink]]` / `@Mention` LSP autocomplete with hover previews.
- **Cryptographic Vault & Revision Snapshots**: Per-book Merkle Tree integrity verification, salted SHA-256 passphrase edit locking, and per-scene snapshot history with one-click restore.
- **Lore Guardian & Semantic Search**: Real-time canon consistency checks with persistent dismissed-warning memory and vector-based semantic lore search across the active book.
- **Linguistic Telemetry**: Locale-calibrated readability index, sentence cadence, dialogue ratio, vocabulary richness (TTR), cliché/repetition warnings, and 8-segment pacing timeline.
- **World Codex**: Manage Characters, Divine Artifacts, Demiurges, and Sanctuaries with real mention/link metrics, Grid/Table layouts, relation linking, and inline quick editing.
- **Story Board & Chronology Timeline**: Drag-free act-by-act Kanban board with normalized ordering, plus a multi-track Timeline Matrix with an SVG dramatic tension curve and milestone editing.
- **Interactive 2D Cosmos Graph**: Force-directed relationship graph with entity filtering, node dragging, search teleportation, minimap radar, and link management.
- **Immersive Reader Mode**: Four reading atmospheres (`Ivory`, `Parchment`, `Sepia`, `Midnight`), spread/scroll toggle, column width presets, and silk bookmark state.
- **Import & Export with Native Path Selection**: Save anywhere on Desktop or Android (SAF) in `.wstory` (Gzip + Base64 compressed archive with checksum verification), compiled `.html` edition, Markdown `.md`, or `.txt` with smart chapter splitting.
- **Localization & Themes**: Built-in `English` (base), `Russian`, `German`, and `Spanish` translations, plus `Dark`, `Amber`, `Sepia`, `Paper`, and `OLED` themes adapted for notched, curved, and foldable Android displays.

---

## Tech Stack

- **Frontend**: Vue 3 (`<script setup lang="ts">`), TypeScript 5.7, Vite, Marked
- **Backend**: Rust, Tauri 2 (`dialog`, `fs`, `opener`, `updater`), `rusqlite` (bundled SQLite + FTS5), `ropey`, `sha2`, `flate2`

---

## Development & Build

### Prerequisites

- **Node.js** 20+
- **Rust** (stable toolchain)
- **Tauri v2 system prerequisites** (WebView2 on Windows, WebKitGTK 4.1 on Linux, Xcode on macOS; Android SDK + NDK + JDK 17 for Android builds)

> **Windows Note for Android NDK**: Keep the repository in an ASCII-only path (e.g., `C:\Dev\Wingnite-Story`) when building Android targets so the NDK linker `ld.lld` does not fail on non-ASCII characters.

### Commands

```bash
npm ci
npm run build
```

Run desktop dev mode or build release bundle:

```bash
npm run tauri dev
npm run tauri build
```

Run Android build:

```bash
npx tauri android init
npx tauri android build --apk
```

---

## Project Structure

```text
src/
  components/       UI views and modular subcomponents (studio, codex, graph, timeline, mobile, layout, modals)
  composables/      State & IPC hooks (stories, projects, vault, rope, lore, lsp, graph, sprint, i18n, device)
  constants/        App metadata, theme meta colors, cross-locale entity classifiers
  i18n/             Typed localization engine and en/ru/de/es dictionaries
  styles/           Modular reset, theme tokens, markdown styles, scrollbars
  types/            Domain TypeScript interfaces
  utils/            Markdown & callout renderer

src-tauri/
  capabilities/     Scoped Tauri v2 permissions (default, desktop, mobile)
  src/
    commands/       Thin Tauri IPC command handlers
    db/             SQLite WAL pool and schema migrations
    models/         Serde models split by domain
    services/       Vault cryptography, text analyzer, lore engine, chapter parser, twist engine
```

---

## License

Copyright (C) 2026 Kettle Ignition Power Studio (K. I. P. Studio)

This project is licensed under the **GNU General Public License v3.0 only** (`GPL-3.0-only`). See the [LICENSE](LICENSE) file for details.