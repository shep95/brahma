# nikon D5100 NEF development

validated 2026-10-11 with three locally supplied nefs: DSC_0453 (portrait),
DSC_0469 and DSC_0477. photos, extracted previews and metadata dumps stay local;
regression fixtures are synthetic.

## findings and change

all three files contain 4992 × 3280, 14-bit nikon lossy type-2 sensor data
with a normal `44 20` linearization table. their as-shot white balances are
already decoded correctly. they have no colour calibration, so the neutral
fallback treats white-balanced camera channels as linear srgb and looks flat.
the nikon `44 40` quarter-span decoder issue addressed elsewhere does not apply.

this branch uses the existing embedded-JPEG rendering work in
[brahma #2135](https://github.com/shep95/brahma/pull/2135), head
`d1a91d1f0dff1ed7ac74767b602051e88198b2e5`. that work fits a matrix and monotone
tone curve with held-out acceptance checks. it supplies the appearance fix;
this follow-up preserves that rendering in `import_raw_tuned`. fitting happens
at default exposure and camera wb before applying user controls, so changing
exposure, temperature or tint cannot be absorbed by a new fit. unusable previews
still use the documented neutral fallback, and calibrated sensors skip fitting.

aligning green-channel gradients of the developed sensor plane against each
camera's full 4928 × 3264 JPEG identifies the same (8, 8) crop origin. searching
even offsets x=0…56, y=0…16 gives best normalized correlations of 0.944, 0.698
and 0.684 for the three files respectively. a model- and sensor-size-matched
D5100 image-area entry removes the magenta edge and applies before orientation;
portrait output is 3264 × 4928. it preserves an explicit file crop and leaves
black, white, wb and colour calibration unchanged.

the camera JPEG is a rendering reference, not a calibration chart. its picture
style, local tone mapping and clipping cannot all be reproduced by one matrix
and curve. no D5100 colour matrix or unrelated camera profile was added.

after applying the measured crop, all three fits pass their held-out acceptance
checks. mean CIE76 δe on the held-out grid against each camera JPEG is:

| file | neutral fallback | fitted rendering | final upright size |
|---|---:|---:|---|
| DSC_0453 | 13.68 | 2.07 | 3264 × 4928 |
| DSC_0469 | 13.90 | 2.45 | 4928 × 3264 |
| DSC_0477 | 7.62 | 1.62 | 4928 × 3264 |

these values measure appearance against the file's JPEG, not calibration accuracy.
full-resolution default, +0.5 ev and temperature +20/tint −15 imports were exported
for each file and independently verified as RGB16 tiffs with the ProPhoto-compatible
ICC profile. default tuned pixels and ICC bytes equal normal import exactly. local
whole-frame and 100% edge comparisons were inspected against the camera JPEG and
installed #houseofasher 0.6.0: the border is gone and colour/contrast improve, with
remaining differences in sunset highlight saturation and noise/detail rendering.

## regression coverage

synthetic tests verify identical normal/default-tuned pixels, preserved fitted
appearance with positive/negative exposure and wb edits, exact application of
user gains after fitting, neutral fallback for missing or unsuitable previews,
and graceful truncated-input handling. image-area tests verify the D5100 crop,
unchanged sensor metadata, explicit-crop precedence and exact model/size guards.
all final imports retain 16-bit RGB in the ProPhoto-compatible working profile.
