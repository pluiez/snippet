//! IPC commands. Per ARCHITECTURE.md §3.4, command granularity is "business
//! action" — `save_template`, not `write_file`.

use crate::auto_paste;
use crate::color;
use crate::onboarding;
use crate::palette;
use crate::paths;
use crate::render;
use crate::schema::{
    Bootstrap, DataFolderStatus, Settings, TagColorMap, Template, Variable, VariableColorMap,
    VariableType, CURRENT_SCHEMA_VERSION,
};
use crate::search;
use crate::state::AppState;
use crate::storage;
use chrono::Utc;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tracing::{info, warn};
use ts_rs::TS;
use uuid::Uuid;

const TEMPLATES_CHANGED_EVENT: &str = "templates-changed";
const COLORS_CHANGED_EVENT: &str = "colors-changed";
const SETTINGS_CHANGED_EVENT: &str = "settings-changed";

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export, export_to = "../src/lib/bindings/")]
#[serde(rename_all = "camelCase")]
pub struct TemplateSummary {
    pub id: Uuid,
    pub display_name: String,
    pub is_pinned: bool,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export, export_to = "../src/lib/bindings/")]
#[serde(rename_all = "camelCase")]
pub struct FillDialogState {
    pub template: Template,
    pub initial_values: HashMap<Uuid, String>,
    pub ordered_variables: Vec<Variable>,
}

/// Result of `apply_template`. Tells the frontend whether autoPaste ran and,
/// if it didn't, why (so toast wording can differ).
#[derive(Clone, Debug, Serialize, TS)]
#[ts(export, export_to = "../src/lib/bindings/")]
#[serde(rename_all = "camelCase")]
pub struct ApplyOutcome {
    /// true → auto-pasted into the previously-focused window;
    /// false → clipboard-only.
    pub pasted: bool,
    /// "disabled" if autoPaste setting is off; "failed" if it was attempted
    /// but errored (HWND invalid, OS refused focus, etc.). null when pasted=true.
    pub reason: Option<String>,
}

#[tauri::command]
pub fn list_templates(state: State<'_, AppState>) -> Result<Vec<TemplateSummary>, String> {
    let map = state
        .templates
        .lock()
        .map_err(|e| format!("templates lock poisoned: {e}"))?;
    let mut out: Vec<TemplateSummary> = map
        .values()
        .map(|t| TemplateSummary {
            id: t.id,
            display_name: t.display_name.clone(),
            is_pinned: t.is_pinned,
            tags: t.tags.clone(),
        })
        .collect();
    out.sort_by(|a, b| {
        b.is_pinned
            .cmp(&a.is_pinned)
            .then_with(|| a.display_name.cmp(&b.display_name))
    });
    Ok(out)
}

