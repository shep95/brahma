//! OKLab / OKLCH: the perceptually uniform colour space the recommendations are computed in
//! (Björn Ottosson, 2020). Rotating the hue at constant lightness and chroma keeps perceived
//! saturation steady, which is what makes generated palettes look harmonious without hand
//! correction. Conversions follow the published matrices for sRGB (D65).

/// A colour as lightness (0..1), chroma (0..~0.4) and hue angle (degrees, 0..360).
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Oklch {
    pub l: f32,
    pub c: f32,
    pub h: f32,
}

impl Oklch {
    pub const fn new(l: f32, c: f32, h: f32) -> Self {
        Self { l, c, h }
    }

    /// From 8-bit sRGB.
    pub fn from_srgb8(rgb: [u8; 3]) -> Self {
        let lin = [srgb_to_linear(f32::from(rgb[0]) / 255.0), srgb_to_linear(f32::from(rgb[1]) / 255.0), srgb_to_linear(f32::from(rgb[2]) / 255.0)];
        let [l, a, b] = linear_to_oklab(lin);
        let c = (a * a + b * b).sqrt();
        let h = if c < 1e-5 { 0.0 } else { b.atan2(a).to_degrees().rem_euclid(360.0) };
        Self { l, c, h }
    }

    /// To 8-bit sRGB. A colour outside the sRGB gamut keeps its lightness and hue and loses
    /// chroma until it fits (the usual "reduce chroma" gamut mapping), so a generated palette
    /// stays harmonious instead of clipping to a different hue.
    pub fn to_srgb8(self) -> [u8; 3] {
        let l = self.l.clamp(0.0, 1.0);
        let mut c = self.c.max(0.0);
        for _ in 0..24 {
            if let Some(rgb) = in_gamut(l, c, self.h) {
                return rgb;
            }
            c *= 0.85;
        }
        in_gamut(l, 0.0, self.h).unwrap_or([0, 0, 0])
    }

    pub fn hex(self) -> String {
        let [r, g, b] = self.to_srgb8();
        format!("#{r:02x}{g:02x}{b:02x}")
    }

    /// The hue `degrees` further round the wheel, same lightness and chroma.
    pub fn rotate(self, degrees: f32) -> Self {
        Self { h: (self.h + degrees).rem_euclid(360.0), ..self }
    }

    pub fn with_l(self, l: f32) -> Self {
        Self { l: l.clamp(0.0, 1.0), ..self }
    }

    pub fn with_c(self, c: f32) -> Self {
        Self { c: c.max(0.0), ..self }
    }

    /// Mix towards `other` by `t` in OKLab (straight through the space, so midpoints don't go
    /// muddy the way sRGB mixing does).
    pub fn mix(self, other: Self, t: f32) -> Self {
        let a = self.to_oklab();
        let b = other.to_oklab();
        let t = t.clamp(0.0, 1.0);
        let m = [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t];
        Self::from_oklab(m)
    }

    pub fn to_oklab(self) -> [f32; 3] {
        let r = self.h.to_radians();
        [self.l, self.c * r.cos(), self.c * r.sin()]
    }

    pub fn from_oklab([l, a, b]: [f32; 3]) -> Self {
        let c = (a * a + b * b).sqrt();
        let h = if c < 1e-5 { 0.0 } else { b.atan2(a).to_degrees().rem_euclid(360.0) };
        Self { l, c, h }
    }

    /// Perceptual distance to `other` (OKLab Euclidean; about 0.02 is a just-noticeable step).
    pub fn distance(self, other: Self) -> f32 {
        let a = self.to_oklab();
        let b = other.to_oklab();
        ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
    }
}

fn in_gamut(l: f32, c: f32, h: f32) -> Option<[u8; 3]> {
    let r = h.to_radians();
    let lin = oklab_to_linear([l, c * r.cos(), c * r.sin()]);
    let eps = -0.0005;
    if lin.iter().any(|v| *v < eps || *v > 1.0005 || !v.is_finite()) {
        return None;
    }
    Some([to8(lin[0]), to8(lin[1]), to8(lin[2])])
}

fn to8(linear: f32) -> u8 {
    (linear_to_srgb(linear.clamp(0.0, 1.0)) * 255.0).round().clamp(0.0, 255.0) as u8
}

pub fn srgb_to_linear(v: f32) -> f32 {
    if v <= 0.04045 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
}

