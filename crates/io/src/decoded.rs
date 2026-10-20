//! Decoding a photo off the UI thread in the browser: the heavy half of [`crate::import`] (the
//! codec) runs in a web worker with its own copy of the WebAssembly module, and the result
//! crosses back as one flat byte envelope ([`pack`] / [`unpack`]) the main thread turns into a
//! document ([`import_decoded`]) with the cheap steps that need the session (palette mode, the
//! file encoding, EXR tagging). Only the formats whose decode is the cost are eligible
//! ([`worker_eligible`]): the common photo formats. Everything else keeps the inline path.
//!
//! The envelope is a private wire format between the two copies of the same build, not a file
//! format: little-endian lengths, no versioning beyond a magic word.

use brahma_codecs::{self as codecs, ChannelLayout, Format, Image, Metadata, SampleType};

use crate::{ImportResult, IoError};

const MAGIC: &[u8; 4] = b"BDC1";

/// Formats worth a worker round trip: the photo formats whose decoding takes real time and
/// whose document needs nothing but the bytes afterwards.
pub fn worker_eligible(name: &str, bytes: &[u8]) -> bool {
    if crate::is_psd(bytes) || crate::is_svg_input(name, bytes) || crate::raw::is_raw(bytes) {
        return false;
    }
    matches!(codecs::detect(bytes), Some(Format::Jpeg | Format::Png | Format::WebP | Format::Heif | Format::Avif | Format::Bmp | Format::Qoi))
}

/// Decode `bytes` (the worker's half) into an envelope for [`unpack`].
pub fn decode_for_worker(name: &str, bytes: &[u8]) -> Result<Vec<u8>, IoError> {
    if !worker_eligible(name, bytes) {
        return Err(IoError::Unsupported(format!("{name}: not a format the worker decodes")));
    }
    let format = codecs::detect(bytes).ok_or(codecs::CodecError::UnknownFormat)?;
    let img = codecs::decode_as(format, bytes)?;
    Ok(pack(&img))
}

fn layout_code(l: ChannelLayout) -> u8 {
    match l {
        ChannelLayout::Gray => 0,
        ChannelLayout::GrayA => 1,
        ChannelLayout::Rgb => 2,
        ChannelLayout::Rgba => 3,
        ChannelLayout::Cmyk => 4,
        ChannelLayout::CmykA => 5,
    }
}

fn layout_of(code: u8) -> Option<ChannelLayout> {
    Some(match code {
        0 => ChannelLayout::Gray,
        1 => ChannelLayout::GrayA,
        2 => ChannelLayout::Rgb,
        3 => ChannelLayout::Rgba,
        4 => ChannelLayout::Cmyk,
        5 => ChannelLayout::CmykA,
        _ => return None,
    })
}

fn sample_code(s: SampleType) -> u8 {
    match s {
        SampleType::U8 => 0,
        SampleType::U16 => 1,
        SampleType::F16 => 2,
        SampleType::F32 => 3,
    }
}

fn sample_of(code: u8) -> Option<SampleType> {
    Some(match code {
        0 => SampleType::U8,
        1 => SampleType::U16,
        2 => SampleType::F16,
        3 => SampleType::F32,
        _ => return None,
    })
}

fn put_block(out: &mut Vec<u8>, b: &[u8]) {
    out.extend_from_slice(&(b.len() as u32).to_le_bytes());
    out.extend_from_slice(b);
}

