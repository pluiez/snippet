//! File I/O: atomic writes, JSON load/save, template scan, schema-version checks.

use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{info, warn};
use uuid::Uuid;

use crate::schema::{Template, CURRENT_SCHEMA_VERSION};

/// Atomic JSON write: serialize → write to `<path>.tmp` → rename onto `<path>`.
pub fn atomic_write<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating parent dir for {}", path.display()))?;
    }
    let json = serde_json::to_vec_pretty(value).context("serializing JSON")?;
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, &json).with_context(|| format!("writing {}", tmp.display()))?;
    std::fs::rename(&tmp, path)
        .with_context(|| format!("renaming {} → {}", tmp.display(), path.display()))?;
    Ok(())
}

pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("parsing JSON at {}", path.display()))
}

/// Load a typed config file; if missing, version-mismatched, or corrupt,
/// write a default and return that. Used for bootstrap / settings / color
/// maps / last-used (i.e. single-doc files, not template files).
///
/// Returns `(value, recovered)` where `recovered` is true if the file was
/// missing, corrupt, or had a mismatched schema version (i.e. the default
/// was written). Callers can use this to surface startup warnings.
pub fn load_or_init<T>(
    path: &Path,
    default: impl FnOnce() -> T,
    schema_version: impl Fn(&T) -> u32,
) -> Result<(T, bool)>
where
    T: Serialize + DeserializeOwned,
{
    if path.exists() {
        match read_json::<T>(path) {
            Ok(v) if schema_version(&v) == CURRENT_SCHEMA_VERSION => Ok((v, false)),
            Ok(v) => {
                warn!(
                    path = ?path,
                    found = schema_version(&v),
                    expected = CURRENT_SCHEMA_VERSION,
                    "schema version mismatch; replacing with default"
                );
                let d = default();
                atomic_write(path, &d)?;
                Ok((d, true))
            }
            Err(e) => {
                warn!(path = ?path, error = ?e, "corrupt JSON; replacing with default");
                let d = default();
                atomic_write(path, &d)?;
                Ok((d, true))
            }
        }
    } else {
        let d = default();
        atomic_write(path, &d)?;
        info!(path = ?path, "created default file");
        Ok((d, false))
    }
}

/// Make sure dataFolder/templates/ and dataFolder/templates/.invalid/ exist.
pub fn ensure_data_folder_structure(root: &Path) -> Result<()> {
    let templates = root.join("templates");
    let invalid = templates.join(".invalid");
    std::fs::create_dir_all(&templates)
        .with_context(|| format!("creating {}", templates.display()))?;
    std::fs::create_dir_all(&invalid)
        .with_context(|| format!("creating {}", invalid.display()))?;
    Ok(())
}