pub fn linear_to_srgb(v: f32) -> f32 {
    if v <= 0.003_130_8 { v * 12.92 } else { 1.055 * v.powf(1.0 / 2.4) - 0.055 }
}

fn linear_to_oklab([r, g, b]: [f32; 3]) -> [f32; 3] {
    let l = 0.412_221_47 * r + 0.536_332_55 * g + 0.051_445_995 * b;
    let m = 0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b;
    let s = 0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b;
    let (l, m, s) = (l.cbrt(), m.cbrt(), s.cbrt());
    [
        0.210_454_26 * l + 0.793_617_8 * m - 0.004_072_047 * s,
        1.977_998_5 * l - 2.428_592_2 * m + 0.450_593_7 * s,
        0.025_904_037 * l + 0.782_771_77 * m - 0.808_675_77 * s,
    ]
}

fn oklab_to_linear([l, a, b]: [f32; 3]) -> [f32; 3] {
    let l_ = l + 0.396_337_78 * a + 0.215_803_76 * b;
    let m_ = l - 0.105_561_346 * a - 0.063_854_17 * b;
    let s_ = l - 0.089_484_18 * a - 1.291_485_5 * b;
    let (l_, m_, s_) = (l_ * l_ * l_, m_ * m_ * m_, s_ * s_ * s_);
    [
        4.076_741_7 * l_ - 3.307_711_6 * m_ + 0.230_969_94 * s_,
        -1.268_438 * l_ + 2.609_757_4 * m_ - 0.341_319_38 * s_,
        -0.004_196_086_3 * l_ - 0.703_418_6 * m_ + 1.707_614_7 * s_,
    ]
}

/// Parse `#rgb`, `#rrggbb` or `#rrggbbaa` (alpha ignored).
pub fn parse_hex(s: &str) -> Option<[u8; 3]> {
    let s = s.trim().trim_start_matches('#');
    let v = |i: usize| u8::from_str_radix(s.get(i..i + 2)?, 16).ok();
    match s.len() {
        3 => {
            let d = |i: usize| u8::from_str_radix(s.get(i..i + 1)?, 16).ok().map(|x| x * 17);
            Some([d(0)?, d(1)?, d(2)?])
        }
        6 | 8 => Some([v(0)?, v(2)?, v(4)?]),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_srgb_through_oklch() {
        for rgb in [[0, 0, 0], [255, 255, 255], [255, 0, 0], [0, 128, 255], [20, 19, 16], [201, 168, 90], [128, 128, 128]] {
            let back = Oklch::from_srgb8(rgb).to_srgb8();
            for i in 0..3 {
                assert!((i32::from(back[i]) - i32::from(rgb[i])).abs() <= 1, "{rgb:?} -> {back:?}");
            }
        }
        assert!((Oklch::from_srgb8([255, 255, 255]).l - 1.0).abs() < 0.001);
        assert!(Oklch::from_srgb8([0, 0, 0]).l.abs() < 0.001);
    }

    #[test]
    fn out_of_gamut_colours_lose_chroma_not_hue() {
        let loud = Oklch::new(0.6, 0.4, 30.0);
        let rgb = loud.to_srgb8();
        let back = Oklch::from_srgb8(rgb);
        assert!((back.h - 30.0).abs() < 4.0, "{back:?}");
        assert!(back.c < 0.4);
        assert!((back.l - 0.6).abs() < 0.03, "{back:?}");
    }

    #[test]
    fn mixing_in_oklab_passes_through_a_clean_midpoint() {
        let red = Oklch::from_srgb8([255, 0, 0]);
        let green = Oklch::from_srgb8([0, 255, 0]);
        let mid = red.mix(green, 0.5);
        // sRGB's midpoint is the muddy (128, 128, 0); the OKLab one keeps lightness between the two.
        assert!(mid.l > (red.l + green.l) / 2.0 - 0.05, "{mid:?}");
        assert!(mid.c > 0.05);
    }

    #[test]
    fn hex_parsing_accepts_short_long_and_alpha_forms() {
        assert_eq!(parse_hex("#fff"), Some([255, 255, 255]));
        assert_eq!(parse_hex("141310"), Some([20, 19, 16]));
        assert_eq!(parse_hex("#c9a85aff"), Some([201, 168, 90]));
        assert_eq!(parse_hex("#12"), None);
        assert_eq!(parse_hex("#zzzzzz"), None);
        assert_eq!(parse_hex(""), None);
    }
}
