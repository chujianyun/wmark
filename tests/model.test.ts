import { describe, it, expect } from "vitest";
import {
  defaultSpec,
  specError,
  exportError,
  mergeFiles,
  summary,
  type Imported,
} from "../src/model";
describe("data contracts", () => {
  it("validates empty and oversized text", () => {
    expect(specError({ ...defaultSpec, text: "" })).toBeTruthy();
    expect(specError({ ...defaultSpec, text: "中".repeat(101) })).toBeTruthy();
    expect(specError({ ...defaultSpec, text: "中文 ©" })).toBeNull();
  });
  it("requires an image for logo mode", () =>
    expect(specError({ ...defaultSpec, kind: "logo" })).toBeTruthy());
  it("rejects path traversal in suffix", () =>
    expect(
      exportError({
        directory: "/out",
        format: "png",
        quality: 90,
        suffix: "../../foo",
        background: "#ffffff",
      }),
    ).toBeTruthy());
  it("merges duplicate import results and allows corrected files", () => {
    const file: Imported = {
      name: "x.png",
      path: "/x.png",
      width: 0,
      height: 0,
      error: "bad",
      thumbnail: "",
    };
    const good = { ...file, width: 100, height: 100, error: null };
    expect(mergeFiles([file], [good])).toEqual([good]);
  });
  it("reports cancelled pending files separately from failures", () => {
    expect(
      summary({
        total: 3,
        cancelled: true,
        files: [
          { source: "a", output: "b", error: null },
          { source: "x", output: null, error: "bad" },
        ],
      }),
    ).toEqual({ success: 1, failed: 1, pending: 1 });
  });
});
