//! Data schemas. See SPEC.md §3 for field definitions and constraints.
//! All types derive `TS` for double-ended type sync; serde camelCase to match
//! the JSON examples in SPEC.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;
use uuid::Uuid;

/// Current schema version for templates and config files.
/// Bump when introducing breaking changes; add a migration in the schema-migration chain.
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/lib/bindings/")]
#[serde(rename_all = "camelCase")]
pub struct Template {
    pub schema_version: u32,
    pub id: Uuid,
    pub display_name: String,
    pub body: String,
    pub variables: Vec<Variable>,
    pub tags: Vec<String>,
    pub is_pinned: bool,
    pub created_at: String,
    pub updated_at: String,
    pub last_used_at: Option<String>,
    pub use_count: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/lib/bindings/")]
#[serde(rename_all = "camelCase")]
pub struct Variable {
    pub guid: Uuid,
    pub display_name: String,
    #[serde(rename = "type")]
    pub variable_type: VariableType,
    pub options: Option<Vec<String>>,
    pub required: bool,
    pub fill_from_clipboard: bool,
    pub remember_last_used: bool,
    pub static_default: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/lib/bindings/")]
#[serde(rename_all = "lowercase")]
pub enum VariableType {
    Text,
    Enum,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/lib/bindings/")]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub schema_version: u32,
    pub hotkey: String,
    pub auto_paste: bool,
    pub theme: ThemePreference,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            hotkey: "Ctrl+Alt+Space".to_string(),
            auto_paste: false,
            theme: ThemePreference::System,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/lib/bindings/")]
#[serde(rename_all = "lowercase")]
pub enum ThemePreference {
    Light,
    Dark,
    System,
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/lib/bindings/")]
#[serde(rename_all = "camelCase")]
pub struct Bootstrap {
    pub schema_version: u32,
    pub data_folder_path: Option<String>,
}

impl Default for Bootstrap {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            data_folder_path: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/lib/bindings/")]
#[serde(rename_all = "camelCase")]
pub struct VariableColorMap {
    pub schema_version: u32,
    pub map: HashMap<String, String>,
}

impl Default for VariableColorMap {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            map: HashMap::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/lib/bindings/")]
#[serde(rename_all = "camelCase")]
pub struct TagColorMap {
    pub schema_version: u32,
    pub map: HashMap<String, String>,
}

impl Default for TagColorMap {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            map: HashMap::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/lib/bindings/")]
#[serde(rename_all = "camelCase")]
pub struct LastUsed {
    pub schema_version: u32,
    pub values: HashMap<String, String>,
}

impl Default for LastUsed {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            values: HashMap::new(),
        }
    }
}
