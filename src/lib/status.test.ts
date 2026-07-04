import { describe, it, expect } from "vitest";
import { statusText } from "./status";
import { ERR_CODE_SEP, type Translator } from "./errors";

const t: Translator = (key, opts) => {
  if (key === "status.rust.S_TRANSCRIBING") return "文字起こし中…";
  if (key === "status.rust.S_MODEL_DOWNLOAD_PCT") return `DL ${opts?.values?.detail}%`;
  return key; // 未知キーはキー自身（svelte-i18n 互換）
};

describe("statusText", () => {
  it("既知コードはローカライズ文言へ写像する", () => {
    expect(statusText("S_TRANSCRIBING", t)).toBe("文字起こし中…");
  });

  it("detail(U+001F区切り)をプレースホルダに補間する", () => {
    expect(statusText(`S_MODEL_DOWNLOAD_PCT${ERR_CODE_SEP}42`, t)).toBe("DL 42%");
  });

  it("未知コードは detail（無ければコード）へフォールバックし区切り文字を露出しない", () => {
    expect(statusText(`S_UNKNOWN${ERR_CODE_SEP}raw detail`, t)).toBe("raw detail");
    expect(statusText("S_UNKNOWN", t)).toBe("S_UNKNOWN");
  });

  it("翻訳器なしでは detail/コードを返す", () => {
    expect(statusText(`S_TRANSCRIBING${ERR_CODE_SEP}x`)).toBe("x");
  });

  it("コード以外の生文字列（空含む）はそのまま", () => {
    expect(statusText("", t)).toBe("");
    expect(statusText("plain text", t)).toBe("plain text");
  });
});
