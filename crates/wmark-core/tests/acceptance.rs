use image::{DynamicImage, ImageEncoder, Rgba, RgbaImage};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};
use wmark_core::*;
fn image() -> DynamicImage {
    DynamicImage::ImageRgba8(RgbaImage::from_pixel(320, 240, Rgba([20, 30, 40, 255])))
}
fn spec() -> Watermark {
    Watermark {
        text: "悟鸣 © Hello <&>".into(),
        size: 8.,
        ..Default::default()
    }
}
fn setup() -> (tempfile::TempDir, PathBuf, ExportOptions) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("sample.png");
    image().save(&path).unwrap();
    let options = ExportOptions {
        directory: dir.path().into(),
        format: "png".into(),
        quality: 90,
        suffix: "_watermarked".into(),
        background: "#ffffff".into(),
    };
    (dir, path, options)
}
fn changed(im: &DynamicImage) -> Vec<(u32, u32)> {
    im.to_rgba8()
        .enumerate_pixels()
        .filter(|(_, _, p)| p.0 != [20, 30, 40, 255])
        .map(|(x, y, _)| (x, y))
        .collect()
}
#[test]
fn r03_chinese_and_xml_characters_render() {
    let out = render(&image(), &spec()).unwrap();
    assert!(changed(&out).len() > 100);
    assert_eq!((out.width(), out.height()), (320, 240));
}
#[test]
fn r03_zero_opacity_is_pixel_identical() {
    let out = render(
        &image(),
        &Watermark {
            opacity: 0.,
            ..spec()
        },
    )
    .unwrap();
    assert_eq!(out.to_rgba8(), image().to_rgba8());
}
#[test]
fn r03_bold_changes_output() {
    let a = render(&image(), &spec()).unwrap();
    let b = render(
        &image(),
        &Watermark {
            bold: true,
            ..spec()
        },
    )
    .unwrap();
    assert_ne!(a.to_rgba8(), b.to_rgba8());
}
#[test]
fn r03_nine_anchors_stay_inside_margin() {
    for y in [0., 0.5, 1.] {
        for x in [0., 0.5, 1.] {
            let out = render(
                &image(),
                &Watermark {
                    x,
                    y,
                    angle: 35.,
                    text: "水印".into(),
                    ..spec()
                },
            )
            .unwrap();
            let points = changed(&out);
            assert!(!points.is_empty());
            assert!(points
                .iter()
                .all(|(x, y)| *x >= 10 && *x < 310 && *y >= 10 && *y < 230));
        }
    }
}
#[test]
fn r03_top_left_and_bottom_right_differ() {
    let a = changed(
        &render(
            &image(),
            &Watermark {
                x: 0.,
                y: 0.,
                text: "A".into(),
                ..spec()
            },
        )
        .unwrap(),
    );
    let b = changed(
        &render(
            &image(),
            &Watermark {
                x: 1.,
                y: 1.,
                text: "A".into(),
                ..spec()
            },
        )
        .unwrap(),
    );
    assert!(a.iter().map(|p| p.0).max().unwrap() < b.iter().map(|p| p.0).min().unwrap());
}
#[test]
fn r03_tile_covers_multiple_regions() {
    let out = render(
        &image(),
        &Watermark {
            tile: true,
            text: "样片".into(),
            ..spec()
        },
    )
    .unwrap();
    let pts = changed(&out);
    assert!(pts.iter().any(|p| p.0 < 80 && p.1 < 80));
    assert!(pts.iter().any(|p| p.0 > 200 && p.1 > 150));
}
#[test]
fn r03_rotated_long_text_fits() {
    let out = render(
        &image(),
        &Watermark {
            text: "水".repeat(100),
            angle: -90.,
            size: 30.,
            ..spec()
        },
    )
    .unwrap();
    assert!(!changed(&out).is_empty());
}
#[test]
fn r03_transparent_logo_preserves_background() {
    let (dir, _, _) = setup();
    let logo = dir.path().join("logo.png");
    RgbaImage::from_pixel(40, 40, Rgba([255, 0, 0, 128]))
        .save(&logo)
        .unwrap();
    let out = render(
        &image(),
        &Watermark {
            kind: "logo".into(),
            logo_path: Some(logo),
            x: 0.5,
            y: 0.5,
            opacity: 100.,
            ..spec()
        },
    )
    .unwrap()
    .to_rgba8();
    assert_eq!(out.get_pixel(0, 0).0, [20, 30, 40, 255]);
    let p = out.get_pixel(160, 120);
    assert!(p[0] > 130 && p[0] < 145);
    assert_eq!(p[3], 255);
}
#[test]
fn r03_invalid_inputs_rejected() {
    for s in [
        Watermark {
            text: "".into(),
            ..spec()
        },
        Watermark {
            size: f32::NAN,
            ..spec()
        },
        Watermark {
            color: "#GGGGGG".into(),
            ..spec()
        },
        Watermark {
            kind: "invalid".into(),
            ..spec()
        },
        Watermark {
            kind: "logo".into(),
            logo_path: None,
            ..spec()
        },
        Watermark {
            text: "x".repeat(101),
            ..spec()
        },
        Watermark {
            text: "a\nb".into(),
            ..spec()
        },
        Watermark { x: 1.1, ..spec() },
    ] {
        assert!(render(&image(), &s).is_err());
    }
}
#[test]
fn r01_missing_and_corrupt_are_errors() {
    let (dir, _, _) = setup();
    assert!(load_image(&dir.path().join("missing")).is_err());
    let p = dir.path().join("bad.png");
    std::fs::write(&p, b"not an image").unwrap();
    assert!(load_image(&p).is_err());
}
#[test]
fn r01_format_is_detected_from_bytes() {
    let (dir, path, _) = setup();
    let renamed = dir.path().join("image.jpg");
    std::fs::copy(path, &renamed).unwrap();
    let im = load_image(&renamed).unwrap();
    assert_eq!(im.width(), 320);
}
#[test]
fn r08_animated_png_rejected() {
    let (dir, path, _) = setup();
    let mut bytes = std::fs::read(path).unwrap();
    let chunk = [
        0, 0, 0, 8, b'a', b'c', b'T', b'L', 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    bytes.splice(33..33, chunk);
    let p = dir.path().join("animated.png");
    std::fs::write(&p, bytes).unwrap();
    assert!(load_image(&p).unwrap_err().to_string().contains("动态"));
}
#[test]
fn r08_animated_webp_rejected() {
    let (dir, _, _) = setup();
    let p = dir.path().join("animated.webp");
    std::fs::write(&p, b"RIFF\x14\0\0\0WEBPANIM\x06\0\0\0\0\0\0\0\0\0").unwrap();
    assert!(load_image(&p).unwrap_err().to_string().contains("动态"));
}
#[test]
fn r06_all_output_formats_decode() {
    let (_dir, path, mut o) = setup();
    for format in ["png", "jpg", "webp", "original"] {
        o.format = format.into();
        let out = export_one(&path, &spec(), &o, &AtomicBool::new(false)).unwrap();
        let decoded = load_image(&out).unwrap();
        assert_eq!((decoded.width(), decoded.height()), (320, 240));
        assert!(!changed(&decoded).is_empty());
    }
}
#[test]
fn r06_original_and_existing_files_never_overwritten() {
    let (_dir, path, mut o) = setup();
    let original = std::fs::read(&path).unwrap();
    o.suffix = "".into();
    let a = export_one(&path, &spec(), &o, &AtomicBool::new(false)).unwrap();
    let b = export_one(&path, &spec(), &o, &AtomicBool::new(false)).unwrap();
    assert_ne!(a, path);
    assert_ne!(a, b);
    assert_eq!(std::fs::read(path).unwrap(), original);
    assert!(a.exists() && b.exists());
}
#[test]
fn r04_export_matches_core_preview_pixels() {
    let (_dir, path, o) = setup();
    let source = load_image(&path).unwrap();
    let expected = render(&source, &spec()).unwrap();
    let out = export_one(&path, &spec(), &o, &AtomicBool::new(false)).unwrap();
    assert_eq!(load_image(&out).unwrap().to_rgba8(), expected.to_rgba8());
}
#[test]
fn r06_transparent_jpeg_uses_background() {
    let (_dir, path, mut o) = setup();
    RgbaImage::from_pixel(320, 240, Rgba([0, 0, 0, 0]))
        .save(&path)
        .unwrap();
    o.format = "jpg".into();
    o.background = "#ff0000".into();
    let out = export_one(
        &path,
        &Watermark {
            opacity: 0.,
            ..spec()
        },
        &o,
        &AtomicBool::new(false),
    )
    .unwrap();
    let p = load_image(&out).unwrap().to_rgb8().get_pixel(10, 10).0;
    assert!(p[0] > 250 && p[1] < 5 && p[2] < 5);
}
#[test]
fn r06_suffix_traversal_and_invalid_directory_rejected() {
    let (_dir, _, mut o) = setup();
    o.suffix = "/../../escape".into();
    assert!(o.validate().is_err());
    o.suffix = "_ok".into();
    o.directory = o.directory.join("missing");
    assert!(o.validate().is_err());
}
#[test]
fn r07_partial_failure_does_not_stop_batch() {
    let (dir, path, o) = setup();
    let missing = dir.path().join("missing.jpg");
    let mut progress = Vec::new();
    let result = run_batch(
        &[path.clone(), missing, path],
        &spec(),
        &o,
        &AtomicBool::new(false),
        |r| progress.push(r.files.len()),
    )
    .unwrap();
    assert_eq!(progress, vec![1, 2, 3]);
    assert_eq!(
        result.files.iter().filter(|f| f.output.is_some()).count(),
        2
    );
    assert!(result.files[1].error.is_some());
}
#[test]
fn r07_cancel_retains_only_completed_outputs() {
    let (_dir, path, o) = setup();
    let cancel = AtomicBool::new(false);
    let result = run_batch(&[path.clone(), path], &spec(), &o, &cancel, |_| {
        cancel.store(true, Ordering::SeqCst)
    })
    .unwrap();
    assert!(result.cancelled);
    assert_eq!(result.files.len(), 1);
    assert!(result.files[0].output.as_ref().unwrap().exists());
}
#[test]
fn r07_precancel_writes_nothing() {
    let (dir, path, o) = setup();
    let before = std::fs::read_dir(dir.path()).unwrap().count();
    let result = run_batch(&[path], &spec(), &o, &AtomicBool::new(true), |_| {}).unwrap();
    assert!(result.cancelled);
    assert!(result.files.is_empty());
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), before);
}
#[test]
fn r07_retry_failed_after_fix() {
    let (dir, _, o) = setup();
    let p = dir.path().join("late.png");
    let first = run_batch(
        std::slice::from_ref(&p),
        &spec(),
        &o,
        &AtomicBool::new(false),
        |_| {},
    )
    .unwrap();
    assert!(first.files[0].error.is_some());
    image().save(&p).unwrap();
    let retry = run_batch(
        &[first.files[0].source.clone()],
        &spec(),
        &o,
        &AtomicBool::new(false),
        |_| {},
    )
    .unwrap();
    assert!(retry.files[0].output.is_some());
}
#[test]
fn r08_exif_orientation_and_metadata_removed() {
    let (dir, _, mut o) = setup();
    let p = dir.path().join("oriented.jpg");
    let exif: Vec<u8> = vec![
        b'I', b'I', 42, 0, 8, 0, 0, 0, 1, 0, 0x12, 1, 3, 0, 1, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0,
    ];
    let mut bytes = Vec::new();
    let mut encoder = image::codecs::jpeg::JpegEncoder::new(&mut bytes);
    encoder.set_exif_metadata(exif).unwrap();
    encoder.encode_image(&image()).unwrap();
    std::fs::write(&p, bytes).unwrap();
    let loaded = load_image(&p).unwrap();
    assert_eq!((loaded.width(), loaded.height()), (240, 320));
    o.format = "jpg".into();
    let out = export_one(&p, &spec(), &o, &AtomicBool::new(false)).unwrap();
    let file = std::fs::File::open(out).unwrap();
    assert!(exif::Reader::new()
        .read_from_container(&mut std::io::BufReader::new(file))
        .is_err());
}
#[test]
fn r08_srgb_icc_is_accepted() {
    let (dir, _, _) = setup();
    let p = dir.path().join("icc.png");
    let mut bytes = Vec::new();
    let mut encoder = image::codecs::png::PngEncoder::new(&mut bytes);
    encoder
        .set_icc_profile(lcms2::Profile::new_srgb().icc().unwrap())
        .unwrap();
    encoder
        .write_image(
            image().to_rgba8().as_raw(),
            320,
            240,
            image::ExtendedColorType::Rgba8,
        )
        .unwrap();
    std::fs::write(&p, bytes).unwrap();
    let loaded = load_image(&p).unwrap();
    assert_eq!(loaded.to_rgba8().get_pixel(0, 0).0, [20, 30, 40, 255]);
}
#[test]
fn r07_hundred_image_batch_completes() {
    let (_dir, path, o) = setup();
    let result = run_batch(
        &vec![path; 100],
        &spec(),
        &o,
        &AtomicBool::new(false),
        |_| {},
    )
    .unwrap();
    assert_eq!(result.files.len(), 100);
    assert!(result.files.iter().all(|f| f.output.is_some()));
}
#[test]
fn r01_large_encoded_file_rejected_before_reading() {
    let (dir, _, _) = setup();
    let path = dir.path().join("huge.png");
    let file = std::fs::File::create(&path).unwrap();
    file.set_len(201 * 1024 * 1024).unwrap();
    assert!(load_image(&path)
        .unwrap_err()
        .to_string()
        .contains("200 MiB"));
}
#[test]
fn r01_unsupported_format_rejected() {
    let (dir, _, _) = setup();
    let path = dir.path().join("image.gif");
    std::fs::write(&path, b"GIF89a\x01\0\x01\0\0\0\0").unwrap();
    assert!(load_image(&path)
        .unwrap_err()
        .to_string()
        .contains("仅支持"));
}
#[test]
fn r06_concurrent_exports_do_not_clobber() {
    let (_dir, path, options) = setup();
    let threads: Vec<_> = (0..4)
        .map(|_| {
            let path = path.clone();
            let options = options.clone();
            std::thread::spawn(move || {
                export_one(&path, &spec(), &options, &AtomicBool::new(false)).unwrap()
            })
        })
        .collect();
    let paths: std::collections::HashSet<_> =
        threads.into_iter().map(|t| t.join().unwrap()).collect();
    assert_eq!(paths.len(), 4);
    assert!(paths.iter().all(|p| p.exists()));
}
#[test]
fn r08_oversized_dimensions_rejected() {
    let (dir, path, _) = setup();
    let mut bytes = std::fs::read(path).unwrap();
    bytes[16..20].copy_from_slice(&8000u32.to_be_bytes());
    bytes[20..24].copy_from_slice(&6000u32.to_be_bytes());
    let mut crc = 0xffffffffu32;
    for b in &bytes[12..29] {
        crc ^= u32::from(*b);
        for _ in 0..8 {
            crc = (crc >> 1) ^ if crc & 1 != 0 { 0xedb88320 } else { 0 };
        }
    }
    bytes[29..33].copy_from_slice(&(!crc).to_be_bytes());
    let large = dir.path().join("large.png");
    std::fs::write(&large, bytes).unwrap();
    assert!(load_image(&large)
        .unwrap_err()
        .to_string()
        .contains("4000 万"));
}
