import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { vi, beforeEach, it, expect } from "vitest";
import { api } from "../src/api";
import App from "../src/App";
import { defaultSpec } from "../src/model";
vi.mock("../src/api", () => ({
  api: {
    load: vi.fn(),
    save: vi.fn(),
    chooseImages: vi.fn(),
    importImages: vi.fn(),
    preview: vi.fn(),
    defaultOutput: vi.fn(),
    chooseDirectory: vi.fn(),
    export: vi.fn(),
    cancel: vi.fn(),
    openOutput: vi.fn(),
    onProgress: vi.fn(),
    onDrop: vi.fn(),
  },
}));
const file = {
  path: "/sample.png",
  name: "sample.png",
  width: 800,
  height: 600,
  thumbnail: "data:image/png;base64,AA==",
  error: null,
};
beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(api.load).mockResolvedValue({
    schemaVersion: 1,
    templates: [],
    lastSpec: null,
  });
  vi.mocked(api.save).mockResolvedValue();
  vi.mocked(api.chooseImages).mockResolvedValue(["/sample.png"]);
  vi.mocked(api.importImages).mockResolvedValue([file]);
  vi.mocked(api.preview).mockResolvedValue("data:image/png;base64,AA==");
  vi.mocked(api.defaultOutput).mockResolvedValue("/out");
  vi.mocked(api.onProgress).mockResolvedValue(() => {});
  vi.mocked(api.onDrop).mockResolvedValue(() => {});
  vi.mocked(api.cancel).mockResolvedValue();
  vi.mocked(api.export).mockResolvedValue({
    total: 1,
    cancelled: false,
    files: [{ source: file.path, output: "/out/sample.png", error: null }],
  });
});
async function imported() {
  const u = userEvent.setup();
  render(<App />);
  await u.click(screen.getByRole("button", { name: "选择图片", exact: true }));
  await waitFor(() => expect(api.importImages).toHaveBeenCalled());
  await screen.findByText("800 × 600 px");
  return u;
}
it("R01/R02 starts empty and imports a real backend result", async () => {
  await imported();
  expect(screen.getByText("应用于全部 1 张")).toBeInTheDocument();
  expect(
    screen.getByRole("button", { name: "导出图片", exact: true }),
  ).toBeEnabled();
});
it("R03/R04 changes preview parameters using certificate preset", async () => {
  const u = await imported();
  await u.click(screen.getByRole("button", { name: "证件保护" }));
  expect(screen.getByLabelText("全图平铺")).toBeChecked();
  expect(screen.getByLabelText("水印内容")).toHaveValue(
    "仅供业务办理使用 · 再次复印无效",
  );
  await waitFor(() =>
    expect(api.preview).toHaveBeenLastCalledWith(
      file.path,
      expect.objectContaining({ tile: true, angle: -30 }),
      false,
      expect.any(Number),
    ),
  );
});
it("R03 blocks export for empty text and missing logo", async () => {
  const u = await imported();
  await u.clear(screen.getByLabelText("水印内容"));
  expect(
    screen.getByRole("button", { name: "导出图片", exact: true }),
  ).toBeDisabled();
  await u.click(screen.getByRole("button", { name: "图片水印", exact: true }));
  expect(screen.getAllByText("请先上传图片水印")).toHaveLength(2);
  expect(
    screen.getByRole("button", { name: "导出图片", exact: true }),
  ).toBeDisabled();
});
it("R06/R07 exports through options dialog and displays completion", async () => {
  const u = await imported();
  await u.click(screen.getByRole("button", { name: "导出图片", exact: true }));
  await screen.findByRole("dialog");
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "开始导出" })).toBeEnabled(),
  );
  await u.click(screen.getByRole("button", { name: "开始导出" }));
  await screen.findByRole("heading", { name: "导出完成" });
  expect(api.export).toHaveBeenCalledWith(
    [file.path],
    defaultSpec,
    expect.objectContaining({ directory: "/out", suffix: "_watermarked" }),
    expect.any(String),
  );
});
it("R06 selected scope prevents empty export", async () => {
  const u = await imported();
  await u.click(screen.getByRole("checkbox", { name: "全选图片" }));
  await u.click(screen.getByRole("button", { name: "导出图片", exact: true }));
  await u.selectOptions(screen.getByLabelText("导出范围"), "selected");
  expect(screen.getByRole("button", { name: "开始导出" })).toBeDisabled();
});
it("R07 export errors are visible and allow recovery", async () => {
  vi.mocked(api.export).mockRejectedValue(new Error("磁盘空间不足"));
  const u = await imported();
  await u.click(screen.getByRole("button", { name: "导出图片", exact: true }));
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "开始导出" })).toBeEnabled(),
  );
  await u.click(screen.getByRole("button", { name: "开始导出" }));
  await screen.findByText("磁盘空间不足");
  expect(
    screen.getByRole("button", { name: "导出图片", exact: true }),
  ).toBeEnabled();
});
it("R05 restores persisted template and last settings", async () => {
  vi.mocked(api.load).mockResolvedValue({
    schemaVersion: 1,
    lastSpec: { ...defaultSpec, text: "已恢复" },
    templates: [
      {
        id: "saved",
        name: "我的模板一",
        spec: { ...defaultSpec, text: "模板文字" },
      },
    ],
  });
  render(<App />);
  await waitFor(() =>
    expect(screen.getByLabelText("水印内容")).toHaveValue("已恢复"),
  );
  await userEvent.selectOptions(
    screen.getByLabelText("选择已保存模板"),
    "saved",
  );
  expect(screen.getByLabelText("水印内容")).toHaveValue("模板文字");
});
it("R05 saves current text settings as a named template", async () => {
  const u = await imported();
  await u.click(screen.getByRole("button", { name: "保存模板", exact: true }));
  await u.type(screen.getByLabelText("模板名称"), "作品署名");
  await u.click(screen.getByRole("button", { name: "保存当前设置" }));
  await waitFor(() =>
    expect(api.save).toHaveBeenCalledWith(
      expect.objectContaining({
        templates: [expect.objectContaining({ name: "作品署名" })],
      }),
    ),
  );
});
it("R01 import failure is surfaced and import remains available", async () => {
  vi.mocked(api.importImages).mockRejectedValue(new Error("读取失败"));
  render(<App />);
  await userEvent.click(
    screen.getByRole("button", { name: "选择图片", exact: true }),
  );
  await screen.findByText("读取失败");
  expect(
    screen.getByRole("button", { name: "导入图片", exact: true }),
  ).toBeEnabled();
});
it("R03 range values remain numeric in renderer requests", async () => {
  await imported();
  fireEvent.change(screen.getByRole("slider", { name: "不透明度" }), {
    target: { value: "42" },
  });
  await waitFor(() =>
    expect(api.preview).toHaveBeenLastCalledWith(
      file.path,
      expect.objectContaining({ opacity: 42 }),
      false,
      expect.any(Number),
    ),
  );
});
it("R02 removes an imported file and returns to empty state", async () => {
  const u = await imported();
  await u.click(screen.getByRole("button", { name: "移除 sample.png" }));
  expect(
    screen.getByRole("button", { name: "选择图片", exact: true }),
  ).toBeInTheDocument();
  expect(
    screen.getByRole("button", { name: "导出图片", exact: true }),
  ).toBeDisabled();
});
it("R07 cancellation retains completed outputs and reports unprocessed files", async () => {
  let finish!: (r: import("../src/model").BatchResult) => void;
  vi.mocked(api.export).mockImplementation(
    () =>
      new Promise((resolve) => {
        finish = resolve;
      }),
  );
  const u = await imported();
  await u.click(screen.getByRole("button", { name: "导出图片", exact: true }));
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "开始导出" })).toBeEnabled(),
  );
  await u.click(screen.getByRole("button", { name: "开始导出" }));
  await u.click(screen.getByRole("button", { name: "取消任务" }));
  expect(api.cancel).toHaveBeenCalledOnce();
  finish({ total: 1, cancelled: true, files: [] });
  await screen.findByRole("heading", { name: "已取消" });
  expect(screen.getByText(/未处理 1 \/ 共 1/)).toBeInTheDocument();
});
it("R07 retry uses only failures and original export snapshot", async () => {
  vi.mocked(api.export).mockResolvedValueOnce({
    total: 1,
    cancelled: false,
    files: [{ source: file.path, output: null, error: "temporary" }],
  });
  const u = await imported();
  await u.click(screen.getByRole("button", { name: "导出图片", exact: true }));
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "开始导出" })).toBeEnabled(),
  );
  await u.click(screen.getByRole("button", { name: "开始导出" }));
  await screen.findByRole("button", { name: "仅重试失败项" });
  await u.clear(screen.getByLabelText("水印内容"));
  await u.type(screen.getByLabelText("水印内容"), "修改后的设置");
  await u.click(screen.getByRole("button", { name: "仅重试失败项" }));
  await waitFor(() => expect(api.export).toHaveBeenCalledTimes(2));
  expect(vi.mocked(api.export).mock.calls[1][1].text).toBe(defaultSpec.text);
});
it("R04 requests original on comparison then returns to watermark", async () => {
  await imported();
  const button = screen.getByRole("button", { name: "按住查看原图" });
  fireEvent.keyDown(button, { key: " " });
  await waitFor(() =>
    expect(api.preview).toHaveBeenLastCalledWith(
      file.path,
      defaultSpec,
      true,
      expect.any(Number),
    ),
  );
  fireEvent.keyUp(button, { key: " " });
  await waitFor(() =>
    expect(api.preview).toHaveBeenLastCalledWith(
      file.path,
      defaultSpec,
      false,
      expect.any(Number),
    ),
  );
});
it("R05 deletes a saved template and persists removal", async () => {
  vi.mocked(api.load).mockResolvedValue({
    schemaVersion: 1,
    lastSpec: defaultSpec,
    templates: [{ id: "x", name: "删除测试", spec: defaultSpec }],
  });
  render(<App />);
  await screen.findByRole("button", { name: "管理模板" });
  await userEvent.click(screen.getByRole("button", { name: "管理模板" }));
  await userEvent.click(
    screen.getByRole("button", { name: "删除模板 删除测试" }),
  );
  await waitFor(() =>
    expect(api.save).toHaveBeenCalledWith(
      expect.objectContaining({ templates: [] }),
    ),
  );
});
