import { describe, it, expect } from "vitest";
import { elapsedSeconds, buildNoteContent } from "./note";

describe("elapsedSeconds", () => {
  it("ミリ秒差を秒に丸める", () => {
    expect(elapsedSeconds(1000, 6000)).toBe(5);
  });
  it("四捨五入する", () => {
    expect(elapsedSeconds(0, 2600)).toBe(3);
  });
  it("負の経過は0に丸める", () => {
    expect(elapsedSeconds(5000, 1000)).toBe(0);
  });
});

describe("buildNoteContent", () => {
  it("録音秒を含む", () => {
    expect(buildNoteContent(12)).toContain("録音 12s");
  });
});
