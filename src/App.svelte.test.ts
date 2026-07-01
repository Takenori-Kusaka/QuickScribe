// @vitest-environment jsdom
// App.svelte のコンポーネントテスト(#402/#481-item13)。
// 主要フロー(録音トグル/ジャーナル/メモ整形/設定操作)を @testing-library/svelte で駆動し、
// App.svelte の実効カバレッジを上げる。Tauri の各APIはモック(invoke はコマンド別に返す)。
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";

// i18n は import 時に起動ロケールを解決する。App の import より前に locale=ja と
// provider=ollama(鍵不要=整形が通る)を固定する(hoistedは全importより先に走る)。
const { invokeMock, openMock } = vi.hoisted(() => {
  try {
    globalThis.localStorage?.setItem("locale", "ja");
    globalThis.localStorage?.setItem("provider", "ollama");
  } catch {
    /* jsdom 前提 */
  }
  return { invokeMock: vi.fn(), openMock: vi.fn() };
});

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: openMock }));
vi.mock("@tauri-apps/plugin-updater", () => ({ check: vi.fn().mockResolvedValue(null) }));
vi.mock("@tauri-apps/plugin-process", () => ({ relaunch: vi.fn().mockResolvedValue(undefined) }));
vi.mock("@tauri-apps/plugin-autostart", () => ({
  enable: vi.fn().mockResolvedValue(undefined),
  disable: vi.fn().mockResolvedValue(undefined),
  isEnabled: vi.fn().mockResolvedValue(false),
}));

import App from "./App.svelte";

// コマンド別のダミー応答。未指定コマンドは undefined。
function defaultInvoke(cmd: string): unknown {
  switch (cmd) {
    case "list_entries":
      return [
        {
          path: "/x/refined-1.md",
          name: "refined-1.md",
          created: "2026-06-01T10:00:00",
          kind: "refined",
          tags: ["仕事"],
          preview: "プレビュー本文",
        },
      ];
    case "read_text_file":
      return "メモの本文テキスト";
    case "refine_text":
      return "整形された結果テキスト";
    default:
      return undefined;
  }
}

beforeEach(() => {
  invokeMock.mockReset();
  invokeMock.mockImplementation(async (cmd: string) => defaultInvoke(cmd));
  openMock.mockReset();
});

describe("App.svelte 起動・基本描画", () => {
  it("見出しと録音ボタンが描画される", async () => {
    render(App);
    expect(
      await screen.findByRole("heading", { name: "QuickScribe", level: 1 }),
    ).toBeInTheDocument();
    const btn = document.querySelector('[data-testid="record-btn"]');
    expect(btn).toBeInTheDocument();
    expect(btn).toBeVisible();
  });

  it("歯車ボタンで設定ダイアログが開く", async () => {
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: "設定" }));
    expect(await screen.findByRole("dialog")).toBeInTheDocument();
  });
});

describe("App.svelte 録音トグル", () => {
  it("録音ボタン押下で start_recording が呼ばれる", async () => {
    render(App);
    const btn = document.querySelector('[data-testid="record-btn"]') as HTMLButtonElement;
    await fireEvent.click(btn);
    expect(invokeMock).toHaveBeenCalledWith("start_recording", expect.anything());
  });
});

describe("App.svelte ジャーナル", () => {
  it("ジャーナルを開くと list_entries の結果が表示される", async () => {
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: "ジャーナル" }));
    expect(await screen.findByText("プレビュー本文")).toBeInTheDocument();
    expect(invokeMock).toHaveBeenCalledWith("list_entries");
  });
});

describe("App.svelte メモ整形フロー", () => {
  it("メモから整形すると refine_text の結果が表示される", async () => {
    openMock.mockResolvedValue("/path/to/memo.txt");
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: /メモ/ }));
    expect(await screen.findByText("整形された結果テキスト")).toBeInTheDocument();
    expect(invokeMock).toHaveBeenCalledWith("refine_text", expect.anything());
  });

  it("空のメモは整形せずエラー表示（無駄なAPIを呼ばない）", async () => {
    openMock.mockResolvedValue("/path/to/empty.txt");
    invokeMock.mockImplementation(async (cmd: string) =>
      cmd === "read_text_file" ? "   " : defaultInvoke(cmd),
    );
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: /メモ/ }));
    expect(await screen.findByRole("alert")).toBeInTheDocument();
    expect(invokeMock).not.toHaveBeenCalledWith("refine_text", expect.anything());
  });
});

describe("App.svelte 設定操作", () => {
  it("設定の保存で set_save_settings が呼ばれる", async () => {
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: "設定" }));
    const saveBtn = await screen.findByRole("button", { name: "保存" });
    await fireEvent.click(saveBtn);
    expect(invokeMock).toHaveBeenCalledWith("set_save_settings", expect.anything());
  });
});
