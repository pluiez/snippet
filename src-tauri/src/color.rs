//! Color generation + reconcile for the central color maps.
//!
//! - `random_oklch`: SPEC §6.2 — L 0.45-0.65, C 0.10-0.20, H 0-360 random,
//!   resampled until contrast against white ≥ 4.5:1 (WCAG AA for normal text).
//! - `reconcile_colors`: bidirectional sync between templates and color maps.
//!   Adds missing entries (so every live variable / tag has a color) AND
//!   removes orphans (SPEC §6.6). Covers SPEC §13 invariants 5, 6, 11.

use crate::state::AppState;
use crate::storage;
use anyhow::Result;
use rand::Rng;
use std::collections::HashSet;
use tracing::info;

/// SPEC §13 invariant 10 target: WCAG contrast ≥ 4.5:1 against white. We
/// require a small guard above the target in the generator because the
/// returned string is rounded (3 decimals for L/C, 1 for H); re-parsing
/// the rounded form can shift contrast by ~0.001-0.002, and what the user
/// actually renders is the rounded form. Without the guard the stored
/// color may compute back to 4.499 — failing the invariant in practice.
const CONTRAST_TARGET: f32 = 4.5;
const CONTRAST_GUARD: f32 = 0.05;

pub fn random_oklch() -> String {
    let mut rng = rand::thread_rng();
    loop {
        let l: f32 = rng.gen_range(0.45..=0.65);
        let c: f32 = rng.gen_range(0.10..=0.20);
        let h: f32 = rng.gen_range(0.0..360.0);
        if contrast_against_white(l, c, h) >= CONTRAST_TARGET + CONTRAST_GUARD {
            return format!("oklch({l:.3} {c:.3} {h:.1})");
        }
        // else loop and resample (rare for L 0.45-0.65 / C 0.10-0.20)
    }
}

/// OKLCh → linear sRGB. May return out-of-gamut values (negative or > 1).
fn oklch_to_linear_rgb(l: f32, c: f32, h: f32) -> (f32, f32, f32) {
    let h_rad = h.to_radians();
    let a = c * h_rad.cos();
    let b = c * h_rad.sin();

    // OKLab to LMS (Björn Ottosson's formulas).
    let l_ = l + 0.3963377774 * a + 0.2158037573 * b;
    let m_ = l - 0.1055613458 * a - 0.0638541728 * b;
    let s_ = l - 0.0894841775 * a - 1.2914855480 * b;

    let l3 = l_ * l_ * l_;
    let m3 = m_ * m_ * m_;
    let s3 = s_ * s_ * s_;

    // LMS to linear sRGB.
    let r = 4.0767416621 * l3 - 3.3077115913 * m3 + 0.2309699292 * s3;
    let g = -1.2684380046 * l3 + 2.6097574011 * m3 - 0.3413193965 * s3;
    let b = -0.0041960863 * l3 - 0.7034186147 * m3 + 1.7076147010 * s3;

    (r, g, b)
}

