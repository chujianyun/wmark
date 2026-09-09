use serde::{Deserialize, Serialize};
use std::path::PathBuf;
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct Error(pub String);
pub type Result<T> = std::result::Result<T, Error>;
impl From<image::ImageError> for Error {
    fn from(e: image::ImageError) -> Self {
        Self(format!("图片处理失败：{e}"))
    }
}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self(format!("文件读写失败：{e}"))
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Watermark {
    pub kind: String,
    pub text: String,
    pub bold: bool,
    pub color: String,
    pub size: f32,
    pub opacity: f32,
    pub angle: f32,
    pub tile: bool,
    pub spacing: f32,
    pub x: f32,
    pub y: f32,
    pub margin: f32,
    pub logo_path: Option<PathBuf>,
}
impl Default for Watermark {
    fn default() -> Self {
        Self {
            kind: "text".into(),
            text: "© WMARK PHOTOGRAPHY".into(),
            bold: false,
            color: "#ffffff".into(),
            size: 3.,
            opacity: 80.,
            angle: 0.,
            tile: false,
            spacing: 12.,
            x: 1.,
            y: 1.,
            margin: 5.,
            logo_path: None,
        }
    }
}
impl Watermark {
    pub fn validate(&self) -> Result<()> {
        for (name, v, min, max) in [
            ("大小", self.size, 1., 30.),
            ("不透明度", self.opacity, 0., 100.),
            ("角度", self.angle, -90., 90.),
            ("间距", self.spacing, 1., 100.),
            ("横坐标", self.x, 0., 1.),
            ("纵坐标", self.y, 0., 1.),
            ("边距", self.margin, 0., 20.),
        ] {
            if !v.is_finite() || !(min..=max).contains(&v) {
                return Err(Error(format!("{name}超出允许范围")));
            }
        }
        parse_color(&self.color)?;
        match self.kind.as_str() {
            "text" => {
                if self.text.trim().is_empty()
                    || self.text.chars().count() > 100
                    || self.text.chars().any(char::is_control)
                {
                    return Err(Error("文字应为 1–100 个字符，不支持换行或控制字符".into()));
                }
            }
            "logo" => {
                if self.logo_path.is_none() {
                    return Err(Error("请先选择图片水印".into()));
                }
            }
            _ => return Err(Error("未知水印类型".into())),
        };
        Ok(())
    }
}
pub fn parse_color(s: &str) -> Result<[u8; 3]> {
    if s.len() != 7 || !s.starts_with('#') || !s[1..].bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(Error("颜色必须是 #RRGGBB".into()));
    }
    Ok([
        u8::from_str_radix(&s[1..3], 16).unwrap(),
        u8::from_str_radix(&s[3..5], 16).unwrap(),
        u8::from_str_radix(&s[5..7], 16).unwrap(),
    ])
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExportOptions {
    pub directory: PathBuf,
    pub format: String,
    pub quality: u8,
    pub suffix: String,
    pub background: String,
}
impl ExportOptions {
    pub fn validate(&self) -> Result<()> {
        if !["original", "png", "jpg", "webp"].contains(&self.format.as_str()) {
            return Err(Error("不支持的输出格式".into()));
        }
        if !(1..=100).contains(&self.quality) {
            return Err(Error("JPG 质量应为 1–100".into()));
        }
        if self.suffix.chars().count() > 40
            || self
                .suffix
                .chars()
                .any(|c| c.is_control() || "<>:\"/\\|?*".contains(c))
            || self.suffix.ends_with([' ', '.'])
        {
            return Err(Error("文件后缀包含无效字符".into()));
        }
        parse_color(&self.background)?;
        if !self.directory.is_dir() {
            return Err(Error("请选择存在的输出文件夹".into()));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileResult {
    pub source: PathBuf,
    pub output: Option<PathBuf>,
    pub error: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BatchResult {
    pub files: Vec<FileResult>,
    pub cancelled: bool,
    pub total: usize,
}
