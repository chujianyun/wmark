use crate::{
    load_image, model::parse_color, render, BatchResult, Error, ExportOptions, FileResult, Result,
    Watermark,
};
use image::{DynamicImage, ImageEncoder, ImageFormat};
use std::{
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};
pub fn export_one(
    source: &Path,
    spec: &Watermark,
    options: &ExportOptions,
    cancel: &AtomicBool,
) -> Result<PathBuf> {
    options.validate()?;
    spec.validate()?;
    if cancel.load(Ordering::Relaxed) {
        return Err(Error("任务已取消".into()));
    }
    let input = load_image(source)?;
    let output = render(&input, spec)?;
    if cancel.load(Ordering::Relaxed) {
        return Err(Error("任务已取消".into()));
    }
    let format = if options.format == "original" {
        let reader = image::ImageReader::open(source)?.with_guessed_format()?;
        match reader.format() {
            Some(ImageFormat::Jpeg) => "jpg",
            Some(ImageFormat::WebP) => "webp",
            _ => "png",
        }
    } else {
        &options.format
    };
    let mut temp = tempfile::NamedTempFile::new_in(&options.directory)?;
    {
        let mut writer = BufWriter::new(temp.as_file_mut());
        match format {
            "jpg" => {
                let bg = parse_color(&options.background)?;
                let rgba = output.to_rgba8();
                let mut rgb = image::RgbImage::new(output.width(), output.height());
                for (d, s) in rgb.pixels_mut().zip(rgba.pixels()) {
                    let a = s[3] as u32;
                    for c in 0..3 {
                        d[c] = ((s[c] as u32 * a + bg[c] as u32 * (255 - a) + 127) / 255) as u8;
                    }
                }
                image::codecs::jpeg::JpegEncoder::new_with_quality(&mut writer, options.quality)
                    .encode_image(&DynamicImage::ImageRgb8(rgb))?;
            }
            "webp" => {
                image::codecs::webp::WebPEncoder::new_lossless(&mut writer).write_image(
                    output.to_rgba8().as_raw(),
                    output.width(),
                    output.height(),
                    image::ExtendedColorType::Rgba8,
                )?;
            }
            _ => {
                image::codecs::png::PngEncoder::new(&mut writer).write_image(
                    output.to_rgba8().as_raw(),
                    output.width(),
                    output.height(),
                    image::ExtendedColorType::Rgba8,
                )?;
            }
        }
        writer.flush()?;
    }
    temp.as_file().sync_all()?;
    if cancel.load(Ordering::Relaxed) {
        return Err(Error("任务已取消".into()));
    }
    let stem = source
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("image");
    let safe: String = stem
        .chars()
        .take(100)
        .map(|c| {
            if c.is_control() || "<>:\"/\\|?*".contains(c) {
                '_'
            } else {
                c
            }
        })
        .collect();
    for i in 0..10000 {
        let number = if i == 0 {
            String::new()
        } else {
            format!("_{i}")
        };
        let target = options
            .directory
            .join(format!("{safe}{}{number}.{format}", options.suffix));
        match temp.persist_noclobber(&target) {
            Ok(_) => return Ok(target),
            Err(e) if e.error.kind() == std::io::ErrorKind::AlreadyExists => {
                temp = e.file;
            }
            Err(e) => return Err(e.error.into()),
        }
    }
    Err(Error("同名文件过多，请修改后缀".into()))
}
pub fn run_batch<F: FnMut(&BatchResult)>(
    sources: &[PathBuf],
    spec: &Watermark,
    options: &ExportOptions,
    cancel: &AtomicBool,
    mut progress: F,
) -> Result<BatchResult> {
    options.validate()?;
    spec.validate()?;
    let mut result = BatchResult {
        total: sources.len(),
        ..Default::default()
    };
    for source in sources {
        if cancel.load(Ordering::Relaxed) {
            result.cancelled = true;
            break;
        }
        let res = export_one(source, spec, options, cancel);
        if cancel.load(Ordering::Relaxed) && res.is_err() {
            result.cancelled = true;
            break;
        }
        let file = match res {
            Ok(output) => FileResult {
                source: source.clone(),
                output: Some(output),
                error: None,
            },
            Err(e) => FileResult {
                source: source.clone(),
                output: None,
                error: Some(e.to_string()),
            },
        };
        result.files.push(file);
        progress(&result);
    }
    Ok(result)
}
