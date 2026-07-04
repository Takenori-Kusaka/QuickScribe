import { describe, it, expect } from "vitest";
import { errorText, ERR_CODE_SEP, type Translator } from "./errors";

// svelte-i18n の $_ 相当のフェイク翻訳器。未知キーはキー自身を返す。
const catalog: Record<string, string> = {
  "errors.rust.E_STT_MODEL_LOAD": "モデル読込に失敗: {detail}",
  "errors.rust.E_ALREADY_RECORDING": "すでに録音中です。",
  "errors.unknown": "An unknown error occurred.",
};
const t: Translator = (key, opts) => {
  const tmpl = catalog[key];
  if (tmpl === undefined) return key;
  const v = opts?.values ?? {};
  return tmpl.replace(/\{(\w+)\}/g, (_, k) => String(v[k] ?? ""));
};

describe("errorText", () => {
  it("文字列はそのまま(trim)返す", () => {
    expect(errorText("鍵が未設定です")).toBe("鍵が未設定です");
    expect(errorText("  余白あり  ")).toBe("余白あり");
  });

  it("Error は message を返す", () => {
    expect(errorText(new Error("ネットワークに接続できません"))).toBe(
      "ネットワークに接続できません",
    );
  });

  it("message を持つオブジェクトから取り出す", () => {
    expect(errorText({ message: "APIが拒否しました" })).toBe("APIが拒否しました");
  });

  it("空・null・未知の値はフォールバック", () => {
    expect(errorText("")).toBe("原因不明のエラーが発生しました。");
    expect(errorText(null)).toBe("原因不明のエラーが発生しました。");
    expect(errorText(undefined)).toBe("原因不明のエラーが発生しました。");
  });

  it("空 message の Error は name にフォールバック", () => {
    const e = new Error("");
    expect(errorText(e)).toBe("Error");
  });

  it("message が文字列でないオブジェクトは String() へフォールバック", () => {
    expect(errorText({ message: 42 })).toBe("[object Object]");
  });

  it("数値など非オブジェクトは String() で文字列化する", () => {
    expect(errorText(42)).toBe("42");
  });
});

describe("errorText — Rust エラーコード (#462)", () => {
  it("既知コード＋詳細を翻訳し {detail} を補間する", () => {
    const raw = `E_STT_MODEL_LOAD${ERR_CODE_SEP}no such file`;
    expect(errorText(raw, t)).toBe("モデル読込に失敗: no such file");
  });

  it("詳細なしの既知コードを翻訳する", () => {
    expect(errorText("E_ALREADY_RECORDING", t)).toBe("すでに録音中です。");
  });

  it("翻訳器なしでも区切り文字を露出せず詳細部を返す", () => {
    const raw = `E_STT_MODEL_LOAD${ERR_CODE_SEP}no such file`;
    expect(errorText(raw)).toBe("no such file");
  });

  it("未知コードは詳細部へ安全にフォールバックする", () => {
    const raw = `E_TOTALLY_UNKNOWN${ERR_CODE_SEP}boom`;
    expect(errorText(raw, t)).toBe("boom");
  });

  it("未知コードで詳細も無ければコード自身を返す", () => {
    expect(errorText("E_TOTALLY_UNKNOWN", t)).toBe("E_TOTALLY_UNKNOWN");
  });

  it("コードに見えない生文字列は翻訳器を渡しても素通し", () => {
    expect(errorText("普通のエラー文", t)).toBe("普通のエラー文");
  });

  it("空・null は翻訳器があれば errors.unknown を使う", () => {
    expect(errorText("", t)).toBe("An unknown error occurred.");
    expect(errorText(null, t)).toBe("An unknown error occurred.");
  });

  it("翻訳器なしの空値は既定フォールバック", () => {
    expect(errorText("")).toBe("原因不明のエラーが発生しました。");
  });

  it("翻訳器が errors.unknown を解決できなければ既定フォールバック", () => {
    // 未知キーをキー自身で返す翻訳器（カタログ未整備時）→ FALLBACK 側へ。
    const echo: Translator = (key) => key;
    expect(errorText("", echo)).toBe("原因不明のエラーが発生しました。");
  });
});
