import { describe, it, expect } from "vitest";
import { elapsedSeconds, buildNoteContent, estimateRemaining, formatRemaining } from "./note";

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

describe("estimateRemaining", () => {
  it("50%なら経過と同じだけ残る", () => {
    expect(estimateRemaining(30, 50)).toBe(30);
  });
  it("0%や100%は0", () => {
    expect(estimateRemaining(30, 0)).toBe(0);
    expect(estimateRemaining(30, 100)).toBe(0);
  });
});

describe("formatRemaining", () => {
  it("60秒未満は秒表示", () => {
    expect(formatRemaining(45)).toBe("残り 約45秒");
  });
  it("分と秒", () => {
    expect(formatRemaining(125)).toBe("残り 約2分5秒");
  });
  it("ちょうど分", () => {
    expect(formatRemaining(120)).toBe("残り 約2分");
  });
  it("0以下は空", () => {
    expect(formatRemaining(0)).toBe("");
  });
  it("翻訳関数を渡すと eta.* キーで解決する（#401）", () => {
    const t = (key: string, opts?: { values?: Record<string, string> }) =>
      `${key}:${JSON.stringify(opts?.values)}`;
    expect(formatRemaining(45, t)).toBe('eta.seconds:{"s":"45"}');
    expect(formatRemaining(120, t)).toBe('eta.minutes:{"m":"2"}');
    expect(formatRemaining(125, t)).toBe('eta.min_sec:{"m":"2","s":"5"}');
  });
});
