# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Cross-platform WebDAV desktop client built with **Tauri 2 + Svelte 5**. Connects to WebDAV servers, browses files, previews multiple formats (text, images, PDF, audio/video, DOCX, XLSX), edits text in-place, and performs file operations (upload, download, create folder, delete, rename, copy, move). Supports i18n (Chinese/English) and dark mode.

## Development Commands

```bash
# Development (starts Vite dev server + Tauri window)
pnpm tauri dev

# Production build
pnpm tauri build

# Frontend only (no Tauri window)
pnpm dev

# Type checking (Svelte/TypeScript)
pnpm check

# Rust linting/formatting (run from src-tauri/)
cd src-tauri && cargo clippy
cd src-tauri && cargo fmt --check
```

No test framework is configured (no Rust `#[test]`, no JS test runner).

## Architecture

Two-layer architecture communicating via Tauri IPC (`invoke`):

**Rust backend** (`src-tauri/`): All WebDAV protocol interaction via `reqwest_dav`. Tauri commands are organized in `src-tauri/src/commands/` by domain: `connection`, `files`, `upload`, `download`, `operations`, `preview`, `edit`. State lives in `AppState` (`src-tauri/src/webdav/mod.rs`). Errors are unified via `AppError` (`src-tauri/src/error.rs`) which implements `Serialize` for IPC transport.

**Svelte 5 frontend** (`src/`): Single-page app using Svelte 5 runes (`$state`) for reactivity. `src/lib/api.ts` wraps all `invoke()` calls. Store modules in `src/lib/stores/` (`browser`, `connections`, `preview`, `toast`, `theme`, `dialog`) encapsulate domain state with getter functions for reactivity. Components organized by feature in `src/lib/components/` (layout, connection, browser, preview, common).

### Data flow

Component → store function → `api.ts` `invoke()` → Rust command → `AppState.get_client()` → WebDAV operation → result returned via IPC → store updates reactive state → Svelte re-renders

### Video streaming architecture

WKWebView does not support custom URI schemes for `<video>` playback. Instead, a local HTTP proxy server (`src-tauri/src/streaming.rs`) forwards Range requests to WebDAV:

1. At app startup, `start_http_server()` binds a TCP listener on `localhost:0` (random port) in a dedicated tokio runtime (2 worker threads)
2. `tokio::net::TcpListener::from_std()` MUST be called inside `rt.block_on()` to register with the IO driver
3. `start_video_stream` command stores `StreamState { webdav_path, base_url, auth_header }` in `AppState::stream_paths` (a `SharedStreams = Arc<Mutex<HashMap<String, StreamState>>>`) and returns `http://localhost:{port}/stream/{uuid}`
4. The HTTP proxy forwards Range headers as-is to WebDAV, then streams response body chunks via `response.chunk()` (NOT `response.bytes()` which would buffer the entire file)
5. Response headers include `Accept-Ranges: bytes`, forwarded `Content-Range`/`Content-Length`, CORS headers, and MIME type guessed from file extension

Critical constraints: must use `localhost` (not `127.0.0.1`) to match the webview origin; must stream rather than buffer large files.

### Key patterns

- `AppState::get_client()` clones the `WebDavClient` before async work to avoid holding the Mutex lock across `.await`
- `FileMetadata` uses `Serialize` only (Rust→frontend), `ConnectionProfile` derives both `Serialize`/`Deserialize` (bidirectional)
- Theme is JS-driven (`theme.svelte.ts`): applies `light`/`dark`/`auto` class on `<html>`, persists choice in localStorage, default is `dark`; CSS variables in `app.css` respond to these classes
- i18n uses `svelte-i18n` with locale files in `src/lib/i18n/`
- **Timeouts**: All WebDAV operations have `tokio::time::timeout` wrappers (10-300s depending on operation type)
- **Path normalization**: `WebDavClient::normalize_path()` ensures all paths start with `/` (handles root edge case)
- **Request cancellation**: Preview operations use `AbortController` signal passed to `invoke()` for canceling in-flight downloads
- **Preview size limits**: 50MB limit for binary previews; oversized files show friendly error instead of hanging
- **Text edit size limit**: 5MB max for text file editing; larger files return an error

## Key Dependencies

Rust: `tauri 2`, `reqwest 0.13`, `reqwest_dav 0.3`, `tokio 1` (full), `tauri-plugin-store/dialog/fs 2`, `thiserror 2`, `serde/serde_json 1`, `uuid 1`, `base64 0.22`, `log 0.4`

Frontend: `svelte ^5`, `@tauri-apps/api ^2`, `pdfjs-dist ^5`, `docx-preview ^0.3`, `xlsx ^0.18` (SheetJS), `svelte-i18n ^4`, `tailwindcss ^4`, `vite ^8`

## Requirements

- Node.js >= 18, Rust >= 1.77, pnpm
- Tauri 2 CLI: `pnpm add -D @tauri-apps/cli`

## Release

GitHub Actions workflow (`.github/workflows/release.yml`) builds on `v*` tag push. Targets: macOS ARM (dmg), macOS Intel (dmg), Windows x64 (exe). Creates a draft GitHub Release for manual publish.

```bash
git tag v0.1.0
git push origin v0.1.0
```
