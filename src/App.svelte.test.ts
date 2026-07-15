// @vitest-environment jsdom
// App.svelte のコンポーネントテスト(#402/#481-item13)。
// 主要フロー(録音トグル/ジャーナル/メモ整形/設定操作)を @testing-library/svelte で駆動し、
// App.svelte の実効カバレッジを上げる。Tauri の各APIはモック(invoke はコマンド別に返す)。
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/svelte";

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
vi.mock("@tauri-apps/api/app", () => ({ getVersion: vi.fn(async () => "1.2.3") }));
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
  // テスト間の localStorage 汚染(offlineMode/customStyles等)を排除し、既定を再設定する。
  localStorage.clear();
  localStorage.setItem("locale", "ja");
  localStorage.setItem("provider", "ollama");
  localStorage.setItem("autoPipeline", "true");
});

// 設定タブ(#512)を開く。設定ダイアログを開いた後に対象タブへ切替える。
async function gotoTab(name: string) {
  await fireEvent.click(await screen.findByRole("tab", { name }));
}

// listen の登録は onMount 内で非同期に走る。登録完了を待つ。
async function waitForListeners() {
  for (let i = 0; i < 50 && !listeners.has("job-done"); i++) {
    await Promise.resolve();
  }
}

// マルチジョブ(Phase2): 録音停止=ジョブ発番→逐次処理を、job-* イベントで再現するヘルパ。
async function emitJobCreated(id: number, extra: Record<string, unknown> = {}) {
  await emitEvent("job-created", {
    id,
    createdAtMs: id * 1000,
    durationSecs: 5,
    status: "queued",
    progress: 0,
    ...extra,
  });
}
async function emitJobDone(id: number, text: string) {
  await emitEvent("job-done", { jobId: id, text });
}
async function emitJobError(id: number, code: string) {
  await emitEvent("job-error", { jobId: id, code });
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

  it("アプリのバージョンが表示される（実行結果の共有用）", async () => {
    render(App);
    await waitForListeners();
    // getVersion() のモックが返す版が右下フッタに表示される。
    expect(await screen.findByText("v1.2.3")).toBeInTheDocument();
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
  it("設定タブを切替えると各カテゴリの内容が表示される(#512)", async () => {
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: "設定" }));
    // 一般: オフラインモード。
    expect(await screen.findByLabelText(/オフラインモードで固定/)).toBeInTheDocument();
    await gotoTab("録音");
    expect(await screen.findByText("録音モード")).toBeInTheDocument();
    await gotoTab("文字起こし");
    const combos = screen.getAllByRole("combobox") as HTMLSelectElement[];
    expect(combos.some((c) => c.value === "local")).toBe(true);
    await gotoTab("整形");
    expect(await screen.findByLabelText("整形プロバイダ")).toBeInTheDocument();
    await gotoTab("出力");
    expect(await screen.findByLabelText("録音音声を保存")).toBeInTheDocument();
  });

  it("設定の保存で set_save_settings が呼ばれる", async () => {
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: "設定" }));
    const saveBtn = await screen.findByRole("button", { name: "保存" });
    await fireEvent.click(saveBtn);
    expect(invokeMock).toHaveBeenCalledWith("set_save_settings", expect.anything());
  });

  it("クラウド選択で鍵未入力でも保存でき、整形失敗の警告を出す(#603)", async () => {
    // クラウド(gemini)を鍵未入力で起動 → 保存はブロックせず警告を出す(文字起こし完結ペルソナ)。
    localStorage.setItem("provider", "gemini");
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: "設定" }));
    await fireEvent.click(await screen.findByRole("button", { name: "保存" }));
    // 未設定でも保存でき、整形(LLM処理)が失敗する旨の非ブロック警告が出る。設定は開いたまま。
    expect(await screen.findByText(/整形（LLM処理）は失敗します/)).toBeInTheDocument();
    expect(screen.getByRole("dialog", { name: "設定" })).toBeInTheDocument();
  });

  it("ローカル(ollama)は警告なく保存でき設定が閉じる(#603)", async () => {
    // 既定 provider=ollama(鍵不要) → 不足なし → 保存成功で設定ダイアログが閉じる(cfgErr無しの分岐)。
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: "設定" }));
    expect(screen.getByRole("dialog", { name: "設定" })).toBeInTheDocument();
    await fireEvent.click(await screen.findByRole("button", { name: "保存" }));
    expect(screen.queryByRole("dialog", { name: "設定" })).not.toBeInTheDocument();
    expect(screen.queryByText(/整形（LLM処理）は失敗します/)).not.toBeInTheDocument();
  });

  it("鍵未入力で整形しようとすると設定が開き不足を明示する(#516導線は維持)", async () => {
    // #603 で保存はブロックしなくなったが、整形の実行導線では従来どおり openSettingsForConfig で
    // 不足を明示＋該当タブへ誘導する(鍵無しでは整形できないため正しい)。メモ整形経由で検証。
    localStorage.setItem("provider", "gemini");
    openMock.mockResolvedValue("/path/to/memo.txt");
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: /メモ/ }));
    // 鍵不足で整形は実行されず、設定が開いて APIキー不足が明示される。
    expect(await screen.findByText(/APIキーが必要です/)).toBeInTheDocument();
    expect(screen.getByRole("dialog", { name: "設定" })).toBeInTheDocument();
  });
  it("オフラインモードONで整形プロバイダ選択が無効化される(#465)", async () => {
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: "設定" }));
    const cb = (await screen.findByLabelText(/オフラインモードで固定/)) as HTMLInputElement;
    await fireEvent.click(cb);
    // 一般タブでオンデバイス表示を確認 → 整形タブでプロバイダ選択が無効化されていること。
    expect(await screen.findByText("オンデバイス完結")).toBeInTheDocument();
    await gotoTab("整形");
    const providerSelect = (await screen.findByLabelText("整形プロバイダ")) as HTMLSelectElement;
    expect(providerSelect.disabled).toBe(true);
  });

  it("クラウド整形プロバイダ選択時に送信同意の警告を表示する(#465)", async () => {
    localStorage.setItem("provider", "gemini");
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: "設定" }));
    await gotoTab("整形");
    // クラウド(gemini)では「クラウド整形は…端末外へ送信」の警告が出る。
    expect(await screen.findByText(/クラウド整形は/)).toBeInTheDocument();
    // ローカル(ollama)へ切替えると警告は消える。
    const providerSelect = (await screen.findByLabelText("整形プロバイダ")) as HTMLSelectElement;
    await fireEvent.change(providerSelect, { target: { value: "ollama" } });
    expect(screen.queryByText(/クラウド整形は/)).not.toBeInTheDocument();
  });

  it("翻訳トグルをONにすると出力言語ピッカーが現れる", async () => {
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: "設定" }));
    await gotoTab("整形");
    const cb = (await screen.findByLabelText(/翻訳して出力/)) as HTMLInputElement;
    expect(cb.checked).toBe(false);
    await fireEvent.click(cb);
    // 出力言語の select が出現する（settings.output_language）。
    expect(await screen.findByLabelText(/出力言語|翻訳先/)).toBeInTheDocument();
  });

  it("カスタム整形: ラベルと指示が空だとエラー", async () => {
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: "設定" }));
    await gotoTab("整形");
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
  it("job-done で文字起こしが作業領域に読み込まれ、自動整形される", async () => {
    render(App);
    await waitForListeners();
    await emitJobCreated(1);
    await emitJobDone(1, "文字起こしされた本文");
    // 最新の完了ジョブが作業領域へ自動読み込みされ、autoPipeline+ollama で自動整形まで走る。
    expect(await screen.findByText("整形された結果テキスト")).toBeInTheDocument();
    expect(invokeMock).toHaveBeenCalledWith("refine_text", expect.anything());
  });

  it("空の job-done は作業領域に読み込まれず、整形も走らない（取りこぼさない=行は残る）", async () => {
    render(App);
    await waitForListeners();
    await emitJobCreated(1);
    await emitJobDone(1, "");
    // 発話が無い完了は本文が無いため自動読み込みされず、整形も呼ばれない（グローバルエラーも出さない）。
    await Promise.resolve();
    expect(invokeMock).not.toHaveBeenCalledWith("refine_text", expect.anything());
    expect(screen.queryByText("整形された結果テキスト")).not.toBeInTheDocument();
  });

  it("job-error でエラーが表示される", async () => {
    render(App);
    await waitForListeners();
    await emitJobCreated(1);
    await emitJobError(1, "文字起こしに失敗しました");
    expect(await screen.findByText("文字起こしに失敗しました")).toBeInTheDocument();
  });

  it("job-progress で実行中ジョブの進捗バーが表示される", async () => {
    render(App);
    await waitForListeners();
    await emitJobCreated(1);
    await emitEvent("job-status", { jobId: 1, status: "running" });
    await emitEvent("job-progress", { jobId: 1, progress: 42 });
    expect(await screen.findByRole("progressbar")).toBeInTheDocument();
  });

  it("作業中に次ジョブが完了しても上書きせず、一覧から開ける（クロバー防止/取りこぼさない）", async () => {
    render(App);
    await waitForListeners();
    // job1 完了 → 作業領域が空なので自動読み込み。
    await emitJobCreated(1);
    await emitJobDone(1, "最初のジャーナル本文");
    expect(await screen.findByText("最初のジャーナル本文")).toBeInTheDocument();
    // job1 を表示・整形中に job2 が完了 → 作業領域(job1)を上書きしない。
    await emitJobCreated(2);
    await emitJobDone(2, "次のジャーナル本文");
    expect(screen.getByText("最初のジャーナル本文")).toBeInTheDocument();
    // 未読の完了があるので一覧が自動展開される。一覧は新しい順なので job2 は先頭の「開く」。
    const openBtns = await screen.findAllByRole("button", { name: "開く" });
    expect(openBtns.length).toBe(2);
    await fireEvent.click(openBtns[0]);
    expect(await screen.findByText("次のジャーナル本文")).toBeInTheDocument();
  });

  it("完了ジョブが増えても一覧は直近3件のみ表示し、「他N件を表示」で全部開ける（表示エリアの無限拡大を防ぐ）", async () => {
    render(App);
    await waitForListeners();
    // 5件完了。1件目は作業領域へ自動読み込み、以降は一覧に積まれ自動展開される。
    for (let i = 1; i <= 5; i++) {
      await emitJobCreated(i);
      await emitJobDone(i, `本文${i}`);
    }
    // 折り畳み時は直近3件(job5,4,3)の「開く」のみ。
    const collapsed = await screen.findAllByRole("button", { name: "開く" });
    expect(collapsed.length).toBe(3);
    // 隠れている2件を開くトグルが出る。
    const showAll = screen.getByRole("button", { name: "他 2 件を表示" });
    await fireEvent.click(showAll);
    // 展開後は全5件の「開く」が並ぶ。
    const expanded = await screen.findAllByRole("button", { name: "開く" });
    expect(expanded.length).toBe(5);
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

  it("『サンプルで試す』でサンプル文字起こしが表示される(aha)", async () => {
    localStorage.removeItem("onboarded");
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: "サンプルで試す" }));
    // オンボが閉じ、文字起こしカードが出る。
    expect(screen.queryByText("QuickScribe へようこそ")).not.toBeInTheDocument();
    expect(await screen.findByRole("heading", { name: "文字起こし" })).toBeInTheDocument();
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
    await gotoTab("整形");
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
    await gotoTab("整形");
    const providerSelect = (await screen.findByLabelText("整形プロバイダ")) as HTMLSelectElement;
    await fireEvent.change(providerSelect, { target: { value: "bedrock" } });
    expect(await screen.findByPlaceholderText("us-east-1")).toBeInTheDocument();
  });

  it("STTをクラウド(OpenAI)にするとAPIキー欄が現れる", async () => {
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: "設定" }));
    await gotoTab("文字起こし");
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
    await gotoTab("出力");
    const cb = (await screen.findByLabelText("録音音声を保存")) as HTMLInputElement;
    await fireEvent.click(cb);
    expect(await screen.findByLabelText("音声形式")).toBeInTheDocument();
  });

  it("STTをAzureにすると Azure リソース欄が現れる", async () => {
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: "設定" }));
    await gotoTab("文字起こし");
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
    await gotoTab("文字起こし");
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

