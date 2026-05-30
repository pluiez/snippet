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
