import { describe, it, expect } from "vitest";
import { displayShortcut, accelFromEvent } from "./shortcut";

describe("displayShortcut", () => {
  it("CommandOrControl をプラットフォームで出し分ける", () => {
    expect(displayShortcut("CommandOrControl+Shift+R", false)).toBe("Ctrl+Shift+R");
    expect(displayShortcut("CommandOrControl+Shift+R", true)).toBe("Cmd+Shift+R");
  });

  it("Alt は mac で Option", () => {
    expect(displayShortcut("Alt+Q", false)).toBe("Alt+Q");
    expect(displayShortcut("Alt+Q", true)).toBe("Option+Q");
  });

  it("Super/Meta は Win / Cmd", () => {
    expect(displayShortcut("Super+J", false)).toBe("Win+J");
    expect(displayShortcut("Meta+J", true)).toBe("Cmd+J");
  });

  it("未知トークンはそのまま", () => {
    expect(displayShortcut("Control+F5", false)).toBe("Ctrl+F5");
  });
});

const ev = (p: Partial<KeyboardEvent>): KeyboardEvent => p as KeyboardEvent;

describe("accelFromEvent", () => {
  it("修飾＋英字を accelerator に", () => {
    expect(accelFromEvent(ev({ key: "r", ctrlKey: true, shiftKey: true }))).toBe(
      "CommandOrControl+Shift+R",
    );
  });

  it("修飾キー単体は null", () => {
    expect(accelFromEvent(ev({ key: "Control", ctrlKey: true }))).toBeNull();
  });

  it("修飾なしの単打は null（誤爆防止）", () => {
    expect(accelFromEvent(ev({ key: "a" }))).toBeNull();
  });

  it("Arrow 系は接頭辞を除去", () => {
    expect(accelFromEvent(ev({ key: "ArrowUp", altKey: true }))).toBe("Alt+Up");
  });

  it("ファンクションキーはそのまま", () => {
    expect(accelFromEvent(ev({ key: "F5", ctrlKey: true }))).toBe("CommandOrControl+F5");
  });
});
