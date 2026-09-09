import {
  useState,
  useEffect,
  useRef,
  useCallback,
  type PointerEvent,
} from "react";
import * as Dialog from "@radix-ui/react-dialog";
import { useVirtualizer } from "@tanstack/react-virtual";
import {
  Plus,
  Download,
  ImagePlus,
  ShieldCheck,
  X,
  FolderOpen,
  Check,
  RotateCcw,
  Type,
  Image as ImageIcon,
  ZoomIn,
  ZoomOut,
  Move,
  LoaderCircle,
  Trash2,
} from "lucide-react";
import { api } from "./api";
import {
  defaultSpec,
  builtinTemplates,
  specError,
  exportError,
  mergeFiles,
  summary,
  type Watermark,
  type Imported,
  type Template,
  type ExportOptions,
  type BatchResult,
} from "./model";
function Range({
  label,
  value,
  min,
  max,
  step = 1,
  unit = "",
  onChange,
}: {
  label: string;
  value: number;
  min: number;
  max: number;
  step?: number;
  unit?: string;
  onChange: (n: number) => void;
}) {
  return (
    <label className="field">
      <span className="range-label">
        {label}
        <output>
          {value}
          {unit}
        </output>
      </span>
      <input
        type="range"
        aria-label={label}
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(e) => onChange(+e.target.value)}
      />
    </label>
  );
}
function Modal({
  open,
  onOpenChange,
  title,
  description,
  children,
}: {
  open: boolean;
  onOpenChange: (v: boolean) => void;
  title: string;
  description: string;
  children: React.ReactNode;
}) {
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="overlay" />
        <Dialog.Content className="modal">
          <div className="modal-head">
            <Dialog.Title>{title}</Dialog.Title>
            <Dialog.Close aria-label="关闭" className="icon">
              <X size={18} />
            </Dialog.Close>
          </div>
          <Dialog.Description className="muted">
            {description}
          </Dialog.Description>
          {children}
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
export default function App() {
  const [files, setFiles] = useState<Imported[]>([]),
    [selected, setSelected] = useState<Set<string>>(new Set()),
    [active, setActive] = useState(""),
    [spec, setSpec] = useState<Watermark>(defaultSpec),
    [templates, setTemplates] = useState<Template[]>([]);
  const [message, setMessage] = useState(""),
    [importing, setImporting] = useState(false),
    [preview, setPreview] = useState(""),
    [previewError, setPreviewError] = useState(""),
    [loading, setLoading] = useState(false),
    [original, setOriginal] = useState(false),
    [zoom, setZoom] = useState(1);
  const [exportOpen, setExportOpen] = useState(false),
    [templateOpen, setTemplateOpen] = useState(false),
    [templateName, setTemplateName] = useState(""),
    [scope, setScope] = useState("all"),
    [options, setOptions] = useState<ExportOptions>({
      directory: "",
      format: "original",
      quality: 90,
      suffix: "_watermarked",
      background: "#ffffff",
    });
  const [busy, setBusy] = useState(false),
    [cancelRequested, setCancelRequested] = useState(false),
    [result, setResult] = useState<BatchResult | null>(null),
    [ready, setReady] = useState(false);
  const request = useRef(0),
    job = useRef(""),
    snapshot = useRef<{ spec: Watermark; options: ExportOptions } | null>(null),
    listRef = useRef<HTMLDivElement>(null),
    importLock = useRef(false),
    filesRef = useRef<Imported[]>([]),
    saveTimer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  filesRef.current = files;
  const current = files.find((f) => f.path === active),
    valid = files.filter((f) => !f.error),
    invalid = specError(spec);
  const virtual = useVirtualizer({
    count: files.length,
    getScrollElement: () => listRef.current,
    estimateSize: () => 78,
    overscan: 6,
    initialRect: { width: 220, height: 400 },
  });
  const notify = useCallback(
    (e: unknown) => setMessage(e instanceof Error ? e.message : String(e)),
    [],
  );
  const change = <K extends keyof Watermark>(k: K, v: Watermark[K]) =>
    setSpec((s) => ({ ...s, [k]: v }));
  const importPaths = useCallback(
    async (paths: string[]) => {
      if (!paths.length) return;
      if (
        new Set([...filesRef.current.map((f) => f.path), ...paths]).size > 1000
      ) {
        notify("每个批次最多 1000 张，请先移除部分图片");
        return;
      }
      if (importLock.current) {
        notify("正在导入，请稍后再试");
        return;
      }
      importLock.current = true;
      setImporting(true);
      try {
        const incoming = await api.importImages(paths);
        setFiles((old) => mergeFiles(old, incoming));
        const good = incoming.filter((f) => !f.error);
        if (good.length) {
          setActive(good[0].path);
          setSelected((old) => new Set([...old, ...good.map((f) => f.path)]));
        }
        const bad = incoming.filter((f) => f.error);
        notify(
          `导入完成：${good.length} 张成功${bad.length ? `，${bad.length} 张失败（查看列表原因）` : ""}`,
        );
      } catch (e) {
        notify(e);
      } finally {
        setImporting(false);
        importLock.current = false;
      }
    },
    [notify],
  );
  const importDialog = async () => {
    try {
      await importPaths(await api.chooseImages());
    } catch (e) {
      notify(e);
    }
  };
  useEffect(() => {
    let disposed = false;
    api
      .load()
      .then((p) => {
        if (!disposed) {
          setTemplates(p.templates);
          if (p.lastSpec) setSpec(p.lastSpec);
        }
      })
      .catch(notify)
      .finally(() => {
        if (!disposed) setReady(true);
      });
    const listeners = [
      api.onDrop((paths) => void importPaths(paths)),
      api.onProgress((e) => {
        if (e.jobId === job.current) setResult(e.result);
      }),
    ];
    return () => {
      disposed = true;
      listeners.forEach((p) => {
        void p.then((off) => off()).catch(() => {});
      });
    };
  }, [importPaths, notify]);
  useEffect(() => {
    if (!ready || invalid) return;
    clearTimeout(saveTimer.current);
    saveTimer.current = setTimeout(() => {
      void api
        .save({ schemaVersion: 1, templates, lastSpec: spec })
        .catch(notify);
    }, 500);
    return () => clearTimeout(saveTimer.current);
  }, [spec, templates, ready, invalid, notify]);
  useEffect(() => {
    const id = ++request.current;
    setPreviewError("");
    if (!current || current.error || (!original && invalid)) {
      setPreview("");
      setLoading(false);
      return;
    }
    if (busy) return;
    setLoading(true);
    const timer = setTimeout(() => {
      api
        .preview(current.path, spec, original, id)
        .then((url) => {
          if (request.current === id) {
            setPreview(url);
            setLoading(false);
          }
        })
        .catch((e) => {
          if (request.current === id) {
            setPreview("");
            setPreviewError(String(e));
            setLoading(false);
          }
        });
    }, 180);
    return () => clearTimeout(timer);
  }, [current, spec, original, invalid, busy]);
  useEffect(() => {
    setZoom(1);
  }, [active]);
  const toggle = (path: string) =>
    setSelected((old) => {
      const next = new Set(old);
      if (next.has(path)) next.delete(path);
      else next.add(path);
      return next;
    });
  const remove = (path: string) => {
    setFiles((old) => {
      const next = old.filter((f) => f.path !== path);
      if (active === path) setActive(next.find((f) => !f.error)?.path ?? "");
      return next;
    });
    setSelected((old) => new Set([...old].filter((p) => p !== path)));
  };
  const chooseLogo = async () => {
    try {
      const paths = await api.chooseImages();
      if (!paths.length) return;
      if (
        new Set([...filesRef.current.map((f) => f.path), ...paths]).size > 1000
      ) {
        notify("每个批次最多 1000 张，请先移除部分图片");
        return;
      }
      const [file] = await api.importImages([paths[0]]);
      if (file.error) throw new Error(file.error);
      setSpec((s) => ({ ...s, kind: "logo", logoPath: file.path }));
    } catch (e) {
      notify(e);
    }
  };
  const openExport = async () => {
    setExportOpen(true);
    if (!options.directory)
      try {
        const directory = await api.defaultOutput();
        setOptions((o) => ({ ...o, directory }));
      } catch (e) {
        notify(e);
      }
  };
  const beginExport = async (
    paths: string[],
    s: Watermark = spec,
    o: ExportOptions = options,
  ) => {
    const error = specError(s) || exportError(o);
    if (error) {
      notify(error);
      return;
    }
    if (!paths.length) {
      notify("请至少选择一张有效图片");
      return;
    }
    if (busy) return;
    setExportOpen(false);
    setBusy(true);
    setCancelRequested(false);
    const id = crypto.randomUUID();
    job.current = id;
    snapshot.current = {
      spec: structuredClone(s),
      options: structuredClone(o),
    };
    setResult({ files: [], cancelled: false, total: paths.length });
    try {
      const r = await api.export(paths, s, o, id);
      setResult(r);
      notify(
        r.cancelled
          ? "导出已取消，已完成的图片已保留"
          : "导出结束，请查看任务结果",
      );
    } catch (e) {
      setResult(null);
      notify(e);
    } finally {
      setBusy(false);
    }
  };
  const retry = () => {
    if (result && snapshot.current)
      void beginExport(
        result.files.filter((f) => f.error).map((f) => f.source),
        snapshot.current.spec,
        snapshot.current.options,
      );
  };
  const counts = result ? summary(result) : null;
  function position(e: PointerEvent<HTMLImageElement>) {
    if (original || spec.tile || busy) return;
    const rect = e.currentTarget.getBoundingClientRect();
    change("x", Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width)));
    change("y", Math.max(0, Math.min(1, (e.clientY - rect.top) / rect.height)));
  }
  const saveTemplate = () => {
    if (!templateName.trim() || templateName.length > 40) {
      notify("模板名称应为 1–40 个字符");
      return;
    }
    if (invalid) {
      notify(invalid);
      return;
    }
    if (templates.length >= 100) {
      notify("最多保存 100 个模板");
      return;
    }
    setTemplates((t) => [
      ...t,
      {
        id: crypto.randomUUID(),
        name: templateName.trim(),
        spec: structuredClone(spec),
      },
    ]);
    setTemplateOpen(false);
    setTemplateName("");
    notify("模板已加入，正在保存");
  };
  return (
    <div className="shell">
      <header>
        <div className="brand">w</div>
        <strong className="wordmark">
          wmark<span>.</span>
        </strong>
        <span className="header-label">水印工作台</span>
        <div className="header-right">
          <span className="local">
            <ShieldCheck size={16} /> 文件仅在本地处理
          </span>
          <button
            onClick={() => void importDialog()}
            disabled={importing || busy}
          >
            <Plus size={16} />
            导入图片
          </button>
        </div>
      </header>
      <div className="workspace">
        <aside>
          <div className="eyebrow">YOUR COLLECTION</div>
          <div className="section-head">
            <h2>图片列表</h2>
            <span className="count">{files.length}</span>
          </div>
          <div className="list-controls">
            <label>
              <input
                type="checkbox"
                aria-label="全选图片"
                checked={
                  valid.length > 0 && valid.every((f) => selected.has(f.path))
                }
                onChange={(e) =>
                  setSelected(
                    e.target.checked
                      ? new Set(valid.map((f) => f.path))
                      : new Set(),
                  )
                }
              />
              全选
            </label>
            <span>已选 {valid.filter((f) => selected.has(f.path)).length}</span>
          </div>
          <div className="file-list" ref={listRef}>
            <div
              style={{ height: virtual.getTotalSize(), position: "relative" }}
            >
              {virtual.getVirtualItems().map((row) => {
                const file = files[row.index];
                return (
                  <div
                    className={`file ${active === file.path ? "active" : ""} ${file.error ? "bad" : ""}`}
                    key={file.path}
                    style={{
                      position: "absolute",
                      top: 0,
                      left: 0,
                      width: "100%",
                      height: 78,
                      transform: `translateY(${row.start}px)`,
                    }}
                  >
                    <input
                      aria-label={`选择 ${file.name}`}
                      type="checkbox"
                      disabled={!!file.error}
                      checked={selected.has(file.path)}
                      onChange={() => toggle(file.path)}
                    />
                    <button
                      className="file-main"
                      onClick={() => setActive(file.path)}
                      title={file.error ?? file.name}
                    >
                      {file.thumbnail ? (
                        <img src={file.thumbnail} alt="" />
                      ) : (
                        <ImageIcon size={32} />
                      )}
                      <span>
                        <strong>{file.name}</strong>
                        <small>
                          {file.error
                            ? "导入失败 · 点击查看"
                            : `${file.width} × ${file.height}`}
                        </small>
                      </span>
                    </button>
                    <button
                      className="remove icon"
                      aria-label={`移除 ${file.name}`}
                      onClick={() => remove(file.path)}
                      disabled={busy}
                    >
                      <X size={14} />
                    </button>
                  </div>
                );
              })}
            </div>
          </div>
          <button
            className="add"
            onClick={() => void importDialog()}
            disabled={importing || busy}
          >
            {importing ? (
              <LoaderCircle className="spin" size={15} />
            ) : (
              <Plus size={15} />
            )}
            添加图片
          </button>
          <div className="aside-note">
            支持 JPG、PNG、静态 WebP
            <br />
            单图最多 4000 万像素
            <br />
            <br />
            原图始终保留，
            <br />
            导出自动移除位置信息。
          </div>
        </aside>
        <main>
          <div className="workspace-head">
            <div>
              <h1>为作品，留下你的印记。</h1>
              <p className="muted">调整一次，让每一张都有你的风格。</p>
            </div>
            <span className="pill">
              {busy ? "正在导出" : `应用于全部 ${valid.length} 张`}
            </span>
          </div>
          <div className={`stage ${!files.length ? "empty" : ""}`}>
            {!current ? (
              <div className="empty-state">
                <div className="empty-icon">
                  <ImagePlus size={38} />
                </div>
                <h2>把图片拖到这里</h2>
                <p>添加照片、证件或作品，开始制作水印</p>
                <button
                  className="primary"
                  onClick={() => void importDialog()}
                  disabled={importing}
                >
                  {importing ? "正在读取图片…" : "选择图片"}
                </button>
                <small>JPG / PNG / WebP · 全程离线</small>
              </div>
            ) : current.error ? (
              <div className="empty-state error">
                <h2>这张图片无法导入</h2>
                <p>{current.error}</p>
                <button onClick={() => void importPaths([current.path])}>
                  重新读取
                </button>
              </div>
            ) : invalid && !original ? (
              <div className="empty-state">
                <p>{invalid}</p>
              </div>
            ) : previewError ? (
              <div className="empty-state error">
                <p>{previewError}</p>
                <button onClick={() => setSpec((s) => ({ ...s }))}>
                  重试预览
                </button>
              </div>
            ) : (
              <>
                <div className="image-area">
                  {preview && (
                    <img
                      className="preview"
                      src={preview}
                      alt="水印效果预览"
                      style={{
                        maxWidth: zoom === 1 ? "100%" : "none",
                        maxHeight: zoom === 1 ? "100%" : "none",
                        width: zoom === 1 ? "auto" : `${zoom * 100}%`,
                      }}
                      tabIndex={0}
                      draggable={false}
                      onPointerDown={(e) => {
                        e.currentTarget.setPointerCapture(e.pointerId);
                        position(e);
                      }}
                      onPointerMove={(e) => {
                        if (e.buttons === 1) position(e);
                      }}
                      onKeyDown={(e) => {
                        if (spec.tile || original) return;
                        if (
                          [
                            "ArrowLeft",
                            "ArrowRight",
                            "ArrowUp",
                            "ArrowDown",
                          ].includes(e.key)
                        ) {
                          e.preventDefault();
                          const step = e.shiftKey ? 0.05 : 0.01;
                          setSpec((s) => ({
                            ...s,
                            x: Math.max(
                              0,
                              Math.min(
                                1,
                                s.x +
                                  (e.key === "ArrowRight"
                                    ? step
                                    : e.key === "ArrowLeft"
                                      ? -step
                                      : 0),
                              ),
                            ),
                            y: Math.max(
                              0,
                              Math.min(
                                1,
                                s.y +
                                  (e.key === "ArrowDown"
                                    ? step
                                    : e.key === "ArrowUp"
                                      ? -step
                                      : 0),
                              ),
                            ),
                          }));
                        }
                      }}
                    />
                  )}
                </div>
                {loading && (
                  <span className="preview-loading">
                    <LoaderCircle className="spin" size={16} />
                    更新预览
                  </span>
                )}
              </>
            )}
          </div>
          <div className="canvas-info">
            <span>
              {current && !current.error
                ? `${current.width} × ${current.height} px`
                : "未选择图片"}
            </span>
            <button
              disabled={!current || !!current.error || busy}
              onPointerDown={(e) => {
                e.currentTarget.setPointerCapture(e.pointerId);
                setOriginal(true);
              }}
              onPointerUp={() => setOriginal(false)}
              onPointerCancel={() => setOriginal(false)}
              onKeyDown={(e) => {
                if (e.key === " " || e.key === "Enter") {
                  e.preventDefault();
                  setOriginal(true);
                }
              }}
              onKeyUp={() => setOriginal(false)}
              onBlur={() => setOriginal(false)}
            >
              按住查看原图
            </button>
            <div className="zoom">
              <button
                aria-label="缩小"
                onClick={() => setZoom((z) => Math.max(1, z - 0.25))}
              >
                <ZoomOut size={16} />
              </button>
              <button onClick={() => setZoom(1)}>
                {zoom === 1 ? "自适应" : `${Math.round(zoom * 100)}%`}
              </button>
              <button
                aria-label="放大"
                onClick={() => setZoom((z) => Math.min(3, z + 0.25))}
              >
                <ZoomIn size={16} />
              </button>
            </div>
          </div>
          <div className="presets">
            <span>快捷模板</span>
            {builtinTemplates.map((t) => (
              <button key={t.id} onClick={() => setSpec({ ...t.spec })}>
                {t.name}
              </button>
            ))}
            <button onClick={() => setTemplateOpen(true)}>
              <Plus size={14} />
              保存模板
            </button>
          </div>
          {templates.length > 0 && (
            <div className="saved-templates">
              <label>
                我的模板
                <select
                  aria-label="选择已保存模板"
                  value=""
                  onChange={(e) => {
                    const t = templates.find((t) => t.id === e.target.value);
                    if (t) setSpec({ ...t.spec });
                  }}
                >
                  <option value="">选择模板…</option>
                  {templates.map((t) => (
                    <option key={t.id} value={t.id}>
                      {t.name}
                    </option>
                  ))}
                </select>
              </label>
              <button onClick={() => setTemplateOpen(true)}>管理模板</button>
            </div>
          )}
          {result && (
            <section className="job" aria-label="导出任务">
              <div className="section-head">
                <h2>
                  {busy
                    ? "正在导出"
                    : result.cancelled
                      ? "已取消"
                      : counts?.failed
                        ? "导出结束 · 存在失败"
                        : "导出完成"}
                </h2>
                {busy ? (
                  <button
                    onClick={() => {
                      setCancelRequested(true);
                      void api.cancel().catch(notify);
                    }}
                    disabled={cancelRequested}
                  >
                    {cancelRequested ? "正在取消…" : "取消任务"}
                  </button>
                ) : (
                  <button
                    className="icon"
                    aria-label="关闭任务结果"
                    onClick={() => setResult(null)}
                  >
                    <X size={16} />
                  </button>
                )}
              </div>
              <progress value={result.files.length} max={result.total} />
              <p>
                成功 {counts?.success} · 失败 {counts?.failed} ·{" "}
                {busy ? "待处理" : "未处理"} {counts?.pending} / 共{" "}
                {result.total} 张
              </p>
              {result.files.some((f) => f.error) && (
                <details>
                  <summary>查看失败原因</summary>
                  <ul>
                    {result.files
                      .filter((f) => f.error)
                      .map((f) => (
                        <li key={f.source}>
                          {f.source.split(/[/\\]/).pop()}：{f.error}
                        </li>
                      ))}
                  </ul>
                </details>
              )}
              {!busy && (
                <div className="row">
                  <button
                    onClick={() =>
                      void api
                        .openOutput(
                          snapshot.current?.options.directory ??
                            options.directory,
                        )
                        .catch(notify)
                    }
                  >
                    <FolderOpen size={16} />
                    打开输出文件夹
                  </button>
                  {!!counts?.failed && (
                    <button onClick={retry}>
                      <RotateCcw size={16} />
                      仅重试失败项
                    </button>
                  )}
                </div>
              )}
            </section>
          )}
        </main>
        <section className="settings">
          <div className="section-head">
            <h2>水印设置</h2>
            <button
              className="icon"
              aria-label="重置水印"
              onClick={() => setSpec({ ...defaultSpec })}
            >
              <RotateCcw size={16} />
            </button>
          </div>
          <div className="seg">
            <button
              aria-pressed={spec.kind === "text"}
              className={spec.kind === "text" ? "selected" : ""}
              onClick={() => change("kind", "text")}
            >
              <Type size={16} />
              文字水印
            </button>
            <button
              aria-pressed={spec.kind === "logo"}
              className={spec.kind === "logo" ? "selected" : ""}
              onClick={() => change("kind", "logo")}
            >
              <ImageIcon size={16} />
              图片水印
            </button>
          </div>
          {spec.kind === "text" ? (
            <>
              <label className="field">
                水印内容
                <input
                  value={spec.text}
                  maxLength={100}
                  onChange={(e) => change("text", e.target.value)}
                />
              </label>
              <label className="field">
                字体
                <select
                  value={spec.bold ? "bold" : "regular"}
                  onChange={(e) => change("bold", e.target.value === "bold")}
                >
                  <option value="regular">思源黑体 · 常规</option>
                  <option value="bold">思源黑体 · 加粗</option>
                </select>
              </label>
            </>
          ) : (
            <div className="field">
              <button onClick={() => void chooseLogo()} disabled={busy}>
                <ImagePlus size={16} />
                {spec.logoPath ? "更换图片水印" : "上传图片水印"}
              </button>
              <p className="help">
                {spec.logoPath?.split(/[/\\]/).pop() ??
                  "建议使用带透明背景的 PNG / WebP"}
              </p>
            </div>
          )}
          <Range
            label={spec.kind === "logo" ? "图片大小" : "文字大小"}
            value={spec.size}
            min={1}
            max={30}
            step={0.5}
            unit="%"
            onChange={(v) => change("size", v)}
          />
          <Range
            label="不透明度"
            value={spec.opacity}
            min={0}
            max={100}
            unit="%"
            onChange={(v) => change("opacity", v)}
          />
          {spec.kind === "text" && (
            <label className="color-field">
              文字颜色
              <input
                type="color"
                value={spec.color}
                onChange={(e) => change("color", e.target.value)}
              />
              <span>{spec.color.toUpperCase()}</span>
            </label>
          )}
          <div className="line" />
          <label className="switchrow">
            全图平铺
            <input
              type="checkbox"
              checked={spec.tile}
              onChange={(e) => change("tile", e.target.checked)}
            />
          </label>
          <Range
            label="旋转角度"
            value={spec.angle}
            min={-90}
            max={90}
            unit="°"
            onChange={(v) => change("angle", v)}
          />
          {spec.tile ? (
            <Range
              label="平铺间距"
              value={spec.spacing}
              min={1}
              max={50}
              unit="%"
              onChange={(v) => change("spacing", v)}
            />
          ) : (
            <>
              <span className="label">水印位置</span>
              <div className="grid9">
                {[
                  "左上",
                  "中上",
                  "右上",
                  "左中",
                  "居中",
                  "右中",
                  "左下",
                  "中下",
                  "右下",
                ].map((name, i) => (
                  <button
                    key={name}
                    aria-label={name}
                    aria-pressed={
                      spec.x === (i % 3) / 2 && spec.y === Math.floor(i / 3) / 2
                    }
                    onClick={() =>
                      setSpec((s) => ({
                        ...s,
                        x: (i % 3) / 2,
                        y: Math.floor(i / 3) / 2,
                      }))
                    }
                  >
                    {["↖", "↑", "↗", "←", "·", "→", "↙", "↓", "↘"][i]}
                  </button>
                ))}
              </div>
              <Range
                label="安全边距"
                value={spec.margin}
                min={0}
                max={20}
                unit="%"
                onChange={(v) => change("margin", v)}
              />
            </>
          )}
          <p className="help">
            <Move size={13} />{" "}
            单点水印可在预览中拖动，方向键微调。过长文字自动缩小以保留边距。
          </p>
          {invalid && (
            <p className="error" role="status">
              {invalid}
            </p>
          )}
          <button
            className="primary export"
            disabled={!valid.length || !!invalid || busy || importing}
            onClick={() => void openExport()}
          >
            <Download size={17} />
            {busy ? "正在导出…" : "导出图片"}
          </button>
          <p className="help center">保留原始尺寸 · 不覆盖原文件</p>
        </section>
      </div>
      <footer>
        <span>WMARK / v0.1.0</span>
        <span>
          {importing
            ? "正在读取文件…"
            : busy
              ? "任务使用开始时的水印设置"
              : "本地处理 · Mac + Windows"}
        </span>
      </footer>
      {message && (
        <div className="notification" role="status">
          <span>{message}</span>
          <button aria-label="关闭提示" onClick={() => setMessage("")}>
            <X size={16} />
          </button>
        </div>
      )}
      <Modal
        open={exportOpen}
        onOpenChange={setExportOpen}
        title="导出图片"
        description="输出为新文件，重名自动编号。原始图片保持不变。"
      >
        <label className="field">
          导出范围
          <select value={scope} onChange={(e) => setScope(e.target.value)}>
            <option value="all">全部有效图片（{valid.length} 张）</option>
            <option value="selected">
              已选图片（{valid.filter((f) => selected.has(f.path)).length} 张）
            </option>
          </select>
        </label>
        <label className="field">
          输出文件夹
          <div className="directory">
            <input aria-label="输出文件夹" value={options.directory} readOnly />
            <button
              onClick={() => {
                void api
                  .chooseDirectory()
                  .then((directory) => {
                    if (directory) setOptions((o) => ({ ...o, directory }));
                  })
                  .catch(notify);
              }}
            >
              <FolderOpen size={16} />
              选择
            </button>
          </div>
        </label>
        <div className="row">
          <label className="field">
            格式
            <select
              value={options.format}
              onChange={(e) =>
                setOptions((o) => ({
                  ...o,
                  format: e.target.value as ExportOptions["format"],
                }))
              }
            >
              <option value="original">保持原格式</option>
              <option value="png">PNG · 无损</option>
              <option value="jpg">JPG</option>
              <option value="webp">WebP · 无损</option>
            </select>
          </label>
          <label className="field">
            文件后缀
            <input
              value={options.suffix}
              onChange={(e) =>
                setOptions((o) => ({ ...o, suffix: e.target.value }))
              }
              maxLength={40}
            />
          </label>
        </div>
        {(options.format === "jpg" || options.format === "original") && (
          <>
            <Range
              label="JPG 质量"
              value={options.quality}
              min={1}
              max={100}
              onChange={(v) => setOptions((o) => ({ ...o, quality: v }))}
            />
            <label className="color-field">
              JPG 背景色
              <input
                type="color"
                value={options.background}
                onChange={(e) =>
                  setOptions((o) => ({ ...o, background: e.target.value }))
                }
              />
            </label>
          </>
        )}
        <div className="privacy">
          <ShieldCheck size={18} />
          <span>
            自动移除 EXIF / GPS 等原始元数据。输出按 sRGB 处理，透明 JPG
            使用所选背景。
          </span>
        </div>
        {exportError(options) && (
          <p className="error">{exportError(options)}</p>
        )}
        <div className="modal-actions">
          <button onClick={() => setExportOpen(false)}>取消</button>
          <button
            className="primary"
            disabled={
              !!exportError(options) ||
              busy ||
              (scope === "selected" && !valid.some((f) => selected.has(f.path)))
            }
            onClick={() =>
              void beginExport(
                valid
                  .filter((f) => scope === "all" || selected.has(f.path))
                  .map((f) => f.path),
              )
            }
          >
            <Download size={16} />
            开始导出
          </button>
        </div>
      </Modal>
      <Modal
        open={templateOpen}
        onOpenChange={setTemplateOpen}
        title="我的水印模板"
        description="保存当前设置，下次打开直接使用。Logo 模板仍需保留原图片文件。"
      >
        <label className="field">
          模板名称
          <input
            value={templateName}
            maxLength={40}
            placeholder="例如：作品署名"
            onChange={(e) => setTemplateName(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") saveTemplate();
            }}
          />
        </label>
        <button
          className="primary"
          disabled={!!invalid || !templateName.trim()}
          onClick={saveTemplate}
        >
          <Check size={16} />
          保存当前设置
        </button>
        <div className="template-list">
          {templates.map((t) => (
            <div key={t.id}>
              <button
                onClick={() => {
                  setSpec({ ...t.spec });
                  setTemplateOpen(false);
                }}
              >
                {t.name}
              </button>
              <button
                aria-label={`删除模板 ${t.name}`}
                onClick={() =>
                  setTemplates((old) => old.filter((v) => v.id !== t.id))
                }
              >
                <Trash2 size={16} />
              </button>
            </div>
          ))}
        </div>
      </Modal>
    </div>
  );
}
