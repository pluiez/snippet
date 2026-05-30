//! Template search and ranking. See SPEC §7 (weights, fuzzy, pinyin) and
//! §13 invariants 7-8 (weight ordering, lastUsedAt tiebreaker).

use crate::commands::TemplateSummary;
use crate::render;
use crate::schema::Template;
use crate::state::AppState;
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};
use pinyin::ToPinyin;
use std::cmp::Ordering;

const WEIGHT_DISPLAY_NAME: f32 = 1.0;
const WEIGHT_TAG: f32 = 0.8;
const WEIGHT_BODY: f32 = 0.3;

pub fn search(state: &AppState, query: &str) -> Vec<TemplateSummary> {
    let templates: Vec<Template> = match state.templates.lock() {
        Ok(map) => map.values().cloned().collect(),
        Err(_) => return Vec::new(),
    };

    let q = query.trim();
    if q.is_empty() {
        return default_order(templates);
    }

    let mut matcher = Matcher::new(Config::DEFAULT);
    let pattern = Pattern::parse(q, CaseMatching::Smart, Normalization::Smart);

    let mut scored: Vec<(f32, Template)> = Vec::new();
    for t in templates {
        let name_score = score_field(&t.display_name, &pattern, &mut matcher);
        let tag_score = t
            .tags
            .iter()
            .map(|tag| score_field(tag, &pattern, &mut matcher))
            .fold(0.0_f32, f32::max);
        let body_text = render::body_for_search(&t);
        let body_score = score_field(&body_text, &pattern, &mut matcher);

        // SPEC §7.1: weighted per field, total = MAX (not sum).
        let weighted = (name_score * WEIGHT_DISPLAY_NAME)
            .max(tag_score * WEIGHT_TAG)
            .max(body_score * WEIGHT_BODY);

        if weighted > 0.0 {
            scored.push((weighted, t));
        }
    }

    scored.sort_by(|a, b| {
        b.0.partial_cmp(&a.0)
            .unwrap_or(Ordering::Equal)
            .then_with(|| cmp_last_used_desc(&a.1, &b.1))
    });

    scored
        .into_iter()
        .map(|(_, t)| TemplateSummary {
            id: t.id,
            display_name: t.display_name,
            is_pinned: t.is_pinned,
            tags: t.tags,
        })
        .collect()
}

fn default_order(mut templates: Vec<Template>) -> Vec<TemplateSummary> {
    // SPEC §7.4 空查询: pinned first, then lastUsedAt desc (displayName as final tiebreaker).
    templates.sort_by(|a, b| {
        b.is_pinned
            .cmp(&a.is_pinned)
            .then_with(|| cmp_last_used_desc(a, b))
    });
    templates
        .into_iter()
        .map(|t| TemplateSummary {
            id: t.id,
            display_name: t.display_name,
            is_pinned: t.is_pinned,
            tags: t.tags,
        })
        .collect()
}

fn cmp_last_used_desc(a: &Template, b: &Template) -> Ordering {
    match (&a.last_used_at, &b.last_used_at) {
        (Some(x), Some(y)) => y.cmp(x),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => a.display_name.cmp(&b.display_name),
    }
}

fn score_field(text: &str, pattern: &Pattern, matcher: &mut Matcher) -> f32 {
    let direct = match_score(text, pattern, matcher);
    if !contains_chinese(text) {
        return direct;
    }
    let full = pinyin_full(text);
    let initial = pinyin_initial(text);
    direct
        .max(match_score(&full, pattern, matcher))
        .max(match_score(&initial, pattern, matcher))
}

fn match_score(haystack: &str, pattern: &Pattern, matcher: &mut Matcher) -> f32 {
    let mut buf: Vec<char> = Vec::new();
    let utf32 = Utf32Str::new(haystack, &mut buf);
    pattern
        .score(utf32, matcher)
        .map(|s| s as f32)
        .unwrap_or(0.0)
}

fn contains_chinese(s: &str) -> bool {
    s.chars().any(|c| matches!(c, '\u{4E00}'..='\u{9FFF}'))
}

/// Per-character pinyin (default pronunciation). Multi-character compound
/// words rely on the `pinyin` crate's default mapping; uncommon homographs
/// (e.g. 行 in 行业) may not match a "hangye" query but still match the
/// dictionary-default pronunciation. SPEC §13 invariant 12 acknowledges
/// dictionary-driven default-pronunciation behavior.
fn pinyin_full(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c.to_pinyin() {
            Some(p) => out.push_str(p.plain()),
            None => out.push(c),
        }
    }
    out
}

fn pinyin_initial(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c.to_pinyin() {
            Some(p) => {
                if let Some(first) = p.plain().chars().next() {
                    out.push(first);
                }
            }
            None => out.push(c),
        }
    }
    out
}