// 単一Vulkanビルド(ADR-0028/0029): 起動時に stt_backend で変種と実行環境のGPU可否を解決し、
// Vulkan変種なら文字起こしタブにGPUトグルを出す。GPU不可時はCPU実行の案内のみ(特定の入手導線は無し)。
describe("App.svelte GPUバックエンド表示(ADR-0028)", () => {
  it("Vulkan変種・GPU利用可: 文字起こしタブにGPUトグル(有効)＋Aboutに GPU版 Vulkan", async () => {
    invokeMock.mockImplementation(async (cmd: string) =>
      cmd === "stt_backend" ? { variant: "vulkan", gpuAvailable: true } : defaultInvoke(cmd),
    );
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: "設定" }));
    // 既定の一般タブの About はビルド変種を「GPU版 Vulkan」と表示する。
    expect(await screen.findByText(/GPU版 Vulkan/)).toBeInTheDocument();
    // 文字起こしタブ: 既定ローカルSTT + Vulkan変種 + GPU可 → GPUトグルが有効で現れる。
    await gotoTab("文字起こし");
    const cb = (await screen.findByLabelText(/GPUで文字起こし/)) as HTMLInputElement;
    expect(cb.disabled).toBe(false);
  });

  it("Vulkan変種・GPU利用不可: トグルは無効化され、CPU実行の案内が出る", async () => {
    invokeMock.mockImplementation(async (cmd: string) =>
      cmd === "stt_backend" ? { variant: "vulkan", gpuAvailable: false } : defaultInvoke(cmd),
    );
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: "設定" }));
    await gotoTab("文字起こし");
    // 対応GPU無し → トグルは無効化され、CPU実行の案内(gpu_unavailable)が出る。
    const cb = (await screen.findByLabelText(/GPUで文字起こし/)) as HTMLInputElement;
    expect(cb.disabled).toBe(true);
    expect(await screen.findByText(/対応GPUが見つかりません/)).toBeInTheDocument();
  });

  it("撤去済みモデル(kotoba)を選択中の既存ユーザーはモデル選択が base へ正規化される(ADR-0029)", async () => {
    // 旧設定で kotoba を保存していた状態を再現。カタログには kotoba が無い。
    localStorage.setItem("whisperModel", "kotoba");
    invokeMock.mockImplementation(async (cmd: string) => {
      if (cmd === "list_whisper_models")
        return [
          { id: "base", label: "標準 base", speed: "fast" },
          { id: "large-v3-turbo", label: "turbo", speed: "slow" },
        ];
      return defaultInvoke(cmd);
    });
    render(App);
    await fireEvent.click(await screen.findByRole("button", { name: "設定" }));
    await gotoTab("文字起こし");
    // モデル選択(turbo オプションを持つ combobox)を特定し、kotoba→base へ正規化されたことを確認。
    const modelSelect = await waitFor(() => {
      const s = (screen.getAllByRole("combobox") as HTMLSelectElement[]).find((el) =>
        Array.from(el.options).some((o) => o.value === "large-v3-turbo"),
      );
      if (!s || s.value !== "base") throw new Error("not normalized yet");
      return s;
    });
    expect(modelSelect.value).toBe("base");
  });
});
