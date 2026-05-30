//! Initial-value computation for the fill dialog. SPEC §4.5 priority order:
//! clipboard (if `fillFromClipboard`) → last-used (if `rememberLastUsed`) →
//! `staticDefault` → empty. Each step validates against the variable's type
//! constraints (enum membership); SPEC §13 invariant 3 specifically covers
//! the enum last-used fallback when options change underneath a stored value.
//!
//! Extracted from `commands.rs` so the priority logic can be unit-tested
//! without a Tauri runtime.

use crate::schema::{Variable, VariableType};
use std::collections::HashMap;

/// Compute the initial value to surface in the fill dialog for `var`,
/// applying SPEC §4.5 priority and type validation at every step.
pub fn compute_initial_value(
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

/// True iff `value` satisfies the variable's type constraints. For `enum`
/// the value must appear in `options`; `text` accepts anything (empty is
/// validated elsewhere — see `compute_initial_value`'s empty-skip).
pub fn is_valid_for_variable(var: &Variable, value: &str) -> bool {
    match var.variable_type {
        VariableType::Text => true,
        VariableType::Enum => var
            .options
            .as_ref()
            .map_or(false, |opts| opts.iter().any(|o| o == value)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn mk_text_variable(
        name: &str,
        fill_clip: bool,
        remember: bool,
        default: Option<&str>,
    ) -> Variable {
        Variable {
            guid: Uuid::new_v4(),
            display_name: name.to_string(),
            variable_type: VariableType::Text,
            options: None,
            required: false,
            fill_from_clipboard: fill_clip,
            remember_last_used: remember,
            static_default: default.map(String::from),
        }
    }

    fn mk_enum_variable(
        name: &str,
        options: Vec<&str>,
        default: Option<&str>,
        remember: bool,
    ) -> Variable {
        Variable {
            guid: Uuid::new_v4(),
            display_name: name.to_string(),
            variable_type: VariableType::Enum,
            options: Some(options.into_iter().map(String::from).collect()),
            required: false,
            fill_from_clipboard: false,
            remember_last_used: remember,
            static_default: default.map(String::from),
        }
    }

    /// SPEC §13 invariant 3 (primary): a remembered last-used value for an
    /// enum variable that is no longer in `options` is dropped, and the
    /// priority cascade falls through to `staticDefault`.
    #[test]
    fn invariant_3_enum_last_used_falls_back_when_stale() {
        let var = mk_enum_variable("Language", vec!["简体", "繁体"], Some("简体"), true);
        let mut last_used = HashMap::new();
        last_used.insert("language".to_string(), "日文".to_string());

        let v = compute_initial_value(&var, None, &last_used);
        assert_eq!(v, "简体");
    }

    /// SPEC §13 invariant 3 (extension): when both last-used AND
    /// staticDefault are stale (neither in `options`), the final value is
    /// empty — never a stale enum option leaking through.
    #[test]
    fn invariant_3_enum_falls_to_empty_when_default_also_stale() {
        let var = mk_enum_variable("Lang", vec!["简体", "繁体"], Some("日文"), true);
        let mut last_used = HashMap::new();
        last_used.insert("lang".to_string(), "韩文".to_string());

        let v = compute_initial_value(&var, None, &last_used);
        assert_eq!(v, "");
    }

    /// SPEC §4.5 priority: clipboard wins over last-used and staticDefault
    /// when `fillFromClipboard` is on and the clipboard has eligible content.
    #[test]
    fn priority_clipboard_wins() {
        let var = mk_text_variable("note", true, true, Some("default"));
        let mut last_used = HashMap::new();
        last_used.insert("note".to_string(), "remembered".to_string());
        let v = compute_initial_value(&var, Some("from-clip"), &last_used);
        assert_eq!(v, "from-clip");
    }

    #[test]
    fn priority_last_used_wins_when_clipboard_disabled() {
        let var = mk_text_variable("note", false, true, Some("default"));
        let mut last_used = HashMap::new();
        last_used.insert("note".to_string(), "remembered".to_string());
        let v = compute_initial_value(&var, Some("from-clip"), &last_used);
        assert_eq!(v, "remembered");
    }

    #[test]
    fn priority_static_default_wins_when_others_off() {
        let var = mk_text_variable("note", false, false, Some("default"));
        let last_used = HashMap::new();
        let v = compute_initial_value(&var, Some("from-clip"), &last_used);
        assert_eq!(v, "default");
    }

    #[test]
    fn priority_empty_when_all_layers_miss() {
        let var = mk_text_variable("note", false, false, None);
        let last_used = HashMap::new();
        let v = compute_initial_value(&var, None, &last_used);
        assert_eq!(v, "");
    }

    /// last-used keying is lowercased displayName (SPEC §5.5 — values are
    /// shared across templates by case-insensitive name).
    #[test]
    fn last_used_key_is_lowercased_displayname() {
        let var = mk_text_variable("Language", false, true, None);
        let mut last_used = HashMap::new();
        last_used.insert("language".to_string(), "zh".to_string());
        let v = compute_initial_value(&var, None, &last_used);
        assert_eq!(v, "zh");
    }

    /// Empty clipboard / last-used strings are treated as "nothing
    /// remembered" and skipped — the cascade falls through to staticDefault.
    #[test]
    fn empty_clipboard_or_last_used_skipped() {
        let var = mk_text_variable("note", true, true, Some("default"));
        let mut last_used = HashMap::new();
        last_used.insert("note".to_string(), "".to_string());
        let v = compute_initial_value(&var, Some(""), &last_used);
        assert_eq!(v, "default");
    }

    /// is_valid_for_variable: text accepts anything; enum requires options
    /// membership.
    #[test]
    fn is_valid_for_variable_rules() {
        let text_var = mk_text_variable("t", false, false, None);
        assert!(is_valid_for_variable(&text_var, ""));
        assert!(is_valid_for_variable(&text_var, "anything"));

        let enum_var = mk_enum_variable("e", vec!["a", "b"], None, false);
        assert!(is_valid_for_variable(&enum_var, "a"));
        assert!(is_valid_for_variable(&enum_var, "b"));
        assert!(!is_valid_for_variable(&enum_var, "c"));
        assert!(!is_valid_for_variable(&enum_var, ""));
    }

    /// Enum staticDefault that is out-of-options is dropped (same validation
    /// path as last-used). Final value falls to empty.
    #[test]
    fn invariant_3_enum_static_default_out_of_options_drops() {
        let var = mk_enum_variable("Lang", vec!["a", "b"], Some("not-in-options"), false);
        let last_used = HashMap::new();
        let v = compute_initial_value(&var, None, &last_used);
        assert_eq!(v, "");
    }
}
