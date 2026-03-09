# Robustness & Polish Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Harden the download manager against concurrent retries and partial files, add a timeout to AI move requests, fix README inaccuracies, and clean up stale Rust code.

**Architecture:** Incremental hardening of existing systems — no new features, just making existing ones more reliable. Download manager gets a mutex guard and partial file cleanup. KataGo AI move requests get a timeout so the UI never hangs indefinitely. README gets corrected to match reality.

**Tech Stack:** Rust (Tauri v2), Svelte 5, reqwest, tokio

---

## Task 1: Guard `retry_downloads` against concurrent calls

The `retry_downloads` command calls `run_initial_downloads`, which has no protection against being invoked while a download is already in progress. Two rapid clicks on "Retry" would spawn duplicate download tasks racing on the same files.

### Files:
- Modify: `src-tauri/src/download_manager.rs`

### Step 1: Add an `AtomicBool` download-in-progress guard

At the top of `download_manager.rs`, add a static `AtomicBool`:

```rust
use std::sync::atomic::{AtomicBool, Ordering};

static DOWNLOADING: AtomicBool = AtomicBool::new(false);
```

### Step 2: Guard `run_initial_downloads` with the flag

At the top of `run_initial_downloads`, add:

```rust
if DOWNLOADING.swap(true, Ordering::SeqCst) {
    tracing::info!("Downloads already in progress, skipping");
    return;
}
```

At the very end of the function (after all download logic), add:

```rust
DOWNLOADING.store(false, Ordering::SeqCst);
```

This must be the last line, after all `emit_status` calls.

### Step 3: Verify

Run: `cargo clippy --workspace -- -D warnings`
Expected: passes clean

Run: `cargo test --workspace`
Expected: all tests pass

### Step 4: Commit

```bash
git add src-tauri/src/download_manager.rs
git commit -m "fix: guard retry_downloads against concurrent calls"
```

---

## Task 2: Clean up partial files on download failure

If a download fails mid-stream, a partial file is left on disk. On next startup, the download manager sees the file exists and skips re-downloading — but the file is corrupt/incomplete.

### Files:
- Modify: `src-tauri/src/setup.rs`

### Step 1: Add cleanup on error in `download_file`

In `setup.rs`, modify the `download_file` function. After the `loop` block, the function currently returns `Ok(())`. Wrap the entire download-and-write logic so that if any step after file creation fails, the partial file is deleted.

Replace the current `download_file` function with:

```rust
fn download_file(
    url: &str,
    dest: &Path,
    on_progress: impl Fn(u64, u64),
) -> Result<(), String> {
    let response = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .build()
        .map_err(|e| format!("http client: {e}"))?
        .get(url)
        .send()
        .map_err(|e| format!("download request: {e}"))?
        .error_for_status()
        .map_err(|e| format!("download failed: {e}"))?;

    let total = response.content_length().unwrap_or(0);
    let mut reader = response;
    let mut file = fs::File::create(dest).map_err(|e| format!("create file: {e}"))?;
    let mut downloaded: u64 = 0;
    let mut buf = [0u8; 65_536];

    let result: Result<(), String> = (|| {
        loop {
            let n = reader.read(&mut buf).map_err(|e| format!("read: {e}"))?;
            if n == 0 {
                break;
            }
            file.write_all(&buf[..n])
                .map_err(|e| format!("write: {e}"))?;
            downloaded += n as u64;
            on_progress(downloaded, total);
        }
        file.flush().map_err(|e| format!("flush: {e}"))?;
        Ok(())
    })();

    if result.is_err() {
        // Remove partial file so next attempt starts fresh
        let _ = fs::remove_file(dest);
    }

    result
}
```

### Step 2: Verify

Run: `cargo clippy --workspace -- -D warnings`
Expected: passes clean

Run: `cargo test --workspace`
Expected: all tests pass

### Step 3: Commit

```bash
git add src-tauri/src/setup.rs
git commit -m "fix: delete partial files on download failure"
```

---

## Task 3: Add timeout to AI move requests

`request_ai_move` in `commands/katago.rs` awaits a oneshot receiver with no timeout. If KataGo crashes or hangs, the UI stays in "AI is thinking..." forever. The review system already uses a 10-second timeout (`commands/review.rs:19`). AI moves should get the same treatment.

