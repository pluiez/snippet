//! Application runtime state shared across IPC commands.

use crate::schema::{LastUsed, Settings, TagColorMap, Template, VariableColorMap};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicIsize;
use std::sync::Mutex;
use uuid::Uuid;

pub struct AppState {
    /// Resolved data folder root (custom path from bootstrap or OS default).
    pub data_folder: PathBuf,
    /// In-memory template index, keyed by template UUID.
    pub templates: Mutex<HashMap<Uuid, Template>>,
    /// Last-used variable values, keyed by lowercased displayName (SPEC §5.5).
    pub last_used: Mutex<LastUsed>,
    /// Variable color map, keyed by lowercased variable displayName (SPEC §6).
    pub variable_colors: Mutex<VariableColorMap>,
    /// Tag color map, keyed by lowercased tag (SPEC §6).
    pub tag_colors: Mutex<TagColorMap>,
    /// App settings (hotkey, autoPaste, theme). Slice 6 consumes auto_paste.
    pub settings: Mutex<Settings>,
    /// Foreground window handle captured at the moment of hotkey press
    /// (Windows only; 0 elsewhere). Per ARCHITECTURE §6 the capture must be
    /// the first synchronous step in the hotkey callback. Slice 6 (autoPaste)
    /// reads it to send Ctrl+V back to the previous app.
    pub cached_hwnd: AtomicIsize,
}

impl AppState {
    pub fn new(
        data_folder: PathBuf,
        templates: HashMap<Uuid, Template>,
        last_used: LastUsed,
        variable_colors: VariableColorMap,
        tag_colors: TagColorMap,
        settings: Settings,
    ) -> Self {
        Self {
            data_folder,
            templates: Mutex::new(templates),
            last_used: Mutex::new(last_used),
            variable_colors: Mutex::new(variable_colors),
            tag_colors: Mutex::new(tag_colors),
            settings: Mutex::new(settings),
            cached_hwnd: AtomicIsize::new(0),
        }
    }

    pub fn templates_dir(&self) -> PathBuf {
        self.data_folder.join("templates")
    }

    pub fn last_used_path(&self) -> PathBuf {
        self.data_folder.join("last-used.json")
    }

    pub fn variable_colors_path(&self) -> PathBuf {
        self.data_folder.join("variable-colors.json")
    }

    pub fn tag_colors_path(&self) -> PathBuf {
        self.data_folder.join("tag-colors.json")
    }

    pub fn settings_path(&self) -> PathBuf {
        self.data_folder.join("settings.json")
    }
}
