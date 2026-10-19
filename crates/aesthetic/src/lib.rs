//! Aesthetic recommendations: colour schemes, type, placement and measurement, and motion
//! timing, computed from a design-research corpus rather than guessed.
//!
//! The crate was built from a set of research notes ("aesthetic research loop": colour
//! systems, typography, layout, motion grammar, perception psychology, emotional design) in
//! two forms:
//!
//! - [`rules`]: the numbers and choices the notes settle on, as code. OKLCH palettes with
//!   harmony by hue rotation; dark-mode tokens as their own values; modular type scales and
//!   pairings per emotional register; an 8-pt grid, Fitts-sized targets, gestalt spacing and
//!   Weber-ratio steps; the 100/200/300/500 ms motion vocabulary with easing by purpose.
//! - [`corpus`]: the notes themselves, distilled by `scripts/distill.py` into short searchable
//!   entries, so a recommendation can cite the pattern it rests on.
//!
//! [`analyze`] reads an image's statistics (lightness, contrast, edge density, chroma, hue
//! spread) and names the register they predict, following the finding that the emotional
//! response to a picture arrives before its content is recognised and is driven by exactly
//! those statistics.
//!
//! No UI, no I/O: the engine exposes this as commands (`aesthetic.recommend`,
//! `aesthetic.analyze`, `aesthetic.search`) and the interface draws the result.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

pub mod analyze;
pub mod corpus;
pub mod oklch;
pub mod rules;

pub use analyze::{Analysis, analyze};
pub use corpus::{Entry, search};
pub use oklch::{Oklch, parse_hex};
pub use rules::{Density, Layout, Motion, Palette, Register, Request, Surface, Swatch, Typography};

use serde::Serialize;

/// A full recommendation: the four families plus the corpus entries that back it.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Recommendation {
    pub request: Request,
    pub palette: Palette,
    pub typography: Typography,
    pub layout: Layout,
    pub motion: Motion,
    /// The research entries most relevant to the request, best first.
    pub principles: Vec<Entry>,
}

/// Recommend for `req`. Never fails: every register, surface and density has an answer.
pub fn recommend(req: &Request) -> Recommendation {
    let (palette, typography, layout, motion) = rules::recommend(req);
    let query = format!(
        "{} color palette oklch typography scale layout spacing gestalt motion easing {}",
        req.register.name(),
        match req.surface {
            Surface::Dark => "dark mode halation",
            Surface::Light => "light contrast",
        }
    );
    let principles = search(&query, 8).into_iter().map(|(_, e)| e.clone()).collect();
    Recommendation { request: req.clone(), palette, typography, layout, motion, principles }
}

/// Recommend from an image: its analysis sets the register and the dominant colour becomes
/// the base, unless the request already names them.
pub fn recommend_for_image(mut req: Request, analysis: &Analysis) -> Recommendation {
    if req.register == Register::Neutral {
        req.register = analysis.register;
    }
    if req.base.is_none() && analysis.dominant.c > 0.03 {
        req.base = Some(analysis.dominant.to_srgb8());
    }
    recommend(&req)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_recommendation_carries_all_four_families_and_cites_the_corpus() {
        let r = recommend(&Request { register: Register::Calm, ..Request::default() });
        assert_eq!(r.palette.swatches.len(), 11);
        assert!(!r.typography.sizes_px.is_empty());
        assert!(r.layout.min_target_px > 0 && r.motion.standard_ms > 0);
        assert!(r.principles.len() >= 4, "{:?}", r.principles.iter().map(|e| &e.title).collect::<Vec<_>>());
        let json = serde_json::to_value(&r).ok();
        assert!(json.is_some_and(|v| v["palette"]["swatches"][0]["hex"].is_string()));
    }

    #[test]
    fn an_image_sets_the_register_and_base_when_the_request_leaves_them_open() {
        let mut rgba = Vec::new();
        for _ in 0..(32 * 32) {
            rgba.extend_from_slice(&[30, 80, 230, 255]);
        }
        let a = analyze(32, 32, &rgba).unwrap_or_else(|| Analysis {
            mean_lightness: 0.5,
            contrast: 0.0,
            edge_density: 0.0,
            mean_chroma: 0.2,
            hue_spread: 0.0,
            dominant: Oklch::from_srgb8([30, 80, 230]),
            register: Register::Calm,
            confidence: 1.0,
            notes: vec![],
        });
        let r = recommend_for_image(Request::default(), &a);
        assert_eq!(r.request.register, a.register);
        let brand = r.palette.swatches.iter().find(|s| s.role == "brand").map(|s| s.oklch.h).unwrap_or(0.0);
        assert!((brand - a.dominant.h).abs() < 3.0, "{brand} vs {}", a.dominant.h);
        // An explicit register wins over the image's.
        let r2 = recommend_for_image(Request { register: Register::Playful, ..Request::default() }, &a);
        assert_eq!(r2.request.register, Register::Playful);
    }
}
