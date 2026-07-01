// @vitest-environment jsdom
// App.svelte のコンポーネントテスト(#402/#481-item13)。
// 主要フロー(録音トグル/ジャーナル/メモ整形/設定操作)を @testing-library/svelte で駆動し、
// App.svelte の実効カバレッジを上げる。Tauri の各APIはモック(invoke はコマンド別に返す)。
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";

// i18n は import 時に起動ロケールを解決する。App の import より前に locale=ja と
// provider=ollama(鍵不要=整形が通る)を固定する(hoistedは全importより先に走る)。
const {
  invokeMock,
  openMock,
  listeners,
  listenMock,
  checkMock,
  enableMock,
  disableMock,
  relaunchMock,
} = vi.hoisted(() => {
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
  return {
    invokeMock: vi.fn(),
    openMock: vi.fn(),
    listeners,
    listenMock,
    checkMock: vi.fn(),
    enableMock: vi.fn(),
    disableMock: vi.fn(),
    relaunchMock: vi.fn(),
  };
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
vi.mock("@tauri-apps/plugin-updater", () => ({ check: checkMock }));
vi.mock("@tauri-apps/plugin-process", () => ({ relaunch: relaunchMock }));
vi.mock("@tauri-apps/plugin-autostart", () => ({
  enable: enableMock,
  disable: disableMock,
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
  checkMock.mockReset();
  checkMock.mockResolvedValue(null);
  enableMock.mockReset();
  enableMock.mockResolvedValue(undefined);
  disableMock.mockReset();
  disableMock.mockResolvedValue(undefined);
  relaunchMock.mockReset();
  relaunchMock.mockResolvedValue(undefined);
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

describe("App.svelte オンボーディング", () => {
  it("初回起動でオンボーディングが表示され、スキップで消える", async () => {
    localStorage.removeItem("onboarded");
    render(App);
    expect(await screen.findByText("QuickScribe へようこそ")).toBeInTheDocument();
    await fireEvent.click(await screen.findByRole("button", { name: "スキップ" }));
    expect(screen.queryByText("QuickScribe へようこそ")).not.toBeInTheDocument();
  });
});

describe("App.svelte 更新確認", () => {
  it("更新を確認で最新メッセージが表示される", async () => {
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: "設定" }));
    await fireEvent.click(await screen.findByRole("button", { name: "更新を確認" }));
    expect(await screen.findByText("お使いのバージョンは最新です。")).toBeInTheDocument();
  });
});

describe("App.svelte エントリを開く", () => {
  it("エントリをクリックすると本文が読み込まれ表示される", async () => {
    invokeMock.mockImplementation(async (cmd: string) =>
      cmd === "read_text_file" ? "エントリの本文内容" : defaultInvoke(cmd),
    );
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: "ジャーナル" }));
    await fireEvent.click(await screen.findByText("プレビュー本文"));
    expect(await screen.findByText("エントリの本文内容")).toBeInTheDocument();
    expect(invokeMock).toHaveBeenCalledWith("read_text_file", { path: "/x/refined-1.md" });
  });
});

describe("App.svelte カスタム整形パターン", () => {
  it("ラベルと指示を入力して追加すると一覧に現れる", async () => {
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: "設定" }));
    // カスタム整形パターンは折りたたみ <details> 内なので開く。
    await fireEvent.click(await screen.findByText("カスタム整形パターン"));
    const nameInput = await screen.findByPlaceholderText(/パターン名/);
    const instrInput = await screen.findByPlaceholderText(/AIへの指示/);
    await fireEvent.input(nameInput, { target: { value: "議事録モード" } });
    await fireEvent.input(instrInput, { target: { value: "決定事項とToDoを分ける" } });
    await fireEvent.click(await screen.findByRole("button", { name: "カスタムパターンを追加" }));
    // 追加後、カスタム一覧と整形スタイルの選択肢の両方に現れる。
    expect((await screen.findAllByText("議事録モード")).length).toBeGreaterThan(0);
  });
});