/// Scan dataFolder/templates/*.json and load all valid templates.
/// Corrupt or version-mismatched files are moved to .invalid/.
///
/// Returns `(templates, invalid_count)` where `invalid_count` is the number
/// of files that were moved to `.invalid/`. Callers can use this to surface
/// startup warnings.
pub fn load_templates(templates_dir: &Path) -> Result<(HashMap<Uuid, Template>, usize)> {
    let mut out = HashMap::new();
    let mut invalid_count: usize = 0;
    if !templates_dir.exists() {
        return Ok((out, 0));
    }
    for entry in std::fs::read_dir(templates_dir)
        .with_context(|| format!("reading {}", templates_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().map_or(true, |e| e != "json") {
            continue;
        }
        match read_json::<Template>(&path) {
            Ok(t) if t.schema_version == CURRENT_SCHEMA_VERSION => {
                out.insert(t.id, t);
            }
            Ok(t) => {
                warn!(
                    path = ?path,
                    found = t.schema_version,
                    expected = CURRENT_SCHEMA_VERSION,
                    "template schema version mismatch; moving to .invalid/"
                );
                if let Err(e) = move_to_invalid(&path) {
                    warn!(path = ?path, error = ?e, "failed to move to .invalid/");
                }
                invalid_count += 1;
            }
            Err(e) => {
                warn!(path = ?path, error = ?e, "template parse failed; moving to .invalid/");
                if let Err(e2) = move_to_invalid(&path) {
                    warn!(path = ?path, error = ?e2, "failed to move to .invalid/");
                }
                invalid_count += 1;
            }
        }
    }
    Ok((out, invalid_count))
}

fn move_to_invalid(path: &Path) -> Result<()> {
    let parent = path.parent().context("template has no parent dir")?;
    let name = path.file_name().context("template has no file name")?;
    let dest = parent.join(".invalid").join(name);
    if let Some(d_parent) = dest.parent() {
        std::fs::create_dir_all(d_parent)?;
    }
    std::fs::rename(path, &dest)
        .with_context(|| format!("moving {} → {}", path.display(), dest.display()))?;
    Ok(())
}

pub fn template_path(templates_dir: &Path, id: &Uuid) -> PathBuf {
    templates_dir.join(format!("{}.json", id))
}

pub fn save_template(templates_dir: &Path, template: &Template) -> Result<()> {
    let path = template_path(templates_dir, &template.id);
    atomic_write(&path, template)
}

pub fn delete_template(templates_dir: &Path, id: &Uuid) -> Result<()> {
    let path = template_path(templates_dir, id);
    if path.exists() {
        std::fs::remove_file(&path)
            .with_context(|| format!("removing {}", path.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{Bootstrap, Settings, Variable, VariableType};
    use tempfile::tempdir;

    fn mk_template(name: &str) -> Template {
        Template {
            schema_version: CURRENT_SCHEMA_VERSION,
            id: Uuid::new_v4(),
            display_name: name.to_string(),
            body: "body".to_string(),
            variables: vec![],
            tags: vec![],
            is_pinned: false,
            created_at: "2026-05-31T00:00:00Z".to_string(),
            updated_at: "2026-05-31T00:00:00Z".to_string(),
            last_used_at: None,
            use_count: 0,
        }
    }

    /// `atomic_write` creates missing parent directories so a fresh data
    /// folder doesn't need to be pre-seeded by the caller.
    #[test]
    fn atomic_write_creates_parent_dirs() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("a/b/c/file.json");
        let value = Settings::default();
        atomic_write(&path, &value).unwrap();
        assert!(path.exists(), "file should exist after atomic_write");
    }

    /// `atomic_write` → `read_json` round-trips a typed value identically.
    #[test]
    fn atomic_write_round_trips_value() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("settings.json");
        let mut value = Settings::default();
        value.hotkey = "Ctrl+Alt+K".to_string();
        value.auto_paste = true;
        atomic_write(&path, &value).unwrap();
        let loaded: Settings = read_json(&path).unwrap();
        assert_eq!(loaded.hotkey, "Ctrl+Alt+K");
        assert!(loaded.auto_paste);
        assert_eq!(loaded.schema_version, CURRENT_SCHEMA_VERSION);
    }

    /// `load_or_init` on a non-existent file writes the default and returns
    /// `recovered=false` — this is first-launch init, not recovery.
    #[test]
    fn load_or_init_creates_default_when_missing() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("bootstrap.json");
        let (value, recovered) =
            load_or_init(&path, Bootstrap::default, |b| b.schema_version).unwrap();
        assert!(!recovered, "missing file is not a recovery");
        assert_eq!(value.schema_version, CURRENT_SCHEMA_VERSION);
        assert!(path.exists(), "default file should be written to disk");
    }

    /// `load_or_init` replaces corrupt JSON with the default and signals
    /// `recovered=true` so callers can surface a startup warning toast.
    #[test]
    fn load_or_init_recovers_corrupt_json() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("settings.json");
        std::fs::write(&path, b"{not valid json").unwrap();
        let (value, recovered) =
            load_or_init(&path, Settings::default, |s| s.schema_version).unwrap();
        assert!(recovered, "corrupt JSON must be flagged as recovered");
        assert_eq!(value.schema_version, CURRENT_SCHEMA_VERSION);
        // File on disk should now be the default, not the corrupt content.
        let reread = std::fs::read_to_string(&path).unwrap();
        assert!(reread.contains("schemaVersion"));
    }

    /// `load_or_init` also recovers when the schema version doesn't match —
    /// no silent half-loaded data when a future format hits an older binary
    /// or vice versa.
    #[test]
    fn load_or_init_recovers_version_mismatch() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("settings.json");
        let bogus = r#"{"schemaVersion": 99, "hotkey": "Ctrl+Alt+Space", "autoPaste": false, "theme": "system"}"#;
        std::fs::write(&path, bogus).unwrap();
        let (value, recovered) =
            load_or_init(&path, Settings::default, |s| s.schema_version).unwrap();
        assert!(recovered, "version mismatch must be flagged as recovered");
        assert_eq!(value.schema_version, CURRENT_SCHEMA_VERSION);
    }

    /// `load_templates` loads every valid `.json` file in the directory and
    /// reports `invalid_count=0` when none are malformed.
    #[test]
    fn load_templates_loads_valid_files() {
        let tmp = tempdir().unwrap();
        let dir = tmp.path().join("templates");
        std::fs::create_dir(&dir).unwrap();
        let t1 = mk_template("a");
        let t2 = mk_template("b");
        save_template(&dir, &t1).unwrap();
        save_template(&dir, &t2).unwrap();

        let (map, invalid) = load_templates(&dir).unwrap();
        assert_eq!(map.len(), 2);
        assert_eq!(invalid, 0);
        assert!(map.contains_key(&t1.id));
        assert!(map.contains_key(&t2.id));
    }

    /// Malformed JSON in `templates/` is moved to `templates/.invalid/` and
    /// counted toward `invalid_count` so the startup-warning toast can show
    /// the right number, while valid neighbors still load normally.
    #[test]
    fn load_templates_moves_invalid_to_subdir() {
        let tmp = tempdir().unwrap();
        ensure_data_folder_structure(tmp.path()).unwrap();
        let dir = tmp.path().join("templates");

        let good = mk_template("good");
        save_template(&dir, &good).unwrap();
        let bad_path = dir.join("bad.json");
        std::fs::write(&bad_path, b"{not valid").unwrap();

        let (map, invalid) = load_templates(&dir).unwrap();
        assert_eq!(map.len(), 1, "valid template should still load");
        assert_eq!(invalid, 1, "one invalid file expected");
        assert!(
            !bad_path.exists(),
            "bad file should be moved out of templates/"
        );
        let invalid_dest = dir.join(".invalid").join("bad.json");
        assert!(invalid_dest.exists(), "bad file should land in .invalid/");
    }

    /// `save_template` writes a JSON file that `load_templates` reads back
    /// with all fields preserved — including variables, tags, and metadata.
    #[test]
    fn save_template_then_load_round_trip() {
        let tmp = tempdir().unwrap();
        let dir = tmp.path().join("templates");
        std::fs::create_dir(&dir).unwrap();

        let mut t = mk_template("round-trip");
        let guid = Uuid::new_v4();
        t.body = format!("hello {{{}}}", guid);
        t.tags = vec!["urgent".to_string(), "work".to_string()];
        t.is_pinned = true;
        t.use_count = 5;
        t.variables.push(Variable {
            guid,
            display_name: "name".to_string(),
            variable_type: VariableType::Text,
            options: None,
            required: true,
            fill_from_clipboard: false,
            remember_last_used: true,
            static_default: Some("default".to_string()),
        });

        save_template(&dir, &t).unwrap();
        let (map, _) = load_templates(&dir).unwrap();
        let loaded = map.get(&t.id).expect("template not found after load");

        assert_eq!(loaded.display_name, "round-trip");
        assert_eq!(loaded.body, t.body);
        assert_eq!(loaded.tags, vec!["urgent", "work"]);
        assert!(loaded.is_pinned);
        assert_eq!(loaded.use_count, 5);
        assert_eq!(loaded.variables.len(), 1);
        assert_eq!(loaded.variables[0].display_name, "name");
        assert!(loaded.variables[0].required);
        assert_eq!(
            loaded.variables[0].static_default.as_deref(),
            Some("default")
        );
        assert!(loaded.variables[0].remember_last_used);
    }

    /// `delete_template` removes the file from disk; subsequent
    /// `load_templates` returns a map without that template.
    #[test]
    fn delete_template_removes_file() {
        let tmp = tempdir().unwrap();
        let dir = tmp.path().join("templates");
        std::fs::create_dir(&dir).unwrap();

        let t = mk_template("doomed");
        save_template(&dir, &t).unwrap();
        assert!(template_path(&dir, &t.id).exists());

        delete_template(&dir, &t.id).unwrap();
        assert!(!template_path(&dir, &t.id).exists());

        let (map, _) = load_templates(&dir).unwrap();
        assert!(!map.contains_key(&t.id));
    }

    /// `ensure_data_folder_structure` creates `templates/` and
    /// `templates/.invalid/` idempotently so callers don't need to pre-check.
    #[test]
    fn ensure_data_folder_structure_creates_dirs() {
        let tmp = tempdir().unwrap();
        ensure_data_folder_structure(tmp.path()).unwrap();
        assert!(tmp.path().join("templates").is_dir());
        assert!(tmp.path().join("templates").join(".invalid").is_dir());
        // Idempotent: running twice doesn't error.
        ensure_data_folder_structure(tmp.path()).unwrap();
    }
}
