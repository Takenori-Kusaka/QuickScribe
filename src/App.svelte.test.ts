// @vitest-environment jsdom
// App.svelte の最小コンポーネントテスト(#402 Phase2)。
// 目的は「コンポーネントテスト基盤が動くこと」の実証＝起動して主要UIが描画されること。
// Tauri の各APIはブラウザ外実体のためモックする(invoke等は no-op/既定値)。
import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";

// i18n は ./lib/i18n の import 時に起動ロケールを解決する(localStorage > navigator > ja)。
// App の import より前に locale=ja を固定し、描画テキストを安定させる(hoistedは全importより先に走る)。
vi.hoisted(() => {
  try {
    globalThis.localStorage?.setItem("locale", "ja");
  } catch {
    /* localStorage 未定義環境では無視（このテストは jsdom 前提） */
  }
});

// --- Tauri モック（App.svelte が import する全モジュール） ---
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));
vi.mock("@tauri-apps/api/event", () => ({
  // listen は購読解除関数を返す契約。
  listen: vi.fn().mockResolvedValue(() => {}),
}));
vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn().mockResolvedValue(null),
}));
vi.mock("@tauri-apps/plugin-updater", () => ({
  check: vi.fn().mockResolvedValue(null),
}));
vi.mock("@tauri-apps/plugin-process", () => ({
  relaunch: vi.fn().mockResolvedValue(undefined),
}));
vi.mock("@tauri-apps/plugin-autostart", () => ({
  enable: vi.fn().mockResolvedValue(undefined),
  disable: vi.fn().mockResolvedValue(undefined),
  isEnabled: vi.fn().mockResolvedValue(false),
}));

import App from "./App.svelte";

describe("App.svelte（コンポーネント基盤の実証）", () => {
  it("起動して見出しと録音ボタンが描画される", async () => {
    render(App);
    // ブランド名はロケール非依存。
    const heading = await screen.findByRole("heading", { name: "QuickScribe", level: 1 });
    expect(heading).toBeInTheDocument();
    // 録音ボタン（data-testid）が表示される。
    const btn = document.querySelector('[data-testid="record-btn"]');
    expect(btn).toBeInTheDocument();
    expect(btn).toBeVisible();
  });

  it("歯車ボタンで設定ダイアログが開く", async () => {
    render(App);
    // ヘッダの設定ボタン（aria-label=「設定」/ja）。
    const settingsBtn = await screen.findByRole("button", { name: "設定" });
    await fireEvent.click(settingsBtn);
    const dialog = await screen.findByRole("dialog");
    expect(dialog).toBeInTheDocument();
  });
});
