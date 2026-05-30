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
    rank(&templates, query)
}

/// Score and rank `templates` for `query` per SPEC §7. Extracted from
/// `search` so the ranking logic can be unit-tested without an AppState.
///
/// Non-empty queries filter to templates with weighted score > 0, sorted by
/// score desc with `lastUsedAt` desc as tiebreaker. Empty queries fall
/// through to `default_order` (pinned first, then `lastUsedAt` desc, then
/// displayName as the final fallback).
pub fn rank(templates: &[Template], query: &str) -> Vec<TemplateSummary> {
    let q = query.trim();
    if q.is_empty() {
        return default_order(templates);
    }

    let mut matcher = Matcher::new(Config::DEFAULT);
    let pattern = Pattern::parse(q, CaseMatching::Smart, Normalization::Smart);

    let mut scored: Vec<(f32, &Template)> = Vec::new();
    for t in templates {
        let name_score = score_field(&t.display_name, &pattern, &mut matcher);
        let tag_score = t
            .tags
            .iter()
            .map(|tag| score_field(tag, &pattern, &mut matcher))
            .fold(0.0_f32, f32::max);
        let body_text = render::body_for_search(t);
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
            .then_with(|| cmp_last_used_desc(a.1, b.1))
    });

    scored
        .into_iter()
        .map(|(_, t)| TemplateSummary {
            id: t.id,
            display_name: t.display_name.clone(),
            is_pinned: t.is_pinned,
            tags: t.tags.clone(),
        })
        .collect()
}

