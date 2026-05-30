//! First-launch onboarding helpers. SPEC §11 three-way choice:
//! default-new / custom-new / import-existing.
//!
//! All functions here are pure (or local file IO only) — IPC wrappers live in
//! `commands.rs` and read/write `bootstrap.json` via `storage::atomic_write`.

use anyhow::{Context, Result};
use std::path::Path;

use crate::schema::DataFolderStatus;

/// Markers that strongly suggest a folder is a Snippet data folder.
/// If ANY of these exist at the root, we treat it as a valid Snippet install
/// for the "import existing" flow.
const SNIPPET_MARKERS: &[&str] = &[
    "templates",
    "settings.json",
    "variable-colors.json",
    "tag-colors.json",
    "last-used.json",
];

/// Classify a candidate path for the onboarding picker.
///
/// Rules:
/// - Path doesn't exist → DoesNotExist
/// - Path exists but is not a directory → OccupiedByOther
/// - Path exists, is a directory, has any Snippet marker → ValidSnippet
/// - Path exists, is a directory, totally empty → Empty
/// - Path exists, is a directory, has non-Snippet content → OccupiedByOther
pub fn classify_path(path: &Path) -> Result<DataFolderStatus> {
    if !path.exists() {
        return Ok(DataFolderStatus::DoesNotExist);
    }
    if !path.is_dir() {
        return Ok(DataFolderStatus::OccupiedByOther);
    }
    for marker in SNIPPET_MARKERS {
        if path.join(marker).exists() {
            return Ok(DataFolderStatus::ValidSnippet);
        }
    }
    let mut iter = std::fs::read_dir(path)
        .with_context(|| format!("reading {}", path.display()))?;
    if iter.next().is_none() {
        Ok(DataFolderStatus::Empty)
    } else {
        Ok(DataFolderStatus::OccupiedByOther)
    }
}

/// True if the user must be sent through onboarding before the rest of the
/// app initializes.
pub fn needs_onboarding(bootstrap: &crate::schema::Bootstrap) -> bool {
    !bootstrap.onboarding_complete
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn classify_nonexistent() {
        let tmp = tempdir().unwrap();
        let p = tmp.path().join("nope");
        assert_eq!(classify_path(&p).unwrap(), DataFolderStatus::DoesNotExist);
    }

    #[test]
    fn classify_empty_dir() {
        let tmp = tempdir().unwrap();
        assert_eq!(classify_path(tmp.path()).unwrap(), DataFolderStatus::Empty);
    }

    #[test]
    fn classify_valid_snippet_with_templates_dir() {
        let tmp = tempdir().unwrap();
        fs::create_dir(tmp.path().join("templates")).unwrap();
        assert_eq!(
            classify_path(tmp.path()).unwrap(),
            DataFolderStatus::ValidSnippet
        );
    }

    #[test]
    fn classify_valid_snippet_with_settings_only() {
        let tmp = tempdir().unwrap();
        fs::write(tmp.path().join("settings.json"), "{}").unwrap();
        assert_eq!(
            classify_path(tmp.path()).unwrap(),
            DataFolderStatus::ValidSnippet
        );
    }

    #[test]
    fn classify_occupied_by_other() {
        let tmp = tempdir().unwrap();
        fs::write(tmp.path().join("readme.txt"), "hello").unwrap();
        assert_eq!(
            classify_path(tmp.path()).unwrap(),
            DataFolderStatus::OccupiedByOther
        );
    }

    #[test]
    fn classify_file_not_dir() {
        let tmp = tempdir().unwrap();
        let f = tmp.path().join("file.txt");
        fs::write(&f, "x").unwrap();
        assert_eq!(classify_path(&f).unwrap(), DataFolderStatus::OccupiedByOther);
    }
}