### Files:
- Modify: `src-tauri/src/commands/katago.rs`

### Step 1: Add timeout to the response await

In `request_ai_move`, find this block (around line 205 in the current file):

```rust
    let response = {
        let rx = {
            let katago = state.katago.lock().await;
            let client = katago
                .as_ref()
                .ok_or(AppError::KataGo("Engine not available".into()))?;
            client.query_fire(query).await?
        };
        // Lock released — await response without holding state.katago
        rx.await.map_err(|_| AppError::KataGo("Engine response cancelled".into()))?
    };
```

Replace with:

```rust
    let response = {
        let rx = {
            let katago = state.katago.lock().await;
            let client = katago
                .as_ref()
                .ok_or(AppError::KataGo("Engine not available".into()))?;
            client.query_fire(query).await?
        };
        // Lock released — await response without holding state.katago
        match tokio::time::timeout(std::time::Duration::from_secs(30), rx).await {
            Ok(Ok(resp)) => resp,
            Ok(Err(_)) => {
                let _ = app.emit("ai-thinking", false);
                return Err(AppError::KataGo("Engine response cancelled".into()));
            }
            Err(_) => {
                let _ = app.emit("ai-thinking", false);
                return Err(AppError::KataGo(
                    "AI move timed out after 30 seconds. The engine may be overloaded.".into(),
                ));
            }
        }
    };
```

### Step 2: Verify

Run: `cargo clippy --workspace -- -D warnings`
Expected: passes clean

Run: `cargo test --workspace`
Expected: all tests pass

### Step 3: Commit

```bash
git add src-tauri/src/commands/katago.rs
git commit -m "fix: add 30s timeout to AI move requests"
```

---

## Task 4: Reset download state before retry

When `retry_downloads` is called, it calls `run_initial_downloads` which checks if the asset is already `Ready` and skips it. But for assets in `Error` state, the function proceeds with the download — however, the status displayed to the user still shows the old error until the download progress event fires. Reset error states to `NotInstalled` at the start so the UI immediately reflects the retry.

### Files:
- Modify: `src-tauri/src/download_manager.rs`

### Step 1: Reset error states in `retry_downloads`

Replace the current `retry_downloads` function:

```rust
#[tauri::command]
pub async fn retry_downloads(app_handle: tauri::AppHandle) {
    // Reset error states so UI shows "starting" immediately
    {
        let mut s = global_status().lock().unwrap();
        if matches!(s.katago, DownloadState::Error { .. }) {
            s.katago = DownloadState::NotInstalled;
        }
        if matches!(s.llm, DownloadState::Error { .. }) {
            s.llm = DownloadState::NotInstalled;
        }
    }
    emit_status(&app_handle);
    run_initial_downloads(app_handle).await;
}
```

### Step 2: Verify

Run: `cargo clippy --workspace -- -D warnings`
Expected: passes clean

Run: `cargo test --workspace`
Expected: all tests pass

### Step 3: Commit

```bash
git add src-tauri/src/download_manager.rs
git commit -m "fix: reset error states before retry so UI updates immediately"
```

---

## Task 5: Fix README inaccuracies

The README says "Pixi.js" in two places, but the board renderer is SVG-based (`BoardSvg.svelte`). It also omits the local LLM coaching feature.

### Files:
- Modify: `README.md`

### Step 1: Fix tech stack description

Line 3 currently reads:
```
A free, open-source desktop app that teaches Go (Weiqi/Baduk) to beginners through AI coaching. Built with Rust, Tauri v2, Svelte 5, and Pixi.js.
```

Change to:
```
A free, open-source desktop app that teaches Go (Weiqi/Baduk) to beginners through AI coaching. Built with Rust, Tauri v2, and Svelte 5.
```

### Step 2: Fix project structure

Line 66 currently reads:
```
  src/                   # Svelte 5 frontend: Pixi.js board, stores, views
```

Change to:
```
  src/                   # Svelte 5 frontend: SVG board, stores, views
```

### Step 3: Add LLM coaching to features list

