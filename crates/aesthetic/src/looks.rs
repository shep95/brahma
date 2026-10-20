//! One-click looks: small stacks of adjustment layers that move a picture towards a register
//! the research describes, non-destructively. A look is a recipe (adjustment kind + params per
//! step); the engine turns it into a named group of adjustment layers so it can be switched
//! off for a before/after and removed in one go. Each look says which register it serves and
//! why, and [`look_suggestions`] ranks them for a picture from its statistics.

use serde::Serialize;
use serde_json::{Value, json};

use crate::analyze::Analysis;
use crate::rules::Register;

/// One adjustment layer of a look: the adjustment kind (the suffix of
/// `layer.newAdjustmentLayer.<kind>`) and its parameters.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LookStep {
    pub kind: &'static str,
    pub params: Value,
}

/// A look as the catalogue lists it.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Look {
    /// The id used by `aesthetic.look` and in the group's name (`look: <id>`).
    pub id: &'static str,
    pub title: &'static str,
    /// The register it pushes a picture towards.
    pub register: Register,
    /// The principle, in a sentence.
    pub why: &'static str,
    pub steps: Vec<LookStep>,
}

fn step(kind: &'static str, params: Value) -> LookStep {
    LookStep { kind, params }
}

/// Every look, in catalogue order.
pub fn looks() -> Vec<Look> {
    vec![
        Look {
            id: "calm",
            title: "calm",
            register: Register::Calm,
            why: "few edges, low contrast, high lightness and one hue family read as calm: the contrast eases, the midtones lift, and the colours settle towards one family",
            steps: vec![
                step("brightnessContrast", json!({"brightness": 12, "contrast": -18})),
                step("vibrance", json!({"vibrance": -10, "saturation": -12})),
                step("photoFilter", json!({"color": "#9fb8d6", "density": 12, "preserveLuminosity": true})),
            ],
        },
        Look {
            id: "warm",
            title: "warm and kind",
            register: Register::Playful,
            why: "warm hues (gold, orange) read as energy and welcome; a warming filter with gentle vibrance is the shortest way there",
            steps: vec![
                step("photoFilter", json!({"color": "#ec8a00", "density": 22, "preserveLuminosity": true})),
                step("vibrance", json!({"vibrance": 18, "saturation": 0})),
                step("brightnessContrast", json!({"brightness": 4, "contrast": 6})),
            ],
        },
        Look {
            id: "trust",
            title: "cool trust",
            register: Register::Trust,
            why: "cool hues and restrained chroma read as trust and competence; the shadows cool, the highlights stay clean, saturation steps back",
            steps: vec![
                step("colorBalance", json!({"shadows": [-8, 0, 14], "midtones": [-4, 0, 6], "highlights": [0, 0, 2], "preserveLuminosity": true})),
                step("vibrance", json!({"vibrance": -6, "saturation": -10})),
                step("brightnessContrast", json!({"brightness": 0, "contrast": 8})),
            ],
        },
        Look {
            id: "urgent",
            title: "urgent",
            register: Register::Urgency,
            why: "high contrast, warm chroma and dense edges read as arousal: contrast up, a touch of red in the midtones, saturation forward",
            steps: vec![
                step("brightnessContrast", json!({"brightness": -4, "contrast": 28})),
                step("colorBalance", json!({"shadows": [6, -2, -8], "midtones": [10, 0, -6], "highlights": [4, 0, -2], "preserveLuminosity": true})),
                step("vibrance", json!({"vibrance": 14, "saturation": 10})),
            ],
        },
        Look {
            id: "wonder",
            title: "wonder",
            register: Register::Wonder,
            why: "deep shadows with lit highlights and a cool-to-warm split read as awe: the tonal range widens and the shadows go blue while the lights go gold",
            steps: vec![
                step("brightnessContrast", json!({"brightness": -8, "contrast": 22})),
                step("colorBalance", json!({"shadows": [-10, -4, 18], "midtones": [0, 0, 0], "highlights": [12, 6, -10], "preserveLuminosity": true})),
                step("vibrance", json!({"vibrance": 12, "saturation": -4})),
            ],
        },
        Look {
            id: "matte",
            title: "quiet matte",
            register: Register::Neutral,
            why: "lifted blacks and softened contrast read as considered and editorial; nothing shouts, the content leads",
            steps: vec![
                step("levels", json!({"outBlack": 18, "outWhite": 248})),
                step("brightnessContrast", json!({"brightness": 0, "contrast": -10})),
                step("vibrance", json!({"vibrance": 0, "saturation": -8})),
            ],
        },
        Look {
            id: "mono",
            title: "monochrome",
            register: Register::Neutral,
            why: "one hue family is the strongest unifier: black and white removes colour as a variable so light, shape and edge carry the picture",
            steps: vec![step("blackWhite", json!({"reds": 50, "yellows": 70, "greens": 40, "cyans": 60, "blues": 20, "magentas": 80})), step("brightnessContrast", json!({"brightness": 0, "contrast": 12}))],
        },
        Look {
            id: "sepia",
            title: "sepia",
            register: Register::Calm,
            why: "a warm single-hue tone reads as memory and ease: monochrome with a light sepia tint and softened contrast",
            steps: vec![
                step("blackWhite", json!({"tint": true, "tintColor": "#e1d3b3"})),
                step("brightnessContrast", json!({"brightness": 4, "contrast": -6})),
            ],
        },
    ]
}

