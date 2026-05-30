mod auto_paste;
mod color;
mod commands;
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
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use tracing::{info, warn};
use ts_rs::TS;

use crate::commands::{
    apply_template, delete_template, duplicate_template, get_settings, get_tag_colors,
    get_template, get_variable_colors, hide_palette, list_templates, prepare_fill_dialog,
    random_color, save_settings, save_tag_colors, save_template, save_variable_colors,
    search_templates, set_pinned, show_main_window, show_palette,
};
use crate::schema::{Bootstrap, LastUsed, Settings, TagColorMap, VariableColorMap};
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
            palette::show_main_window(app);
        }))
        .plugin(tauri_plugin_clipboard_manager::init())
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
        ])
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let label = window.label();
                if label == "main" || label == "palette" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .setup(|app| {
            let app_state =
                init_app_state(app.handle()).expect("failed to initialize app state");
            app.manage(app_state);

            let hotkey = Shortcut::new(
                Some(Modifiers::CONTROL | Modifiers::ALT),
                Code::Space,
            );
            match app.global_shortcut().register(hotkey) {
                Ok(()) => info!("global hotkey registered: Ctrl+Alt+Space"),
                Err(e) => warn!(error = ?e, "failed to register global hotkey"),
            }

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
                        palette::show_main_window(tray.app_handle());
                    }
                })
                .build(app)?;

            info!("tray icon created");
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error building tauri application");

    app.run(|app, event| {
        if let RunEvent::ExitRequested { .. } = event {
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

/// Run all the startup file IO and build the AppState.
/// See ARCHITECTURE §5 启动流程.
fn init_app_state(app: &AppHandle) -> anyhow::Result<AppState> {
    let boot_path = paths::bootstrap_path(app)?;
    let bootstrap: Bootstrap = storage::load_or_init(
        &boot_path,
        Bootstrap::default,
        |b| b.schema_version,
    )?;
    info!(path = ?boot_path, ?bootstrap, "bootstrap loaded");

    let data_folder = match &bootstrap.data_folder_path {
        Some(p) => std::path::PathBuf::from(p),
        None => paths::default_data_folder(app)?,
    };

    storage::ensure_data_folder_structure(&data_folder)?;
    info!(path = ?data_folder, "dataFolder ready");

    let settings: Settings = storage::load_or_init(
        &data_folder.join("settings.json"),
        Settings::default,
        |s| s.schema_version,
    )?;
    let variable_colors: VariableColorMap = storage::load_or_init(
        &data_folder.join("variable-colors.json"),
        VariableColorMap::default,
        |c| c.schema_version,
    )?;
    let tag_colors: TagColorMap = storage::load_or_init(
        &data_folder.join("tag-colors.json"),
        TagColorMap::default,
        |c| c.schema_version,
    )?;
    let last_used: LastUsed = storage::load_or_init(
        &data_folder.join("last-used.json"),
        LastUsed::default,
        |l| l.schema_version,
    )?;

    let templates_dir = data_folder.join("templates");
    let templates = storage::load_templates(&templates_dir)?;
    info!(count = templates.len(), "templates loaded");

    let state = AppState::new(
        data_folder,
        templates,
        last_used,
        variable_colors,
        tag_colors,
        settings,
    );

    if let Err(e) = color::reconcile_colors(&state) {
        warn!(error = ?e, "startup color reconcile failed");
    }

    Ok(state)
}

fn init_tracing() {
    use tracing_subscriber::EnvFilter;
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("snippet_lib=debug,snippet=debug,tauri=info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
