// @vitest-environment jsdom
import { describe, it, expect, vi, beforeEach } from "vitest";
import { flushSync } from "svelte";

const invokeMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invokeMock(...a) }));

import { createShortcut } from "./shortcut-store.svelte";

const t = (key: string) => key; // key を素通しする翻訳器。

beforeEach(() => {
  invokeMock.mockReset();
  invokeMock.mockResolvedValue(undefined);
  localStorage.clear();
});

describe("createShortcut", () => {
  it("startCapture/cancelCapture が capturing を切り替える", () => {
    const s = createShortcut(t);
    s.startCapture();
    flushSync();
    expect(s.capturing).toBe(true);
    s.cancelCapture();
    flushSync();
    expect(s.capturing).toBe(false);
  });

  it("onCaptureKeydown: Escape はキャプチャをやめ登録しない", () => {
    const s = createShortcut(t);
    s.startCapture();
    s.onCaptureKeydown(new KeyboardEvent("keydown", { key: "Escape" }));
    flushSync();
    expect(s.capturing).toBe(false);
    expect(invokeMock).not.toHaveBeenCalled();
  });

  it("onCaptureKeydown: 修飾キー無しはヒントを出し待機継続", () => {
    const s = createShortcut(t);
    s.startCapture();
    s.onCaptureKeydown(new KeyboardEvent("keydown", { key: "R" }));
    flushSync();
    expect(s.capturing).toBe(true);
    expect(s.shortcutMsg).toBe("settings.shortcut_modifier_required");
  });

  it("onCaptureKeydown: 有効な組合せはショートカットを更新し登録する", async () => {
    const s = createShortcut(t);
    s.startCapture();
    s.onCaptureKeydown(new KeyboardEvent("keydown", { key: "R", ctrlKey: true, shiftKey: true }));
    await Promise.resolve();
    flushSync();
    expect(s.recordShortcut).toBe("CommandOrControl+Shift+R");
    expect(s.capturing).toBe(false);
    expect(invokeMock).toHaveBeenCalledWith("set_record_shortcut", {
      accelerator: "CommandOrControl+Shift+R",
    });
    expect(localStorage.getItem("recordShortcut")).toBe("CommandOrControl+Shift+R");
  });

  it("applyShortcut 失敗時は失敗メッセージを出す", async () => {
    invokeMock.mockRejectedValueOnce(new Error("x"));
    const s = createShortcut(t);
    await s.applyShortcut();
    flushSync();
    expect(s.shortcutMsg).toBe("errors.hotkey_failed");
  });

  it("resetShortcut は既定へ戻す", async () => {
    const s = createShortcut(t);
    s.recordShortcut = "CommandOrControl+X";
    s.resetShortcut();
    await Promise.resolve();
    flushSync();
    expect(s.recordShortcut).toBe("CommandOrControl+Shift+R");
  });

  it("display は表記変換した文字列を返す", () => {
    const s = createShortcut(t);
    expect(typeof s.display()).toBe("string");
    expect(s.display().length).toBeGreaterThan(0);
  });

  it("onCaptureKeydown: キャプチャ中でなければ何もしない", () => {
    const s = createShortcut(t);
    s.onCaptureKeydown(new KeyboardEvent("keydown", { key: "R", ctrlKey: true }));
    flushSync();
    expect(invokeMock).not.toHaveBeenCalled();
    expect(s.recordShortcut).toBe("CommandOrControl+Shift+R");
  });

  it("onCaptureKeydown: 修飾キー単体は確定せず待機継続", () => {
    const s = createShortcut(t);
    s.startCapture();
    s.onCaptureKeydown(new KeyboardEvent("keydown", { key: "Shift", shiftKey: true }));
    flushSync();
    expect(s.capturing).toBe(true);
    expect(invokeMock).not.toHaveBeenCalled();
    expect(s.shortcutMsg).toBe("");
  });

  it("navigator.platform が無い環境でも UA から Mac を判定する", () => {
    vi.stubGlobal("navigator", { userAgent: "Macintosh" }); // platform 未定義 → ?? "" 側。
    try {
      const s = createShortcut(t);
      expect(s.isMac).toBe(true);
    } finally {
      vi.unstubAllGlobals();
    }
  });
});
