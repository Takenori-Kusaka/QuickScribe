import { describe, it, expect } from "vitest";
import {
  ALL_PROVIDERS,
  PROVIDER_LABELS,
  LOCAL_PROVIDERS,
  AWS_PROVIDERS,
  FALLBACK_MODELS,
  KEY_PLACEHOLDERS,
  STT_LABELS,
  STT_CLOUD,
  STT_KEY_PLACEHOLDERS,
  STT_MODEL_PLACEHOLDERS,
  REFINE_STYLES,
  MODEL_TTL_MS,
  DISCOVERY_MAX,
  DEFAULT_SHORTCUT,
} from "./constants";

describe("整形プロバイダ定義の整合(SSOT)", () => {
  it("ALL_PROVIDERS が各マップを過不足なく覆う", () => {
    const keys = (o: object) => Object.keys(o).sort();
    const all = [...ALL_PROVIDERS].sort();
    expect(keys(PROVIDER_LABELS)).toEqual(all);
    expect(keys(FALLBACK_MODELS)).toEqual(all);
    expect(keys(KEY_PLACEHOLDERS)).toEqual(all);
  });

  it("LOCAL/AWS は ALL_PROVIDERS の部分集合かつ互いに素", () => {
    for (const p of [...LOCAL_PROVIDERS, ...AWS_PROVIDERS]) {
      expect(ALL_PROVIDERS).toContain(p);
    }
    expect(LOCAL_PROVIDERS.some((p) => AWS_PROVIDERS.includes(p))).toBe(false);
  });
});

describe("STTプロバイダ定義の整合(SSOT)", () => {
  it("STT_CLOUD は STT_LABELS に存在し local を含まない", () => {
    for (const p of STT_CLOUD) expect(STT_LABELS[p]).toBeTruthy();
    expect(STT_CLOUD).not.toContain("local");
  });

  it("クラウドSTTの鍵/モデルのプレースホルダが STT_CLOUD を覆う", () => {
    const cloud = [...STT_CLOUD].sort();
    expect(Object.keys(STT_KEY_PLACEHOLDERS).sort()).toEqual(cloud);
    expect(Object.keys(STT_MODEL_PLACEHOLDERS).sort()).toEqual(cloud);
  });
});

describe("整形スタイルと定数", () => {
  it("REFINE_STYLES の value は一意で主要4スタイルを含む(ブレストはvisionのコア価値)", () => {
    const values = REFINE_STYLES.map((s) => s.value);
    expect(new Set(values).size).toBe(values.length);
    expect(values).toEqual(
      expect.arrayContaining(["structured", "verbatim", "summary", "brainstorm"]),
    );
    for (const s of REFINE_STYLES) expect(s.desc.length).toBeGreaterThan(0);
  });

  it("定数が妥当", () => {
    expect(MODEL_TTL_MS).toBe(24 * 60 * 60 * 1000);
    expect(DISCOVERY_MAX).toBeGreaterThan(0);
    expect(DEFAULT_SHORTCUT).toContain("CommandOrControl");
  });
});
