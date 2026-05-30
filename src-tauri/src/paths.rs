//! Path resolution for the bootstrap pointer and data folder.
//! See SPEC.md §3.5 for the bootstrap-vs-settings split.

use anyhow::{Context, Result};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// User-facing subdir name used under both OS config dir and OS data dir.
/// Tauri's built-in `app_data_dir()` / `app_config_dir()` would use the bundle
/// identifier (`app.snippet/`), which is unfriendly when the user browses the
/// folder. SPEC.md §11's `%APPDATA%\<app>\` reads as the product name.
const APP_SUBDIR: &str = "Snippet";

/// `<OS user config dir>/Snippet/bootstrap.json` — per-device, never synced.
pub fn bootstrap_path(app: &AppHandle) -> Result<PathBuf> {
    let base = app
        .path()
        .config_dir()
        .context("resolving OS config dir")?;
    Ok(base.join(APP_SUBDIR).join("bootstrap.json"))
}

/// `<OS user data dir>/Snippet/` — used when `bootstrap.dataFolderPath` is None.
pub fn default_data_folder(app: &AppHandle) -> Result<PathBuf> {
    let base = app.path().data_dir().context("resolving OS data dir")?;
    Ok(base.join(APP_SUBDIR))
}