/// The image as one byte envelope: header, ICC, EXIF, XMP, text pairs, then the samples.
pub fn pack(img: &Image) -> Vec<u8> {
    let (w, h) = img.dimensions();
    let mut out = Vec::with_capacity(img.data().len() + 256 + img.icc.as_ref().map_or(0, Vec::len));
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&w.to_le_bytes());
    out.extend_from_slice(&h.to_le_bytes());
    out.push(layout_code(img.layout()));
    out.push(sample_code(img.sample_type()));
    let (dx, dy) = img.meta.dpi.unwrap_or((0.0, 0.0));
    out.extend_from_slice(&dx.to_le_bytes());
    out.extend_from_slice(&dy.to_le_bytes());
    put_block(&mut out, img.icc.as_deref().unwrap_or(&[]));
    put_block(&mut out, img.meta.exif.as_deref().unwrap_or(&[]));
    put_block(&mut out, img.meta.xmp.as_deref().unwrap_or("").as_bytes());
    out.extend_from_slice(&(img.meta.text.len().min(u32::MAX as usize) as u32).to_le_bytes());
    for (k, v) in img.meta.text.iter().take(u32::MAX as usize) {
        put_block(&mut out, k.as_bytes());
        put_block(&mut out, v.as_bytes());
    }
    let warnings: Vec<String> = img.warnings.iter().map(ToString::to_string).collect();
    put_block(&mut out, warnings.join("\n").as_bytes());
    put_block(&mut out, img.data());
    out
}

struct Reader<'a> {
    b: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], IoError> {
        let end = self.at.checked_add(n).filter(|e| *e <= self.b.len()).ok_or_else(|| IoError::Unsupported("decoded image envelope is cut short".into()))?;
        let s = self.b.get(self.at..end).ok_or_else(|| IoError::Unsupported("decoded image envelope is cut short".into()))?;
        self.at = end;
        Ok(s)
    }
    fn u32(&mut self) -> Result<u32, IoError> {
        let s = self.take(4)?;
        Ok(u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
    }
    fn f32(&mut self) -> Result<f32, IoError> {
        Ok(f32::from_bits(self.u32()?))
    }
    fn u8(&mut self) -> Result<u8, IoError> {
        Ok(self.take(1)?[0])
    }
    fn block(&mut self) -> Result<&'a [u8], IoError> {
        let n = self.u32()? as usize;
        self.take(n)
    }
    fn text(&mut self) -> Result<String, IoError> {
        String::from_utf8(self.block()?.to_vec()).map_err(|_| IoError::Unsupported("decoded image envelope holds bad text".into()))
    }
}

/// The image back from [`pack`]'s envelope, with the decoder's warnings as text. Hostile or
/// truncated input is an error, never a panic: every length is checked and the sample buffer
/// must match the header exactly.
pub fn unpack(env: &[u8]) -> Result<(Image, Vec<String>), IoError> {
    let mut r = Reader { b: env, at: 0 };
    if r.take(4)? != MAGIC {
        return Err(IoError::Unsupported("not a decoded image envelope".into()));
    }
    let (w, h) = (r.u32()?, r.u32()?);
    let layout = layout_of(r.u8()?).ok_or_else(|| IoError::Unsupported("decoded image envelope: bad layout".into()))?;
    let sample = sample_of(r.u8()?).ok_or_else(|| IoError::Unsupported("decoded image envelope: bad sample type".into()))?;
    let (dx, dy) = (r.f32()?, r.f32()?);
    let icc = r.block()?.to_vec();
    let exif = r.block()?.to_vec();
    let xmp = r.text()?;
    let n = r.u32()? as usize;
    let mut text = Vec::with_capacity(n.min(1024));
    for _ in 0..n {
        text.push((r.text()?, r.text()?));
    }
    let warnings = r.text()?;
    let data = r.block()?.to_vec();
    let expected = (w as usize).checked_mul(h as usize).and_then(|p| p.checked_mul(layout.channels())).and_then(|p| p.checked_mul(sample.bytes()));
    if expected != Some(data.len()) {
        return Err(IoError::Unsupported("decoded image envelope: sample buffer does not match its header".into()));
    }
    let mut img = Image::from_raw(w, h, layout, sample, data)?;
    img.icc = (!icc.is_empty()).then_some(icc);
    img.meta = Metadata {
        exif: (!exif.is_empty()).then_some(exif),
        xmp: (!xmp.is_empty()).then_some(xmp),
        dpi: (dx > 0.0 && dy > 0.0).then_some((dx, dy)),
        text,
        ..Default::default()
    };
    Ok((img, warnings.lines().filter(|l| !l.is_empty()).map(str::to_string).collect()))
}

