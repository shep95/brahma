//! The rule base: the numbers and choices the research notes settle on, written as code so the
//! recommender computes with them rather than quoting them. Each rule names its source entry in
//! the corpus (`src/corpus.rs`) in the comments, so a reader can go from a recommendation to
//! the reasoning behind it.
//!
//! The four recommendation families: colour schemes (OKLCH, harmony by hue rotation at constant
//! lightness and chroma, dark-mode tokens as their own values), type (pairings by register, a
//! modular scale, line length, optical sizing), placement and measurement (an 8-pt grid, target
//! sizes from Fitts, spacing from gestalt proximity, step ratios above the Weber fraction) and
//! motion timing (the 100/200/300/500 ms vocabulary with easing by purpose).

use crate::oklch::Oklch;
use serde::{Deserialize, Serialize};

/// The emotional register a design aims at (Norman's visceral level): the research maps each
/// to concrete colour, type, layout and motion choices.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Register {
    Neutral,
    Trust,
    Calm,
    Urgency,
    Wonder,
    Playful,
}

impl Register {
    pub const ALL: [Register; 6] = [Register::Neutral, Register::Trust, Register::Calm, Register::Urgency, Register::Wonder, Register::Playful];

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "neutral" | "" => Some(Register::Neutral),
            "trust" | "trustworthy" | "authority" => Some(Register::Trust),
            "calm" | "serene" | "quiet" => Some(Register::Calm),
            "urgency" | "urgent" | "energy" | "action" => Some(Register::Urgency),
            "wonder" | "awe" | "dramatic" => Some(Register::Wonder),
            "playful" | "fun" | "joy" => Some(Register::Playful),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Register::Neutral => "neutral",
            Register::Trust => "trust",
            Register::Calm => "calm",
            Register::Urgency => "urgency",
            Register::Wonder => "wonder",
            Register::Playful => "playful",
        }
    }

    /// The hue the register leans to when the caller gives no base colour, and the chroma and
    /// lightness band it works in (corpus: "emotional design", specific visual choices).
    fn hue_hint(self) -> (f32, f32, f32) {
        match self {
            Register::Neutral => (260.0, 0.04, 0.55),
            Register::Trust => (250.0, 0.09, 0.50),  // cool, muted: blue → authority, stability
            Register::Calm => (215.0, 0.06, 0.70),   // sky, water, mist: cool and light
            Register::Urgency => (30.0, 0.20, 0.60), // warm and saturated: fire, blood
            Register::Wonder => (290.0, 0.16, 0.45), // deep, unexpected combinations
            Register::Playful => (85.0, 0.19, 0.78), // warm yellows, bright greens, magentas
        }
    }
}

/// Light or dark surfaces: the research is explicit that dark mode is its own set of token
/// values, not an inversion (halation, chroma shift, elevation by glow rather than shadow).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Surface {
    Dark,
    Light,
}

/// How much goes on a screen; sets the type scale ratio and spacing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Density {
    Dense,
    Comfortable,
    Airy,
}

/// What a recommendation is asked for.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub register: Register,
    /// The brand or key colour to build around; none picks the register's own hue.
    pub base: Option<[u8; 3]>,
    pub surface: Surface,
    pub density: Density,
    /// Touch input: larger targets and spacing.
    pub touch: bool,
}

