use wmark_core::*;
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let input = load_image(std::path::Path::new(&args[1])).unwrap();
    let output = load_image(std::path::Path::new(&args[2])).unwrap();
    let spec = Watermark {
        text: "仅供业务办理使用 · 再次复印无效".into(),
        size: 2.5,
        opacity: 28.,
        angle: -30.,
        tile: true,
        ..Default::default()
    };
    let expected = render(&input, &spec).unwrap();
    assert_eq!(expected.to_rgba8(), output.to_rgba8());
    println!(
        "Desktop output matches Rust rendering exactly: {}x{}",
        output.width(),
        output.height()
    );
}
