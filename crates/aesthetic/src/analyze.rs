//! What an image says before anyone reads it: the research notes that the emotional response
//! to a picture arrives about 150 ms before its content is recognised, and that it is driven by
//! the image's statistics, not its subject. Calm environments have low spatial frequency (few
//! sharp edges), low contrast, high luminance and uniform hue; arousing ones have dense edges,
//! high contrast and varied hue. This module measures those statistics on a small RGBA copy of
//! a document and names the register they predict, so the recommender can work from the
//! picture rather than from a guess.

use crate::oklch::Oklch;
use crate::rules::Register;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Analysis {
    /// Mean OKLab lightness of the opaque pixels (0..1).
    pub mean_lightness: f32,
    /// Standard deviation of lightness: the global contrast (0..~0.5).
    pub contrast: f32,
    /// Mean absolute lightness difference between neighbouring pixels: the spatial-frequency
    /// proxy (0 flat .. ~0.3 dense texture).
    pub edge_density: f32,
    /// Mean chroma (0 grey .. ~0.3 vivid).
    pub mean_chroma: f32,
    /// How much hues vary (0 one hue .. 1 everything).
    pub hue_spread: f32,
    /// The chroma-weighted dominant hue, as an OKLCH colour at the image's mean lightness.
    pub dominant: Oklch,
    /// The register the statistics predict.
    pub register: Register,
    /// 0..1: how clearly the statistics point at that register.
    pub confidence: f32,
    /// Plain-language notes on what was measured and what would move it.
    pub notes: Vec<String>,
}

/// Analyse `rgba` (8-bit, `width * height * 4` bytes). Transparent pixels are ignored. Meant
/// for a thumbnail (a few hundred pixels a side); larger inputs are sampled on a stride.
pub fn analyze(width: u32, height: u32, rgba: &[u8]) -> Option<Analysis> {
    let (w, h) = (width as usize, height as usize);
    if w == 0 || h == 0 || rgba.len() < w.checked_mul(h)?.checked_mul(4)? {
        return None;
    }
    let stride = w.max(h).div_ceil(256).max(1);
    let sw = w.div_ceil(stride);
    let sh = h.div_ceil(stride);
    // Lightness per sampled pixel (NaN = transparent) and the colour sums.
    let mut light = vec![f32::NAN; sw * sh];
    let mut n = 0usize;
    let (mut sum_l, mut sum_c, mut sum_a, mut sum_b, mut sum_cw) = (0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32);
    let mut hue_hist = [0.0f32; 12];
    for sy in 0..sh {
        for sx in 0..sw {
            let (x, y) = (sx * stride, sy * stride);
            let i = (y * w + x) * 4;
            let Some(px) = rgba.get(i..i + 4) else { continue };
            if px[3] < 8 {
                continue;
            }
            let c = Oklch::from_srgb8([px[0], px[1], px[2]]);
            if let Some(slot) = light.get_mut(sy * sw + sx) {
                *slot = c.l;
            }
            n += 1;
            sum_l += c.l;
            sum_c += c.c;
            let [_, a, b] = c.to_oklab();
            sum_a += a * c.c;
            sum_b += b * c.c;
            sum_cw += c.c;
            if c.c > 0.03 {
                let bin = ((c.h / 30.0) as usize).min(11);
                hue_hist[bin] += c.c;
            }
        }
    }
    if n == 0 {
        return None;
    }
    let nf = n as f32;
    let mean_lightness = sum_l / nf;
    let variance = light.iter().filter(|v| v.is_finite()).map(|v| (v - mean_lightness).powi(2)).sum::<f32>() / nf;
    let contrast = variance.sqrt();
    let mean_chroma = sum_c / nf;
    // Edge density: mean |ΔL| to the right and down neighbours.
    let (mut edge_sum, mut edges) = (0.0f32, 0usize);
    for sy in 0..sh {
        for sx in 0..sw {
            let Some(&v) = light.get(sy * sw + sx) else { continue };
            if !v.is_finite() {
                continue;
            }
            if sx + 1 < sw
                && let Some(&r) = light.get(sy * sw + sx + 1)
                && r.is_finite()
            {
                edge_sum += (v - r).abs();
                edges += 1;
            }
            if sy + 1 < sh
                && let Some(&d) = light.get((sy + 1) * sw + sx)
                && d.is_finite()
            {
                edge_sum += (v - d).abs();
                edges += 1;
            }
        }
    }
    let edge_density = if edges == 0 { 0.0 } else { edge_sum / edges as f32 };
    // Hue spread: 1 - the share of chroma in the dominant 90° of the wheel.
    let total: f32 = hue_hist.iter().sum();
    let hue_spread = if total <= 0.0 {
        0.0
    } else {
        let best = (0..12).map(|i| (0..3).map(|k| hue_hist[(i + k) % 12]).sum::<f32>()).fold(0.0f32, f32::max);
        1.0 - best / total
    };
    let dominant = if sum_cw > 1e-6 {
        let c = Oklch::from_oklab([mean_lightness, sum_a / sum_cw, sum_b / sum_cw]);
        c.with_c(c.c.max(mean_chroma))
    } else {
        Oklch::new(mean_lightness, 0.0, 0.0)
    };

    // Scores per register from the research's statistical signatures.
    let calm = score(1.0 - edge_density * 4.0) * 0.35 + score(1.0 - contrast * 3.0) * 0.25 + score(mean_lightness) * 0.2 + score(1.0 - hue_spread) * 0.2;
    let arousal = score(edge_density * 4.0) * 0.4 + score(contrast * 3.0) * 0.35 + score(hue_spread) * 0.25;
    let warm = dominant.c > 0.04 && (dominant.h < 90.0 || dominant.h > 330.0);
    let cool = dominant.c > 0.04 && (180.0..=300.0).contains(&dominant.h);
    let urgency = arousal * 0.6 + score(mean_chroma * 5.0) * 0.25 + if warm { 0.15 } else { 0.0 };
    let trust = score(1.0 - hue_spread) * 0.3 + score(contrast * 2.5) * 0.3 + if cool { 0.25 } else { 0.0 } + score(1.0 - mean_chroma * 5.0) * 0.15;
    let playful = score(mean_chroma * 5.0) * 0.45 + score(hue_spread) * 0.35 + score(mean_lightness) * 0.2;
    let wonder = score(1.0 - mean_lightness) * 0.35 + score(hue_spread * 0.7) * 0.25 + score(contrast * 3.0) * 0.2 + score(mean_chroma * 4.0) * 0.2;
    let candidates = [(Register::Calm, calm), (Register::Urgency, urgency), (Register::Trust, trust), (Register::Playful, playful), (Register::Wonder, wonder)];
    let (register, best) = candidates.iter().copied().fold((Register::Neutral, 0.0f32), |acc, c| if c.1 > acc.1 { c } else { acc });
    let second = candidates.iter().map(|c| c.1).filter(|v| *v < best).fold(0.0f32, f32::max);
    let confidence = ((best - second) * 2.0 + best * 0.3).clamp(0.0, 1.0);

    let mut notes = Vec::new();
    notes.push(format!(
        "lightness {:.2}, contrast {:.2}, edge density {:.2}, chroma {:.2}, hue spread {:.2}",
        mean_lightness, contrast, edge_density, mean_chroma, hue_spread
    ));
    if edge_density > 0.12 {
        notes.push("dense edges read as arousing before the content registers; soften or blur secondary areas to calm it".into());
    } else if edge_density < 0.03 {
        notes.push("very few edges: calm, but a single sharp focal element would give the eye somewhere to land".into());
    }
    if mean_lightness > 0.85 {
        notes.push("bright and airy: text and controls over it need the dark end of the scale for contrast".into());
    } else if mean_lightness < 0.25 {
        notes.push("dark field: avoid pure white type on it (halation); use lightness about 0.92 and a touch heavier weight".into());
    }
    if hue_spread > 0.6 {
        notes.push("many hues compete; pulling secondary hues towards the dominant one in oklch would unify it".into());
    }
    Some(Analysis { mean_lightness, contrast, edge_density, mean_chroma, hue_spread, dominant, register, confidence, notes })
}