#[tauri::command]
pub fn search_templates(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<TemplateSummary>, String> {
    Ok(search::search(&state, &query))
}

#[tauri::command]
pub fn get_template(
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<Option<Template>, String> {
    let map = state
        .templates
        .lock()
        .map_err(|e| format!("templates lock poisoned: {e}"))?;
    Ok(map.get(&id).cloned())
}

#[tauri::command]
pub fn save_template(
    app: AppHandle,
    state: State<'_, AppState>,
    template: Template,
) -> Result<(), String> {
    let mut template = template;
    template.updated_at = Utc::now().to_rfc3339();

    let templates_dir = state.templates_dir();
    storage::save_template(&templates_dir, &template)
        .map_err(|e| format!("save_template failed: {e:#}"))?;

    {
        let mut map = state
            .templates
            .lock()
            .map_err(|e| format!("templates lock poisoned: {e}"))?;
        info!(id = %template.id, name = %template.display_name, "template saved");
        map.insert(template.id, template.clone());
    }

    ensure_colors_for_template(&app, &state, &template)?;

    app.emit(TEMPLATES_CHANGED_EVENT, ())
        .map_err(|e| format!("emit failed: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn delete_template(
    app: AppHandle,
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<(), String> {
    let templates_dir = state.templates_dir();
    storage::delete_template(&templates_dir, &id)
        .map_err(|e| format!("delete_template failed: {e:#}"))?;

    {
        let mut map = state
            .templates
            .lock()
            .map_err(|e| format!("templates lock poisoned: {e}"))?;
        info!(?id, "template deleted");
        map.remove(&id);
    }

    app.emit(TEMPLATES_CHANGED_EVENT, ())
        .map_err(|e| format!("emit failed: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn duplicate_template(
    app: AppHandle,
    state: State<'_, AppState>,
    source_id: Uuid,
) -> Result<Template, String> {
    let source = {
        let map = state
            .templates
            .lock()
            .map_err(|e| format!("templates lock poisoned: {e}"))?;
        map.get(&source_id)
            .cloned()
            .ok_or_else(|| format!("source template {source_id} not found"))?
    };

    let now = Utc::now().to_rfc3339();
    let new_template = Template {
        schema_version: CURRENT_SCHEMA_VERSION,
        id: Uuid::new_v4(),
        display_name: format!("{} 副本", source.display_name),
        body: source.body,
        variables: source.variables,
        tags: source.tags,
        is_pinned: false,
        created_at: now.clone(),
        updated_at: now,
        last_used_at: None,
        use_count: 0,
    };

    let templates_dir = state.templates_dir();
    storage::save_template(&templates_dir, &new_template)
        .map_err(|e| format!("save_template failed: {e:#}"))?;

    {
        let mut map = state
            .templates
            .lock()
            .map_err(|e| format!("templates lock poisoned: {e}"))?;
        info!(source = %source_id, new = %new_template.id, "template duplicated");
        map.insert(new_template.id, new_template.clone());
    }

    ensure_colors_for_template(&app, &state, &new_template)?;

    app.emit(TEMPLATES_CHANGED_EVENT, ())
        .map_err(|e| format!("emit failed: {e}"))?;
    Ok(new_template)
}

#[tauri::command]
pub fn set_pinned(
    app: AppHandle,
    state: State<'_, AppState>,
    id: Uuid,
    pinned: bool,
) -> Result<(), String> {
    let mut updated = {
        let map = state
            .templates
            .lock()
            .map_err(|e| format!("templates lock poisoned: {e}"))?;
        map.get(&id)
            .cloned()
            .ok_or_else(|| format!("template {id} not found"))?
    };
    if updated.is_pinned == pinned {
        return Ok(());
    }
    updated.is_pinned = pinned;
    updated.updated_at = Utc::now().to_rfc3339();

    let templates_dir = state.templates_dir();
    storage::save_template(&templates_dir, &updated)
        .map_err(|e| format!("save_template failed: {e:#}"))?;

    {
        let mut map = state
            .templates
            .lock()
            .map_err(|e| format!("templates lock poisoned: {e}"))?;
        info!(id = %id, pinned, "template pinned state changed");
        map.insert(id, updated);
    }
    app.emit(TEMPLATES_CHANGED_EVENT, ())
        .map_err(|e| format!("emit failed: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn show_palette(app: AppHandle) -> Result<(), String> {
    palette::show_palette(&app);
    Ok(())
}

#[tauri::command]
pub fn hide_palette(app: AppHandle) -> Result<(), String> {
    palette::hide_palette(&app);
    Ok(())
}

#[tauri::command]
pub fn show_main_window(app: AppHandle) -> Result<(), String> {
    palette::show_main_window(&app);
    Ok(())
}

#[tauri::command]
pub fn prepare_fill_dialog(
    app: AppHandle,
    state: State<'_, AppState>,
    id: Uuid,
) -> Result<FillDialogState, String> {
    let template = {
        let map = state
            .templates
            .lock()
            .map_err(|e| format!("templates lock poisoned: {e}"))?;
        map.get(&id)
            .cloned()
            .ok_or_else(|| format!("template {id} not found"))?
    };

    let clipboard_text = app.clipboard().read_text().ok();

    let last_used_map: HashMap<String, String> = {
        let lu = state
            .last_used
            .lock()
            .map_err(|e| format!("last_used lock poisoned: {e}"))?;
        lu.values.clone()
    };

    let ordered_variables: Vec<Variable> = render::order_variables_by_body_appearance(&template)
        .into_iter()
        .cloned()
        .collect();

    let mut initial_values: HashMap<Uuid, String> = HashMap::new();
    for var in &ordered_variables {
        let v = compute_initial_value(var, clipboard_text.as_deref(), &last_used_map);
        initial_values.insert(var.guid, v);
    }

    Ok(FillDialogState {
        template,
        initial_values,
        ordered_variables,
    })
}

fn compute_initial_value(
    var: &Variable,
    clipboard: Option<&str>,
    last_used_map: &HashMap<String, String>,
) -> String {
    if var.fill_from_clipboard {
        if let Some(text) = clipboard {
            if !text.is_empty() && is_valid_for_variable(var, text) {
                return text.to_string();
            }
        }
    }
    if var.remember_last_used {
        let key = var.display_name.to_lowercase();
        if let Some(stored) = last_used_map.get(&key) {
            if !stored.is_empty() && is_valid_for_variable(var, stored) {
                return stored.clone();
            }
        }
    }
    if let Some(d) = &var.static_default {
        if is_valid_for_variable(var, d) {
            return d.clone();
        }
    }
    String::new()
}

fn is_valid_for_variable(var: &Variable, value: &str) -> bool {
    match var.variable_type {
        VariableType::Text => true,
        VariableType::Enum => var
            .options
            .as_ref()
            .map_or(false, |opts| opts.iter().any(|o| o == value)),
    }
}

#[tauri::command]
pub fn apply_template(
    app: AppHandle,
    state: State<'_, AppState>,
    id: Uuid,
    values: HashMap<Uuid, String>,
) -> Result<ApplyOutcome, String> {
    let template = {
        let map = state
            .templates
            .lock()
            .map_err(|e| format!("templates lock poisoned: {e}"))?;
        map.get(&id)
            .cloned()
            .ok_or_else(|| format!("template {id} not found"))?
    };

    let rendered = render::render(&template.body, &values);

    // SPEC §8.1: clipboard write always happens; failure here is hard error.
    app.clipboard()
        .write_text(rendered.clone())
        .map_err(|e| format!("clipboard write failed: {e}"))?;

    let now = Utc::now().to_rfc3339();
    let mut updated = template.clone();
    updated.last_used_at = Some(now);
    updated.use_count = updated.use_count.saturating_add(1);

    let templates_dir = state.templates_dir();
    if let Err(e) = storage::save_template(&templates_dir, &updated) {
        warn!(error = ?e, "writing updated template metadata failed");
    } else {
        let mut map = state
            .templates
            .lock()
            .map_err(|e| format!("templates lock poisoned: {e}"))?;
        map.insert(updated.id, updated);
    }

    let mut updated_any = false;
    {
        let mut lu = state
            .last_used
            .lock()
            .map_err(|e| format!("last_used lock poisoned: {e}"))?;
        for var in &template.variables {
            if var.remember_last_used {
                if let Some(value) = values.get(&var.guid) {
                    if !value.is_empty() {
                        lu.values
                            .insert(var.display_name.to_lowercase(), value.clone());
                        updated_any = true;
                    }
                }
            }
        }
    }
    if updated_any {
        let lu_path = state.last_used_path();
        let lu_snapshot = {
            let lu = state
                .last_used
                .lock()
                .map_err(|e| format!("last_used lock poisoned: {e}"))?;
            lu.clone()
        };
        if let Err(e) = storage::atomic_write(&lu_path, &lu_snapshot) {
            warn!(error = ?e, "writing last-used.json failed");
        }
    }

    // SPEC §4.6 / §8.2: if autoPaste is enabled, focus the cached HWND and
    // simulate Ctrl+V. Any failure here falls back to clipboard-only.
    let auto_paste_enabled = {
        let s = state
            .settings
            .lock()
            .map_err(|e| format!("settings lock poisoned: {e}"))?;
        s.auto_paste
    };

    let outcome = if !auto_paste_enabled {
        ApplyOutcome {
            pasted: false,
            reason: Some("disabled".to_string()),
        }
    } else {
        let hwnd = state.cached_hwnd.load(Ordering::Relaxed);
        match auto_paste::paste_into(hwnd) {
            Ok(()) => ApplyOutcome {
                pasted: true,
                reason: None,
            },
            Err(e) => {
                warn!(error = %e, hwnd, "auto-paste failed; falling back to clipboard-only");
                ApplyOutcome {
                    pasted: false,
                    reason: Some("failed".to_string()),
                }
            }
        }
    };

    info!(
        id = %id,
        len = rendered.len(),
        pasted = outcome.pasted,
        reason = ?outcome.reason,
        "template applied",
    );

    app.emit(TEMPLATES_CHANGED_EVENT, ())
        .map_err(|e| format!("emit failed: {e}"))?;
    Ok(outcome)
}

// --- Color management commands -------------------------------------------------

#[tauri::command]
pub fn get_variable_colors(state: State<'_, AppState>) -> Result<VariableColorMap, String> {
    let map = state
        .variable_colors
        .lock()
        .map_err(|e| format!("variable_colors lock poisoned: {e}"))?;
    Ok(map.clone())
}

#[tauri::command]
pub fn get_tag_colors(state: State<'_, AppState>) -> Result<TagColorMap, String> {
    let map = state
        .tag_colors
        .lock()
        .map_err(|e| format!("tag_colors lock poisoned: {e}"))?;
    Ok(map.clone())
}

#[tauri::command]
pub fn save_variable_colors(
    app: AppHandle,
    state: State<'_, AppState>,
    map: HashMap<String, String>,
) -> Result<(), String> {
    let new_map = VariableColorMap {
        schema_version: CURRENT_SCHEMA_VERSION,
        map,
    };
    let path = state.variable_colors_path();
    storage::atomic_write(&path, &new_map).map_err(|e| format!("write failed: {e:#}"))?;
    *state
        .variable_colors
        .lock()
        .map_err(|e| format!("variable_colors lock poisoned: {e}"))? = new_map;
    app.emit(COLORS_CHANGED_EVENT, ())
        .map_err(|e| format!("emit failed: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn save_tag_colors(
    app: AppHandle,
    state: State<'_, AppState>,
    map: HashMap<String, String>,
) -> Result<(), String> {
    let new_map = TagColorMap {
        schema_version: CURRENT_SCHEMA_VERSION,
        map,
    };
    let path = state.tag_colors_path();
    storage::atomic_write(&path, &new_map).map_err(|e| format!("write failed: {e:#}"))?;
    *state
        .tag_colors
        .lock()
        .map_err(|e| format!("tag_colors lock poisoned: {e}"))? = new_map;
    app.emit(COLORS_CHANGED_EVENT, ())
        .map_err(|e| format!("emit failed: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn random_color() -> Result<String, String> {
    Ok(color::random_oklch())
}

// --- Settings commands ---------------------------------------------------------

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    let s = state
        .settings
        .lock()
        .map_err(|e| format!("settings lock poisoned: {e}"))?;
    Ok(s.clone())
}

#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: Settings,
) -> Result<(), String> {
    let mut s = settings;
    // Don't trust frontend's schemaVersion — pin to current.
    s.schema_version = CURRENT_SCHEMA_VERSION;

    let path = state.settings_path();
    storage::atomic_write(&path, &s).map_err(|e| format!("write failed: {e:#}"))?;

    *state
        .settings
        .lock()
        .map_err(|e| format!("settings lock poisoned: {e}"))? = s;

    app.emit(SETTINGS_CHANGED_EVENT, ())
        .map_err(|e| format!("emit failed: {e}"))?;
    Ok(())
}

// --- Onboarding commands -------------------------------------------------------
//
// SPEC §11 三选一: default-path-new / custom-path-new / import-existing.
// These are usable BEFORE AppState is managed (during first-launch); they take
// AppHandle, not State<AppState>. After `complete_onboarding_*` succeeds, the
// shared `crate::complete_onboarding` helper builds AppState and registers it.

#[tauri::command]
pub fn default_data_folder(app: AppHandle) -> Result<String, String> {
    paths::default_data_folder(&app)
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| format!("resolving default data folder: {e:#}"))
}

/// Return the resolved data folder currently in use (custom path from
/// bootstrap, or OS default if bootstrap doesn't specify one). Used by the
/// Settings page to display the active path.
#[tauri::command]
pub fn current_data_folder(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.data_folder.to_string_lossy().to_string())
}

#[tauri::command]
pub fn validate_path_for_new(path: String) -> Result<DataFolderStatus, String> {
    let p = std::path::PathBuf::from(&path);
    onboarding::classify_path(&p).map_err(|e| format!("classify failed: {e:#}"))
}

#[tauri::command]
pub fn validate_path_for_import(path: String) -> Result<DataFolderStatus, String> {
    let p = std::path::PathBuf::from(&path);
    onboarding::classify_path(&p).map_err(|e| format!("classify failed: {e:#}"))
}

#[tauri::command]
pub fn complete_onboarding_default(app: AppHandle) -> Result<(), String> {
    crate::complete_onboarding(&app, |b| {
        b.data_folder_path = None;
    })
    .map_err(|e| format!("complete_onboarding_default failed: {e:#}"))
}

#[tauri::command]
pub fn complete_onboarding_custom_new(app: AppHandle, path: String) -> Result<(), String> {
    let p = std::path::PathBuf::from(&path);
    let status =
        onboarding::classify_path(&p).map_err(|e| format!("classify failed: {e:#}"))?;
    if !matches!(
        status,
        DataFolderStatus::DoesNotExist | DataFolderStatus::Empty
    ) {
        return Err(format!(
            "path not eligible for new install (status: {:?})",
            status
        ));
    }
    crate::complete_onboarding(&app, move |b| {
        b.data_folder_path = Some(path);
    })
    .map_err(|e| format!("complete_onboarding_custom_new failed: {e:#}"))
}

#[tauri::command]
pub fn complete_onboarding_import(app: AppHandle, path: String) -> Result<(), String> {
    let p = std::path::PathBuf::from(&path);
    let status =
        onboarding::classify_path(&p).map_err(|e| format!("classify failed: {e:#}"))?;
    if status != DataFolderStatus::ValidSnippet {
        return Err(format!(
            "path is not a valid Snippet folder (status: {:?})",
            status
        ));
    }
    crate::complete_onboarding(&app, move |b| {
        b.data_folder_path = Some(path);
    })
    .map_err(|e| format!("complete_onboarding_import failed: {e:#}"))
}

/// Quit the app. Used by the Settings page after changing dataFolderPath
/// (SPEC §12: change requires a restart). Bypasses the close-handler hide
/// semantics that main/palette windows use.
#[tauri::command]
pub fn exit_app(app: AppHandle) -> Result<(), String> {
    info!("exit_app requested from frontend");
    app.exit(0);
    Ok(())
}

#[tauri::command]
pub fn set_data_folder_path(app: AppHandle, path: Option<String>) -> Result<(), String> {
    let boot_path = paths::bootstrap_path(&app)
        .map_err(|e| format!("resolving bootstrap path: {e:#}"))?;
    if !boot_path.exists() {
        return Err("bootstrap.json not found — onboarding must be completed first".to_string());
    }
    let mut bootstrap: Bootstrap =
        storage::read_json(&boot_path).map_err(|e| format!("reading bootstrap: {e:#}"))?;
    bootstrap.data_folder_path = path.clone();
    storage::atomic_write(&boot_path, &bootstrap)
        .map_err(|e| format!("writing bootstrap: {e:#}"))?;
    info!(
        ?path,
        "bootstrap.data_folder_path updated; restart required to take effect"
    );
    Ok(())
}

fn ensure_colors_for_template(
    app: &AppHandle,
    state: &State<'_, AppState>,
    template: &Template,
) -> Result<(), String> {
    let mut var_changed = false;
    {
        let mut map = state
            .variable_colors
            .lock()
            .map_err(|e| format!("variable_colors lock poisoned: {e}"))?;
        for v in &template.variables {
            let key = v.display_name.to_lowercase();
            if !key.is_empty() && !map.map.contains_key(&key) {
                map.map.insert(key, color::random_oklch());
                var_changed = true;
            }
        }
    }
    if var_changed {
        let path = state.variable_colors_path();
        let snap = state
            .variable_colors
            .lock()
            .map_err(|e| format!("variable_colors lock poisoned: {e}"))?
            .clone();
        storage::atomic_write(&path, &snap).map_err(|e| format!("write failed: {e:#}"))?;
    }

    let mut tag_changed = false;
    {
        let mut map = state
            .tag_colors
            .lock()
            .map_err(|e| format!("tag_colors lock poisoned: {e}"))?;
        for tag in &template.tags {
            let key = tag.to_lowercase();
            if !key.is_empty() && !map.map.contains_key(&key) {
                map.map.insert(key, color::random_oklch());
                tag_changed = true;
            }
        }
    }
    if tag_changed {
        let path = state.tag_colors_path();
        let snap = state
            .tag_colors
            .lock()
            .map_err(|e| format!("tag_colors lock poisoned: {e}"))?
            .clone();
        storage::atomic_write(&path, &snap).map_err(|e| format!("write failed: {e:#}"))?;
    }

    if var_changed || tag_changed {
        app.emit(COLORS_CHANGED_EVENT, ())
            .map_err(|e| format!("emit failed: {e}"))?;
    }
    Ok(())
}
