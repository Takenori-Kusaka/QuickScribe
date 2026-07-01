// @vitest-environment jsdom
// App.svelte のコンポーネントテスト(#402/#481-item13)。
// 主要フロー(録音トグル/ジャーナル/メモ整形/設定操作)を @testing-library/svelte で駆動し、
// App.svelte の実効カバレッジを上げる。Tauri の各APIはモック(invoke はコマンド別に返す)。
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";

// i18n は import 時に起動ロケールを解決する。App の import より前に locale=ja と
// provider=ollama(鍵不要=整形が通る)を固定する(hoistedは全importより先に走る)。
const { invokeMock, openMock, listeners, listenMock } = vi.hoisted(() => {
  try {
    globalThis.localStorage?.setItem("locale", "ja");
    globalThis.localStorage?.setItem("provider", "ollama");
    globalThis.localStorage?.setItem("autoPipeline", "true");
  } catch {
    /* jsdom 前提 */
  }
  const listeners = new Map<string, (e: { payload: unknown }) => void>();
  // Tauri の listen を模し、イベント名→ハンドラを記録する(テストから emit できるように)。
  const listenMock = vi.fn(async (event: string, handler: (e: { payload: unknown }) => void) => {
    listeners.set(event, handler);
    return () => listeners.delete(event);
  });
  return { invokeMock: vi.fn(), openMock: vi.fn(), listeners, listenMock };
});

// バックエンドからのイベント発火をエミュレートする。
async function emitEvent(event: string, payload: unknown) {
  listeners.get(event)?.({ payload });
  // Svelte のリアクティブ更新をフラッシュ。
  await Promise.resolve();
}

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));
vi.mock("@tauri-apps/api/event", () => ({ listen: listenMock }));
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
  listeners.clear();
});

// listen の登録は onMount 内で非同期に走る。登録完了を待つ。
async function waitForListeners() {
  for (let i = 0; i < 50 && !listeners.has("transcribe-done"); i++) {
    await Promise.resolve();
  }
}

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

  it("翻訳トグルをONにすると出力言語ピッカーが現れる", async () => {
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: "設定" }));
    const cb = (await screen.findByLabelText(/翻訳して出力/)) as HTMLInputElement;
    expect(cb.checked).toBe(false);
    await fireEvent.click(cb);
    // 出力言語の select が出現する（settings.output_language）。
    expect(await screen.findByLabelText(/出力言語|翻訳先/)).toBeInTheDocument();
  });

  it("カスタム整形: ラベルと指示が空だとエラー", async () => {
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: "設定" }));
    // カスタム整形の追加ボタン（指示未入力）を押すとエラー。
    const addBtn = await screen.findByRole("button", { name: /追加/ });
    await fireEvent.click(addBtn);
    expect(await screen.findByRole("alert")).toBeInTheDocument();
  });
});

describe("App.svelte ジャーナル検索", () => {
  it("一致しない検索語でエントリが絞り込まれる", async () => {
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: "ジャーナル" }));
    expect(await screen.findByText("プレビュー本文")).toBeInTheDocument();
    const search = (await screen.findByLabelText(/検索/)) as HTMLInputElement;
    await fireEvent.input(search, { target: { value: "存在しない語XYZ" } });
    expect(screen.queryByText("プレビュー本文")).not.toBeInTheDocument();
  });
});

describe("App.svelte コピー", () => {
  it("整形結果をコピーすると clipboard.writeText が呼ばれる", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.assign(navigator, { clipboard: { writeText } });
    openMock.mockResolvedValue("/path/to/memo.txt");
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: /メモ/ }));
    await screen.findByText("整形された結果テキスト");
    await fireEvent.click(await screen.findByRole("button", { name: /コピー/ }));
    expect(writeText).toHaveBeenCalledWith("整形された結果テキスト");
  });
});

describe("App.svelte バックエンドイベント", () => {
  it("transcribe-done で文字起こしが表示され、自動整形される", async () => {
    render(App);
    await waitForListeners();
    await emitEvent("transcribe-done", "文字起こしされた本文");
    // 文字起こしが表示され、autoPipeline+ollama で自動整形まで走る。
    expect(await screen.findByText("整形された結果テキスト")).toBeInTheDocument();
    expect(invokeMock).toHaveBeenCalledWith("refine_text", expect.anything());
  });

  it("空の transcribe-done は「音声なし」エラーを表示する", async () => {
    render(App);
    await waitForListeners();
    await emitEvent("transcribe-done", "");
    expect(await screen.findByRole("alert")).toBeInTheDocument();
    expect(invokeMock).not.toHaveBeenCalledWith("refine_text", expect.anything());
  });

  it("transcribe-error でエラーが表示される", async () => {
    render(App);
    await waitForListeners();
    await emitEvent("transcribe-error", "文字起こしに失敗しました");
    expect(await screen.findByText("文字起こしに失敗しました")).toBeInTheDocument();
  });

  it("録音停止→progress で進捗バーが表示される", async () => {
    render(App);
    await waitForListeners();
    const btn = document.querySelector('[data-testid="record-btn"]') as HTMLButtonElement;
    await fireEvent.click(btn); // 開始
    await fireEvent.click(btn); // 停止 → transcribing=true
    await emitEvent("progress", 42);
    expect(await screen.findByRole("progressbar")).toBeInTheDocument();
  });

  it("status イベントでステータス文言が表示される", async () => {
    render(App);
    await waitForListeners();
    await emitEvent("status", "音声を読み込み中…");
    expect(await screen.findByText("音声を読み込み中…")).toBeInTheDocument();
  });
});
