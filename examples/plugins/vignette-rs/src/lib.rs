//! Vignette: a brahma filter plug-in that depends on position (plug-in ABI v1,
//! `docs/plugins.md`). The host runs filters in bands; `_image` in the params says where this
//! band sits on the canvas, so the darkening is centred on the picture, not on each band.
#![no_std]

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

const MANIFEST: &str = r#"{
  "id": "org.brahma.example.vignette",
  "name": "Vignette (WebAssembly)…",
  "version": "1.0.0",
  "kind": "filter",
  "author": "brahma contributors",
  "description": "Darkens towards the corners; an example plug-in that uses the band's position on the canvas.",
  "params": {"amount": {"type": "number", "min": 0, "max": 100, "default": 50}},
  "area": "canvas"
}"#;

const HAS_ALPHA: u32 = 1 << 8;

#[unsafe(no_mangle)]
pub extern "C" fn pc_abi_version() -> u32 {
    1
}

#[unsafe(no_mangle)]
pub extern "C" fn pc_manifest() -> u64 {
    ((MANIFEST.len() as u64) << 32) | MANIFEST.as_ptr() as u64
}

#[unsafe(no_mangle)]
pub extern "C" fn pc_alloc(size: u32) -> u32 {
    let pages = (size as usize).div_ceil(65536);
    let old = core::arch::wasm32::memory_grow(0, pages);
    if old == usize::MAX { 0 } else { (old * 65536) as u32 }
}

/// The number after `"<key>":` in `json` (integer or decimal, optional sign), or `default`.
fn param_num(json: &[u8], key: &[u8], default: f32) -> f32 {
    let mut i = 0;
    while i + key.len() + 3 <= json.len() {
        if json[i] == b'"' && json[i + 1..].starts_with(key) && json.get(i + 1 + key.len()) == Some(&b'"') {
            let mut j = i + key.len() + 2;
            while j < json.len() && (json[j] == b':' || json[j] == b' ') {
                j += 1;
            }
            let neg = json.get(j) == Some(&b'-');
            if neg {
                j += 1;
            }
            let (mut v, mut scale, mut frac, mut any) = (0.0f32, 1.0f32, false, false);
            while j < json.len() {
                let c = json[j];
                if c.is_ascii_digit() {
                    let d = f32::from(c - b'0');
                    if frac {
                        scale *= 0.1;
                        v += d * scale;
                    } else {
                        v = v * 10.0 + d;
                    }
                    any = true;
                } else if c == b'.' && !frac {
                    frac = true;
                } else {
                    break;
                }
                j += 1;
            }
            return if any { if neg { -v } else { v } } else { default };
        }
        i += 1;
    }
    default
}

/// # Safety
/// `buf..buf + len` is a block from `pc_alloc` holding `len / 4` samples; `params` points at
/// `params_len` bytes of JSON (the host's contract).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pc_filter(buf: *mut f32, len: u32, width: u32, height: u32, channels: u32, format: u32, params: *const u8, params_len: u32) -> i32 {
    let n = channels as usize;
    if n == 0 || width == 0 || height == 0 {
        return 1;
    }
    // SAFETY: see the function's contract.
    let (px, json) = unsafe { (core::slice::from_raw_parts_mut(buf, len as usize / 4), core::slice::from_raw_parts(params, params_len as usize)) };
    let amount = param_num(json, b"amount", 50.0).clamp(0.0, 100.0) / 100.0;
    // Where this band sits on the canvas (the `_image` object; its keys are unique in the JSON).
    let x0 = param_num(json, b"x", 0.0);
    let y0 = param_num(json, b"y", 0.0);
    let cw = param_num(json, b"canvasWidth", width as f32).max(1.0);
    let ch = param_num(json, b"canvasHeight", height as f32).max(1.0);
    let alpha = format & HAS_ALPHA != 0;
    let colour = if alpha { n - 1 } else { n };
    let (w, h) = (width as usize, height as usize);
    for y in 0..h {
        let dy = ((y0 + y as f32 + 0.5) / ch - 0.5) * 2.0;
        for x in 0..w {
            let dx = ((x0 + x as f32 + 0.5) / cw - 0.5) * 2.0;
            // 0 at the centre, 1 at the corners (distance squared keeps it libm-free).
            let r2 = (dx * dx + dy * dy) * 0.5;
            let r2 = if r2 > 1.0 { 1.0 } else { r2 };
            let k = 1.0 - amount * r2 * r2;
            let p = &mut px[(y * w + x) * n..(y * w + x) * n + n];
            if alpha && p[n - 1] <= 0.0 {
                continue;
            }
            for v in &mut p[..colour] {
                *v *= k;
            }
        }
    }
    0
}
