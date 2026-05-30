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