fn relative_luminance(r: f32, g: f32, b: f32) -> f32 {
    let r = r.clamp(0.0, 1.0);
    let g = g.clamp(0.0, 1.0);
    let b = b.clamp(0.0, 1.0);
    // WCAG luminance formula uses linear-light values (which OKLab → linear sRGB
    // already gives us; no need to gamma-decode).
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn contrast_against_white(l: f32, c: f32, h: f32) -> f32 {
    let (r, g, b) = oklch_to_linear_rgb(l, c, h);
    let lum = relative_luminance(r, g, b);
    (1.0 + 0.05) / (lum + 0.05)
}

/// Bidirectional sync between the templates and the color maps.
///
/// 1. Every variable displayName / tag in any template that lacks a color
///    map entry gets a freshly-generated `random_oklch` color (ensure).
/// 2. Every color map entry whose key no longer appears in any template
///    is removed (GC, SPEC §6.6).
///
/// Persists each map if it changed. Safe to call repeatedly — converges
/// per SPEC §13 invariant 11.
pub fn reconcile_colors(state: &AppState) -> Result<()> {
    let used_vars = collect_used_variable_names(state);
    let used_tags = collect_used_tag_names(state);

    let mut var_added = 0usize;
    let mut var_removed = 0usize;
    {
        let mut map = state
            .variable_colors
            .lock()
            .map_err(|e| anyhow::anyhow!("variable_colors lock: {e}"))?;

        for name in &used_vars {
            if !map.map.contains_key(name.as_str()) {
                map.map.insert(name.clone(), random_oklch());
                var_added += 1;
            }
        }

        let to_remove: Vec<String> = map
            .map
            .keys()
            .filter(|k| !used_vars.contains(k.as_str()))
            .cloned()
            .collect();
        for k in to_remove {
            map.map.remove(&k);
            var_removed += 1;
        }
    }
    if var_added > 0 || var_removed > 0 {
        let path = state.variable_colors_path();
        let snap = state
            .variable_colors
            .lock()
            .map_err(|e| anyhow::anyhow!("variable_colors lock: {e}"))?
            .clone();
        storage::atomic_write(&path, &snap)?;
    }

    let mut tag_added = 0usize;
    let mut tag_removed = 0usize;
    {
        let mut map = state
            .tag_colors
            .lock()
            .map_err(|e| anyhow::anyhow!("tag_colors lock: {e}"))?;

        for name in &used_tags {
            if !map.map.contains_key(name.as_str()) {
                map.map.insert(name.clone(), random_oklch());
                tag_added += 1;
            }
        }

        let to_remove: Vec<String> = map
            .map
            .keys()
            .filter(|k| !used_tags.contains(k.as_str()))
            .cloned()
            .collect();
        for k in to_remove {
            map.map.remove(&k);
            tag_removed += 1;
        }
    }
    if tag_added > 0 || tag_removed > 0 {
        let path = state.tag_colors_path();
        let snap = state
            .tag_colors
            .lock()
            .map_err(|e| anyhow::anyhow!("tag_colors lock: {e}"))?
            .clone();
        storage::atomic_write(&path, &snap)?;
    }

    if var_added + var_removed + tag_added + tag_removed > 0 {
        info!(
            var_added,
            var_removed,
            tag_added,
            tag_removed,
            "color maps reconciled",
        );
    }
    Ok(())
}

fn collect_used_variable_names(state: &AppState) -> HashSet<String> {
    let mut set = HashSet::new();
    if let Ok(map) = state.templates.lock() {
        for t in map.values() {
            for v in &t.variables {
                let key = v.display_name.to_lowercase();
                if !key.is_empty() {
                    set.insert(key);
                }
            }
        }
    }
    set
}

fn collect_used_tag_names(state: &AppState) -> HashSet<String> {
    let mut set = HashSet::new();
    if let Ok(map) = state.templates.lock() {
        for t in map.values() {
            for tag in &t.tags {
                let key = tag.to_lowercase();
                if !key.is_empty() {
                    set.insert(key);
                }
            }
        }
    }
    set
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parse "oklch(L C H)" string emitted by `random_oklch` back into floats
    /// so we can validate the generator's promises about its output range.
    fn parse_oklch(s: &str) -> (f32, f32, f32) {
        let inner = s
            .strip_prefix("oklch(")
            .and_then(|x| x.strip_suffix(")"))
            .unwrap_or_else(|| panic!("not an oklch string: {s}"));
        let parts: Vec<f32> = inner
            .split_whitespace()
            .map(|p| p.parse().expect("oklch component must parse"))
            .collect();
        assert_eq!(parts.len(), 3, "expected 3 components in '{s}'");
        (parts[0], parts[1], parts[2])
    }

    /// SPEC §13 invariant 10: every color produced by `random_oklch` must
    /// have WCAG contrast ≥ 4.5:1 against white. The generator resamples on
    /// failure; this sweep locks that promise in. If a future change removes
    /// the resample loop or widens the L/C ranges into low-contrast territory,
    /// this test fails loudly.
    #[test]
    fn invariant_10_random_oklch_meets_contrast_against_white() {
        for _ in 0..1000 {
            let s = random_oklch();
            let (l, c, h) = parse_oklch(&s);
            let contrast = contrast_against_white(l, c, h);
            assert!(
                contrast >= 4.5,
                "contrast {contrast:.3} < 4.5 for '{s}' (L={l} C={c} H={h})",
            );
            // SPEC §6.2 sampling ranges.
            assert!((0.45..=0.65).contains(&l), "L {l} out of [0.45, 0.65]");
            assert!((0.10..=0.20).contains(&c), "C {c} out of [0.10, 0.20]");
            assert!((0.0..360.0).contains(&h), "H {h} out of [0, 360)");
        }
    }

    /// Sanity anchors for `contrast_against_white` so the OKLab → sRGB →
    /// luminance math doesn't silently drift under refactors. White-on-white
    /// is contrast 1; pure black is the WCAG max of ~21.
    #[test]
    fn contrast_against_white_anchor_values() {
        let white = contrast_against_white(1.0, 0.0, 0.0);
        assert!((white - 1.0).abs() < 0.05, "white-on-white {white}");
        let black = contrast_against_white(0.0, 0.0, 0.0);
        assert!(black > 20.0, "black-on-white should be ~21, got {black}");
    }

    // --- reconcile_colors tests ---------------------------------------------

    use crate::schema::{Template, Variable, VariableType, CURRENT_SCHEMA_VERSION};
    use tempfile::tempdir;
    use uuid::Uuid;

    fn mk_template(name: &str, variables: Vec<&str>, tags: Vec<&str>) -> Template {
        Template {
            schema_version: CURRENT_SCHEMA_VERSION,
            id: Uuid::new_v4(),
            display_name: name.to_string(),
            body: String::new(),
            variables: variables
                .iter()
                .map(|n| Variable {
                    guid: Uuid::new_v4(),
                    display_name: n.to_string(),
                    variable_type: VariableType::Text,
                    options: None,
                    required: false,
                    fill_from_clipboard: false,
                    remember_last_used: false,
                    static_default: None,
                })
                .collect(),
            tags: tags.iter().map(|t| t.to_string()).collect(),
            is_pinned: false,
            created_at: String::new(),
            updated_at: String::new(),
            last_used_at: None,
            use_count: 0,
        }
    }

    /// SPEC §13 invariant 5: GC must not delete entries whose key still
    /// corresponds to a live variable/tag in any template. Reconcile runs
    /// and the existing entries survive — only orphans are touched.
    #[test]
    fn invariant_5_gc_preserves_live_entries() {
        let tmp = tempdir().unwrap();
        let state = AppState::for_test(tmp.path().to_path_buf());

        let t = mk_template("t", vec!["Language"], vec!["work"]);
        state.templates.lock().unwrap().insert(t.id, t);

        // Pre-populate maps for these refs (simulating a prior session that
        // already ensured them).
        state
            .variable_colors
            .lock()
            .unwrap()
            .map
            .insert("language".to_string(), "oklch(0.5 0.15 200.0)".to_string());
        state
            .tag_colors
            .lock()
            .unwrap()
            .map
            .insert("work".to_string(), "oklch(0.5 0.15 30.0)".to_string());

        reconcile_colors(&state).unwrap();

        let v = state.variable_colors.lock().unwrap();
        assert_eq!(
            v.map.get("language").map(String::as_str),
            Some("oklch(0.5 0.15 200.0)"),
            "live variable entry must survive GC unchanged"
        );
        let tg = state.tag_colors.lock().unwrap();
        assert_eq!(
            tg.map.get("work").map(String::as_str),
            Some("oklch(0.5 0.15 30.0)"),
            "live tag entry must survive GC unchanged"
        );
    }

    /// SPEC §13 invariant 6: GC removes entries whose key no longer appears
    /// in any template. With no templates at all, every entry is orphaned.
    #[test]
    fn invariant_6_gc_removes_orphans() {
        let tmp = tempdir().unwrap();
        let state = AppState::for_test(tmp.path().to_path_buf());

        state
            .variable_colors
            .lock()
            .unwrap()
            .map
            .insert("orphan_var".to_string(), "oklch(0.5 0.15 100.0)".to_string());
        state
            .tag_colors
            .lock()
            .unwrap()
            .map
            .insert("orphan_tag".to_string(), "oklch(0.5 0.15 200.0)".to_string());

        reconcile_colors(&state).unwrap();

        assert!(
            state.variable_colors.lock().unwrap().map.is_empty(),
            "orphan variable entries must be GC'd"
        );
        assert!(
            state.tag_colors.lock().unwrap().map.is_empty(),
            "orphan tag entries must be GC'd"
        );
    }

    /// reconcile_colors also ensures colors for live references that don't
    /// yet have one (the "ensure" side of bidirectional sync), not just GC.
    #[test]
    fn reconcile_ensures_missing_entries_for_live_refs() {
        let tmp = tempdir().unwrap();
        let state = AppState::for_test(tmp.path().to_path_buf());

        let t = mk_template("t", vec!["Color"], vec!["urgent"]);
        state.templates.lock().unwrap().insert(t.id, t);

        reconcile_colors(&state).unwrap();

        assert!(state.variable_colors.lock().unwrap().map.contains_key("color"));
        assert!(state.tag_colors.lock().unwrap().map.contains_key("urgent"));
    }

    /// SPEC §13 invariant 11: reconcile_colors is idempotent — running it
    /// twice in a row leaves the maps unchanged after the first run
    /// converges them.
    #[test]
    fn invariant_11_reconcile_converges_after_first_run() {
        let tmp = tempdir().unwrap();
        let state = AppState::for_test(tmp.path().to_path_buf());

        let t1 = mk_template("t1", vec!["A"], vec!["red"]);
        let t2 = mk_template("t2", vec!["B"], vec!["blue"]);
        let t1_id = t1.id;
        let t2_id = t2.id;
        state.templates.lock().unwrap().insert(t1_id, t1);
        state.templates.lock().unwrap().insert(t2_id, t2);

        // Pre-existing orphan that should be GC'd on the first run.
        state
            .variable_colors
            .lock()
            .unwrap()
            .map
            .insert("stale".to_string(), "oklch(0.5 0.15 0.0)".to_string());

        reconcile_colors(&state).unwrap();
        let after_first_vars = state.variable_colors.lock().unwrap().map.clone();
        let after_first_tags = state.tag_colors.lock().unwrap().map.clone();

        reconcile_colors(&state).unwrap();
        let after_second_vars = state.variable_colors.lock().unwrap().map.clone();
        let after_second_tags = state.tag_colors.lock().unwrap().map.clone();

        assert_eq!(after_first_vars, after_second_vars, "var map must converge");
        assert_eq!(after_first_tags, after_second_tags, "tag map must converge");
        assert!(
            !after_first_vars.contains_key("stale"),
            "first run should have removed the orphan"
        );
    }

    /// Color keys are lowercased — `Language` variable and `language`
    /// variable live under the same key. Locked in by collect_used_*_names'
    /// lowercase pass.
    #[test]
    fn gc_keys_are_lowercased() {
        let tmp = tempdir().unwrap();
        let state = AppState::for_test(tmp.path().to_path_buf());

        let t = mk_template("t", vec!["Language"], vec!["Work"]);
        state.templates.lock().unwrap().insert(t.id, t);

        reconcile_colors(&state).unwrap();

        let v = state.variable_colors.lock().unwrap();
        assert!(v.map.contains_key("language"), "key should be lowercased");
        assert!(!v.map.contains_key("Language"));
        let tg = state.tag_colors.lock().unwrap();
        assert!(tg.map.contains_key("work"));
        assert!(!tg.map.contains_key("Work"));
    }
}
