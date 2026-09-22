# Clipo Architecture & Technical Reference

Clipo is a lightweight, high-performance clipboard manager for Windows built with **Tauri v2 (Rust backend)** and **React 19 + TypeScript + Tailwind CSS v4 (Frontend)**.

---

## 1. System Overview

```
┌────────────────────────────────────────────────────────┐
│                   Windows OS (Win32)                   │
│  - GetClipboardSequenceNumber polling (250ms)          │
│  - CF_HDROP, CF_HTML, CF_UNICODETEXT, DIB/RGBA Bitmaps │
│  - Foreground HWND tracking & SendInput Ctrl+V         │
│  - SHGetFileInfoW Executable Icon Extraction           │
└──────────────────────────┬─────────────────────────────┘
                           │
                           ▼
┌────────────────────────────────────────────────────────┐
│                   Rust Backend (Tauri v2)              │
│  - clipboard::monitor (Sequence watcher thread)        │
│  - clipboard::ingest (Format parsing & deduplication)  │
│  - intelligence (Secret & credential filtering)        │
│  - history::History (JSON storage + limits + pinning)  │
│  - windowing & tray (Transient popover + focus hooks)  │
│  - commands (Typed IPC handlers)                       │
└──────────────────────────┬─────────────────────────────┘
                           │ Typed IPC & Tauri Events
                           ▼
┌────────────────────────────────────────────────────────┐
│                React 19 Frontend (SPA)                 │
│  - shared/ipc/api.ts (Type-safe IPC client)            │
│  - shared/hooks/useClipboard.ts (Reactive clip store)  │
│  - widgets/clipboard (SearchBar & CategoryTabs)        │
│  - features/clipboard (ClipItem, AppIcon, Thumbnails)  │
│  - views/app (Floating popover & Settings pages)       │
└────────────────────────────────────────────────────────┘
```

---

## 2. Backend Modules (`src-tauri/src/`)

### A. Clipboard Ingestion (`src-tauri/src/clipboard/`)
- `monitor.rs`: Spawns a dedicated background thread monitoring `GetClipboardSequenceNumber` with a 250ms tick interval. When a sequence change is detected, it invokes `ingest`.
- `ingest.rs`:
  1. Inspects active foreground window and retrieves source process metadata (`exeName`, `friendly name`, `path`).
  2. Evaluates application exclusion lists from `Settings`.
  3. Checks Win32 clipboard format flags for privacy opt-outs (`ExcludeClipboardContentFromMonitorProcessing`, `CanIncludeInClipboardHistory`).
  4. Parses clipboard formats with priority:
     - `CF_HDROP`: File paths and directory references.
     - `CF_HTML` / Rich Text / Plain Text: Validates text content, scans for secrets/credentials via `intelligence.rs`, detects URLs/links.
     - Bitmaps / RGBA Images via `arboard`: Encodes PNG master assets and generates cached thumbnails.
  5. Deduplicates identical consecutive items, appends to `History`, saves to disk, and emits `clips-changed` event to the webview.
- `write.rs`: Restores stored clips back into Windows clipboard in either `Rich` or `Plain` format.

### B. Platform Integration (`src-tauri/src/platform/`)
- `app.rs`: Resolves active foreground `HWND`, process ID, executable name, and window title via Win32 APIs (`GetForegroundWindow`, `GetWindowThreadProcessId`, `QueryFullProcessImageNameW`).
- `clipboard_raw.rs`: Win32 low-level clipboard format queries, format IDs registration, and `HDROP` file path reading/writing.
- `icon.rs`: Extracts native executable icons via `SHGetFileInfoW`, renders to a 32-bit DIB section, converts to RGBA, and encodes as PNG bytes for frontend display.
- `input.rs`: Synthesizes `Ctrl+V` key combination using Win32 `SendInput` after restoring foreground focus to the target application.

### C. State & Storage (`src-tauri/src/state.rs`, `storage.rs`, `history.rs`)
- `AppState`: Holds mutex-guarded `History`, `Settings`, paths, cached app icons, and an atomic tracker for previous foreground window handle (`foreground_window`).
- `Paths`: Manages directory paths in local app data (`%LOCALAPPDATA%/clipo/`):
  - `history.json`: Clipboard metadata and text content.
  - `settings.json`: User preferences.
  - `images/`: Full-size captured PNG images.
  - `thumbs/`: Downscaled image thumbnails.
- `History`: In-memory clip list enforcing `historyLimit` capacity while protecting pinned clips from eviction.

### D. Windowing & Tray (`src-tauri/src/windowing.rs`, `tray.rs`)
- `windowing.rs`:
  - Transient popover window configuration: centered, borderless, always on top, skip taskbar.
  - Automatically captures the active foreground window before opening popover.
  - Automatically hides popover when focus is lost (`WindowEvent::Focused(false)`).
- `tray.rs`: System tray icon with context menu ("Open Clipboard", "Settings", "Clear History", "Quit").

---

## 3. Frontend Architecture (`src/`)

- `shared/types/index.ts`: TypeScript contracts for `ClipView`, `Settings`, `SourceApp`, `ClipKind`, and `ThemeMode`.
- `shared/ipc/api.ts`: Typed IPC wrapper covering all Tauri commands and event listeners.
- `shared/hooks/useClipboard.ts`: Central reactive hook providing search filtering, category filtering, keyboard actions, and automatic sync with `clips-changed` events.
- `shared/hooks/useSettings.ts`: Manages user settings, persistence, and dynamic dark/light theme switching.
- `widgets/clipboard/`: Search bar and category tabs (`All`, `Pinned`, `Text`, `Images`, `Links`, `Files`).
- `features/clipboard/`:
  - `ClipItem.tsx`: Clip card with metadata, source app icon, thumbnail/preview, and quick action buttons.
  - `AppIcon.tsx`: Lazy-loaded base64 application icon with fallback icon.
  - `ClipThumbnail.tsx`: Lazy-loaded image thumbnail.
  - `QuickActions.tsx`: Context buttons (Copy, Paste Plain, Pin/Unpin, Delete).
  - `EmptyState.tsx`: Search and empty history states.
- `views/app/`: Popover layout (`AppLayout.tsx`), clipboard page (`page.tsx`), and settings view (`settings/page.tsx`).

---

## 4. Keyboard Navigation

- **Global Shortcut** (default `Ctrl+Shift+V`): Toggles floating Clipo popover.
- **Up / Down Arrows**: Navigate through clips list.
- **Enter**: Quick paste selected clip into previous application.
- **Shift + Enter**: Paste selected clip as plain text.
- **1 - 9**: Quick paste clip by index.
- **Esc / Focus Lost**: Automatically hides popover.