fn score(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn a_soft_bright_gradient_is_calm_and_a_checkerboard_is_arousing() {
        let sky = image(64, 64, |x, _| [200 + (x / 3) as u8, 220, 245, 255]);
        let a = analyze(64, 64, &sky).unwrap();
        assert_eq!(a.register, Register::Calm, "{a:?}");
        assert!(a.edge_density < 0.03 && a.mean_lightness > 0.8);
        let noise = image(64, 64, |x, y| if (x + y) % 2 == 0 { [255, 40, 40, 255] } else { [20, 20, 20, 255] });
        let b = analyze(64, 64, &noise).unwrap();
        assert!(matches!(b.register, Register::Urgency | Register::Playful), "{b:?}");
        assert!(b.edge_density > 0.3 && b.contrast > 0.2);
    }

    #[test]
    fn transparent_and_degenerate_inputs_are_handled() {
        assert!(analyze(0, 0, &[]).is_none());
        assert!(analyze(4, 4, &[0; 10]).is_none(), "too few bytes");
        let clear = image(8, 8, |_, _| [255, 0, 0, 0]);
        assert!(analyze(8, 8, &clear).is_none(), "nothing opaque");
        let big = image(600, 300, |x, y| [(x % 256) as u8, (y % 256) as u8, 90, 255]);
        let a = analyze(600, 300, &big).unwrap();
        assert!(a.confidence >= 0.0 && a.confidence <= 1.0);
        assert!(a.dominant.l > 0.0);
    }

    #[test]
    fn the_dominant_hue_is_the_chroma_weighted_one() {
        let blue = image(32, 32, |x, _| if x < 24 { [30, 80, 230, 255] } else { [128, 128, 128, 255] });
        let a = analyze(32, 32, &blue).unwrap();
        assert!((a.dominant.h - Oklch::from_srgb8([30, 80, 230]).h).abs() < 5.0, "{a:?}");
    }
}
