mod auto_paste;
mod color;
mod commands;
mod hotkey;
mod onboarding;
mod palette;
mod paths;
mod render;
mod schema;
mod search;
mod state;
mod storage;

use serde::{Deserialize, Serialize};
use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, RunEvent, WindowEvent,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use tracing::{info, warn};
use ts_rs::TS;

use crate::commands::{
    apply_template, complete_onboarding_custom_new, complete_onboarding_default,
    complete_onboarding_import, current_data_folder, default_data_folder, delete_template,
    duplicate_template, exit_app, get_settings, get_startup_warnings, get_tag_colors,
    get_template, get_variable_colors, hide_palette, list_templates, prepare_fill_dialog,
    random_color, save_settings, save_tag_colors, save_template, save_variable_colors,
    search_templates, set_data_folder_path, pause_hotkey, resume_hotkey, set_pinned,
    show_main_window, show_palette, validate_hotkey, validate_path_for_import,
    validate_path_for_new,
};
use crate::schema::{Bootstrap, LastUsed, Settings, StartupWarning, TagColorMap, VariableColorMap};
use crate::state::AppState;

#[derive(Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/lib/bindings/")]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub name: String,
    pub version: String,
}

#[tauri::command]
fn app_info() -> AppInfo {
    AppInfo {
        name: "Snippet".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_tracing();
    info!(version = env!("CARGO_PKG_VERSION"), "starting snippet");

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, args, cwd| {
            info!(?args, ?cwd, "second instance launched, focusing main window");
            // During onboarding (AppState not yet managed), surface the
            // onboarding window instead of the main window.
            if app.try_state::<AppState>().is_none() {
                if let Some(w) = app.get_webview_window("onboarding") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
                return;
            }
            palette::show_main_window(app);
        }))
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        palette::on_hotkey(app);
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            app_info,
            list_templates,
            search_templates,
            get_template,
            save_template,
            delete_template,
            duplicate_template,
            set_pinned,
            prepare_fill_dialog,
            apply_template,
            show_palette,
            hide_palette,
            show_main_window,
            get_variable_colors,
            get_tag_colors,
            save_variable_colors,
            save_tag_colors,
            random_color,
            get_settings,
            save_settings,
            validate_hotkey,
            pause_hotkey,
            resume_hotkey,
            default_data_folder,
            current_data_folder,
            validate_path_for_new,
            validate_path_for_import,
            complete_onboarding_default,
            complete_onboarding_custom_new,
            complete_onboarding_import,
            set_data_folder_path,
            get_startup_warnings,
            exit_app,
        ])
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let label = window.label();
                match label {
                    "main" | "palette" => {
                        api.prevent_close();
                        let _ = window.hide();
                    }
                    "onboarding" => {
                        // SPEC §11: closing onboarding before completing it =
                        // cancel = quit app. The ExitRequested handler below
                        // already guards on `try_state::<AppState>()` so
                        // shutdown GC is skipped when state isn't managed yet.
                        window.app_handle().exit(0);
                    }
                    _ => {}
                }
            }
        })
        .setup(|app| {
            // Phase A: always read/init bootstrap. This step is cheap and runs
            // even on first launch where bootstrap.json doesn't exist yet.
            let bootstrap =
                init_bootstrap(app.handle()).expect("failed to initialize bootstrap");

            // Tray is created unconditionally so the user can re-summon the
            // onboarding window via tray-click during onboarding. The tray
            // click handler discriminates on whether AppState is managed.
            let icon = app
                .default_window_icon()
                .expect("default window icon must be configured in tauri.conf.json")
                .clone();
            TrayIconBuilder::new()
                .icon(icon)
                .tooltip("Snippet")
                .on_tray_icon_event(|tray, event| {
                    if matches!(
                        event,
                        TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        }
                    ) {
                        let handle = tray.app_handle();
                        if handle.try_state::<AppState>().is_none() {
                            if let Some(w) = handle.get_webview_window("onboarding") {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                            return;
                        }
                        palette::show_main_window(handle);
                    }
                })
                .build(app)?;
            info!("tray icon created");

            // Phase B: if onboarding was already completed previously, finish
            // the regular init flow. Otherwise show the onboarding window and
            // wait for the user — finalization continues in
            // `complete_onboarding` (called from IPC).
            if onboarding::needs_onboarding(&bootstrap) {
                info!("onboarding incomplete — showing onboarding window");
                if let Some(w) = app.get_webview_window("onboarding") {
                    let _ = w.show();
                    let _ = w.set_focus();
                } else {
                    warn!("onboarding window not found in tauri.conf.json");
                }
            } else {
                let state = init_full_state(app.handle(), &bootstrap)
                    .expect("failed to initialize app state");
                app.manage(state);
                if let Some(w) = register_hotkey_from_settings(app.handle()) {
                    if let Some(s) = app.try_state::<AppState>() {
                        if let Ok(mut warnings) = s.startup_warnings.lock() {
                            warnings.push(w);
                        }
                    }
                }
            }

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error building tauri application");

    app.run(|app, event| {
        if let RunEvent::ExitRequested { .. } = event {
            // GC only runs if AppState exists — during onboarding cancel
            // (`app.exit(0)` from close handler) state is unmanaged.
            if let Some(state) = app.try_state::<AppState>() {
                if let Err(e) = color::reconcile_colors(&state) {
                    warn!(error = ?e, "shutdown color reconcile failed");
                } else {
                    info!("shutdown color reconcile complete");
                }
            }
        }
    });
}

/// Phase A: read or default-init `bootstrap.json`. Always runs at startup.
fn init_bootstrap(app: &AppHandle) -> anyhow::Result<Bootstrap> {
    let boot_path = paths::bootstrap_path(app)?;
    let (bootstrap, _recovered) =
        storage::load_or_init(&boot_path, Bootstrap::default, |b| b.schema_version)?;
    info!(path = ?boot_path, ?bootstrap, "bootstrap loaded");
    Ok(bootstrap)
}

/// Phase B: resolve data folder, ensure structure, load all config files,
/// scan templates, build AppState. Called either from `setup` (when
/// onboarding was already complete) or from `complete_onboarding` (after the
/// user finishes the picker).
fn init_full_state(app: &AppHandle, bootstrap: &Bootstrap) -> anyhow::Result<AppState> {
    let data_folder = match &bootstrap.data_folder_path {
        Some(p) => std::path::PathBuf::from(p),
        None => paths::default_data_folder(app)?,
    };

    storage::ensure_data_folder_structure(&data_folder)?;
    info!(path = ?data_folder, "dataFolder ready");

    // Write-permission probe: try write + delete a sentinel file.
    let probe_path = data_folder.join(".snippet-write-test");
    match std::fs::write(&probe_path, b"probe") {
        Ok(()) => {
            let _ = std::fs::remove_file(&probe_path);
        }
        Err(e) => {
            warn!(error = ?e, path = ?data_folder, "data folder is not writable");
            use tauri_plugin_dialog::DialogExt;
            app.dialog()
                .message(format!(
                    "数据文件夹无写权限：\n{}\n\n请检查文件夹权限或在设置中更改路径。\n\n错误详情：{}",
                    data_folder.display(),
                    e
                ))
                .title("Snippet 启动错误")
                .blocking_show();
            std::process::exit(1);
        }
    }

    let mut warnings: Vec<StartupWarning> = Vec::new();

    let (settings, settings_recovered) = storage::load_or_init(
        &data_folder.join("settings.json"),
        Settings::default,
        |s| s.schema_version,
    )?;
    if settings_recovered {
        warnings.push(StartupWarning {
            kind: "corrupt_config".to_string(),
            message: "settings.json 已损坏或版本不匹配，已重置为默认设置。".to_string(),
        });
    }

    let (variable_colors, vc_recovered) = storage::load_or_init(
        &data_folder.join("variable-colors.json"),
        VariableColorMap::default,
        |c| c.schema_version,
    )?;
    if vc_recovered {
        warnings.push(StartupWarning {
            kind: "corrupt_config".to_string(),
            message: "variable-colors.json 已损坏，变量颜色已重置。".to_string(),
        });
    }

    let (tag_colors, tc_recovered) = storage::load_or_init(
        &data_folder.join("tag-colors.json"),
        TagColorMap::default,
        |c| c.schema_version,
    )?;
    if tc_recovered {
        warnings.push(StartupWarning {
            kind: "corrupt_config".to_string(),
            message: "tag-colors.json 已损坏，标签颜色已重置。".to_string(),
        });
    }

    let (last_used, lu_recovered) = storage::load_or_init(
        &data_folder.join("last-used.json"),
        LastUsed::default,
        |l| l.schema_version,
    )?;
    if lu_recovered {
        warnings.push(StartupWarning {
            kind: "corrupt_config".to_string(),
            message: "last-used.json 已损坏，上次使用记录已重置。".to_string(),
        });
    }

    let templates_dir = data_folder.join("templates");
    let (templates, invalid_count) = storage::load_templates(&templates_dir)?;
    info!(count = templates.len(), invalid_count, "templates loaded");
    if invalid_count > 0 {
        warnings.push(StartupWarning {
            kind: "invalid_templates".to_string(),
            message: format!(
                "发现 {} 个损坏的模板文件，已移至 templates/.invalid/ 目录。",
                invalid_count
            ),
        });
    }

    let state = AppState::new(
        data_folder,
        templates,
        last_used,
        variable_colors,
        tag_colors,
        settings,
        warnings,
    );

    if let Err(e) = color::reconcile_colors(&state) {
        warn!(error = ?e, "startup color reconcile failed");
    }

    Ok(state)
}

/// Register the global hotkey from `settings.hotkey`. Best-effort: parse
/// failures or registration failures are logged as warnings (not fatal) — the
/// app keeps running, just without a working palette shortcut. The user can
/// fix the keybinding from the Settings page.
///
/// Caller must `app.manage(state)` BEFORE calling this — we read the hotkey
/// string out of `AppState.settings`. Returns `None` on success, or a
/// `StartupWarning` if registration failed.
fn register_hotkey_from_settings(app: &AppHandle) -> Option<StartupWarning> {
    let Some(state) = app.try_state::<AppState>() else {
        warn!("register_hotkey_from_settings called before AppState was managed");
        return None;
    };
    let hotkey_str = match state.settings.lock() {
        Ok(s) => s.hotkey.clone(),
        Err(e) => {
            warn!(error = ?e, "settings lock poisoned; skipping hotkey register");
            return None;
        }
    };
    match hotkey::parse_hotkey(&hotkey_str) {
        Ok(shortcut) => match app.global_shortcut().register(shortcut) {
            Ok(()) => {
                info!(hotkey = %hotkey_str, "global hotkey registered");
                None
            }
            Err(e) => {
                warn!(error = ?e, hotkey = %hotkey_str, "failed to register global hotkey");
                Some(StartupWarning {
                    kind: "hotkey_failed".to_string(),
                    message: format!(
                        "全局热键 '{}' 注册失败，可能被其它程序占用。请在设置中更改。",
                        hotkey_str
                    ),
                })
            }
        },
        Err(e) => {
            warn!(
                error = %e,
                hotkey = %hotkey_str,
                "settings.hotkey unparseable; no hotkey active"
            );
            Some(StartupWarning {
                kind: "hotkey_failed".to_string(),
                message: format!(
                    "热键 '{}' 格式无效，无法注册。请在设置中更改。",
                    hotkey_str
                ),
            })
        }
    }
}

/// Finalize onboarding: write the chosen bootstrap, build AppState, register
/// hotkey, hide the onboarding window. Called from the three
/// `complete_onboarding_*` IPC handlers, each of which supplies a closure
/// that sets `data_folder_path` appropriately.
///
/// Idempotency: if AppState is already managed (the user somehow finished
/// twice), this returns an error rather than panicking on `app.manage`.
pub fn complete_onboarding(
    app: &AppHandle,
    set_data_folder: impl FnOnce(&mut Bootstrap),
) -> anyhow::Result<()> {
    if app.try_state::<AppState>().is_some() {
        anyhow::bail!("onboarding already finalized — AppState is managed");
    }

    let boot_path = paths::bootstrap_path(app)?;
    let mut bootstrap: Bootstrap = if boot_path.exists() {
        storage::read_json(&boot_path)?
    } else {
        Bootstrap::default()
    };
    set_data_folder(&mut bootstrap);
    bootstrap.onboarding_complete = true;
    storage::atomic_write(&boot_path, &bootstrap)?;
    info!(?bootstrap, "bootstrap written; onboarding complete");

    let state = init_full_state(app, &bootstrap)?;
    app.manage(state);
    if let Some(w) = register_hotkey_from_settings(app) {
        if let Some(s) = app.try_state::<AppState>() {
            if let Ok(mut warnings) = s.startup_warnings.lock() {
                warnings.push(w);
            }
        }
    }

    if let Some(w) = app.get_webview_window("onboarding") {
        let _ = w.hide();
    }
    info!("onboarding finalized; app fully initialized");
    Ok(())
}

fn init_tracing() {
    use tracing_subscriber::EnvFilter;
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("snippet_lib=debug,snippet=debug,tauri=info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
