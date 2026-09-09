use image::{DynamicImage, Rgba, RgbaImage};
use std::{sync::atomic::AtomicBool, time::Instant};
use wmark_core::*;
fn main() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("24mp.png");
    let start = Instant::now();
    DynamicImage::ImageRgba8(RgbaImage::from_pixel(6000, 4000, Rgba([24, 76, 58, 255])))
        .save(&input)
        .unwrap();
    let spec = Watermark {
        text: "中文水印 © Wmark".into(),
        tile: true,
        angle: -30.,
        ..Default::default()
    };
    let options = ExportOptions {
        directory: dir.path().into(),
        format: "jpg".into(),
        quality: 90,
        suffix: "_out".into(),
        background: "#ffffff".into(),
    };
    let process = Instant::now();
    let output = export_one(&input, &spec, &options, &AtomicBool::new(false)).unwrap();
    let duration = process.elapsed();
    let im = load_image(&output).unwrap();
    assert_eq!((im.width(), im.height()), (6000, 4000));
    println!(
        "24MP PNG → tiled Chinese watermark → JPG: {:.3}s; output={} bytes; total {:.3}s",
        duration.as_secs_f64(),
        std::fs::metadata(output).unwrap().len(),
        start.elapsed().as_secs_f64()
    );
}
