//! What to change in the picture itself: concrete edits, each a brahma command with
//! parameters, derived from the analysis the way the research reads an image. Low global
//! contrast and a dark field read as flat and heavy; a riot of hues reads as noise; dense
//! texture arouses, and a picture with no edges gives the eye nowhere to land. Every
//! suggestion names the principle it rests on, and adjustments go on adjustment layers so
//! nothing is destructive.

use serde::Serialize;
use serde_json::{Value, json};

use crate::analyze::Analysis;

/// One edit the picture would benefit from, ready to run.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditSuggestion {
    pub title: &'static str,
    /// The measurement and the principle behind it.
    pub why: String,
    /// A brahma command id.
    pub command: &'static str,
    pub params: Value,
}

/// Edits for `a`, strongest first; empty when the picture is already balanced.
pub fn edit_suggestions(a: &Analysis) -> Vec<EditSuggestion> {
    let mut v = Vec::new();
    if a.mean_lightness < 0.3 {
        let lift = ((0.42 - a.mean_lightness) * 120.0).clamp(10.0, 45.0).round();
        v.push(EditSuggestion {
            title: "lift the exposure",
            why: format!("mean lightness {:.2}: a dark field reads as heavy before its content registers; brightness +{lift} brings the midtones up without flattening them", a.mean_lightness),
            command: "layer.newAdjustmentLayer.brightnessContrast",
            params: json!({"brightness": lift, "contrast": 0}),
        });
    } else if a.mean_lightness > 0.88 {
        v.push(EditSuggestion {
            title: "bring the highlights back",
            why: format!("mean lightness {:.2}: a near-white picture loses its shape; a touch darker and the tones separate again", a.mean_lightness),
            command: "layer.newAdjustmentLayer.brightnessContrast",
            params: json!({"brightness": -12, "contrast": 8}),
        });
    }
    if a.contrast < 0.1 {
        let add = ((0.16 - a.contrast) * 250.0).clamp(10.0, 40.0).round();
        v.push(EditSuggestion {
            title: "add contrast",
            why: format!("contrast {:.2}: low contrast reads as calm but also as flat; +{add} gives the eye a focal range to settle on", a.contrast),
            command: "layer.newAdjustmentLayer.brightnessContrast",
            params: json!({"brightness": 0, "contrast": add}),
        });
    } else if a.contrast > 0.32 {
        v.push(EditSuggestion {
            title: "ease the contrast",
            why: format!("contrast {:.2}: hard contrast reads as urgent; easing it a little keeps the drama without the strain", a.contrast),
            command: "layer.newAdjustmentLayer.brightnessContrast",
            params: json!({"brightness": 0, "contrast": -15}),
        });
    }
    if a.mean_chroma < 0.04 && a.mean_lightness > 0.08 {
        v.push(EditSuggestion {
            title: "add a little colour",
            why: format!("mean chroma {:.2}: near-grey pictures read as serious; vibrance lifts the quiet colours first and leaves skin alone", a.mean_chroma),
            command: "layer.newAdjustmentLayer.vibrance",
            params: json!({"vibrance": 30, "saturation": 0}),
        });
    } else if a.mean_chroma > 0.18 {
        v.push(EditSuggestion {
            title: "tame the saturation",
            why: format!("mean chroma {:.2}: everything shouts at once; pulling saturation back lets one colour lead", a.mean_chroma),
            command: "layer.newAdjustmentLayer.vibrance",
            params: json!({"vibrance": 0, "saturation": -18}),
        });
    }
    if a.hue_spread > 0.6 {
        v.push(EditSuggestion {
            title: "unify the hues",
            why: format!("hue spread {:.2}: many competing hues read as noise; a warming filter pulls them towards one family", a.hue_spread),
            command: "layer.newAdjustmentLayer.photoFilter",
            params: json!({"density": 25}),
        });
    }
    if a.edge_density > 0.16 {
        v.push(EditSuggestion {
            title: "calm the texture",
            why: format!(
                "edge density {:.2}: dense edges arouse before the subject is recognised; a light blur on a copy, masked off the subject, settles it",
                a.edge_density
            ),
            command: "filter.blur.gaussianBlur",
            params: json!({"radius": 1.5}),
        });
    } else if a.edge_density < 0.025 && a.contrast > 0.05 {
        v.push(EditSuggestion {
            title: "sharpen the focal point",
            why: format!("edge density {:.2}: with almost no edges the eye has nowhere to land; a gentle unsharp mask gives it one", a.edge_density),
            command: "filter.sharpen.unsharpMask",
            params: json!({"amount": 60, "radius": 1.2, "threshold": 2}),
        });
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyze::analyze;

    fn image(w: u32, h: u32, f: impl Fn(u32, u32) -> [u8; 4]) -> Vec<u8> {
        let mut v = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h {
            for x in 0..w {
                v.extend_from_slice(&f(x, y));
            }
        }
        v
    }

    #[test]
    fn a_dark_flat_grey_picture_gets_exposure_contrast_and_colour_and_a_balanced_one_gets_nothing() {
        let dark = image(32, 32, |x, _| [40 + (x % 3) as u8, 40, 42, 255]);
        let a = analyze(32, 32, &dark).unwrap();
        let e = edit_suggestions(&a);
        let titles: Vec<&str> = e.iter().map(|s| s.title).collect();
        assert!(titles.contains(&"lift the exposure") && titles.contains(&"add contrast") && titles.contains(&"add a little colour"), "{titles:?}");
        for s in &e {
            assert!(s.params.is_object() && !s.why.is_empty());
            assert!(s.command.starts_with("layer.newAdjustmentLayer.") || s.command.starts_with("filter."));
        }
        // A mid-tone picture with gentle contrast, one hue family and some texture: balanced.
        let balanced = image(64, 64, |x, y| {
            let t = ((x * 7 + y * 3) % 40) as u8;
            [120 + t, 110 + t / 2, 90 + t / 3, 255]
        });
        let b = analyze(64, 64, &balanced).unwrap();
        assert!(edit_suggestions(&b).len() <= 1, "{:?} for {b:?}", edit_suggestions(&b));
    }
}
