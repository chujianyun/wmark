export interface Watermark {
  kind: "text" | "logo";
  text: string;
  bold: boolean;
  color: string;
  size: number;
  opacity: number;
  angle: number;
  tile: boolean;
  spacing: number;
  x: number;
  y: number;
  margin: number;
  logoPath: string | null;
}
export const defaultSpec: Watermark = {
  kind: "text",
  text: "© WMARK PHOTOGRAPHY",
  bold: false,
  color: "#ffffff",
  size: 3,
  opacity: 80,
  angle: 0,
  tile: false,
  spacing: 12,
  x: 1,
  y: 1,
  margin: 5,
  logoPath: null,
};
export interface Imported {
  path: string;
  name: string;
  width: number;
  height: number;
  thumbnail: string;
  error: string | null;
}
export interface FileResult {
  source: string;
  output: string | null;
  error: string | null;
}
export interface BatchResult {
  files: FileResult[];
  cancelled: boolean;
  total: number;
}
export interface ExportOptions {
  directory: string;
  format: "original" | "jpg" | "png" | "webp";
  quality: number;
  suffix: string;
  background: string;
}
export interface Template {
  id: string;
  name: string;
  spec: Watermark;
}
export interface Preferences {
  schemaVersion: number;
  templates: Template[];
  lastSpec: Watermark | null;
}
export interface Progress {
  jobId: string;
  result: BatchResult;
}
export const builtinTemplates: Template[] = [
  { id: "photo", name: "摄影署名", spec: { ...defaultSpec } },
  {
    id: "private",
    name: "证件保护",
    spec: {
      ...defaultSpec,
      text: "仅供业务办理使用 · 再次复印无效",
      tile: true,
      angle: -30,
      opacity: 28,
      size: 2.5,
    },
  },
  {
    id: "sample",
    name: "样片预览",
    spec: {
      ...defaultSpec,
      text: "SAMPLE · 样片预览",
      tile: true,
      angle: -30,
      opacity: 35,
    },
  },
];
export function specError(s: Watermark): string | null {
  if (
    s.kind === "text" &&
    (!s.text.trim() ||
      [...s.text].length > 100 ||
      /[\x00-\x1f\x7f]/.test(s.text))
  )
    return "请输入 1–100 个字符，不支持换行";
  if (s.kind === "logo" && !s.logoPath) return "请先上传图片水印";
  return null;
}
export function exportError(o: ExportOptions): string | null {
  if (!o.directory.trim()) return "请选择输出文件夹";
  if (
    /[<>:"/\\|?*\x00-\x1f]/.test(o.suffix) ||
    /[ .]$/.test(o.suffix) ||
    [...o.suffix].length > 40
  )
    return "后缀包含无效字符，或超过 40 个字符";
  return null;
}
export function mergeFiles(old: Imported[], incoming: Imported[]): Imported[] {
  const map = new Map(old.map((f) => [f.path, f]));
  for (const f of incoming) map.set(f.path, f);
  return [...map.values()];
}
export function summary(r: BatchResult) {
  const success = r.files.filter((f) => f.output).length;
  const failed = r.files.filter((f) => f.error).length;
  return { success, failed, pending: r.total - r.files.length };
}
