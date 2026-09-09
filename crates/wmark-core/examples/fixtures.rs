use image::{Rgba, RgbaImage};
fn main() {
    let dir = std::path::PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "artifacts/fixtures".into()),
    );
    std::fs::create_dir_all(&dir).unwrap();
    for (name, w, h) in [("landscape.png", 1400, 1050), ("portrait.png", 900, 1400)] {
        let mut im = RgbaImage::new(w, h);
        for (x, y, p) in im.enumerate_pixels_mut() {
            let hill = 0.5 + 0.09 * (x as f32 / 180.).sin();
            let t = y as f32 / h as f32;
            *p = if t > hill {
                Rgba([
                    (35. + t * 30.) as u8,
                    (88. + t * 30.) as u8,
                    (68. + t * 25.) as u8,
                    255,
                ])
            } else {
                Rgba([
                    (185. + t * 40.) as u8,
                    (205. + t * 20.) as u8,
                    (188. + t * 25.) as u8,
                    255,
                ])
            };
        }
        im.save(dir.join(name)).unwrap();
    }
    let mut logo = RgbaImage::new(160, 160);
    for (x, y, p) in logo.enumerate_pixels_mut() {
        if (30..130).contains(&x) && (30..130).contains(&y) {
            *p = Rgba([255, 210, 50, 180]);
        }
    }
    logo.save(dir.join("logo.png")).unwrap();
    std::fs::write(dir.join("broken.png"), b"broken fixture").unwrap();
    println!("Generated synthetic fixtures in {}", dir.display());
}
