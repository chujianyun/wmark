import { invoke, convertFileSrc, isTauri } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import type {
  Imported,
  Watermark,
  ExportOptions,
  BatchResult,
  Preferences,
  Progress,
} from "./model";
function call<T>(name: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri())
    return Promise.reject(
      new Error("请在 Wmark 桌面应用中使用文件功能（npm run tauri dev）"),
    );
  return invoke(name, args);
}
export const api = {
  importImages: (paths: string[]) =>
    call<Imported[]>("import_images", { paths }),
  chooseImages: async () => {
    if (!isTauri()) throw new Error("请在桌面应用中导入图片");
    const files = await open({
      multiple: true,
      filters: [{ name: "图片", extensions: ["jpg", "jpeg", "png", "webp"] }],
    });
    return files ? (Array.isArray(files) ? files : [files]) : [];
  },
  chooseDirectory: async () => {
    const path = await open({ directory: true, multiple: false });
    return typeof path === "string" ? path : null;
  },
  preview: async (
    path: string,
    spec: Watermark,
    original: boolean,
    requestId: number,
  ) => {
    const p = await call<string>("preview_image", {
      path,
      spec,
      original,
      requestId,
    });
    return convertFileSrc(p) + `?v=${requestId}`;
  },
  export: (
    paths: string[],
    spec: Watermark,
    options: ExportOptions,
    jobId: string,
  ) => call<BatchResult>("export_images", { paths, spec, options, jobId }),
  cancel: () => call<void>("cancel_export"),
  openOutput: (directory: string) => call<void>("open_output", { directory }),
  defaultOutput: () => call<string>("default_output"),
  load: () => call<Preferences>("load_preferences"),
  save: (preferences: Preferences) =>
    call<void>("save_preferences", { preferences }),
  onProgress: (cb: (event: Progress) => void) =>
    isTauri()
      ? listen<Progress>("export-progress", (e) => cb(e.payload))
      : Promise.resolve(() => {}),
  onDrop: (cb: (paths: string[]) => void) =>
    isTauri()
      ? getCurrentWebview().onDragDropEvent((e) => {
          if (e.payload.type === "drop") cb(e.payload.paths);
        })
      : Promise.resolve(() => {}),
};