/// The look with `id`, if any.
pub fn look(id: &str) -> Option<Look> {
    looks().into_iter().find(|l| l.id == id.trim())
}

/// A look the picture would benefit from, with how well it fits.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LookSuggestion {
    pub id: &'static str,
    pub title: &'static str,
    /// 0..1: how much the look's register already shows in the picture (a look that agrees
    /// with the picture reads as polish; one that fights it reads as a mask).
    pub fit: f32,
    pub why: String,
}

/// Looks for `a`, best fit first. The picture's register leads; the looks that share its
/// temperature come next; monochrome and sepia stay available for any picture.
pub fn look_suggestions(a: &Analysis) -> Vec<LookSuggestion> {
    let warm = {
        let h = a.dominant.h;
        !(90.0..=330.0).contains(&h) && a.mean_chroma > 0.03
    };
    let mut v: Vec<LookSuggestion> = looks()
        .into_iter()
        .map(|l| {
            let mut fit: f32 = 0.35;
            if l.register == a.register {
                fit += 0.45 * a.confidence.clamp(0.0, 1.0) + 0.1;
            }
            match l.id {
                "warm" | "urgent" if warm => fit += 0.1,
                "trust" | "calm" if !warm => fit += 0.1,
                "mono" | "sepia" if a.mean_chroma < 0.05 => fit += 0.15,
                "matte" if a.contrast > 0.28 => fit += 0.1,
                _ => {}
            }
            let why = format!("{} ({})", l.why, l.register.name());
            LookSuggestion { id: l.id, title: l.title, fit: fit.clamp(0.0, 1.0), why }
        })
        .collect();
    v.sort_by(|x, y| y.fit.partial_cmp(&x.fit).unwrap_or(std::cmp::Ordering::Equal).then(x.id.cmp(y.id)));
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_catalogue_is_well_formed() {
        let all = looks();
        let mut ids = std::collections::HashSet::new();
        for l in &all {
            assert!(ids.insert(l.id), "duplicate look {}", l.id);
            assert!(!l.steps.is_empty(), "{}", l.id);
            assert!(l.steps.iter().all(|s| s.params.is_object()), "{}", l.id);
        }
        assert!(look("warm").is_some());
        assert!(look(" mono ").is_some());
        assert!(look("nope").is_none());
    }

    #[test]
    fn suggestions_follow_the_register() {
        // A near-white, low-contrast picture reads as calm: the calm look leads.
        let a = crate::analyze(4, 4, &[250u8; 4 * 4 * 4]).unwrap();
        let s = look_suggestions(&a);
        assert_eq!(s.len(), looks().len());
        assert_eq!(s[0].id, "calm", "{s:?}");
        assert!(s.windows(2).all(|w| w[0].fit >= w[1].fit));
        assert!(s.iter().any(|x| x.id == "mono" && x.fit > 0.4), "grey pictures invite monochrome: {s:?}");
    }
}
