use crate::{Error, Result, Watermark};
use base64::{engine::general_purpose::STANDARD, Engine};
use image::{DynamicImage, ImageDecoder, ImageFormat, ImageReader};
use resvg::{tiny_skia, usvg};
use std::{
    io::Cursor,
    path::Path,
    sync::{Arc, OnceLock},
};
pub const MAX_PIXELS: u64 = 40_000_000;
fn options() -> &'static usvg::Options<'static> {
    static OPT: OnceLock<usvg::Options<'static>> = OnceLock::new();
    OPT.get_or_init(|| {
        let mut db = usvg::fontdb::Database::new();
        db.load_font_data(
            include_bytes!("../../../assets/fonts/NotoSansCJKsc-Regular.otf").to_vec(),
        );
        usvg::Options {
            font_family: "Noto Sans CJK SC".into(),
            fontdb: Arc::new(db),
            ..Default::default()
        }
    })
}
fn animated(bytes: &[u8], format: ImageFormat) -> bool {
    let mut p = if format == ImageFormat::Png { 8 } else { 12 };
    while p + 8 <= bytes.len() {
        let (len, tag, extra) = if format == ImageFormat::Png {
            (
                u32::from_be_bytes(bytes[p..p + 4].try_into().unwrap()) as usize,
                &bytes[p + 4..p + 8],
                12,
            )
        } else {
            (
                u32::from_le_bytes(bytes[p + 4..p + 8].try_into().unwrap()) as usize,
                &bytes[p..p + 4],
                8,
            )
        };
        if (format == ImageFormat::Png && tag == b"acTL")
            || (format == ImageFormat::WebP && tag == b"ANIM")
        {
            return true;
        }
        let Some(next) = p
            .checked_add(len)
            .and_then(|n| n.checked_add(extra))
            .and_then(|n| {
                n.checked_add(if format == ImageFormat::WebP {
                    len % 2
                } else {
                    0
                })
            })
        else {
            break;
        };
        if next <= p {
            break;
        }
        p = next;
    }
    false
}
pub fn load_image(path: &Path) -> Result<DynamicImage> {
    let meta = std::fs::metadata(path)?;
    if !meta.is_file() || meta.len() > 200 * 1024 * 1024 {
        return Err(Error("文件超过 200 MiB 或不是普通文件".into()));
    }
    let bytes = std::fs::read(path)?;
    let format = image::guess_format(&bytes)?;
    if ![ImageFormat::Jpeg, ImageFormat::Png, ImageFormat::WebP].contains(&format) {
        return Err(Error("仅支持 JPG、PNG、静态 WebP".into()));
    }
    if animated(&bytes, format) {
        return Err(Error("不支持动态图片，请先转换为静态图片".into()));
    }
    let mut reader = ImageReader::with_format(Cursor::new(bytes), format);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(20000);
    limits.max_image_height = Some(20000);
    limits.max_alloc = Some(400 * 1024 * 1024);
    reader.limits(limits);
    let mut decoder = reader.into_decoder()?;
    let (w, h) = decoder.dimensions();
    if u64::from(w) * u64::from(h) > MAX_PIXELS {
        return Err(Error("图片超过 4000 万像素，请缩小后再试".into()));
    }
    let orientation = decoder.orientation()?;
    let profile = decoder.icc_profile()?;
    let mut im = DynamicImage::from_decoder(decoder)?;
    im.apply_orientation(orientation);
    if let Some(profile) = profile {
        let src =
            lcms2::Profile::new_icc(&profile).map_err(|_| Error("无法读取 ICC 色彩配置".into()))?;
        if src.color_space() != lcms2::ColorSpaceSignature::RgbData {
            return Err(Error("暂不支持非 RGB ICC，请先转换为 sRGB".into()));
        }
        let dst = lcms2::Profile::new_srgb();
        let transform = lcms2::Transform::<[u8; 3], [u8; 3]>::new(
            &src,
            lcms2::PixelFormat::RGB_8,
            &dst,
            lcms2::PixelFormat::RGB_8,
            lcms2::Intent::Perceptual,
        )
        .map_err(|e| Error(format!("色彩转换失败：{e}")))?;
        let mut rgba = im.to_rgba8();
        let mut row = vec![[0u8; 3]; w.max(h) as usize];
        for y in 0..rgba.height() {
            for x in 0..rgba.width() {
                let p = rgba.get_pixel(x, y);
                row[x as usize] = [p[0], p[1], p[2]];
            }
            transform.transform_in_place(&mut row[..rgba.width() as usize]);
            for x in 0..rgba.width() {
                let p = rgba.get_pixel_mut(x, y);
                p.0[..3].copy_from_slice(&row[x as usize]);
            }
        }
        im = DynamicImage::ImageRgba8(rgba);
    }
    Ok(im)
}
pub fn png_bytes(im: &DynamicImage) -> Result<Vec<u8>> {
    let mut out = Cursor::new(Vec::new());
    im.write_to(&mut out, ImageFormat::Png)?;
    Ok(out.into_inner())
}
fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
fn stamp(spec: &Watermark, short: f32) -> Result<tiny_skia::Pixmap> {
    let size = short * spec.size / 100.;
    if spec.kind == "logo" {
        let im = load_image(
            spec.logo_path
                .as_ref()
                .ok_or_else(|| Error("缺少 Logo".into()))?,
        )?;
        let width = (size * 6.).max(1.) as u32;
        let im = im.resize(width, width, image::imageops::FilterType::Lanczos3);
        return tiny_skia::Pixmap::decode_png(&png_bytes(&im)?).map_err(|e| Error(e.to_string()));
    }
    let svg=format!("<svg xmlns='http://www.w3.org/2000/svg' width='24000' height='400'><text x='10' y='200' font-family='Noto Sans CJK SC' font-size='100' fill='{}' stroke='{}' {}>{}</text></svg>",spec.color,if spec.bold {spec.color.as_str()}else{"none"},if spec.bold {"stroke-width='2' paint-order='stroke' stroke-linejoin='round'"}else{""},escape(&spec.text));
    let tree = usvg::Tree::from_str(&svg, options()).map_err(|e| Error(e.to_string()))?;
    let bbox = tree.root().abs_layer_bounding_box();
    let scale = (size / 100.)
        .min(short * 2. / bbox.width())
        .min(short / bbox.height());
    let width = (bbox.width() * scale).ceil().max(1.) as u32 + 4;
    let height = (bbox.height() * scale).ceil().max(1.) as u32 + 4;
    let mut pix =
        tiny_skia::Pixmap::new(width, height).ok_or_else(|| Error("水印尺寸过大".into()))?;
    resvg::render(
        &tree,
        tiny_skia::Transform::from_row(
            scale,
            0.,
            0.,
            scale,
            2. - bbox.x() * scale,
            2. - bbox.y() * scale,
        ),
        &mut pix.as_mut(),
    );
    Ok(pix)
}
pub fn render(source: &DynamicImage, spec: &Watermark) -> Result<DynamicImage> {
    spec.validate()?;
    let (w, h) = (source.width(), source.height());
    let short = w.min(h) as f32;
    let pix = stamp(spec, short)?;
    let angle = spec.angle.to_radians();
    let (sin, cos) = angle.sin_cos();
    let (sw, sh) = (pix.width() as f32, pix.height() as f32);
    let (bw, bh) = (
        sw * cos.abs() + sh * sin.abs(),
        sw * sin.abs() + sh * cos.abs(),
    );
    let margin = short * spec.margin / 100.;
    let scale = 1f32
        .min(((w as f32 - 2. * margin) / bw).max(0.01))
        .min(((h as f32 - 2. * margin) / bh).max(0.01));
    let (bw, bh) = (bw * scale, bh * scale);
    let mut layer = tiny_skia::Pixmap::new(w, h).ok_or_else(|| Error("无法分配图像内存".into()))?;
    let paint = tiny_skia::PixmapPaint {
        opacity: spec.opacity / 100.,
        quality: tiny_skia::FilterQuality::Bicubic,
        ..Default::default()
    };
    let mut draw = |x: f32, y: f32| {
        let (a, b) = (cos * scale, sin * scale);
        let transform = tiny_skia::Transform::from_row(
            a,
            b,
            -b,
            a,
            x - a * sw / 2. + b * sh / 2.,
            y - b * sw / 2. - a * sh / 2.,
        );
        layer.draw_pixmap(0, 0, pix.as_ref(), &paint, transform, None);
    };
    if spec.tile {
        let step_x = bw + short * spec.spacing / 100.;
        let step_y = bh + short * spec.spacing / 100.;
        let mut y = bh / 2.;
        while y < h as f32 + bh {
            let mut x = bw / 2.;
            while x < w as f32 + bw {
                draw(x, y);
                x += step_x;
            }
            y += step_y;
        }
    } else {
        draw(
            margin + bw / 2. + spec.x * (w as f32 - 2. * margin - bw).max(0.),
            margin + bh / 2. + spec.y * (h as f32 - 2. * margin - bh).max(0.),
        );
    }
    let mut output = source.to_rgba8();
    for (dst, sp) in output.pixels_mut().zip(layer.pixels()) {
        let src = sp.demultiply();
        let sa = u32::from(src.alpha());
        let da = u32::from(dst[3]);
        let total = sa * 255 + da * (255 - sa);
        if total == 0 {
            continue;
        }
        let colors = [src.red(), src.green(), src.blue()];
        for i in 0..3 {
            dst[i] = ((u32::from(colors[i]) * sa * 255
                + u32::from(dst[i]) * da * (255 - sa)
                + total / 2)
                / total) as u8;
        }
        dst[3] = ((total + 127) / 255) as u8;
    }
    Ok(DynamicImage::ImageRgba8(output))
}
pub fn data_url(im: &DynamicImage) -> Result<String> {
    Ok(format!(
        "data:image/png;base64,{}",
        STANDARD.encode(png_bytes(im)?)
    ))
}
