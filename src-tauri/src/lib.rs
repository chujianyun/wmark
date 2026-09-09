use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
};
use tauri::{Emitter, Manager, State};
use wmark_core::{BatchResult, ExportOptions, Watermark};
type Reply<T> = Result<T, String>;
#[derive(Default)]
struct Inner {
    paths: Mutex<HashSet<PathBuf>>,
    directories: Mutex<HashSet<PathBuf>>,
    work: Mutex<()>,
    busy: AtomicBool,
    preview_generation: AtomicU64,
    cancel: AtomicBool,
}
#[derive(Default, Clone)]
struct Backend(Arc<Inner>);
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Imported {
    path: PathBuf,
    name: String,
    width: u32,
    height: u32,
    thumbnail: String,
    error: Option<String>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Template {
    id: String,
    name: String,
    spec: Watermark,
}
#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Preferences {
    schema_version: u8,
    templates: Vec<Template>,
    last_spec: Option<Watermark>,
}
fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}
fn authorized(state: &Backend, path: &Path) -> Reply<PathBuf> {
    if !path.exists() && state.0.paths.lock().map_err(err)?.contains(path) {
        return Ok(path.to_path_buf());
    }
    let p = path.canonicalize().map_err(err)?;
    if !state.0.paths.lock().map_err(err)?.contains(&p) {
        return Err("文件尚未导入，请重新选择".into());
    }
    Ok(p)
}
fn authorize_spec(state: &Backend, spec: &Watermark) -> Reply<()> {
    spec.validate().map_err(err)?;
    if spec.kind == "logo" {
        if let Some(p) = &spec.logo_path {
            authorized(state, p)?;
        }
    }
    Ok(())
}
#[tauri::command]
async fn import_images(paths: Vec<PathBuf>, state: State<'_, Backend>) -> Reply<Vec<Imported>> {
    if paths.len() > 1000 {
        return Err("每次最多导入 1000 张图片".into());
    }
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = state.0.work.lock().map_err(err)?;
        let mut out = Vec::new();
        for path in paths {
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            let result = (|| {
                let p = path.canonicalize().map_err(err)?;
                let im = wmark_core::load_image(&p).map_err(err)?;
                let thumb = wmark_core::imaging::data_url(&im.thumbnail(96, 96)).map_err(err)?;
                state.0.paths.lock().map_err(err)?.insert(p.clone());
                Ok::<_, String>((p, im.width(), im.height(), thumb))
            })();
            match result {
                Ok((path, width, height, thumbnail)) => out.push(Imported {
                    path,
                    name,
                    width,
                    height,
                    thumbnail,
                    error: None,
                }),
                Err(error) => out.push(Imported {
                    path,
                    name,
                    width: 0,
                    height: 0,
                    thumbnail: String::new(),
                    error: Some(error),
                }),
            }
        }
        Ok(out)
    })
    .await
    .map_err(err)?
}
#[tauri::command]
async fn preview_image(
    path: PathBuf,
    spec: Watermark,
    original: bool,
    request_id: u64,
    app: tauri::AppHandle,
    state: State<'_, Backend>,
) -> Reply<String> {
    let path = authorized(&state, &path)?;
    if !original {
        authorize_spec(&state, &spec)?;
    }
    let state = state.inner().clone();
    let generation = state.0.preview_generation.fetch_add(1, Ordering::SeqCst) + 1;
    let cache = app.path().app_cache_dir().map_err(err)?;
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = state.0.work.lock().map_err(err)?;
        if state.0.preview_generation.load(Ordering::SeqCst) != generation {
            return Err("过期预览已取消".into());
        }
        std::fs::create_dir_all(&cache).map_err(err)?;
        let im = wmark_core::load_image(&path).map_err(err)?;
        let rendered = if original {
            im
        } else {
            wmark_core::render(&im, &spec).map_err(err)?
        };
        let name = cache.join(format!("preview-{}.png", request_id % 4));
        std::fs::write(
            &name,
            wmark_core::png_bytes(&rendered.thumbnail(1600, 1600)).map_err(err)?,
        )
        .map_err(err)?;
        Ok(name.to_string_lossy().into_owned())
    })
    .await
    .map_err(err)?
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Progress {
    job_id: String,
    result: BatchResult,
}
#[tauri::command]
async fn export_images(
    paths: Vec<PathBuf>,
    spec: Watermark,
    options: ExportOptions,
    job_id: String,
    app: tauri::AppHandle,
    state: State<'_, Backend>,
) -> Reply<BatchResult> {
    if paths.is_empty() || paths.len() > 1000 {
        return Err("请选择 1–1000 张图片导出".into());
    }
    authorize_spec(&state, &spec)?;
    options.validate().map_err(err)?;
    let paths = paths
        .iter()
        .map(|p| authorized(&state, p))
        .collect::<Reply<Vec<_>>>()?;
    let state = state.inner().clone();
    if state.0.busy.swap(true, Ordering::SeqCst) {
        return Err("已有导出任务正在运行".into());
    }
    state.0.cancel.store(false, Ordering::SeqCst);
    let cleanup = state.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let _guard = state.0.work.lock().map_err(err)?;
        state
            .0
            .directories
            .lock()
            .map_err(err)?
            .insert(options.directory.canonicalize().map_err(err)?);
        wmark_core::run_batch(&paths, &spec, &options, &state.0.cancel, |result| {
            let _ = app.emit(
                "export-progress",
                Progress {
                    job_id: job_id.clone(),
                    result: result.clone(),
                },
            );
        })
        .map_err(err)
    })
    .await
    .map_err(err)
    .and_then(|r| r);
    cleanup.0.busy.store(false, Ordering::SeqCst);
    result
}
#[tauri::command]
fn cancel_export(state: State<'_, Backend>) {
    state.0.cancel.store(true, Ordering::SeqCst);
}
#[tauri::command]
fn open_output(directory: PathBuf, state: State<'_, Backend>) -> Reply<()> {
    let path = directory.canonicalize().map_err(err)?;
    if !state.0.directories.lock().map_err(err)?.contains(&path) {
        return Err("尚未向该文件夹导出".into());
    }
    open::that(path).map_err(err)
}
#[tauri::command]
fn default_output(app: tauri::AppHandle) -> Reply<String> {
    let path = app
        .path()
        .picture_dir()
        .or_else(|_| app.path().document_dir())
        .map_err(err)?
        .join("wmark-output");
    std::fs::create_dir_all(&path).map_err(err)?;
    Ok(path.to_string_lossy().into_owned())
}
fn pref_path(app: &tauri::AppHandle) -> Reply<PathBuf> {
    let dir = app.path().app_config_dir().map_err(err)?;
    std::fs::create_dir_all(&dir).map_err(err)?;
    Ok(dir.join("preferences.json"))
}
#[tauri::command]
fn load_preferences(app: tauri::AppHandle, state: State<'_, Backend>) -> Reply<Preferences> {
    let path = pref_path(&app)?;
    if !path.exists() {
        return Ok(Preferences {
            schema_version: 1,
            ..Default::default()
        });
    }
    let bytes = std::fs::read(path).map_err(err)?;
    if bytes.len() > 1024 * 1024 {
        return Err("配置文件过大".into());
    }
    let prefs: Preferences =
        serde_json::from_slice(&bytes).map_err(|_| "设置文件损坏，请重新保存模板".to_string())?;
    validate_prefs(&prefs)?;
    for spec in prefs
        .templates
        .iter()
        .map(|t| &t.spec)
        .chain(prefs.last_spec.iter())
    {
        if let Some(p) = &spec.logo_path {
            if let Ok(p) = p.canonicalize() {
                state.0.paths.lock().map_err(err)?.insert(p);
            }
        }
    }
    Ok(prefs)
}
fn validate_prefs(p: &Preferences) -> Reply<()> {
    if p.schema_version != 1 || p.templates.len() > 100 {
        return Err("不支持的配置版本或模板过多".into());
    }
    for t in &p.templates {
        if t.name.trim().is_empty() || t.name.chars().count() > 40 || t.id.len() > 100 {
            return Err("模板名称或标识无效".into());
        }
        t.spec.validate().map_err(err)?;
    }
    if let Some(s) = &p.last_spec {
        s.validate().map_err(err)?;
    }
    Ok(())
}
#[tauri::command]
fn save_preferences(
    preferences: Preferences,
    app: tauri::AppHandle,
    state: State<'_, Backend>,
) -> Reply<()> {
    validate_prefs(&preferences)?;
    for s in preferences
        .templates
        .iter()
        .map(|t| &t.spec)
        .chain(preferences.last_spec.iter())
    {
        authorize_spec(&state, s)?;
    }
    let path = pref_path(&app)?;
    if path.exists() {
        let old = std::fs::read(&path).map_err(err)?;
        let valid = serde_json::from_slice::<Preferences>(&old)
            .ok()
            .is_some_and(|p| validate_prefs(&p).is_ok());
        if !valid {
            std::fs::copy(
                &path,
                path.with_file_name(format!(
                    "preferences-recovery-{}.json",
                    uuid::Uuid::new_v4()
                )),
            )
            .map_err(err)?;
        }
    }
    let mut temp = tempfile::NamedTempFile::new_in(path.parent().unwrap()).map_err(err)?;
    use std::io::Write;
    temp.write_all(&serde_json::to_vec_pretty(&preferences).map_err(err)?)
        .map_err(err)?;
    temp.as_file().sync_all().map_err(err)?;
    temp.persist(path).map_err(err)?;
    Ok(())
}
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Backend::default())
        .invoke_handler(tauri::generate_handler![
            import_images,
            preview_image,
            export_images,
            cancel_export,
            open_output,
            default_output,
            load_preferences,
            save_preferences
        ])
        .run(tauri::generate_context!())
        .expect("Unable to start Wmark");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unknown_path_is_denied() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("x.png");
        std::fs::write(&p, b"test").unwrap();
        assert!(authorized(&Backend::default(), &p).is_err());
    }
    #[test]
    fn missing_imported_path_reaches_per_file_error_handling() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("gone.png");
        let state = Backend::default();
        state.0.paths.lock().unwrap().insert(p.clone());
        assert_eq!(authorized(&state, &p).unwrap(), p);
    }
    #[test]
    fn invalid_preference_version_rejected() {
        assert!(validate_prefs(&Preferences {
            schema_version: 99,
            ..Default::default()
        })
        .is_err());
    }
    #[test]
    fn malformed_template_rejected() {
        let p = Preferences {
            schema_version: 1,
            templates: vec![Template {
                id: "id".into(),
                name: "".into(),
                spec: Watermark::default(),
            }],
            last_spec: None,
        };
        assert!(validate_prefs(&p).is_err());
    }
    #[test]
    fn preference_roundtrip_preserves_spec() {
        let p = Preferences {
            schema_version: 1,
            templates: vec![Template {
                id: "id".into(),
                name: "署名".into(),
                spec: Watermark::default(),
            }],
            last_spec: Some(Watermark::default()),
        };
        let bytes = serde_json::to_vec(&p).unwrap();
        let decoded: Preferences = serde_json::from_slice(&bytes).unwrap();
        validate_prefs(&decoded).unwrap();
        assert_eq!(decoded.templates[0].spec, Watermark::default());
    }
}