fn default_order(templates: &[Template]) -> Vec<TemplateSummary> {
    let mut sorted: Vec<&Template> = templates.iter().collect();
    // SPEC §7.4 空查询: pinned first, then lastUsedAt desc (displayName as final tiebreaker).
    sorted.sort_by(|a, b| {
        b.is_pinned
            .cmp(&a.is_pinned)
            .then_with(|| cmp_last_used_desc(*a, *b))
    });
    sorted
        .into_iter()
        .map(|t| TemplateSummary {
            id: t.id,
            display_name: t.display_name.clone(),
            is_pinned: t.is_pinned,
            tags: t.tags.clone(),
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

#[cfg(test)]
mod tests {
    use super::*;

    /// SPEC §13 invariant 12: pinyin haystacks use the pinyin crate's
    /// default per-character reading. These asserts lock the documented v1
    /// behavior so a crate upgrade that changes default readings fails
    /// loudly here rather than silently breaking user search.
    #[test]
    fn invariant_12_pinyin_full_default_readings() {
        assert_eq!(pinyin_full("翻译"), "fanyi");
        // "模" defaults to mó (not mú), so "模板" indexes as "moban".
        assert_eq!(pinyin_full("模板"), "moban");
        assert_eq!(pinyin_full("邮箱"), "youxiang");
    }

    #[test]
    fn invariant_12_pinyin_initial_default_readings() {
        assert_eq!(pinyin_initial("翻译"), "fy");
        assert_eq!(pinyin_initial("模板"), "mb");
        assert_eq!(pinyin_initial("邮箱"), "yx");
    }

    /// Heteronym lock: "重" has multiple readings ("zhong", "chong"); "行"
    /// has "xing" and "hang". Per SPEC §13 invariant 12, only the crate
    /// default reading is indexed — users searching "hangye" for "行业"
    /// rely on the nucleo fuzzy fallback, not pinyin.
    #[test]
    fn invariant_12_heteronyms_use_default_reading() {
        assert_eq!(pinyin_full("重"), "zhong");
        assert_eq!(pinyin_full("行"), "xing");
        assert_eq!(pinyin_initial("重"), "z");
        assert_eq!(pinyin_initial("行"), "x");
    }

    /// Non-Chinese characters pass through unchanged so mixed strings still
    /// behave on their ASCII portion.
    #[test]
    fn pinyin_passes_through_ascii() {
        assert_eq!(pinyin_full("翻译a"), "fanyia");
        assert_eq!(pinyin_initial("翻译a"), "fya");
    }

    /// `contains_chinese` gates whether pinyin haystacks are produced at all.
    #[test]
    fn contains_chinese_detection() {
        assert!(contains_chinese("翻译"));
        assert!(contains_chinese("hello 翻译"));
        assert!(!contains_chinese("hello"));
        assert!(!contains_chinese(""));
    }

    // --- rank() tests --------------------------------------------------------

    use crate::schema::CURRENT_SCHEMA_VERSION;
    use uuid::Uuid;

    fn mk_template_for_rank(
        display_name: &str,
        body: &str,
        tags: Vec<&str>,
        last_used_at: Option<&str>,
        is_pinned: bool,
    ) -> Template {
        Template {
            schema_version: CURRENT_SCHEMA_VERSION,
            id: Uuid::new_v4(),
            display_name: display_name.to_string(),
            body: body.to_string(),
            variables: vec![],
            tags: tags.into_iter().map(String::from).collect(),
            is_pinned,
            created_at: String::new(),
            updated_at: String::new(),
            last_used_at: last_used_at.map(String::from),
            use_count: 0,
        }
    }

    /// SPEC §13 invariant 7: per-field weights are 1.0 (displayName) / 0.8
    /// (tag) / 0.3 (body) and the template total is MAX, not SUM. So a
    /// displayName hit must outrank a body-only hit even when the raw
    /// nucleo body score is higher.
    #[test]
    fn invariant_7_displayname_weight_outranks_body() {
        let by_name = mk_template_for_rank("foobar", "unrelated body", vec![], None, false);
        let by_body = mk_template_for_rank("unrelated", "foobar", vec![], None, false);
        let out = rank(&[by_body.clone(), by_name.clone()], "foobar");
        assert!(out.len() >= 2);
        assert_eq!(
            out[0].id, by_name.id,
            "displayName hit must outrank body hit"
        );
    }

    /// SPEC §13 invariant 7: tag hit (weight 0.8) outranks body hit (0.3).
    #[test]
    fn invariant_7_tag_weight_outranks_body() {
        let by_tag = mk_template_for_rank("unrelated", "no match", vec!["foobar"], None, false);
        let by_body = mk_template_for_rank("unrelated", "foobar text", vec![], None, false);
        let out = rank(&[by_body.clone(), by_tag.clone()], "foobar");
        assert!(out.len() >= 2);
        assert_eq!(
            out[0].id, by_tag.id,
            "tag hit (0.8) must outrank body hit (0.3)"
        );
    }

    /// SPEC §13 invariant 8: same weighted score → more-recent lastUsedAt
    /// wins. RFC3339 strings sort chronologically by lexicographic compare,
    /// which is what `cmp_last_used_desc` relies on.
    #[test]
    fn invariant_8_same_score_breaks_tie_by_last_used_desc() {
        let older =
            mk_template_for_rank("foobar", "x", vec![], Some("2025-01-01T00:00:00Z"), false);
        let newer =
            mk_template_for_rank("foobar", "x", vec![], Some("2026-01-01T00:00:00Z"), false);
        let out = rank(&[older.clone(), newer.clone()], "foobar");
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].id, newer.id, "newer lastUsedAt wins tie");
        assert_eq!(out[1].id, older.id);
    }

    /// SPEC §13 invariant 8 (None case): a template with no lastUsedAt
    /// sorts after a template that has one — never-used is older than
    /// ever-used.
    #[test]
    fn invariant_8_none_last_used_sorts_after_some() {
        let used =
            mk_template_for_rank("foobar", "x", vec![], Some("2025-01-01T00:00:00Z"), false);
        let never = mk_template_for_rank("foobar", "x", vec![], None, false);
        let out = rank(&[never.clone(), used.clone()], "foobar");
        assert_eq!(out[0].id, used.id);
        assert_eq!(out[1].id, never.id);
    }

    /// SPEC §7.4: empty query → pinned first, then lastUsedAt desc.
    #[test]
    fn empty_query_pinned_first_then_last_used_desc() {
        let pinned = mk_template_for_rank("a", "", vec![], Some("2025-01-01T00:00:00Z"), true);
        let recent = mk_template_for_rank("b", "", vec![], Some("2026-01-01T00:00:00Z"), false);
        let older = mk_template_for_rank("c", "", vec![], Some("2024-01-01T00:00:00Z"), false);
        let out = rank(&[recent.clone(), older.clone(), pinned.clone()], "");
        assert_eq!(out.len(), 3);
        assert_eq!(out[0].id, pinned.id, "pinned float to top");
        assert_eq!(out[1].id, recent.id, "newer lastUsedAt next");
        assert_eq!(out[2].id, older.id);
    }

    /// SPEC §7.4: final tiebreaker for never-used templates is displayName.
    #[test]
    fn empty_query_displayname_ties_break_never_used() {
        let b = mk_template_for_rank("bravo", "", vec![], None, false);
        let a = mk_template_for_rank("alpha", "", vec![], None, false);
        let out = rank(&[b.clone(), a.clone()], "");
        assert_eq!(out[0].id, a.id, "alpha < bravo lexicographically");
        assert_eq!(out[1].id, b.id);
    }

    /// Templates with no field match (weighted score 0) are excluded from
    /// non-empty query results entirely.
    #[test]
    fn nonempty_query_excludes_zero_score() {
        let hit = mk_template_for_rank("foobar", "", vec![], None, false);
        let miss = mk_template_for_rank("xyzzy", "abcdef", vec!["unrelated"], None, false);
        let out = rank(&[hit.clone(), miss.clone()], "foobar");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].id, hit.id);
    }
}