describe("App.svelte モーメンタリ録音", () => {
  it("record-press イベントで録音が開始される（モーメンタリ）", async () => {
    localStorage.setItem("recordMode", "momentary");
    render(App);
    await waitForListeners();
    await emitEvent("record-press", null);
    expect(invokeMock).toHaveBeenCalledWith("start_recording", expect.anything());
    localStorage.removeItem("recordMode");
  });
});

describe("App.svelte AWSプロバイダ", () => {
  it("プロバイダを Bedrock にすると region 入力が現れる", async () => {
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: "設定" }));
    const providerSelect = (await screen.findByLabelText("整形プロバイダ")) as HTMLSelectElement;
    await fireEvent.change(providerSelect, { target: { value: "bedrock" } });
    expect(await screen.findByPlaceholderText("us-east-1")).toBeInTheDocument();
  });

  it("STTをクラウド(OpenAI)にするとAPIキー欄が現れる", async () => {
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: "設定" }));
    // STTプロバイダの select は value=local で一意に特定できる。
    const combos = screen.getAllByRole("combobox") as HTMLSelectElement[];
    const sttSelect = combos.find((c) => c.value === "local")!;
    expect(sttSelect).toBeTruthy();
    const before = document.querySelectorAll('input[type="password"]').length;
    await fireEvent.change(sttSelect, { target: { value: "openai" } });
    // クラウドSTTの鍵入力(password)が増える。
    const after = document.querySelectorAll('input[type="password"]').length;
    expect(after).toBeGreaterThan(before);
  });
});

describe("App.svelte 横断発見", () => {
  it("エントリが1件のときは横断発見ボタンが出ない", async () => {
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: "ジャーナル" }));
    await screen.findByText("プレビュー本文");
    expect(screen.queryByRole("button", { name: /横断発見/ })).not.toBeInTheDocument();
  });

  it("2件以上あると横断発見が実行され結果が表示される", async () => {
    invokeMock.mockImplementation(async (cmd: string) => {
      if (cmd === "list_entries")
        return [
          {
            path: "/a.md",
            name: "a.md",
            created: "2026-06-01T10:00:00",
            kind: "refined",
            tags: [],
            preview: "A本文",
          },
          {
            path: "/b.md",
            name: "b.md",
            created: "2026-06-02T10:00:00",
            kind: "refined",
            tags: [],
            preview: "B本文",
          },
        ];
      if (cmd === "refine_text") return "横断発見の結果";
      return undefined;
    });
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: "ジャーナル" }));
    await screen.findByText("A本文");
    await fireEvent.click(await screen.findByRole("button", { name: /横断発見/ }));
    expect(await screen.findByText("横断発見の結果")).toBeInTheDocument();
    expect(invokeMock).toHaveBeenCalledWith("refine_text", expect.anything());
  });
});

describe("App.svelte 音声保存設定", () => {
  it("録音音声を保存をONにすると音声形式の選択が現れる", async () => {
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: "設定" }));
    const cb = (await screen.findByLabelText("録音音声を保存")) as HTMLInputElement;
    await fireEvent.click(cb);
    expect(await screen.findByLabelText("音声形式")).toBeInTheDocument();
  });

  it("STTをAzureにすると Azure リソース欄が現れる", async () => {
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: "設定" }));
    const combos = screen.getAllByRole("combobox") as HTMLSelectElement[];
    const sttSelect = combos.find((c) => c.value === "local")!;
    await fireEvent.change(sttSelect, { target: { value: "azure" } });
    // Azure 固有の追加フィールドが現れる(password鍵 + azureリソース)。
    expect(document.querySelectorAll('input[type="password"]').length).toBeGreaterThan(0);
  });
});

describe("App.svelte エントリ表示の閉じる", () => {
  it("エントリを開いてから戻ると一覧へ戻る", async () => {
    invokeMock.mockImplementation(async (cmd: string) =>
      cmd === "read_text_file" ? "本文ABC" : defaultInvoke(cmd),
    );
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: "ジャーナル" }));
    await fireEvent.click(await screen.findByText("プレビュー本文"));
    expect(await screen.findByText("本文ABC")).toBeInTheDocument();
    // 閉じるで一覧へ。
    const closeButtons = screen.getAllByRole("button", { name: "閉じる" });
    await fireEvent.click(closeButtons[closeButtons.length - 1]);
    expect(screen.queryByText("本文ABC")).not.toBeInTheDocument();
  });
});

