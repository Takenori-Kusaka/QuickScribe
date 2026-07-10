import { describe, it, expect } from "vitest";
import { detectSpeakers, buildSpeakerRenames } from "./speakers";
import { applyCorrections } from "./corrections";

describe("detectSpeakers", () => {
  it("本文中の [話者N] を重複なく昇順で抽出する", () => {
    const t = "[話者2] こんにちは\n[話者1] どうも\n[話者2] またね";
    expect(detectSpeakers(t)).toEqual(["話者1", "話者2"]);
  });

  it("タイムスタンプ [HH:MM:SS] は話者として拾わない", () => {
    const t = "[00:01:23][話者1] はい\n[00:02:00] ラベル無し行";
    expect(detectSpeakers(t)).toEqual(["話者1"]);
  });

  it("話者ラベルが無ければ空配列（リネームUI非表示の根拠 R10）", () => {
    expect(detectSpeakers("ただの文字起こし。")).toEqual([]);
    expect(detectSpeakers("")).toEqual([]);
  });

  it("10以上でも数値順にソート（文字列順ではない）", () => {
    const t = "[話者10] a\n[話者2] b\n[話者1] c";
    expect(detectSpeakers(t)).toEqual(["話者1", "話者2", "話者10"]);
  });
});

describe("buildSpeakerRenames", () => {
  it("入力名を [話者N]→[名前] の一括置換候補に変換する", () => {
    const corr = buildSpeakerRenames({ 話者1: "田中", 話者2: "佐藤" });
    expect(corr).toEqual([
      { original: "[話者1]", suggestion: "[田中]", reason: "", replace: true },
      { original: "[話者2]", suggestion: "[佐藤]", reason: "", replace: true },
    ]);
  });

  it("空欄の名前はスキップする（R8 空欄は変更しない）", () => {
    const corr = buildSpeakerRenames({ 話者1: "田中", 話者2: "  ", 話者3: "" });
    expect(corr).toHaveLength(1);
    expect(corr[0].suggestion).toBe("[田中]");
  });

  it("既存 applyCorrections と連携して本文を全置換できる（R9 再利用）", () => {
    const text = "[話者1] やあ\n[話者2] どうも\n[話者1] またね";
    const corr = buildSpeakerRenames({ 話者1: "田中", 話者2: "佐藤" });
    const { text: out, applied } = applyCorrections(text, corr);
    expect(out).toBe("[田中] やあ\n[佐藤] どうも\n[田中] またね");
    expect(applied).toBe(2);
  });
});
