//! Posterize: a brahma filter plug-in with one parameter (plug-in ABI v1, `docs/plugins.md`).
//!
//! `no_std`, no allocator, no imports. The host validates `levels` (2–32, default 4) and hands
//! it over as JSON; the few lines of `param_int` below read it without an allocator.
#![no_std]

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

const MANIFEST: &str = r#"{
  "id": "org.brahma.example.posterize",
  "name": "Posterize (WebAssembly)…",
  "version": "1.0.0",
  "kind": "filter",
  "author": "brahma contributors",
  "description": "Snaps each colour channel to a few levels; an example plug-in with a parameter.",
  "params": {"levels": {"type": "int", "min": 2, "max": 32, "default": 4}}
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

/// The integer after `"<key>":` in `json` (the host writes compact JSON with no spaces
/// around the colon), or `default`.
fn param_int(json: &[u8], key: &[u8], default: i64) -> i64 {
    let mut i = 0;
    'outer: while i + key.len() + 3 <= json.len() {
        if json[i] == b'"' && json[i + 1..].starts_with(key) && json.get(i + 1 + key.len()) == Some(&b'"') {
            let mut j = i + key.len() + 2;
            while j < json.len() && (json[j] == b':' || json[j] == b' ') {
                j += 1;
            }
            let neg = json.get(j) == Some(&b'-');
            if neg {
                j += 1;
            }
            let mut v: i64 = 0;
            let mut any = false;
            while j < json.len() && json[j].is_ascii_digit() {
                v = v.saturating_mul(10).saturating_add(i64::from(json[j] - b'0'));
                any = true;
                j += 1;
            }
            if !any {
                break 'outer;
            }
            return if neg { -v } else { v };
        }
        i += 1;
    }
    default
}

/// # Safety
/// `buf..buf + len` is a block from `pc_alloc` holding `len / 4` samples; `params` points at
/// `params_len` bytes of JSON (the host's contract).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pc_filter(buf: *mut f32, len: u32, _width: u32, _height: u32, channels: u32, format: u32, params: *const u8, params_len: u32) -> i32 {
    let n = channels as usize;
    if n == 0 {
        return 1;
    }
    // SAFETY: see the function's contract.
    let (px, json) = unsafe { (core::slice::from_raw_parts_mut(buf, len as usize / 4), core::slice::from_raw_parts(params, params_len as usize)) };
    let levels = param_int(json, b"levels", 4).clamp(2, 32) as f32;
    let alpha = format & HAS_ALPHA != 0;
    let colour = if alpha { n - 1 } else { n };
    let steps = levels - 1.0;
    for p in px.chunks_exact_mut(n) {
        if alpha && p[n - 1] <= 0.0 {
            continue;
        }
        for v in &mut p[..colour] {
            let c = if *v < 0.0 { 0.0 } else if *v > 1.0 { 1.0 } else { *v };
            // Round to the nearest level without libm.
            let q = c * steps + 0.5;
            *v = (q as i32) as f32 / steps;
        }
    }
    0
}