describe("App.svelte 習慣ストリーク", () => {
  it("直近に連続した記録があるとストリークバッジが出る", async () => {
    const iso = (d: Date) => d.toISOString().slice(0, 10);
    const today = new Date();
    const y1 = new Date(today.getTime() - 86400000);
    const y2 = new Date(today.getTime() - 2 * 86400000);
    invokeMock.mockImplementation(async (cmd: string) => {
      if (cmd === "list_entries")
        return [today, y1, y2].map((d, i) => ({
          path: `/e${i}.md`,
          name: `e${i}.md`,
          created: `${iso(d)}T09:00:00`,
          kind: "refined",
          tags: [],
          preview: `本文${i}`,
        }));
      return undefined;
    });
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: "ジャーナル" }));
    expect(await screen.findByText(/連続/)).toBeInTheDocument();
  });
});

describe("App.svelte 追加フロー", () => {
  it("コピー失敗時はエラーメッセージが出る", async () => {
    const writeText = vi.fn().mockRejectedValue(new Error("denied"));
    Object.assign(navigator, { clipboard: { writeText } });
    openMock.mockResolvedValue("/m.txt");
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: /メモ/ }));
    await screen.findByText("整形された結果テキスト");
    await fireEvent.click(await screen.findByRole("button", { name: /コピー/ }));
    expect(writeText).toHaveBeenCalled();
  });

  it("横断発見の結果を閉じると消える", async () => {
    invokeMock.mockImplementation(async (cmd: string) => {
      if (cmd === "list_entries")
        return [
          {
            path: "/a.md",
            name: "a.md",
            created: "2026-06-01T10:00:00",
            kind: "refined",
            tags: [],
            preview: "A本文",
          },
          {
            path: "/b.md",
            name: "b.md",
            created: "2026-06-02T10:00:00",
            kind: "refined",
            tags: [],
            preview: "B本文",
          },
        ];
      if (cmd === "refine_text") return "横断発見の結果Z";
      return undefined;
    });
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: "ジャーナル" }));
    await screen.findByText("A本文");
    await fireEvent.click(await screen.findByRole("button", { name: /横断発見/ }));
    expect(await screen.findByText("横断発見の結果Z")).toBeInTheDocument();
    // 発見結果の閉じるボタン群のいずれかで消える。
    const closes = screen.getAllByRole("button", { name: "閉じる" });
    await fireEvent.click(closes[closes.length - 1]);
    expect(screen.queryByText("横断発見の結果Z")).not.toBeInTheDocument();
  });

  it("STTをクラウドにして保存すると set_stt_settings が呼ばれる", async () => {
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: "設定" }));
    const combos = screen.getAllByRole("combobox") as HTMLSelectElement[];
    const sttSelect = combos.find((c) => c.value === "local")!;
    await fireEvent.change(sttSelect, { target: { value: "openai" } });
    await fireEvent.click(await screen.findByRole("button", { name: "保存" }));
    expect(invokeMock).toHaveBeenCalledWith("set_stt_settings", expect.anything());
  });

  it("自動起動トグルで enable/disable が呼ばれる", async () => {
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: "設定" }));
    const cb = (await screen.findByLabelText(/自動起動|OS起動時|ログイン時/)) as HTMLInputElement;
    await fireEvent.click(cb);
    expect(enableMock).toHaveBeenCalled();
  });

  it("更新ありのとき: ダウンロード→インストールが走り再起動導線が出る", async () => {
    checkMock.mockResolvedValue({
      version: "9.9.9",
      downloadAndInstall: async (
        cb: (e: { event: string; data: { contentLength?: number; chunkLength?: number } }) => void,
      ) => {
        cb({ event: "Started", data: { contentLength: 100 } });
        cb({ event: "Progress", data: { chunkLength: 100 } });
      },
    });
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: "設定" }));
    await fireEvent.click(await screen.findByRole("button", { name: "更新を確認" }));
    // バージョンが表示され、再起動ボタンが現れる。
    expect(await screen.findByText(/9\.9\.9/)).toBeInTheDocument();
  });
});
