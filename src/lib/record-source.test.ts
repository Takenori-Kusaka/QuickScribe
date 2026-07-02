import { describe, it, expect } from "vitest";
import { parseRecordSource } from "./record-source";

describe("parseRecordSource", () => {
  it("kind|id を分解する", () => {
    expect(parseRecordSource("input|")).toEqual({ kind: "input", id: "" });
    expect(parseRecordSource("mix|")).toEqual({ kind: "mix", id: "" });
    expect(parseRecordSource("input|MIC-123")).toEqual({ kind: "input", id: "MIC-123" });
  });
  it("id に含まれる '|' は最初の区切りのみで分け、以降は温存する", () => {
    expect(parseRecordSource("mix|render|abc")).toEqual({ kind: "mix", id: "render|abc" });
  });
  it("区切り無しは kind のみ（id は空・末尾欠落しない）", () => {
    expect(parseRecordSource("input")).toEqual({ kind: "input", id: "" });
    expect(parseRecordSource("")).toEqual({ kind: "", id: "" });
  });
});