impl Default for Request {
    fn default() -> Self {
        Self { register: Register::Neutral, base: None, surface: Surface::Dark, density: Density::Comfortable, touch: false }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Swatch {
    pub role: &'static str,
    pub oklch: Oklch,
    pub hex: String,
    pub why: &'static str,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Palette {
    pub harmony: &'static str,
    pub swatches: Vec<Swatch>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Typography {
    pub heading: &'static str,
    pub body: &'static str,
    pub mono: &'static str,
    pub pairing_why: &'static str,
    /// The modular scale ratio and the sizes it produces from the base (px).
    pub scale_ratio: f32,
    pub scale_name: &'static str,
    pub base_px: f32,
    pub sizes_px: Vec<f32>,
    pub line_height: f32,
    /// Characters per line for body text: the research's comfortable band.
    pub line_length_chars: [u32; 2],
    pub tracking_body: f32,
    pub notes: Vec<&'static str>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Layout {
    pub grid_px: u32,
    /// Smallest comfortable target (pointer or touch).
    pub min_target_px: u32,
    /// Gap inside a group of related things, and between groups: gestalt proximity needs the
    /// second to be clearly larger than the first.
    pub gap_related_px: u32,
    pub gap_groups_px: u32,
    pub margin_px: u32,
    /// Where the primary action goes for the reading pattern the register implies.
    pub primary_action: &'static str,
    pub shadow: &'static str,
    pub notes: Vec<&'static str>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Motion {
    pub micro_ms: u32,
    pub standard_ms: u32,
    pub large_ms: u32,
    pub enter_easing: &'static str,
    pub exit_easing: &'static str,
    pub spring: &'static str,
    pub notes: Vec<&'static str>,
}

/// Build the four families from a request. Pure and total: any request yields a result.
pub fn recommend(req: &Request) -> (Palette, Typography, Layout, Motion) {
    (palette(req), typography(req), layout(req), motion(req))
}

/// Colour: the base in OKLCH, then a harmony by rotating hue at constant lightness and chroma
/// (corpus: "oklch"), surfaces and text as their own values per mode (corpus: "dark mode and
/// adaptive theming"), and the register's chroma band on top.
pub fn palette(req: &Request) -> Palette {
    let (hint_h, hint_c, hint_l) = req.register.hue_hint();
    let mut base = match req.base {
        Some(rgb) => Oklch::from_srgb8(rgb),
        None => Oklch::new(hint_l, hint_c, hint_h),
    };
    // A near-grey base takes the register's hue; the chroma band is the register's.
    if base.c < 0.02 {
        base = Oklch::new(base.l, hint_c, hint_h);
    }
    let dark = req.surface == Surface::Dark;
    // Brand on dark: lighter and a little less chroma (corpus: dark mode, colour saturation shift).
    let brand = if dark { base.with_l((base.l + 0.12).clamp(0.55, 0.82)).with_c(base.c * 0.85) } else { base.with_l(base.l.clamp(0.40, 0.65)) };
    let (harmony, accent) = match req.register {
        // Split-complementary accents keep a trustworthy scheme from going flat.
        Register::Trust | Register::Neutral => ("analogous with a split-complementary accent", brand.rotate(150.0)),
        Register::Calm => ("analogous (±30°), low chroma", brand.rotate(30.0).with_c(brand.c * 0.8)),
        Register::Urgency => ("complementary", brand.rotate(180.0)),
        Register::Wonder => ("triadic", brand.rotate(120.0)),
        Register::Playful => ("tetradic", brand.rotate(90.0)),
    };
    let tint = brand.c * 0.1; // the slight hue on surfaces: not pure black, not pure white
    let (surface_1, surface_2, surface_3, text_1, text_2) = if dark {
        // oklch(0.10 0.02 h): dark with a slight hue, never #000; text at 0.92 against halation.
        (
            Oklch::new(0.12, tint.max(0.012), brand.h),
            Oklch::new(0.16, tint.max(0.012), brand.h),
            Oklch::new(0.21, tint.max(0.01), brand.h),
            Oklch::new(0.92, 0.0, 0.0),
            Oklch::new(0.66, 0.01, brand.h),
        )
    } else {
        (
            Oklch::new(1.0, 0.0, 0.0),
            Oklch::new(0.97, tint * 0.5, brand.h),
            Oklch::new(0.93, tint * 0.5, brand.h),
            Oklch::new(0.15, 0.02, brand.h),
            Oklch::new(0.42, 0.01, brand.h),
        )
    };
    let on_brand = if brand.l > 0.62 { Oklch::new(0.14, 0.02, brand.h) } else { Oklch::new(0.98, 0.0, 0.0) };
    let mk = |role: &'static str, c: Oklch, why: &'static str| Swatch { role, oklch: c, hex: c.hex(), why };
    let swatches = vec![
        mk("surface-1", surface_1, if dark { "the page: dark with a hint of the brand hue, never pure black" } else { "the page: white" }),
        mk("surface-2", surface_2, "raised surfaces: one step lighter than the page"),
        mk("surface-3", surface_3, "cards and fields: two steps"),
        mk("text-1", text_1, if dark { "primary text at lightness 0.92, not white: no halation" } else { "primary text: near black with the brand's hue" }),
        mk("text-2", text_2, "secondary text"),
        mk("brand", brand, if dark { "the key colour, lighter and less saturated for the dark surface" } else { "the key colour" }),
        mk("on-brand", on_brand, "text on the key colour"),
        mk("accent", accent, "the harmony's second hue, for one thing per screen"),
        mk("brand-soft", brand.mix(surface_2, 0.78), "tinted background for selected or hovered rows (mixed in oklab)"),
        mk("danger", Oklch::new(if dark { 0.70 } else { 0.55 }, 0.19, 25.0), "destructive actions: warm and saturated"),
        mk("success", Oklch::new(if dark { 0.75 } else { 0.55 }, 0.15, 150.0), "confirmations"),
    ];
    Palette { harmony, swatches }
}

/// Type: a pairing per register and a modular scale per density (corpus: "typography",
/// typographic rhythm; "emotional design", type choices per state; the variable-font grade
/// axis for dark mode).
pub fn typography(req: &Request) -> Typography {
    let (heading, body, pairing_why) = match req.register {
        Register::Trust => {
            ("Source Serif 4", "Source Sans 3", "a serif heading carries the long institutional record; its sans companion keeps body text neutral")
        }
        Register::Calm => ("Figtree", "Inter", "a soft geometric heading over a quiet humanist body, both with relaxed tracking"),
        Register::Urgency => ("Archivo Black", "Inter", "a heavy grotesque heading shouts; the body stays plain so the shout is the only loud thing"),
        Register::Wonder => ("Fraunces", "Inter", "an expressive serif with optical sizing for scale contrast against a neutral body"),
        Register::Playful => ("Nunito", "Nunito Sans", "rounded terminals read as friendly; the sans sibling keeps paragraphs readable"),
        Register::Neutral => ("Inter", "Inter", "one family at two weights: the hierarchy comes from size and weight, not from a second voice"),
    };
    let (scale_ratio, scale_name): (f32, &'static str) = match req.density {
        Density::Dense => (1.2, "minor third (1.2): tight enough for information-dense screens"),
        Density::Comfortable => (1.25, "major third (1.25): the usual right choice for interfaces"),
        Density::Airy => (1.333, "perfect fourth (1.333): roomy, for editorial pages"),
    };
    let base_px = if req.touch { 16.0 } else { 14.0 };
    let sizes_px: Vec<f32> = (-1..=5).map(|i| (base_px * scale_ratio.powi(i) * 2.0).round() / 2.0).collect();
    let line_height = match req.density {
        Density::Dense => 1.4,
        Density::Comfortable => 1.5,
        Density::Airy => 1.6,
    };
    let tracking_body = if req.register == Register::Calm { 0.01 } else { 0.0 };
    let mut notes = vec![
        "optical sizing: turn the opsz axis on (`font-optical-sizing: auto`) so small text gets the sturdier cut and display sizes the finer one",
        "keep body lines at 45 to 75 characters; 66 is the sweet spot",
    ];
    if req.surface == Surface::Dark {
        notes.push("on dark surfaces thin strokes look thinner (halation): add a little weight or grade (the GRAD axis) rather than switching to white");
    }
    if req.density == Density::Dense {
        notes.push("at dense settings use weight, not size, to separate levels within a group; size for the group boundaries");
    }
    Typography {
        heading,
        body,
        mono: "JetBrains Mono",
        pairing_why,
        scale_ratio,
        scale_name,
        base_px,
        sizes_px,
        line_height,
        line_length_chars: [45, 75],
        tracking_body,
        notes,
    }
}

/// Placement and measurement (corpus: "gestalt completion in UI", perception psychology, the
/// Fechner-Weber law: a step must exceed about 10 % to register, so spacing goes up in ratios,
/// not in equal increments; Fitts: target size and distance decide acquisition time).
pub fn layout(req: &Request) -> Layout {
    let grid_px = 8;
    let min_target_px = if req.touch { 44 } else { 28 };
    let (gap_related_px, gap_groups_px, margin_px) = match req.density {
        Density::Dense => (4, 12, 12),
        Density::Comfortable => (8, 24, 16),
        Density::Airy => (12, 40, 24),
    };
    let (gap_related_px, gap_groups_px, margin_px) =
        if req.touch { (gap_related_px + 4, gap_groups_px + 8, margin_px + 4) } else { (gap_related_px, gap_groups_px, margin_px) };
    let primary_action = match req.register {
        Register::Urgency => "high and central, with vertical emphasis pointing at it: the eye is being asked to act now",
        Register::Calm => "at the end of the reading path (bottom-right in left-to-right scripts), with horizontal emphasis around it",
        Register::Trust => "where the F-pattern ends its first sweep: top-left heading, action at the right end of the first row, symmetric layout",
        Register::Wonder => "off the expected grid, with scale contrast: one very large element and small ones",
        Register::Playful => "off-grid, allowed to break alignment once; everything else stays aligned so the break reads as intent",
        Register::Neutral => "at the end of the reading path, aligned to the content column",
    };
    let shadow = if req.surface == Surface::Dark {
        "elevation by a 1 px light inner border plus a deep soft ambient shadow; drop shadows vanish on dark surfaces"
    } else {
        "one light source for every shadow: key shadow sharp and offset down, ambient shadow soft and centred"
    };
    let notes = vec![
        "gestalt proximity: the gap between groups must be clearly larger than the gap inside them (at least 2×), or grouping is read from colour instead",
        "spacing steps follow a ratio (8, 12, 16, 24, 32, 48): equal increments fall under the Weber fraction and read as the same",
        "the primary action gets the largest target and the shortest pointer travel from where the eye already is (Fitts)",
        "every shadow derives from the same light position; a mismatched blur or offset reads as wrong before anyone can say why",
    ];
    Layout { grid_px, min_target_px, gap_related_px, gap_groups_px, margin_px, primary_action, shadow, notes }
}

/// Motion timing (corpus: "motion design — 12 principles in UI"): the duration vocabulary and
/// easing by purpose; springs for things that move into place; linear only when the user drives.
pub fn motion(req: &Request) -> Motion {
    let (micro_ms, standard_ms, large_ms) = match req.register {
        Register::Urgency => (90, 180, 280),
        Register::Calm => (160, 300, 450),
        Register::Playful | Register::Wonder => (120, 260, 420),
        Register::Trust | Register::Neutral => (120, 220, 340),
    };
    let spring = match req.register {
        Register::Playful => "stiffness 300, damping 18: visible overshoot, cartoon weight",
        Register::Calm | Register::Trust => "stiffness 220, damping 28: settles without a bounce",
        _ => "stiffness 260, damping 24: one small overshoot",
    };
    Motion {
        micro_ms,
        standard_ms,
        large_ms,
        enter_easing: "ease-out (decelerate into place)",
        exit_easing: "ease-in (accelerate away), shorter than the entrance",
        spring,
        notes: vec![
            "every animation states its purpose: confirm an action, show a state change, reveal where something came from; anything else is noise",
            "stagger lists by about 50 ms per item so the order reads as sequence; the secondary items use heavier damping than the first",
            "scroll-linked motion is linear: the user sets the speed",
            "honour reduced-motion: swap movement for a fade and keep durations under 120 ms",
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_register_and_surface_yields_a_full_palette_in_gamut() {
        for register in Register::ALL {
            for surface in [Surface::Dark, Surface::Light] {
                let req = Request { register, surface, ..Request::default() };
                let p = palette(&req);
                assert_eq!(p.swatches.len(), 11, "{register:?}");
                for s in &p.swatches {
                    assert_eq!(s.hex.len(), 7, "{register:?} {}", s.role);
                    assert!(s.oklch.l >= 0.0 && s.oklch.l <= 1.0);
                }
                let by = |role: &str| p.swatches.iter().find(|s| s.role == role).map(|s| s.oklch).unwrap();
                let (t1, s1, brand) = (by("text-1"), by("surface-1"), by("brand"));
                // Readable: text and page far apart in lightness; dark pages never pure black.
                assert!((t1.l - s1.l).abs() > 0.6, "{register:?} {surface:?}: {t1:?} on {s1:?}");
                if surface == Surface::Dark {
                    assert!(s1.l > 0.08 && t1.l < 0.95, "no #000 and no #fff: {s1:?} {t1:?}");
                    assert!(brand.l >= 0.55, "brand lightened for dark: {brand:?}");
                }
            }
        }
    }

    #[test]
    fn a_base_colour_keeps_its_hue_and_a_grey_base_takes_the_registers_hue() {
        let gold = Request { base: Some([201, 168, 90]), ..Request::default() };
        let p = palette(&gold);
        let brand = p.swatches.iter().find(|s| s.role == "brand").unwrap().oklch;
        assert!((brand.h - Oklch::from_srgb8([201, 168, 90]).h).abs() < 2.0, "{brand:?}");
        let grey = Request { base: Some([128, 128, 128]), register: Register::Trust, ..Request::default() };
        let b2 = palette(&grey).swatches.iter().find(|s| s.role == "brand").unwrap().oklch;
        assert!((b2.h - 250.0).abs() < 2.0 && b2.c > 0.05, "{b2:?}");
    }

    #[test]
    fn the_type_scale_follows_its_ratio_and_touch_raises_the_base() {
        let t = typography(&Request::default());
        assert_eq!(t.base_px, 14.0);
        assert_eq!(t.sizes_px.len(), 7);
        for w in t.sizes_px.windows(2) {
            assert!(w[1] > w[0]);
            assert!((w[1] / w[0] - 1.25).abs() < 0.08, "{:?}", t.sizes_px);
        }
        let touch = typography(&Request { touch: true, density: Density::Dense, ..Request::default() });
        assert_eq!(touch.base_px, 16.0);
        assert!((touch.scale_ratio - 1.2).abs() < 1e-6);
        assert_eq!(touch.line_length_chars, [45, 75]);
    }

    #[test]
    fn layout_measurements_respect_fitts_weber_and_proximity() {
        for density in [Density::Dense, Density::Comfortable, Density::Airy] {
            for touch in [false, true] {
                let l = layout(&Request { density, touch, ..Request::default() });
                assert!(l.min_target_px >= if touch { 44 } else { 28 });
                assert!(l.gap_groups_px >= 2 * l.gap_related_px, "{density:?} {touch}: {l:?}");
                assert_eq!(l.grid_px, 8);
            }
        }
        let m = motion(&Request { register: Register::Urgency, ..Request::default() });
        let c = motion(&Request { register: Register::Calm, ..Request::default() });
        assert!(m.standard_ms < c.standard_ms && m.micro_ms < m.standard_ms && m.standard_ms < m.large_ms);
    }

    #[test]
    fn registers_parse_from_everyday_words() {
        assert_eq!(Register::parse("Calm"), Some(Register::Calm));
        assert_eq!(Register::parse("urgent"), Some(Register::Urgency));
        assert_eq!(Register::parse(""), Some(Register::Neutral));
        assert_eq!(Register::parse("spicy"), None);
    }
}
