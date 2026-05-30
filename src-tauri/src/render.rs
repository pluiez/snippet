//! Template rendering: replace `{<guid>}` placeholders in body with values.
//! Single source of truth for SPEC §13 invariant 10. Frontend has a mirror in
//! `src/lib/render.ts` for live preview without IPC chatter; both must stay in
//! sync — any change to the regex or substitution rule must be made in both.

use crate::schema::{Template, Variable};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;
use uuid::Uuid;

fn placeholder_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"\{([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})\}",
        )
        .expect("placeholder regex")
    })
}

/// Replace each `{<uuid>}` in `body` with the corresponding value.
/// - Orphan placeholder (UUID not in `values`): renders as empty string.
/// - Non-UUID `{...}` content: preserved as literal text.
pub fn render(body: &str, values: &HashMap<Uuid, String>) -> String {
    placeholder_re()
        .replace_all(body, |caps: &regex::Captures| {
            match Uuid::parse_str(&caps[1]) {
                Ok(uuid) => values.get(&uuid).cloned().unwrap_or_default(),
                Err(_) => caps[0].to_string(),
            }
        })
        .into_owned()
}

/// Variables in the order their GUIDs first appear in `template.body`.
/// Variables whose GUID never appears (orphan vars) are excluded — Slice 2
/// hides them; B1 will revisit when the editor surface needs them.
pub fn order_variables_by_body_appearance(template: &Template) -> Vec<&Variable> {
    let mut seen: HashSet<Uuid> = HashSet::new();
    let mut ordered: Vec<&Variable> = Vec::new();
    for caps in placeholder_re().captures_iter(&template.body) {
        if let Ok(uuid) = Uuid::parse_str(&caps[1]) {
            if seen.insert(uuid) {
                if let Some(var) = template.variables.iter().find(|v| v.guid == uuid) {
                    ordered.push(var);
                }
            }
        }
    }
    ordered
}

/// Body in displayName-substituted form, for the search index (per A2 in
/// PROGRESS spec decisions). Each `{<guid>}` becomes `{<displayName>}` so
/// queries like "Language" hit on body. Orphan placeholders are kept verbatim.
pub fn body_for_search(template: &Template) -> String {
    let names: HashMap<Uuid, String> = template
        .variables
        .iter()
        .map(|v| (v.guid, v.display_name.clone()))
        .collect();
    placeholder_re()
        .replace_all(&template.body, |caps: &regex::Captures| {
            match Uuid::parse_str(&caps[1]) {
                Ok(uuid) => match names.get(&uuid) {
                    Some(name) => format!("{{{name}}}"),
                    None => caps[0].to_string(),
                },
                Err(_) => caps[0].to_string(),
            }
        })
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{Template, Variable, VariableType, CURRENT_SCHEMA_VERSION};

    fn mk_template(body: &str, variables: Vec<Variable>) -> Template {
        Template {
            schema_version: CURRENT_SCHEMA_VERSION,
            id: Uuid::new_v4(),
            display_name: "t".to_string(),
            body: body.to_string(),
            variables,
            tags: vec![],
            is_pinned: false,
            created_at: String::new(),
            updated_at: String::new(),
            last_used_at: None,
            use_count: 0,
        }
    }

    fn mk_variable(name: &str) -> Variable {
        Variable {
            guid: Uuid::new_v4(),
            display_name: name.to_string(),
            variable_type: VariableType::Text,
            options: None,
            required: false,
            fill_from_clipboard: false,
            remember_last_used: false,
            static_default: None,
        }
    }

    /// SPEC §13 invariant 1: variable GUID is stable. Renaming a variable's
    /// displayName must not break placeholder resolution — body and values are
    /// both keyed by GUID, not displayName. Verified here by binding values
    /// to the GUID and asserting render still resolves after a rename of the
    /// variable struct.
    #[test]
    fn invariant_1_render_resolves_by_guid_not_displayname() {
        let mut var = mk_variable("Language");
        let guid = var.guid;
        let body = format!("hello {{{}}}", guid);
        let mut values = HashMap::new();
        values.insert(guid, "world".to_string());

        assert_eq!(render(&body, &values), "hello world");

        var.display_name = "语言".to_string();
        assert_eq!(render(&body, &values), "hello world");
    }

    /// SPEC §13 invariant 1 (search-index side): renaming a variable's
    /// displayName updates the displayName that appears in the search
    /// haystack the next time `body_for_search` is computed — the GUID in
    /// body acts as a stable handle into the variable list.
    #[test]
    fn invariant_1_body_for_search_reflects_renamed_variable() {
        let mut var = mk_variable("Language");
        let guid = var.guid;
        let body = format!("t {{{}}}", guid);

        let before = mk_template(&body, vec![var.clone()]);
        assert_eq!(body_for_search(&before), "t {Language}");

        var.display_name = "语言".to_string();
        let after = mk_template(&body, vec![var]);
        assert_eq!(body_for_search(&after), "t {语言}");
    }

    /// SPEC §13 invariant 2 (downstream): if the body still contains a
    /// placeholder whose variable has been deleted, render returns empty for
    /// that placeholder rather than crashing or leaking the GUID literal. The
    /// editor cleans body on variable deletion (frontend); this asserts the
    /// backend doesn't trust that cleanup blindly.
    #[test]
    fn invariant_2_orphan_placeholder_renders_empty() {
        let orphan = Uuid::new_v4();
        let body = format!("a{{{}}}b", orphan);
        let values: HashMap<Uuid, String> = HashMap::new();
        assert_eq!(render(&body, &values), "ab");
    }

    /// body_for_search keeps orphan placeholders verbatim (no value to
    /// substitute) so the search index still has something to grep against.
    #[test]
    fn body_for_search_keeps_orphan_verbatim() {
        let orphan = Uuid::new_v4();
        let body = format!("a {{{}}} b", orphan);
        let template = mk_template(&body, vec![]);
        assert_eq!(body_for_search(&template), body);
    }

    /// Non-UUID `{...}` content is preserved verbatim so literal braces in
    /// snippets aren't accidentally rewritten.
    #[test]
    fn non_uuid_braces_preserved() {
        let values: HashMap<Uuid, String> = HashMap::new();
        assert_eq!(render("{not-a-uuid}", &values), "{not-a-uuid}");
        assert_eq!(render("a{b}c", &values), "a{b}c");
    }

    /// Variables surface in the order their GUIDs first appear in body;
    /// duplicates collapse, orphan placeholders are excluded.
    #[test]
    fn order_variables_by_appearance() {
        let v1 = mk_variable("first");
        let v2 = mk_variable("second");
        let body = format!("{{{}}} - {{{}}} - {{{}}}", v2.guid, v1.guid, v2.guid);
        let template = mk_template(&body, vec![v1.clone(), v2.clone()]);
        let ordered = order_variables_by_body_appearance(&template);
        assert_eq!(ordered.len(), 2);
        assert_eq!(ordered[0].guid, v2.guid);
        assert_eq!(ordered[1].guid, v1.guid);
    }
}