After the line:
```
- **Two visual themes** (Study and Grid) with animated stone placement
```

Add:
```
- **Local AI coaching** via Gemma 3 1B (optional, downloaded on first launch)
```

### Step 4: Verify

Run: `npm run check`
Expected: 0 errors (README changes don't affect type checking, but good to confirm nothing broke)

### Step 5: Commit

```bash
git add README.md
git commit -m "docs: fix README — SVG not Pixi.js, add LLM coaching feature"
```

---

## Task 6: Remove unused `Emitter` import check and dead `setup` import in katago.rs

After the dead code removal in the previous cleanup phase, verify that all imports in `commands/katago.rs` are still needed. The `setup_katago` function was removed but `use crate::setup` is still imported — check if `setup::KataGoStatus`, `setup::binary_path`, `setup::model_path`, `setup::config_path` are still used.

### Files:
- Modify: `src-tauri/src/commands/katago.rs`

### Step 1: Audit imports

Read `commands/katago.rs` and verify each import is used. Current imports:

```rust
use tauri::{AppHandle, Emitter, Manager, State};
use crate::setup;
```

- `Emitter` — used by `start_engine` (app.emit) and `request_ai_move` (app.emit) — **keep**
- `Manager` — used by `app.path()` in `katago_dir` — **keep**
- `crate::setup` — used by `setup::binary_path`, `setup::model_path`, `setup::config_path`, `setup::setup_status`, `setup::KataGoStatus` — check which are still referenced

After removing `get_katago_status`, `setup::setup_status` and `setup::KataGoStatus` may still be referenced by `start_engine` (returns `setup::KataGoStatus`). If so, keep `use crate::setup`. If `setup::KataGoStatus` is the only type used and it's just for the return type, consider whether it's still appropriate — `start_engine` returns `KataGoStatus::Ready` on success.

### Step 2: Run clippy to catch any unused imports

Run: `cargo clippy --workspace -- -D warnings`

If clippy is clean, no changes needed — skip to commit. If it reports unused imports, remove them.

### Step 3: Commit (only if changes were made)

```bash
git add src-tauri/src/commands/katago.rs
git commit -m "refactor: remove unused imports from katago commands"
```

---

## Task 7: Add retry button to OnboardingView download error

The onboarding view shows "Download failed" with only a "Skip for now" button. The other views (HomeView, PlayView, ReviewView) now have Retry buttons. OnboardingView should match.

### Files:
- Modify: `src/views/OnboardingView.svelte`

### Step 1: Add retry button

In `OnboardingView.svelte`, find the download error block (around line 266-270):

```svelte
      {:else if downloadStore.katagoError}
        <div class="text-sm" style="color: var(--danger);">Download failed: {downloadStore.katagoError}</div>
        <button onclick={finishOnboarding} class="rounded px-4 py-2 text-sm" style="background-color: var(--surface-secondary); color: var(--text-secondary);">
          Skip for now
        </button>
```

Replace with:

```svelte
      {:else if downloadStore.katagoError}
        <div class="text-sm" style="color: var(--danger);">Download failed: {downloadStore.katagoError}</div>
        <div class="flex gap-2">
          <button
            onclick={() => downloadStore.retry()}
            class="rounded px-4 py-2 text-sm font-semibold"
            style="background-color: var(--btn-bg); color: var(--btn-text);"
          >
            Retry
          </button>
          <button onclick={finishOnboarding} class="rounded px-4 py-2 text-sm" style="background-color: var(--surface-secondary); color: var(--text-secondary);">
            Skip for now
          </button>
        </div>
```

### Step 2: Verify

Run: `npm run check`
Expected: 0 errors

### Step 3: Commit

```bash
git add src/views/OnboardingView.svelte
git commit -m "feat: add Retry button to onboarding download error"
```

---

## Verification

After all tasks:
- `cargo clippy --workspace -- -D warnings` passes
- `cargo test --workspace` passes (195 tests)
- `npm run check` passes (0 errors)
- Manual test: double-click Retry rapidly — only one download runs
- Manual test: disconnect network during download, reconnect, click Retry — download restarts from 0%
- Manual test: AI move with engine stopped — times out after 30s with clear error message
