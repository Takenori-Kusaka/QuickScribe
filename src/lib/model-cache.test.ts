import { describe, it, expect } from "vitest";
import { isModelCacheFresh } from "./model-cache";

const TTL = 24 * 60 * 60 * 1000;

describe("isModelCacheFresh", () => {
  it("モデル未解決(空)なら常に false", () => {
    expect(isModelCacheFresh("", 1000, 1000, TTL)).toBe(false);
  });
  it("TTL 内なら true", () => {
    const now = 1_000_000;
    expect(isModelCacheFresh("gpt-4o", now - 1000, now, TTL)).toBe(true);
  });
  it("TTL 超過なら false", () => {
    const now = 1_000_000;
    expect(isModelCacheFresh("gpt-4o", now - TTL - 1, now, TTL)).toBe(false);
  });
  it("ちょうど TTL 経過は false（< 比較）", () => {
    const now = 1_000_000;
    expect(isModelCacheFresh("gpt-4o", now - TTL, now, TTL)).toBe(false);
  });
});