/// The main thread's half: the document for an image the worker decoded from `bytes`, with
/// the decoder's `warnings` first, as the inline import would list them.
pub fn import_decoded(name: &str, bytes: &[u8], img: &Image, warnings: &[String]) -> Result<ImportResult, IoError> {
    let mut r = crate::flat::import_decoded_flat(name, bytes, img)?;
    if !warnings.is_empty() {
        let mut all = warnings.to_vec();
        all.append(&mut r.warnings);
        r.warnings = all;
    }
    Ok(r)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png(w: u32, h: u32) -> Vec<u8> {
        let img = Image::from_u8(w, h, ChannelLayout::Rgba, (0..w * h * 4).map(|i| (i * 7 % 251) as u8).collect()).unwrap();
        let img = img.with_meta(Metadata { dpi: Some((300.0, 300.0)), text: vec![("Comment".into(), "hi".into())], ..Default::default() });
        codecs::encode(&img, Format::Png, &codecs::EncodeOptions::default()).unwrap()
    }

    #[test]
    fn the_worker_half_and_the_main_half_make_the_same_document_as_the_inline_import() {
        let bytes = png(12, 9);
        assert!(worker_eligible("a.png", &bytes));
        let env = decode_for_worker("a.png", &bytes).unwrap();
        let (img, warnings) = unpack(&env).unwrap();
        assert!(warnings.is_empty());
        assert_eq!(img.dimensions(), (12, 9));
        assert_eq!(img.meta.dpi, Some((300.0, 300.0)));
        assert_eq!(img.meta.text, vec![("Comment".to_string(), "hi".to_string())]);
        let split = import_decoded("a.png", &bytes, &img, &warnings).unwrap();
        let inline = crate::import("a.png", &bytes).unwrap();
        assert_eq!(split.document.size, inline.document.size);
        assert_eq!(split.document.layers.len(), inline.document.layers.len());
        assert_eq!(split.document.resolution_dpi, inline.document.resolution_dpi);
        assert_eq!(split.document.file_encoding, inline.document.file_encoding);
        let a = brahma_compose::render(&split.document, brahma_geom::Rect::new(0, 0, 12, 9)).px;
        let b = brahma_compose::render(&inline.document, brahma_geom::Rect::new(0, 0, 12, 9)).px;
        assert_eq!(a, b, "the pixels are the same either way");
    }

    #[test]
    fn only_photo_formats_go_to_the_worker_and_broken_envelopes_are_errors() {
        assert!(!worker_eligible("x.svg", b"<svg xmlns='http://www.w3.org/2000/svg'/>"));
        assert!(!worker_eligible("x.psd", b"8BPS\0\x01"));
        assert!(!worker_eligible("x.bin", b"nothing here"));
        assert!(decode_for_worker("x.bin", b"nothing").is_err());
        let env = pack(&Image::from_u8(2, 2, ChannelLayout::Rgb, vec![1; 12]).unwrap());
        assert!(unpack(&env).is_ok());
        for cut in [0, 3, 4, 10, 20, env.len() - 1] {
            assert!(unpack(&env[..cut]).is_err(), "cut at {cut}");
        }
        let mut bad = env.clone();
        bad[12] = 9; // layout code
        assert!(unpack(&bad).is_err());
        let mut short = env.clone();
        let n = short.len();
        short[n - 13] = 11; // the sample block length no longer matches 2 × 2 × 3
        assert!(unpack(&short).is_err());
        assert!(unpack(b"").is_err());
        assert!(unpack(b"BDC1").is_err());
    }
}
