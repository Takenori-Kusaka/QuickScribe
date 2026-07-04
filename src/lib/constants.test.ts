import { describe, it, expect } from "vitest";
import {
  ALL_PROVIDERS,
  PROVIDER_LABEL_KEYS,
  LOCAL_PROVIDERS,
  AWS_PROVIDERS,
  FALLBACK_MODELS,
  KEY_PLACEHOLDER_KEYS,
  STT_PROVIDERS,
  STT_LABEL_KEYS,
  STT_CLOUD,
  STT_KEY_PLACEHOLDER_KEYS,
  STT_MODEL_PLACEHOLDER_KEYS,
  REFINE_STYLES,
  MODEL_TTL_MS,
  DISCOVERY_MAX,
  DEFAULT_SHORTCUT,
} from "./constants";

describe("整形プロバイダ定義の整合(SSOT)", () => {
  it("ALL_PROVIDERS が各マップを過不足なく覆う", () => {
    const keys = (o: object) => Object.keys(o).sort();
    const all = [...ALL_PROVIDERS].sort();
    expect(keys(PROVIDER_LABEL_KEYS)).toEqual(all);
    expect(keys(FALLBACK_MODELS)).toEqual(all);
    expect(keys(KEY_PLACEHOLDER_KEYS)).toEqual(all);
  });

  it("ラベル/プレースホルダは i18n キー(catalog.*)を指す（ハードコード文言を持たない）", () => {
    for (const p of ALL_PROVIDERS) {
      expect(PROVIDER_LABEL_KEYS[p]).toBe(`catalog.providers.${p}`);
      expect(KEY_PLACEHOLDER_KEYS[p]).toBe(`catalog.key_ph.${p}`);
    }
  });

  it("LOCAL/AWS は ALL_PROVIDERS の部分集合かつ互いに素", () => {
    for (const p of [...LOCAL_PROVIDERS, ...AWS_PROVIDERS]) {
      expect(ALL_PROVIDERS).toContain(p);
    }
    expect(LOCAL_PROVIDERS.some((p) => AWS_PROVIDERS.includes(p))).toBe(false);
  });
});

describe("STTプロバイダ定義の整合(SSOT)", () => {
  it("STT_CLOUD は STT_PROVIDERS に存在し local を含まない", () => {
    for (const p of STT_CLOUD) expect(STT_PROVIDERS).toContain(p);
    expect(STT_CLOUD).not.toContain("local");
  });

  it("ラベルは i18n キー(catalog.stt.*)を指す", () => {
    expect(Object.keys(STT_LABEL_KEYS).sort()).toEqual([...STT_PROVIDERS].sort());
    for (const p of STT_PROVIDERS) expect(STT_LABEL_KEYS[p]).toBe(`catalog.stt.${p}`);
  });

  it("クラウドSTTの鍵/モデルのプレースホルダキーが STT_CLOUD を覆う", () => {
    const cloud = [...STT_CLOUD].sort();
    expect(Object.keys(STT_KEY_PLACEHOLDER_KEYS).sort()).toEqual(cloud);
    expect(Object.keys(STT_MODEL_PLACEHOLDER_KEYS).sort()).toEqual(cloud);
  });
});

describe("整形スタイルと定数", () => {
  it("REFINE_STYLES の value は一意で主要4スタイルを含む(ブレストはvisionのコア価値)", () => {
    const values = REFINE_STYLES.map((s) => s.value);
    expect(new Set(values).size).toBe(values.length);
    expect(values).toEqual(
      expect.arrayContaining(["structured", "verbatim", "summary", "brainstorm"]),
    );
    for (const s of REFINE_STYLES) {
      expect(s.labelKey).toBe(`catalog.styles.${s.value}.label`);
      expect(s.descKey).toBe(`catalog.styles.${s.value}.desc`);
    }
  });

  it("定数が妥当", () => {
    expect(MODEL_TTL_MS).toBe(24 * 60 * 60 * 1000);
    expect(DISCOVERY_MAX).toBeGreaterThan(0);
    expect(DEFAULT_SHORTCUT).toContain("CommandOrControl");
  });
});
