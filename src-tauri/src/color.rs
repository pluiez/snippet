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

pub fn random_oklch() -> String {
    let mut rng = rand::thread_rng();
    loop {
        let l: f32 = rng.gen_range(0.45..=0.65);
        let c: f32 = rng.gen_range(0.10..=0.20);
        let h: f32 = rng.gen_range(0.0..360.0);
        if contrast_against_white(l, c, h) >= 4.5 {
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
