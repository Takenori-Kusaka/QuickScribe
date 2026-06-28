import { describe, it, expect } from "vitest";
import { errorText } from "./errors";

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
});
