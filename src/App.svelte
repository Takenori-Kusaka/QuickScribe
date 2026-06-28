<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/plugin-dialog";
  import { check } from "@tauri-apps/plugin-updater";
  import { relaunch } from "@tauri-apps/plugin-process";
  import {
    enable as enableAutostart,
    disable as disableAutostart,
    isEnabled as isAutostartEnabled,
  } from "@tauri-apps/plugin-autostart";
  import { onMount } from "svelte";
  import { estimateRemaining, formatRemaining } from "./lib/note";
  import { parseCorrections, applyCorrections, type Correction } from "./lib/corrections";
  import { errorText } from "./lib/errors";
  import { modal } from "./lib/a11y";
  import {
    type Provider,
    type SttProvider,
    type CustomStyle,
    ALL_PROVIDERS,
    PROVIDER_LABELS,
    LOCAL_PROVIDERS,
    AWS_PROVIDERS,
    FALLBACK_MODELS,
    KEY_PLACEHOLDERS,
    STT_LABELS,
    STT_CLOUD,
    STT_KEY_PLACEHOLDERS,
    STT_MODEL_PLACEHOLDERS,
    REFINE_STYLES,
    MODEL_TTL_MS,
    DISCOVERY_MAX,
    DEFAULT_SHORTCUT,
    MAX_INPUT_MB,
    SUPPORTED_AUDIO_EXTS,
  } from "./lib/constants";

  let recording = $state(false);
  let error = $state<string | null>(null);
  let startedAt = $state<number | null>(null);
  let status = $state<string>("");
  let progress = $state<number>(0);
  let eta = $state<string>("");
  let transcribeStartMs = $state<number | null>(null);
  let segments = $state<string[]>([]);
  let transcript = $state<string | null>(null);
  let refined = $state<string | null>(null);
  // 現在の整形結果がどのスタイルで作られたか(結果からの再整形チップの強調 / S3.5)。
  let refinedStyle = $state<string>("structured");
  let refining = $state(false);
  let busy = $state(false);
  // バックグラウンド文字起こし中（録音ボタンはブロックしない＝録音の非同期化）。
  let transcribing = $state(false);

  // 設定（localStorageに保存。秘密情報はローカル端末内のみ）。
  // 整形プロバイダ: Gemini / Anthropic / OpenAI / ローカル(Ollama) ＋
  // AWS Bedrock / Claude Platform on AWS(ADR-0011)。BYO鍵/資格情報 / ADR-0005。
  // プロバイダ定義・定数は src/lib/constants.ts に集約(SSOT / #401 Phase0)。

  let showSettings = $state(false);

  // 保管庫エントリの横断（S4.3 Phase1）。一覧＋タグ/全文絞り込み＋閲覧。
  type EntrySummary = {
    path: string;
    name: string;
    created: string;
    kind: string;
    tags: string[];
    preview: string;
  };
  let showEntries = $state(false);
  let entries = $state<EntrySummary[]>([]);
  let entriesLoading = $state(false);
  let entrySearch = $state<string>("");
  let selectedTags = $state<string[]>([]);
  let viewingEntry = $state<{ name: string; content: string } | null>(null);
  async function loadEntries() {
    entriesLoading = true;
    try {
      entries = await invoke<EntrySummary[]>("list_entries");
    } catch (e) {
      error = `ジャーナルの読み込みに失敗しました: ${errorText(e)}`;
      entries = [];
    } finally {
      entriesLoading = false;
    }
  }
  function openEntriesPanel() {
    showEntries = true;
    viewingEntry = null;
    void loadEntries();
  }
  function toggleTagFilter(tag: string) {
    selectedTags = selectedTags.includes(tag)
      ? selectedTags.filter((t) => t !== tag)
      : [...selectedTags, tag];
  }
  // 全エントリのタグ集合（絞り込みチップ用・出現頻度降順）。
  const allTags = $derived.by(() => {
    const count = new Map<string, number>();
    for (const e of entries) for (const t of e.tags) count.set(t, (count.get(t) ?? 0) + 1);
    return [...count.entries()].sort((a, b) => b[1] - a[1]).map(([t]) => t);
  });
  // 検索語(name/preview/tags)＋選択タグ(AND)で絞り込んだ一覧。
  const filteredEntries = $derived.by(() => {
    const q = entrySearch.trim().toLowerCase();
    return entries.filter((e) => {
      if (selectedTags.length > 0 && !selectedTags.every((t) => e.tags.includes(t))) return false;
      if (!q) return true;
      const hay = `${e.name} ${e.preview} ${e.tags.join(" ")}`.toLowerCase();
      return hay.includes(q);
    });
  });
  // エントリ種別の日本語ラベル（生の文字起こし/整形済み/メモ）。
  function kindLabel(kind: string): string {
    return kind === "transcript"
      ? "文字起こし"
      : kind === "refined"
        ? "整形済み"
        : kind === "note"
          ? "メモ"
          : kind;
  }
  async function openEntry(e: EntrySummary) {
    try {
      const content = await invoke<string>("read_text_file", { path: e.path });
      viewingEntry = { name: e.name, content };
    } catch (err) {
      error = `エントリを開けませんでした: ${err}`;
    }
  }

  // 横断発見（S4.3 Phase2）。絞り込んだ過去エントリ群をAIで読み解く。
  const DISCOVERY_INSTRUCTION = [
    "- これは過去の複数のジャーナル（日付付き）です。横断して読み解いてください。",
    "- 繰り返し現れるテーマ・関心、感情やトーンの変化・傾向、未解決の問い、次の一歩の候補を抽出する",
    "- 具体的な日付や言葉に触れつつ、本人が気づいていない接続・パターンを丁寧に提示する",
    "- 決めつけず、本人の言葉とニュアンスを尊重する。事実を捏造しない",
  ].join("\n");
  let discovering = $state(false);
  let discoveryResult = $state<string | null>(null);
  let discoveryTruncated = $state(false);
  async function discoverAcross() {
    const cfgErr = refineConfigError();
    if (cfgErr) {
      error = `${cfgErr}設定から入力してください。`;
      showEntries = false;
      showSettings = true;
      return;
    }
    const targets = filteredEntries.slice(0, DISCOVERY_MAX);
    discoveryTruncated = filteredEntries.length > DISCOVERY_MAX;
    if (targets.length < 2) {
      error = "横断発見には2件以上のエントリが必要です（タグ/検索で絞ってからお試しください）。";
      return;
    }
    error = null;
    discovering = true;
    discoveryResult = null;
    try {
      await resolveCurrentModel();
      const parts: string[] = [];
      for (const e of targets) {
        const content = await invoke<string>("read_text_file", { path: e.path });
        const tagStr = e.tags.map((t) => `#${t}`).join(" ");
        parts.push(`### ${e.created} ${tagStr}\n${content}`);
      }
      const args = refineArgs();
      args.text = parts.join("\n\n---\n\n");
      args.customInstruction = DISCOVERY_INSTRUCTION;
      args.save = false; // 発見結果は一時表示（保管庫を汚さない）。
      delete args.tags;
      discoveryResult = await invoke<string>("refine_text", args);
    } catch (e) {
      error = `横断発見に失敗しました: ${errorText(e)}`;
    } finally {
      discovering = false;
    }
  }

  let provider = $state<Provider>("gemini");
  // プロバイダごとに鍵を保持する（切替時に再入力不要）。
  let apiKeys = $state<Record<Provider, string>>({
    gemini: "",
    anthropic: "",
    openai: "",
    ollama: "",
    bedrock: "",
    "claude-aws": "",
  });
  // 実行時に解決した最新モデルID（表示・整形に使用）。
  let resolvedModel = $state<Record<Provider, string>>({
    gemini: "",
    anthropic: "",
    openai: "",
    ollama: "",
    bedrock: "",
    "claude-aws": "",
  });
  let resolvingModel = $state(false);
  let updateMsg = $state<string>("");

  // AWS資格情報(Bedrock / Claude Platform on AWS 共通 / ADR-0011)。
  // ※localStorage平文保存は当面。秘密情報のkeyring化は S3.2 で対応(優先度を上げる)。
  let awsRegion = $state<string>("us-east-1");
  let awsWorkspaceId = $state<string>(""); // Claude Platform on AWS で必須
  let awsAuthMode = $state<"sigv4" | "apikey">("sigv4");
  let awsAccessKey = $state<string>("");
  let awsSecretKey = $state<string>("");
  let awsSessionToken = $state<string>(""); // 一時credのときのみ
  // AWS Bedrock のモデルID(リージョン/アカウント依存のため手入力可)。
  let bedrockModel = $state<string>("");

  // 録音トグルのグローバルホットキー。内部・保存・登録は Tauri アクセラレータ表記
  // ("CommandOrControl+Shift+R")だが、表示は実行環境に合わせて Ctrl/Cmd に変換する。
  let recordShortcut = $state<string>(DEFAULT_SHORTCUT);
  let shortcutMsg = $state<string>("");
  // 録音モード（S1.5 / ADR-0014）: toggle=1押しで開始/停止 / momentary=押している間だけ録音。
  let recordMode = $state<"toggle" | "momentary">("toggle");
  // ホットキーのキャプチャモード（変更ボタン押下中＝キー入力待ち）。
  let capturing = $state<boolean>(false);

  // 実行環境(OS)を判定し、修飾キーを親しみやすい表記にする。
  const IS_MAC =
    typeof navigator !== "undefined" &&
    /mac/i.test(`${navigator.userAgent} ${navigator.platform ?? ""}`);
  // タスクバーウィジェットは Windows 専用機能。設定UIの出し分けに使う。
  const IS_WINDOWS =
    typeof navigator !== "undefined" &&
    /win/i.test(`${navigator.userAgent} ${navigator.platform ?? ""}`);
  // タスクバー上のウィジェット表示（Windows）。既定ON。設定でON/OFF可能。
  let taskbarWidget = $state<boolean>(true);

  // OSログイン時の自動起動（S6.3）。実体はOSに登録されるため、状態はOSから取得する。
  let autoStart = $state<boolean>(false);
  async function loadAutoStart() {
    try {
      autoStart = await isAutostartEnabled();
    } catch (e) {
      console.error("isAutostartEnabled failed", e);
    }
  }
  async function onAutoStartChange() {
    try {
      if (autoStart) await enableAutostart();
      else await disableAutostart();
      // OSの実際の登録状態に同期（失敗時のずれを防ぐ）。
      autoStart = await isAutostartEnabled();
    } catch (e) {
      error = `自動起動の設定に失敗しました: ${errorText(e)}`;
      autoStart = await isAutostartEnabled().catch(() => autoStart);
    }
  }

  // 録音ソース選択（S1.2/S1.3 / #18 #19）。マイク入力＋出力デバイスのループバックを統一。
  // inputDevice: 入力=デバイス名 / ループバック=レンダーデバイスID（空=OS既定）。
  type AudioSource = { id: string; label: string; kind: string };
  let inputDevice = $state<string>("");
  let inputDeviceKind = $state<string>("input");
  let audioSources = $state<AudioSource[]>([]);

  // 利用可能な録音ソースを列挙する（設定UIのプルダウン用）。失敗時は空のまま既定運用。
  async function loadAudioSources() {
    try {
      audioSources = await invoke<AudioSource[]>("list_audio_sources");
    } catch (e) {
      console.error("list_audio_sources failed", e);
      audioSources = [];
    }
  }

  // プルダウンは "kind|id" を値に使う（id がレンダーデバイスIDでも安全に分解できる）。
  function onSourceChange(e: Event) {
    const v = (e.currentTarget as HTMLSelectElement).value;
    const sep = v.indexOf("|");
    inputDeviceKind = v.slice(0, sep);
    inputDevice = v.slice(sep + 1);
  }

  // タスクバーウィジェットの表示有効/無効をバックエンドへ反映する（Windowsのみ実体動作）。
  async function applyTaskbarWidget() {
    try {
      await invoke("set_taskbar_widget", { enabled: taskbarWidget });
    } catch (e) {
      console.error("set_taskbar_widget failed", e);
    }
  }
  function displayShortcut(accel: string): string {
    return accel
      .split("+")
      .map((t) => {
        switch (t) {
          case "CommandOrControl":
          case "CmdOrCtrl":
            return IS_MAC ? "Cmd" : "Ctrl";
          case "Control":
            return "Ctrl";
          case "Super":
          case "Meta":
          case "Command":
            return IS_MAC ? "Cmd" : "Win";
          case "Alt":
            return IS_MAC ? "Option" : "Alt";
          default:
            return t;
        }
      })
      .join("+");
  }

  // 文字起こしメタデータ設定。タイムスタンプは既定ON（整形AIが時間関係を解釈できる）。
  let includeTimestamps = $state<boolean>(true);
  // 停止後に文字起こし→整形まで自動実行する（一気通貫）。
  let autoPipeline = $state<boolean>(false);

  // 保存設定。文字起こしテキスト保持/録音音声保存/形式/保存先フォルダ。
  let keepText = $state<boolean>(true);
  let saveAudio = $state<boolean>(false);
  let audioFormat = $state<string>("opus");
  let saveDir = $state<string>(""); // 空=既定(ドキュメント/QuickScribe)
  // 出力形式（S4.2）: "txt"=本文のみ / "md"=メタデータ付きMarkdown。既定はtxt（後方互換）。
  let outputFormat = $state<string>("txt");
  // 内省タグ（S4.3）。エントリ保存時にメタデータとして付与（カンマ/空白区切りで入力）。
  let entryTags = $state<string>("");
  // 入力文字列をタグ配列へ（カンマ/全角カンマ/空白区切り・重複/空除去・先頭#除去）。
  function parseTags(s: string): string[] {
    const seen = new Set<string>();
    const out: string[] = [];
    for (const raw of s.split(/[,、\s]+/)) {
      const t = raw.trim().replace(/^#+/, "");
      if (t && !seen.has(t)) {
        seen.add(t);
        out.push(t);
      }
    }
    return out;
  }
  // 文字起こし(STT)プロバイダ（S2.4 / ADR-0016）。既定はローカルwhisper（プライバシー）。
  // クラウドは音声を端末外へ送信＝明示選択時のみ。鍵はkeyring保管。
  // STTプロバイダ定義は src/lib/constants.ts に集約(SSOT / #401 Phase0)。
  let sttProvider = $state<SttProvider>("local");
  let sttModel = $state<string>(""); // クラウドのモデルID（空=プロバイダ既定）
  let sttAzureResource = $state<string>(""); // Azureのリソース名（azure時のみ）
  // ローカル whisper のモデル選択（S2.2）。クラウドの sttModel とは分離。
  let whisperModel = $state<string>(""); // 空=既定 base
  let whisperModels = $state<{ id: string; label: string }[]>([]);
  async function loadWhisperModels() {
    try {
      whisperModels = await invoke<{ id: string; label: string }[]>("list_whisper_models");
    } catch (e) {
      console.error("list_whisper_models failed", e);
    }
  }
  // クラウドSTTのAPIキー（プロバイダごと）。keyringに "sttKey:<provider>" で保管。
  let sttKeys = $state<Record<string, string>>({
    groq: "",
    openai: "",
    deepgram: "",
    azure: "",
  });
  // STT設定をバックエンドへ反映。ローカルは whisperModel、クラウドは sttModel を model として渡す。
  async function syncSttSettings() {
    try {
      const isCloud = (STT_CLOUD as string[]).includes(sttProvider);
      await invoke("set_stt_settings", {
        provider: sttProvider,
        model: isCloud ? sttModel : whisperModel,
        apiKey: isCloud ? (sttKeys[sttProvider] ?? "") : "",
        azureResource: sttProvider === "azure" ? sttAzureResource.trim() : "",
      });
    } catch (e) {
      console.error("set_stt_settings failed", e);
    }
  }

  // 整形スタイル(コア価値: 逐語⇄要約⇄ブレストを行き来 / S3.3)。
  // desc は各モードの短い解説(設定のtips・処理画面のツールチップに使う。refine.rs の指示と一致)。
  let refineStyle = $state<string>("structured");
  // 整形スタイル(REFINE_STYLES)・カスタム型は src/lib/constants.ts に集約(SSOT / #401 Phase0)。
  let customStyles = $state<CustomStyle[]>([]);
  // 組み込み＋カスタムを1つの選択肢リストに統合（チップ・設定ドロップダウン・解説で共用）。
  const allStyles = $derived([
    ...REFINE_STYLES,
    ...customStyles.map((c) => ({
      value: `custom:${c.id}`,
      label: c.label || "カスタム",
      desc: c.instruction.trim() || "ユーザー定義の整形指示。",
    })),
  ]);
  // 現在選択中のスタイル(処理画面の表示・解説に使う)。未知値は既定にフォールバック。
  const currentStyle = $derived(allStyles.find((s) => s.value === refineStyle) ?? allStyles[0]);

  // カスタムパターンの編集フォーム状態（新規追加）。
  let newCustomLabel = $state<string>("");
  let newCustomInstruction = $state<string>("");
  function addCustomStyle() {
    const label = newCustomLabel.trim();
    const instruction = newCustomInstruction.trim();
    if (!label || !instruction) {
      error = "カスタムパターンには名前と指示の両方が必要です。";
      return;
    }
    const id =
      typeof crypto !== "undefined" && crypto.randomUUID
        ? crypto.randomUUID().slice(0, 8)
        : String(Date.now());
    customStyles = [...customStyles, { id, label, instruction }];
    newCustomLabel = "";
    newCustomInstruction = "";
    persistCustomStyles();
  }
  function removeCustomStyle(id: string) {
    customStyles = customStyles.filter((c) => c.id !== id);
    // 削除したパターンを選択中なら既定へ戻す。
    if (refineStyle === `custom:${id}`) refineStyle = "structured";
    persistCustomStyles();
  }
  function persistCustomStyles() {
    localStorage.setItem("customStyles", JSON.stringify(customStyles));
  }

  function loadSettings() {
    provider = (localStorage.getItem("provider") as Provider) || "gemini";
    if (!(provider in PROVIDER_LABELS)) provider = "gemini";
    // 解決済みモデルは秘密でないため localStorage。鍵は keyring(loadSecrets)で別途読む(S3.2)。
    for (const p of ALL_PROVIDERS) {
      resolvedModel[p] = localStorage.getItem(`resolvedModel:${p}`) ?? "";
    }
    recordShortcut = localStorage.getItem("recordShortcut") || DEFAULT_SHORTCUT;
    recordMode = localStorage.getItem("recordMode") === "momentary" ? "momentary" : "toggle";
    includeTimestamps = localStorage.getItem("includeTimestamps") !== "false";
    autoPipeline = localStorage.getItem("autoPipeline") === "true";
    keepText = localStorage.getItem("keepText") !== "false";
    saveAudio = localStorage.getItem("saveAudio") === "true";
    audioFormat = localStorage.getItem("audioFormat") || "opus";
    saveDir = localStorage.getItem("saveDir") || "";
    outputFormat = localStorage.getItem("outputFormat") || "txt";
    refineStyle = localStorage.getItem("refineStyle") || "structured";
    sttProvider = (localStorage.getItem("sttProvider") as SttProvider) || "local";
    sttModel = localStorage.getItem("sttModel") || "";
    sttAzureResource = localStorage.getItem("sttAzureResource") || "";
    whisperModel = localStorage.getItem("whisperModel") || "base";
    try {
      customStyles = JSON.parse(localStorage.getItem("customStyles") || "[]");
    } catch {
      customStyles = [];
    }
    // AWS設定(秘密でないもの)。region/workspace_id/認証方式/モデルは localStorage。
    awsRegion = localStorage.getItem("awsRegion") || "us-east-1";
    awsWorkspaceId = localStorage.getItem("awsWorkspaceId") || "";
    awsAuthMode = (localStorage.getItem("awsAuthMode") as "sigv4" | "apikey") || "sigv4";
    bedrockModel = localStorage.getItem("bedrockModel") || "";
    taskbarWidget = localStorage.getItem("taskbarWidget") !== "false";
    inputDevice = localStorage.getItem("inputDevice") || "";
    inputDeviceKind = localStorage.getItem("inputDeviceKind") || "input";
    // 設定スキーマの検証＋版更新（S5.3 / ADR-0017）。破損/旧値は既定へクランプ。
    validateSettings();
    // 秘密情報(API鍵/AWS鍵)は keyring から非同期で読む(S3.2)。
    void loadSecrets();
  }

  // 設定スキーマ版（ADR-0017）。形式が将来変わってもクランプ＋移行で壊れないようにする。
  const SETTINGS_VERSION = 1;
  // enum的な設定値を検証し、不正なら既定へクランプする（破損耐性）。未知キーは保持（非破壊）。
  function validateSettings() {
    if (!(provider in PROVIDER_LABELS)) provider = "gemini";
    if (!(sttProvider in STT_LABELS)) sttProvider = "local";
    if (recordMode !== "toggle" && recordMode !== "momentary") recordMode = "toggle";
    if (outputFormat !== "txt" && outputFormat !== "md") outputFormat = "txt";
    if (audioFormat !== "opus" && audioFormat !== "wav") audioFormat = "opus";
    if (awsAuthMode !== "sigv4" && awsAuthMode !== "apikey") awsAuthMode = "sigv4";
    // 整形スタイルは「組み込み」または「存在するカスタム」以外は既定へ。
    const styleOk =
      REFINE_STYLES.some((s) => s.value === refineStyle) ||
      (refineStyle.startsWith("custom:") &&
        customStyles.some((c) => `custom:${c.id}` === refineStyle));
    if (!styleOk) refineStyle = "structured";
    localStorage.setItem("settingsVersion", String(SETTINGS_VERSION));
  }

  // OSセキュアストレージ(keyring)との橋渡し(S3.2)。鍵は localStorage に置かない。
  async function getSecret(key: string): Promise<string> {
    try {
      return (await invoke<string | null>("get_secret", { key })) ?? "";
    } catch (e) {
      console.error("get_secret failed", key, e);
      return "";
    }
  }
  // 成功時 true。keyring書き込みが失敗したら false(呼び出し側は平文を消さない=鍵を失わない)。
  async function setSecret(key: string, value: string): Promise<boolean> {
    try {
      await invoke("set_secret", { key, value });
      return true;
    } catch (e) {
      console.error("set_secret failed", key, e);
      return false;
    }
  }
  // keyringに無ければ旧localStorage(平文)から移行する。
  // ★移行は keyring書き込みが成功した時だけ平文を削除する(失敗時は保持＝データ損失防止)。
  async function loadSecretMigrating(key: string, legacyLsKey: string): Promise<string> {
    let v = await getSecret(key);
    if (!v) {
      const legacy = localStorage.getItem(legacyLsKey);
      if (legacy) {
        v = legacy;
        if (await setSecret(key, legacy)) {
          localStorage.removeItem(legacyLsKey);
        }
      }
    }
    return v;
  }
  async function loadSecrets() {
    for (const p of ALL_PROVIDERS) {
      apiKeys[p] = await loadSecretMigrating(`apiKey:${p}`, `apiKey:${p}`);
    }
    // 旧バージョン(geminiKey 平文)からの移行(同様に成功時のみ削除)。
    if (!apiKeys.gemini) {
      const legacy = localStorage.getItem("geminiKey");
      if (legacy) {
        apiKeys.gemini = legacy;
        if (await setSecret("apiKey:gemini", legacy)) {
          localStorage.removeItem("geminiKey");
        }
      }
    }
    awsAccessKey = await loadSecretMigrating("awsAccessKey", "awsAccessKey");
    awsSecretKey = await loadSecretMigrating("awsSecretKey", "awsSecretKey");
    awsSessionToken = await loadSecretMigrating("awsSessionToken", "awsSessionToken");
    // クラウドSTTのAPIキー（S2.4）。
    for (const p of STT_CLOUD) sttKeys[p] = await getSecret(`sttKey:${p}`);
    // 鍵を読み込んだらSTT設定をバックエンドへ反映（クラウド選択時に有効化）。
    void syncSttSettings();
  }
  async function saveSecrets() {
    for (const p of ALL_PROVIDERS) await setSecret(`apiKey:${p}`, apiKeys[p]);
    await setSecret("awsAccessKey", awsAccessKey);
    await setSecret("awsSecretKey", awsSecretKey);
    await setSecret("awsSessionToken", awsSessionToken);
    for (const p of STT_CLOUD) await setSecret(`sttKey:${p}`, sttKeys[p] ?? "");
  }

  // 保存設定をバックエンドへ反映する（保存系コマンドが参照）。
  async function syncSaveSettings() {
    try {
      await invoke("set_save_settings", {
        saveDir,
        saveAudio,
        audioFormat,
        keepText,
        outputFormat,
      });
    } catch (e) {
      console.error("set_save_settings failed", e);
    }
  }

  // 保存先フォルダを選ぶ（ディレクトリ選択ダイアログ）。
  async function pickSaveDir() {
    const d = await open({ directory: true, multiple: false });
    if (typeof d === "string") saveDir = d;
  }
  // 出力先フォルダをOSのファイルマネージャで開く（S4.1 R6）。
  async function openVault() {
    try {
      await invoke("open_vault");
    } catch (e) {
      error = `出力先フォルダを開けませんでした: ${errorText(e)}`;
    }
  }
  function saveSettings() {
    localStorage.setItem("provider", provider);
    // 鍵(API鍵/AWS鍵)は keyring に保存する(localStorageには置かない / S3.2)。
    void saveSecrets();
    void applyShortcut();
    localStorage.setItem("includeTimestamps", String(includeTimestamps));
    localStorage.setItem("autoPipeline", String(autoPipeline));
    localStorage.setItem("keepText", String(keepText));
    localStorage.setItem("saveAudio", String(saveAudio));
    localStorage.setItem("audioFormat", audioFormat);
    localStorage.setItem("saveDir", saveDir);
    localStorage.setItem("outputFormat", outputFormat);
    localStorage.setItem("refineStyle", refineStyle);
    localStorage.setItem("sttProvider", sttProvider);
    localStorage.setItem("sttModel", sttModel);
    localStorage.setItem("sttAzureResource", sttAzureResource);
    localStorage.setItem("whisperModel", whisperModel);
    // AWS設定(秘密でないもの)のみ localStorage。
    localStorage.setItem("awsRegion", awsRegion);
    localStorage.setItem("awsWorkspaceId", awsWorkspaceId);
    localStorage.setItem("awsAuthMode", awsAuthMode);
    localStorage.setItem("bedrockModel", bedrockModel);
    localStorage.setItem("taskbarWidget", String(taskbarWidget));
    localStorage.setItem("recordMode", recordMode);
    localStorage.setItem("inputDevice", inputDevice);
    localStorage.setItem("inputDeviceKind", inputDeviceKind);
    void applyTaskbarWidget();
    void syncSaveSettings();
    void syncSttSettings();
    showSettings = false;
    // 鍵が入っていれば現在のプロバイダの最新モデルを取得（強制更新）。
    void resolveCurrentModel(true);
  }

  // キー入力イベントから Tauri アクセラレータ表記を組み立てる（修飾キー＋1キー）。
  function accelFromEvent(e: KeyboardEvent): string | null {
    const k = e.key;
    if (["Control", "Shift", "Alt", "Meta"].includes(k)) return null; // 修飾キー単体は無視
    const parts: string[] = [];
    if (e.ctrlKey || e.metaKey) parts.push("CommandOrControl");
    if (e.shiftKey) parts.push("Shift");
    if (e.altKey) parts.push("Alt");
    let key: string;
    if (k.length === 1) key = k.toUpperCase();
    else if (k.startsWith("Arrow"))
      key = k.slice(5); // ArrowUp -> Up
    else key = k; // F1..F12, Space, Enter 等
    parts.push(key);
    if (parts.length < 2) return null; // 修飾キー無しは誤爆防止のため不可
    return parts.join("+");
  }

  // ホットキー設定UX: 「変更」ボタンでキャプチャモードに入り、押したキーで登録する
  // （VSCode/OBS/ゲーム等の定石。待機表示・Escキャンセル・修飾キー単体は待機継続）。
  function startCapture() {
    capturing = true;
    shortcutMsg = "";
  }
  function cancelCapture() {
    capturing = false;
  }
  function onCaptureKeydown(e: KeyboardEvent) {
    if (!capturing) return;
    e.preventDefault();
    if (e.key === "Escape") {
      cancelCapture();
      return;
    }
    // 修飾キー単体は待機を継続（組合せの確定を待つ）。
    if (["Control", "Shift", "Alt", "Meta"].includes(e.key)) return;
    const accel = accelFromEvent(e);
    if (!accel) {
      // 修飾キー無しは誤爆防止のため不可。ヒントを出して待機継続。
      shortcutMsg = "修飾キー（Ctrl/Alt/Shift）と組み合わせて押してください";
      return;
    }
    recordShortcut = accel;
    capturing = false;
    void applyShortcut();
  }
  function resetShortcut() {
    recordShortcut = DEFAULT_SHORTCUT;
    void applyShortcut();
  }

  // 現在の表記でグローバルホットキーを再登録する。
  async function applyShortcut() {
    try {
      await invoke("set_record_shortcut", { accelerator: recordShortcut });
      localStorage.setItem("recordShortcut", recordShortcut);
      void invoke("set_taskbar_shortcut", { display: displayShortcut(recordShortcut) });
      shortcutMsg = `ホットキーを設定しました: ${displayShortcut(recordShortcut)}`;
    } catch (e) {
      shortcutMsg = `ホットキーを設定できませんでした: ${errorText(e)}`;
    }
  }

  // 現在のプロバイダの最新ミドルレンジモデルを実行時に解決し、キャッシュする。
  // force=false かつキャッシュが新しければ何もしない。鍵未入力なら何もしない。
  async function resolveCurrentModel(force = false) {
    const p = provider;
    // AWS系はモデル一覧APIが別系統のため自動解決しない(フォールバック/手入力)。
    if (AWS_PROVIDERS.includes(p)) return;
    if (!apiKeys[p].trim()) return;
    const at = Number(localStorage.getItem(`resolvedModelAt:${p}`) || 0);
    if (!force && resolvedModel[p] && Date.now() - at < MODEL_TTL_MS) return;
    resolvingModel = true;
    try {
      const m = await invoke<string>("resolve_model", {
        provider: p,
        apiKey: apiKeys[p],
      });
      if (m) {
        resolvedModel[p] = m;
        localStorage.setItem(`resolvedModel:${p}`, m);
        localStorage.setItem(`resolvedModelAt:${p}`, String(Date.now()));
      }
    } catch (e) {
      console.error("resolve_model failed", e);
    } finally {
      resolvingModel = false;
    }
  }

  // 自動アップデート(起動時に非同期チェック→背景DL→完了後に再起動を確認)。
  type UpdateState = "idle" | "downloading" | "ready";
  let updateState = $state<UpdateState>("idle");
  let updateVersion = $state<string>("");
  let updatePct = $state<number>(0);

  async function checkForUpdate(manual = false) {
    try {
      updateMsg = manual ? "更新を確認中…" : "";
      const update = await check();
      if (!update) {
        updateMsg = manual ? "お使いのバージョンは最新です。" : "";
        return;
      }
      updateMsg = "";
      updateVersion = update.version;
      updateState = "downloading";
      let downloaded = 0;
      let total = 0;
      await update.downloadAndInstall((event) => {
        if (event.event === "Started") {
          total = event.data.contentLength ?? 0;
        } else if (event.event === "Progress") {
          downloaded += event.data.chunkLength;
          updatePct = total > 0 ? Math.round((downloaded / total) * 100) : 0;
        }
      });
      updateState = "ready";
    } catch (e) {
      console.error("update check failed", e);
      updateMsg = manual ? `更新確認に失敗: ${errorText(e)}` : "";
    }
  }

  async function restartNow() {
    await relaunch();
  }

  // 音声ファイル(mp3等)を選んで文字起こし→保存する(S1.6)。非同期で実行しUIを固めない。
  async function transcribeFromFile() {
    error = null;
    transcript = null;
    refined = null;
    segments = [];
    progress = 0;
    eta = "";
    transcribeStartMs = null;
    const selected = await open({
      multiple: false,
      filters: [{ name: "音声ファイル", extensions: SUPPORTED_AUDIO_EXTS }],
    });
    if (typeof selected !== "string") return;
    busy = true;
    try {
      const text = await invoke<string>("transcribe_file", {
        path: selected,
        timestamps: includeTimestamps,
      });
      transcript = text;
    } catch (e) {
      error = `文字起こしに失敗しました: ${errorText(e)}`;
    } finally {
      busy = false;
      status = "";
    }
  }

  // 文字起こしを整形（思考整理・要約）する＝コア価値。選択中プロバイダの鍵が必要。
  // プロバイダが整形可能な設定になっているか（鍵/AWS資格情報）。未設定なら理由文を返す。
  function refineConfigError(): string | null {
    if (LOCAL_PROVIDERS.includes(provider)) return null; // Ollamaは鍵不要
    if (AWS_PROVIDERS.includes(provider)) {
      if (!awsRegion.trim()) return "AWSリージョンを設定してください。";
      if (provider === "claude-aws" && !awsWorkspaceId.trim())
        return "Claude Platform on AWS には workspace_id が必要です。";
      if (awsAuthMode === "sigv4") {
        if (!awsAccessKey.trim() || !awsSecretKey.trim())
          return "AWSアクセスキー/シークレットを設定してください。";
      } else if (!apiKeys[provider].trim()) {
        return `${PROVIDER_LABELS[provider]} のAPIキーが必要です。`;
      }
      return null;
    }
    return apiKeys[provider].trim()
      ? null
      : `整形には ${PROVIDER_LABELS[provider]} のAPIキーが必要です。`;
  }

  // refine_text に渡す追加引数（AWS時のみ資格情報。非AWSは undefined＝後方互換）。
  // styleOverride: 結果から別スタイルで整形し直す時に既定スタイルを上書きする(S3.5・行き来)。
  function refineArgs(styleOverride?: string) {
    const base: Record<string, unknown> = {
      text: transcript,
      provider,
      apiKey: apiKeys[provider],
      // 解決済みモデル（空ならバックエンドがフォールバック既定を補完）。Bedrockは手入力モデル。
      model: provider === "bedrock" ? bedrockModel : resolvedModel[provider],
      style: styleOverride ?? refineStyle,
    };
    // カスタムパターン選択時は指示文をバックエンドへ渡す（style既定指示の代わりに使われる）。
    const styleVal = (styleOverride ?? refineStyle) as string;
    if (styleVal.startsWith("custom:")) {
      const cs = customStyles.find((c) => `custom:${c.id}` === styleVal);
      base.customInstruction = cs?.instruction ?? null;
    }
    // 内省タグ（S4.3）。保存時にメタデータとして付与。
    const tags = parseTags(entryTags);
    if (tags.length > 0) base.tags = tags;
    if (AWS_PROVIDERS.includes(provider)) {
      base.region = awsRegion.trim();
      base.workspaceId = awsWorkspaceId.trim();
      base.authMode = awsAuthMode;
      if (awsAuthMode === "sigv4") {
        base.awsAccessKey = awsAccessKey.trim();
        base.awsSecretKey = awsSecretKey.trim();
        base.awsSessionToken = awsSessionToken.trim() || null;
      }
    }
    return base;
  }

  async function refineNow(styleOverride?: string) {
    if (!transcript) return;
    const cfgErr = refineConfigError();
    if (cfgErr) {
      showSettings = true;
      error = `${cfgErr}設定から入力してください。`;
      return;
    }
    error = null;
    refining = true;
    refined = null;
    try {
      // 整形直前に最新モデルを確保（キャッシュが新しければ即返る。AWSはスキップ）。
      await resolveCurrentModel();
      refined = await invoke<string>("refine_text", refineArgs(styleOverride));
      refinedStyle = styleOverride ?? refineStyle; // どのスタイルで整形したか(再整形チップの強調用)。
    } catch (e) {
      error = `整形に失敗しました: ${errorText(e)}`;
    } finally {
      refining = false;
    }
  }

  // 用語補正フェーズ（文字起こし→整形の間）。誤変換疑いの語を検出→確認→置換。
  let checkingTerms = $state(false);
  let corrections = $state<Correction[] | null>(null); // null=未実行 / []=候補なし。型は ./lib/corrections
  // 校正専用の指示。整形(思考整理)ではなく誤変換検出だけを行わせ、<journal>内に区切り行で返させる。
  const CORRECTION_INSTRUCTION = [
    "（重要）今回は思考整理ではなく「校正」だけを行う。整理・要約・再構成・書き換えは一切しない。",
    "- 文字起こしの中で、音声認識の誤変換が疑われる語（文脈に合わない固有名詞・専門用語・カタカナ語・同音異義語）を検出する",
    "- 各候補について、文脈から推定される正しい表記を提案する。確信が低い候補は出さない（多くて15件まで）",
    "- 出力は <journal> と </journal> の中に、候補を1行ずつ `原文の語 ||| 提案する語 ||| 理由(短く)` の形式で書く。候補が無ければ中身は空にする",
    "- 本文の整形や書き換えはしない。区切りは半角の ||| を使う",
  ].join("\n");
  async function suggestCorrections() {
    if (!transcript) return;
    const cfgErr = refineConfigError();
    if (cfgErr) {
      showSettings = true;
      error = `${cfgErr}設定から入力してください。`;
      return;
    }
    error = null;
    checkingTerms = true;
    corrections = null;
    try {
      await resolveCurrentModel();
      const args = refineArgs();
      args.customInstruction = CORRECTION_INSTRUCTION;
      args.save = false; // 校正候補は保存しない。
      delete args.tags;
      const raw = await invoke<string>("refine_text", args);
      corrections = parseCorrections(raw);
    } catch (e) {
      error = `用語チェックに失敗しました: ${errorText(e)}`;
    } finally {
      checkingTerms = false;
    }
  }
  // 選択された候補を文字起こしに適用（全置換）し、確認パネルを閉じる。
  function applyCorrectionsToTranscript() {
    if (!corrections || !transcript) return;
    const { text, applied } = applyCorrections(transcript, corrections);
    transcript = text;
    corrections = null;
    status = applied > 0 ? `用語を${applied}件置換しました` : "";
  }

  // 整形結果をクリップボードへコピー(S3.5・最小操作の利便)。
  let copyMsg = $state<string>("");
  async function copyRefined() {
    if (!refined) return;
    try {
      await navigator.clipboard.writeText(refined);
      copyMsg = "コピーしました";
      setTimeout(() => (copyMsg = ""), 1500);
    } catch (e) {
      console.error("copy failed", e);
    }
  }

  // メモ/テキストファイルを読み込んで整形だけ実行する（文字起こし不要の用途）。
  async function refineFromMemo() {
    error = null;
    const selected = await open({
      multiple: false,
      filters: [{ name: "テキスト/メモ", extensions: ["txt", "md", "markdown", "text"] }],
    });
    if (typeof selected !== "string") return;
    try {
      const text = await invoke<string>("read_text_file", { path: selected });
      transcript = text;
      refined = null;
      segments = [];
      await refineNow();
    } catch (e) {
      error = `メモの整形に失敗しました: ${errorText(e)}`;
    }
  }

  // 録音トグル（S1.1）。開始でマイク収集、停止で文字起こし→保存→表示まで貫通する。
  async function toggle() {
    error = null;
    if (!recording) {
      try {
        await invoke("start_recording", {
          device: inputDevice || null,
          kind: inputDeviceKind || null,
        });
        recording = true;
        startedAt = Date.now();
        // タスクバーボタンに録音中バッジを表示（状態の可視化）。
        void invoke("set_recording_overlay", { recording: true });
        // 新しい録音に向けて表示をリセット。
        transcript = null;
        refined = null;
        segments = [];
        progress = 0;
        eta = "";
        transcribeStartMs = null;
      } catch (e) {
        error = `録音を開始できませんでした: ${errorText(e)}`;
      }
    } else {
      recording = false;
      startedAt = null;
      // タスクバーの録音中バッジを解除。
      void invoke("set_recording_overlay", { recording: false });
      // 文字起こしはバックグラウンドで走る（録音はすぐ再開できる＝非同期）。
      // 結果は transcribe-done / transcribe-error イベントで受け取る。
      transcribing = true;
      try {
        await invoke("stop_recording", { timestamps: includeTimestamps });
      } catch (e) {
        transcribing = false;
        error = `録音の停止に失敗しました: ${errorText(e)}`;
      }
    }
  }

  // 物理トリガーのホットキー押下/解放を録音モードで振り分ける（S1.5 / ADR-0014）。
  // モーメンタリ=押している間だけ録音。解放時は末尾切れ防止に少し遅延して停止（OBS/Discord定石）。
  const MOMENTARY_STOP_DELAY_MS = 300;
  let momentaryStopTimer: ReturnType<typeof setTimeout> | null = null;
  function clearMomentaryTimer() {
    if (momentaryStopTimer !== null) {
      clearTimeout(momentaryStopTimer);
      momentaryStopTimer = null;
    }
  }
  function onRecordPress() {
    if (recordMode === "momentary") {
      clearMomentaryTimer(); // 直前の解放猶予をキャンセルして録音継続。
      if (!recording) void toggle();
    } else {
      void toggle();
    }
  }
  function onRecordRelease() {
    if (recordMode !== "momentary") return;
    clearMomentaryTimer();
    momentaryStopTimer = setTimeout(() => {
      momentaryStopTimer = null;
      if (recording) void toggle();
    }, MOMENTARY_STOP_DELAY_MS);
  }

  onMount(() => {
    loadSettings();
    void applyShortcut();
    void applyTaskbarWidget();
    void loadAutoStart();
    void loadAudioSources();
    void loadWhisperModels();
    void syncSaveSettings();
    void resolveCurrentModel();
    void checkForUpdate();
    // トレイ/メニュー/CLI --toggle-record は常にトグル。
    const unToggle = listen("toggle-record", () => void toggle());
    // ホットキー押下/解放（録音モードで振り分け）。
    const unPress = listen("record-press", () => onRecordPress());
    const unRelease = listen("record-release", () => onRecordRelease());
    // CLI --start-record / --stop-record（自動化・マクロ向け 明示開始/停止）。
    const unStartRec = listen("start-record", () => {
      if (!recording) void toggle();
    });
    const unStopRec = listen("stop-record", () => {
      if (recording) void toggle();
    });
    const unStatus = listen<string>("status", (e) => (status = e.payload));
    const unProgress = listen<number>("progress", (e) => {
      progress = e.payload;
      if (progress > 0 && transcribeStartMs === null) transcribeStartMs = Date.now();
      if (transcribeStartMs && progress > 0 && progress < 100) {
        const elapsed = (Date.now() - transcribeStartMs) / 1000;
        eta = formatRemaining(estimateRemaining(elapsed, progress));
      } else {
        eta = "";
      }
    });
    const unSegment = listen<string>("segment", (e) => {
      const t = e.payload.trim();
      if (t) segments = [...segments, t];
    });
    // バックグラウンド文字起こしの完了/失敗（録音の非同期化）。
    const unDone = listen<string>("transcribe-done", (e) => {
      transcribing = false;
      status = "";
      const text = e.payload;
      if (text) {
        transcript = text;
        // 一気通貫: 自動で整形まで実行（プロバイダが設定済みのときのみ）。
        if (autoPipeline && refineConfigError() === null) void refineNow();
      } else {
        error = "文字起こしできる音声が含まれていませんでした（音声は保存していません）。";
      }
    });
    const unErr = listen<string>("transcribe-error", (e) => {
      transcribing = false;
      status = "";
      error = e.payload;
    });
    return () => {
      clearMomentaryTimer();
      unToggle.then((f) => f());
      unPress.then((f) => f());
      unRelease.then((f) => f());
      unStartRec.then((f) => f());
      unStopRec.then((f) => f());
      unStatus.then((f) => f());
      unProgress.then((f) => f());
      unSegment.then((f) => f());
      unDone.then((f) => f());
      unErr.then((f) => f());
    };
  });
</script>

<main inert={showSettings || showEntries}>
  <div class="content">
    <header>
      <div class="title-row">
        <h1>QuickScribe</h1>
        <div class="header-actions">
          <button
            class="nav-btn"
            title="ジャーナル（過去のエントリを一覧・検索・横断発見）"
            aria-label="ジャーナル"
            onclick={openEntriesPanel}
          >
            <svg
              class="ic"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              aria-hidden="true"
            >
              <path d="M2 6h4" /><path d="M2 10h4" /><path d="M2 14h4" /><path d="M2 18h4" />
              <rect width="16" height="20" x="4" y="2" rx="2" /><path d="M16 2v20" />
            </svg>
            <span>ジャーナル</span>
          </button>
          <button
            class="gear"
            data-testid="settings-btn"
            title="設定"
            aria-label="設定"
            onclick={() => (showSettings = !showSettings)}
          >
            <svg
              class="ic"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              aria-hidden="true"
            >
              <path
                d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"
              />
              <circle cx="12" cy="12" r="3" />
            </svg>
          </button>
        </div>
      </div>
      <p class="tagline">思考整理・自己理解のためのボイスジャーナル</p>
    </header>

    {#if updateState === "downloading"}
      <div class="update-banner">
        <span class="spinner" aria-hidden="true"></span>
        新バージョン {updateVersion} を背景でダウンロード中… {updatePct}%
      </div>
    {:else if updateState === "ready"}
      <div class="update-banner ready">
        <span>新バージョン {updateVersion} の準備ができました。</span>
        <button class="btn-restart" onclick={restartNow}>再起動して更新</button>
      </div>
    {/if}

    <div class="actions">
      <button class="btn primary" class:recording data-testid="record-btn" onclick={toggle}>
        <span class="dot" class:on={recording}></span>
        {recording ? "停止" : "録音開始"}
      </button>

      <button
        class="btn secondary"
        data-testid="file-btn"
        onclick={transcribeFromFile}
        disabled={busy}
      >
        <svg viewBox="0 0 24 24" width="18" height="18" aria-hidden="true">
          <path
            fill="currentColor"
            d="M12 3a3 3 0 0 0-3 3v6a3 3 0 0 0 6 0V6a3 3 0 0 0-3-3Zm5 9a5 5 0 0 1-10 0H5a7 7 0 0 0 6 6.92V21h2v-2.08A7 7 0 0 0 19 12h-2Z"
          />
        </svg>
        音声ファイルから文字起こし
      </button>

      <button
        class="btn secondary"
        data-testid="memo-btn"
        onclick={refineFromMemo}
        disabled={refining}
      >
        <svg viewBox="0 0 24 24" width="18" height="18" aria-hidden="true">
          <path
            fill="currentColor"
            d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8l-6-6Zm0 2 4 4h-4V4ZM8 13h8v2H8v-2Zm0 4h8v2H8v-2Z"
          />
        </svg>
        メモ/テキストから整形
      </button>
    </div>

    <p class="hint">
      対応形式: 音声 {SUPPORTED_AUDIO_EXTS.join(" / ")}（最大 {MAX_INPUT_MB}MB）・テキスト txt / md
    </p>
    <p class="hint">
      録音ホットキー: <code>{displayShortcut(recordShortcut)}</code>（設定で変更可）
    </p>

    {#if busy || transcribing || status}
      <div class="panel">
        <div class="status-row">
          <span class="spinner" aria-hidden="true"></span>
          <span class="status-text">{status || "処理中…"}</span>
        </div>
        {#if progress > 0}
          <div class="progress" role="progressbar" aria-valuenow={progress}>
            <div class="bar" style="width: {progress}%"></div>
          </div>
          <div class="progress-meta">
            <span class="pct">{progress}%</span>
            {#if eta}<span class="eta">{eta}</span>{/if}
          </div>
        {/if}
        {#if segments.length}
          <div class="segments">
            {#each segments as seg}<span>{seg}</span>{/each}
          </div>
        {/if}
      </div>
    {/if}

    {#if transcript}
      <section class="card">
        <div class="card-head">
          <h2>文字起こし</h2>
          <div class="refine-controls">
            <!-- スタイルは設定画面で選ぶ。ここでは「どのスタイルで整形するか」の表示のみ
               (マウスオーバーでモードの解説を表示)。 -->
            <span class="style-indicator" title={currentStyle.desc}>
              整形スタイル: <strong>{currentStyle.label}</strong>
            </span>
            <button
              class="btn small ghost"
              title="誤変換が疑われる用語をAIが検出し、置換を提案します（整形の前に手修正を緩和）"
              onclick={suggestCorrections}
              disabled={checkingTerms || refining}
            >
              {checkingTerms ? "用語を確認中…" : "✓ 用語チェック"}
            </button>
            <button class="btn small" onclick={() => refineNow()} disabled={refining}>
              {refining ? "整形中…" : "✨ 整形する"}
            </button>
          </div>
        </div>
        <div class="scroll">{transcript}</div>

        <!-- 用語補正フェーズ: 誤変換疑いの候補を確認→置換（置換しない選択肢付き）。 -->
        {#if corrections !== null}
          {#if corrections.length === 0}
            <p class="tip">誤変換の疑いがある用語は見つかりませんでした。</p>
          {:else}
            <div class="corrections">
              <div class="corrections-head">
                <span
                  >誤変換の疑い（{corrections.length}件）— 置換する語にチェック、提案は編集可</span
                >
                <button
                  type="button"
                  class="btn small ghost"
                  onclick={() => corrections && corrections.forEach((c) => (c.replace = false))}
                >
                  すべて置換しない
                </button>
              </div>
              <ul class="correction-list">
                {#each corrections as c}
                  <li class="correction-item">
                    <label class="correction-check">
                      <input type="checkbox" bind:checked={c.replace} />
                      <span class="correction-orig">{c.original}</span>
                      <span class="correction-arrow">→</span>
                    </label>
                    <input class="correction-sugg" type="text" bind:value={c.suggestion} />
                    {#if c.reason}<span class="correction-reason" title={c.reason}>{c.reason}</span
                      >{/if}
                  </li>
                {/each}
              </ul>
              <div class="corrections-actions">
                <button class="btn small" onclick={applyCorrectionsToTranscript}
                  >選んだ用語を置換して更新</button
                >
                <button class="btn small ghost" onclick={() => (corrections = null)}
                  >閉じる（置換しない）</button
                >
              </div>
            </div>
          {/if}
        {/if}
        <!-- 内省タグ(S4.3): 整形・保存時にメタデータとして付与。後から束ねて見返す入口。 -->
        <div class="tags-row">
          <input
            class="tags-input"
            type="text"
            bind:value={entryTags}
            placeholder="タグ（任意・カンマ区切り 例: 仕事, 不安, アイデア）"
          />
        </div>
      </section>
    {/if}

    {#if refining}
      <p class="muted center">
        <span class="spinner" aria-hidden="true"></span> AIが思考を整理しています…
      </p>
    {/if}
    {#if refined}
      <section class="card refined">
        <h2>整形（思考整理）</h2>
        <div class="scroll">{refined}</div>
        <!-- 段階的深掘り(S3.5): 結果から別スタイルで整形し直す(再文字起こし不要＝逐語⇄要約⇄ブレストを行き来)。 -->
        <div class="restyle-row">
          <span class="restyle-label">別のスタイルで整形し直す:</span>
          {#each allStyles as s}
            <button
              type="button"
              class="chip"
              class:active={refinedStyle === s.value}
              title={s.desc}
              disabled={refining}
              onclick={() => refineNow(s.value)}>{s.label}</button
            >
          {/each}
          <button type="button" class="chip copy" onclick={copyRefined} disabled={refining}>
            {copyMsg || "コピー"}
          </button>
          <button
            type="button"
            class="chip"
            onclick={openVault}
            title="保存先フォルダをエクスプローラー等で開く"
          >
            <svg
              class="ic-sm"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              aria-hidden="true"
            >
              <path
                d="m6 14 1.5-2.9A2 2 0 0 1 9.24 10H20a2 2 0 0 1 1.94 2.5l-1.54 6a2 2 0 0 1-1.95 1.5H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h3.9a2 2 0 0 1 1.69.9l.81 1.2a2 2 0 0 0 1.67.9H18a2 2 0 0 1 2 2v2"
              />
            </svg>
            出力先を開く
          </button>
        </div>
      </section>
    {/if}

    {#if error}
      <p class="error">{error}</p>
    {/if}
  </div>
</main>

{#if showEntries}
  <div
    class="settings-overlay"
    role="presentation"
    onclick={(e) => {
      if (e.target === e.currentTarget) showEntries = false;
    }}
  >
    <div
      class="settings vault-panel"
      role="dialog"
      aria-modal="true"
      aria-labelledby="vault-title"
      tabindex="-1"
      use:modal={{ onClose: () => (showEntries = false) }}
    >
      <div class="settings-head">
        <h2 id="vault-title">ジャーナル{viewingEntry ? `：${viewingEntry.name}` : ""}</h2>
        <button class="close" aria-label="閉じる" onclick={() => (showEntries = false)}>×</button>
      </div>

      {#if discoveryResult !== null}
        <button class="btn small ghost" onclick={() => (discoveryResult = null)}
          >← 一覧へ戻る</button
        >
        <p class="tip">
          絞り込んだ{Math.min(filteredEntries.length, 30)}件{discoveryTruncated
            ? `（先頭30件のみ・全${filteredEntries.length}件）`
            : ""}からの横断発見です。保存はされません（必要ならコピーしてください）。
        </p>
        <pre class="entry-view">{discoveryResult}</pre>
      {:else if viewingEntry}
        <button class="btn small ghost" onclick={() => (viewingEntry = null)}>← 一覧へ戻る</button>
        <pre class="entry-view">{viewingEntry.content}</pre>
      {:else}
        <input
          class="tags-input"
          type="text"
          bind:value={entrySearch}
          placeholder="🔎 本文・タグ・ファイル名で検索"
        />
        {#if allTags.length > 0}
          <div class="tag-filter">
            {#each allTags as t}
              <button
                type="button"
                class="chip"
                class:active={selectedTags.includes(t)}
                onclick={() => toggleTagFilter(t)}>#{t}</button
              >
            {/each}
          </div>
        {/if}
        {#if filteredEntries.length >= 2}
          <button type="button" class="btn small" disabled={discovering} onclick={discoverAcross}>
            {discovering
              ? "AIが横断的に読み解いています…"
              : `✨ この${filteredEntries.length}件から横断発見`}
          </button>
          <p class="tip">
            絞り込んだ過去エントリから、繰り返すテーマ・感情の傾向・未解決の問い・次の一歩をAIが抽出します（整形プロバイダの鍵が必要・最大30件）。
          </p>
        {/if}
        {#if entriesLoading}
          <p class="muted center"><span class="spinner" aria-hidden="true"></span> 読み込み中…</p>
        {:else if filteredEntries.length === 0}
          <p class="tip">
            {entries.length === 0
              ? "まだエントリがありません。録音・整形するとジャーナルに保存されます。"
              : "条件に合うエントリがありません。"}
          </p>
        {:else}
          <ul class="entry-list">
            {#each filteredEntries as e}
              <li>
                <button type="button" class="entry-item" onclick={() => openEntry(e)}>
                  <div class="entry-meta">
                    <span class="entry-date">{e.created.replace("T", " ")}</span>
                    {#if e.kind}<span class="entry-kind">{kindLabel(e.kind)}</span>{/if}
                  </div>
                  {#if e.tags.length > 0}
                    <div class="entry-tags">
                      {#each e.tags as t}<span class="entry-tag">#{t}</span>{/each}
                    </div>
                  {/if}
                  <div class="entry-preview">{e.preview}</div>
                </button>
              </li>
            {/each}
          </ul>
        {/if}
      {/if}
    </div>
  </div>
{/if}

{#if showSettings}
  <div
    class="settings-overlay"
    role="presentation"
    onclick={(e) => {
      if (e.target === e.currentTarget) showSettings = false;
    }}
  >
    <div
      class="settings"
      role="dialog"
      aria-modal="true"
      aria-labelledby="settings-title"
      tabindex="-1"
      use:modal={{ onClose: () => (showSettings = false) }}
    >
      <div class="settings-head">
        <h2 id="settings-title">設定</h2>
        <button class="close" aria-label="閉じる" onclick={() => (showSettings = false)}>×</button>
      </div>
      <label>
        整形プロバイダ
        <select bind:value={provider} onchange={() => resolveCurrentModel()}>
          <option value="gemini">Gemini</option>
          <option value="anthropic">Anthropic (Claude)</option>
          <option value="openai">OpenAI</option>
          <option value="ollama">ローカル (Ollama)</option>
          <option value="bedrock">AWS Bedrock</option>
          <option value="claude-aws">Claude Platform on AWS</option>
        </select>
      </label>
      {#if LOCAL_PROVIDERS.includes(provider)}
        <p class="muted">
          ローカル (Ollama) は鍵不要で端末内完結（思考の生データを外に出しません）。 事前に <code
            >ollama serve</code
          >
          の起動とモデル取得（例: <code>ollama pull llama3.1</code>）が必要です。
        </p>
      {:else if AWS_PROVIDERS.includes(provider)}
        <!-- AWSプロバイダ(Bedrock / Claude Platform on AWS) / ADR-0011。SigV4 or APIキー。 -->
        <label>
          AWSリージョン
          <input type="text" bind:value={awsRegion} placeholder="us-east-1" autocomplete="off" />
        </label>
        {#if provider === "claude-aws"}
          <label>
            workspace_id（Claude Platform on AWS で必須）
            <input
              type="text"
              bind:value={awsWorkspaceId}
              placeholder="wrkspc_..."
              autocomplete="off"
            />
          </label>
        {/if}
        {#if provider === "bedrock"}
          <label>
            BedrockモデルID（リージョン/アカウント依存・空で既定）
            <input
              type="text"
              bind:value={bedrockModel}
              placeholder="anthropic.claude-sonnet-4-6"
              autocomplete="off"
            />
          </label>
        {/if}
        <label>
          認証方式
          <select bind:value={awsAuthMode}>
            <option value="sigv4">AWS IAMキー (SigV4)</option>
            <option value="apikey">APIキー</option>
          </select>
        </label>
        {#if awsAuthMode === "sigv4"}
          <label>
            AWSアクセスキーID
            <input
              type="password"
              bind:value={awsAccessKey}
              placeholder="AKIA..."
              autocomplete="off"
            />
          </label>
          <label>
            AWSシークレットアクセスキー
            <input type="password" bind:value={awsSecretKey} placeholder="..." autocomplete="off" />
          </label>
          <label>
            セッショントークン（一時credのときのみ・任意）
            <input
              type="password"
              bind:value={awsSessionToken}
              placeholder="（任意）"
              autocomplete="off"
            />
          </label>
        {:else}
          <label>
            {PROVIDER_LABELS[provider]} APIキー（端末内のみ保存）
            <input
              type="password"
              bind:value={apiKeys[provider]}
              placeholder={KEY_PLACEHOLDERS[provider]}
              autocomplete="off"
            />
          </label>
        {/if}
        <p class="muted">秘密情報は端末内に保存します（思考の生データを外に出しません）。</p>
      {:else}
        <label>
          {PROVIDER_LABELS[provider]} APIキー（整形に使用・端末内のみ保存）
          <input
            type="password"
            bind:value={apiKeys[provider]}
            placeholder={KEY_PLACEHOLDERS[provider]}
            autocomplete="off"
          />
        </label>
      {/if}
      <p class="muted model-hint">
        {#if provider === "bedrock"}
          モデル: <code>{bedrockModel || FALLBACK_MODELS[provider]}</code>（Bedrockは手入力/既定）
        {:else if provider === "claude-aws"}
          モデル: <code>{FALLBACK_MODELS[provider]}</code>（Claude Platform on AWS）
        {:else}
          モデル: <code>{resolvedModel[provider] || FALLBACK_MODELS[provider]}</code>
          {#if resolvingModel}（取得中…）{:else if resolvedModel[provider]}（最新を自動取得）{:else}（最新ミドルレンジを自動選択）{/if}
        {/if}
      </p>
      <label>
        整形スタイル（録音後の自動整形にも適用されます）
        <select bind:value={refineStyle}>
          {#each allStyles as s}
            <option value={s.value} title={s.desc}>{s.label}</option>
          {/each}
        </select>
      </label>
      <!-- 選択中スタイルの解説を常時表示(マウスオーバーに頼らず各モードの違いを示す)。 -->
      <p class="style-desc">{currentStyle.desc}</p>

      <!-- カスタム整形パターン(S3.3): ユーザー定義の指示を追加・管理する。 -->
      <div class="meta-group">
        <span class="meta-title">カスタム整形パターン</span>
        {#if customStyles.length > 0}
          <ul class="custom-list">
            {#each customStyles as c}
              <li class="custom-item">
                <span class="custom-name">{c.label}</span>
                <button
                  type="button"
                  class="btn small ghost"
                  onclick={() => removeCustomStyle(c.id)}>削除</button
                >
              </li>
            {/each}
          </ul>
        {/if}
        <input
          class="custom-name-input"
          type="text"
          bind:value={newCustomLabel}
          placeholder="パターン名（例: 議事録、日報、感情ラベル付け）"
        />
        <textarea
          class="custom-instruction-input"
          bind:value={newCustomInstruction}
          rows="4"
          placeholder={"AIへの指示を箇条書きで。例:\n- 決定事項とToDoを分けて箇条書きにする\n- 各ToDoに担当と期限の候補を添える"}
        ></textarea>
        <button type="button" class="btn small" onclick={addCustomStyle}>
          カスタムパターンを追加
        </button>
        <p class="tip">
          作成したパターンは整形スタイルの一覧（上の選択と結果画面のチップ）に並びます。捏造禁止・本文だけを出力する基本ルールは自動で守られます。
        </p>
      </div>
      <span class="meta-title">録音開始/停止のホットキー</span>
      <div class="hotkey-row">
        <button
          type="button"
          class="hotkey-capture"
          class:capturing
          onclick={startCapture}
          onkeydown={onCaptureKeydown}
          onblur={cancelCapture}
        >
          {#if capturing}
            キーを押してください…（Escでキャンセル）
          {:else}
            {displayShortcut(recordShortcut)}
          {/if}
        </button>
        <button type="button" class="btn small ghost" onclick={resetShortcut}>既定に戻す</button>
      </div>
      <p class="tip">
        「{displayShortcut(recordShortcut)}」をクリックして、登録したいキーを押します。
      </p>
      {#if shortcutMsg}<p class="muted">{shortcutMsg}</p>{/if}

      <div class="meta-group">
        <span class="meta-title">録音モード</span>
        <div class="device-row">
          <select bind:value={recordMode}>
            <option value="toggle">トグル（1回押すと開始 / もう1回で停止）</option>
            <option value="momentary">押している間だけ録音（ホールド）</option>
          </select>
        </div>
        <p class="tip">
          「押している間だけ録音」は、物理ボタン（マウス・フットスイッチ等）やホットキーを<strong
            >押し続けている間だけ</strong
          >録音します。離すと停止します（会議の発言・とっさの一言向け）。
        </p>
      </div>

      <div class="meta-group">
        <span class="meta-title">録音ソース（マイク / システム音）</span>
        <div class="device-row">
          <select value={`${inputDeviceKind}|${inputDevice}`} onchange={onSourceChange}>
            <option value="input|">OS既定のマイク</option>
            {#if IS_WINDOWS}
              <option value="mix|">マイク＋システム音（既定の出力 / 同時録音）</option>
            {/if}
            {#each audioSources as s}
              <option value={`${s.kind}|${s.id}`}>{s.label}</option>
            {/each}
          </select>
          <button type="button" class="btn small ghost" onclick={() => void loadAudioSources()}>
            再読込
          </button>
        </div>
        <p class="tip">
          録音元を選びます。「システム音:
          …」はその出力で再生中の音（相手の声・再生音）を録音。「マイク＋システム音」は自分の声と相手の声を同時に録音します（会議・通話向け）。次回の録音開始から反映されます。
        </p>
      </div>

      <div class="meta-group">
        <span class="meta-title">文字起こしエンジン</span>
        <div class="device-row">
          <select bind:value={sttProvider}>
            {#each Object.keys(STT_LABELS) as p}
              <option value={p}>{STT_LABELS[p as SttProvider]}</option>
            {/each}
          </select>
        </div>
        {#if STT_CLOUD.includes(sttProvider)}
          <label>
            APIキー（{STT_LABELS[sttProvider]}）
            <input
              type="password"
              bind:value={sttKeys[sttProvider]}
              placeholder={STT_KEY_PLACEHOLDERS[sttProvider]}
            />
          </label>
          {#if sttProvider === "azure"}
            <label>
              Azure リソース名
              <input
                type="text"
                bind:value={sttAzureResource}
                placeholder="例: myspeechresource（.cognitiveservices.azure.com の前）"
              />
            </label>
          {/if}
          {#if sttProvider !== "azure"}
            <label>
              モデル（任意・空で既定）
              <input
                type="text"
                bind:value={sttModel}
                placeholder={STT_MODEL_PLACEHOLDERS[sttProvider]}
              />
            </label>
          {/if}
          <p class="tip warn">
            ⚠ クラウド文字起こしは<strong>音声を端末外（{STT_LABELS[sttProvider]}）へ送信</strong
            >します。各社は既定でAPI音声を学習利用しないと明言していますが、プライバシー重視なら「ローカル」をお使いください。鍵はこの端末の安全な保管領域に保存されます。
          </p>
        {:else}
          <label>
            モデル
            <select bind:value={whisperModel}>
              {#each whisperModels as m}
                <option value={m.id}>{m.label}</option>
              {/each}
            </select>
          </label>
          <p class="tip">
            ローカルの whisper で端末内完結（音声は外部送信されません）。日本語中心なら <strong
              >kotoba-whisper</strong
            > が高精度です。選んだモデルは初回録音時に自動ダウンロードします（大きいモデルは時間がかかります）。
          </p>
        {/if}
      </div>

      <div class="meta-group">
        <span class="meta-title">文字起こしのメタデータ</span>
        <label class="check">
          <input type="checkbox" bind:checked={includeTimestamps} />
          タイムスタンプを含める
        </label>
        <p class="tip">いつ何を話したかの時刻を残し、AIが話の流れを踏まえて整理します。</p>
        <label class="check">
          <input type="checkbox" bind:checked={autoPipeline} />
          停止後、文字起こしから整形まで自動実行する
        </label>
        <p class="tip">
          録音を止めると、文字起こし→AI整形まで一気に実行します（整形プロバイダの鍵が必要）。
        </p>
      </div>

      <div class="meta-group">
        <span class="meta-title">保存</span>
        <label class="check">
          <input type="checkbox" bind:checked={keepText} />
          文字起こしテキストを保存（.txt）
        </label>
        <label class="check">
          <input type="checkbox" bind:checked={saveAudio} />
          録音音声を保存
        </label>
        {#if saveAudio}
          <label>
            音声形式
            <select bind:value={audioFormat}>
              <option value="opus">Opus（高圧縮・モダン / .opus）</option>
              <option value="wav">WAV（無圧縮・確実）</option>
            </select>
          </label>
          <p class="tip">Opusは小容量でジャーナル向き。WAVは無圧縮で容量大ですが確実です。</p>
        {/if}
        <div class="dir-row">
          <span class="tip">保存先: {saveDir || "既定（ドキュメント/QuickScribe）"}</span>
          <button class="btn small ghost" onclick={pickSaveDir}>変更</button>
          <button class="btn small ghost" onclick={openVault}>出力先フォルダを開く</button>
        </div>
        <label>
          出力形式（生の文字起こし）
          <select bind:value={outputFormat}>
            <option value="txt">プレーンテキスト（.txt・本文のみ）</option>
            <option value="md">Markdown（.md・日時/種別などのメタデータ付き）</option>
          </select>
        </label>
        <p class="tip">
          生の文字起こしの保存形式です。Markdownは先頭に作成日時・種別・タグを付けます。<strong
            >整形結果は構造化Markdownのため常に .md で保存</strong
          >されます。ファイル名も <code>transcript-…</code>（生）/
          <code>refined-…</code>（整形）で区別されます。
        </p>
      </div>

      <div class="meta-group">
        <span class="meta-title">アプリ全般</span>
        {#if IS_WINDOWS}
          <label class="check">
            <input type="checkbox" bind:checked={taskbarWidget} />
            タスクバーに録音ウィジェットを表示する
          </label>
          <p class="tip">
            タスクバー上の録音/停止・ウィンドウ表示ボタン（Windows）。OFFにすると非表示になります。
          </p>
        {/if}
        <label class="check">
          <input
            type="checkbox"
            bind:checked={autoStart}
            onchange={() => void onAutoStartChange()}
          />
          PCのログイン時に自動起動する
        </label>
        <p class="tip">
          OSにログインすると QuickScribe
          を自動で起動し、トレイに常駐します（ウィンドウは出ません）。
        </p>
      </div>

      <div class="settings-actions">
        <button class="btn small" onclick={saveSettings}>保存</button>
        <button class="btn small ghost" onclick={() => checkForUpdate(true)}>更新を確認</button>
      </div>
      {#if updateMsg}<p class="muted">{updateMsg}</p>{/if}
    </div>
  </div>
{/if}

<style>
  /* スクロールバーの領域を常時確保し、設定展開でバーが出てもレイアウトがずれないようにする。 */
  :global(html) {
    scrollbar-gutter: stable;
  }
  :global(body) {
    margin: 0;
    background: #f3f4f6;
  }
  main {
    font-family:
      "Segoe UI",
      system-ui,
      -apple-system,
      "Hiragino Kaku Gothic ProN",
      "Noto Sans JP",
      sans-serif;
    color: #1f2330;
    padding: 1.25rem 1.25rem 1.75rem;
    max-width: 560px;
    margin: 0 auto;
  }

  /* 設定はフルスクリーンのモーダルダイアログとして表示する。 */
  .settings-overlay {
    position: fixed;
    inset: 0;
    background: rgba(17, 24, 39, 0.45);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding: 1.5rem;
    overflow-y: auto;
    z-index: 50;
  }
  .settings-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 0.7rem;
  }
  .settings-head h2 {
    margin: 0;
  }
  .close {
    background: none;
    border: none;
    font-size: 1.4rem;
    line-height: 1;
    color: #6b7280;
    cursor: pointer;
    padding: 0 0.2rem;
  }
  .close:hover {
    color: #1f2330;
  }
  header {
    margin-bottom: 1.1rem;
  }
  .title-row {
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
  }
  h1 {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 700;
  }
  .header-actions {
    position: absolute;
    right: 0;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  /* 主要アクション: ジャーナル（アイコン＋ラベル）。補助操作より視覚的に格上げ。 */
  .nav-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    background: #eef2ff;
    color: #4338ca;
    border: 1px solid #e0e7ff;
    border-radius: 8px;
    padding: 0.4rem 0.7rem;
    font-size: 0.82rem;
    font-weight: 600;
    font-family: inherit;
    cursor: pointer;
  }
  .nav-btn:hover {
    background: #e0e7ff;
    border-color: #c7d2fe;
  }
  .nav-btn .ic {
    width: 1.05rem;
    height: 1.05rem;
  }
  /* 補助操作: 設定（アイコンのみ）。 */
  .gear {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    cursor: pointer;
    opacity: 0.55;
    color: #4b5563;
    line-height: 1;
    padding: 0.3rem;
    border-radius: 8px;
  }
  .gear .ic {
    width: 1.2rem;
    height: 1.2rem;
  }
  .gear:hover {
    opacity: 1;
    background: #f3f4f6;
  }
  .ic-sm {
    width: 0.9rem;
    height: 0.9rem;
    vertical-align: -0.13rem;
    margin-right: 0.15rem;
  }
  .tagline {
    /* 薄いグレー背景上でも AA(4.5:1) を満たす濃さ (#395)。 */
    color: #4b5563;
    font-size: 0.8rem;
    margin: 0.25rem 0 0;
    text-align: center;
  }

  .settings {
    background: #fff;
    border: 1px solid #e5e7eb;
    border-radius: 14px;
    padding: 1.1rem 1.2rem;
    width: 100%;
    max-width: 520px;
    box-shadow: 0 8px 30px rgba(17, 24, 39, 0.2);
    text-align: left;
  }
  .settings h2 {
    margin: 0 0 0.7rem;
    font-size: 0.95rem;
  }
  .settings label {
    display: block;
    font-size: 0.76rem;
    color: #4b5563;
    margin-bottom: 0.7rem;
  }
  .settings input,
  .settings select {
    width: 100%;
    box-sizing: border-box;
    margin-top: 0.25rem;
    padding: 0.5rem 0.6rem;
    border: 1px solid #d1d5db;
    border-radius: 8px;
    font-size: 0.85rem;
    background: #fff;
  }
  .settings-actions {
    display: flex;
    gap: 0.5rem;
  }
  .meta-group {
    border-top: 1px solid #eef0f3;
    padding-top: 0.7rem;
    margin-bottom: 0.8rem;
  }
  .meta-title {
    display: block;
    font-size: 0.76rem;
    color: #4b5563;
    font-weight: 600;
    margin-bottom: 0.45rem;
  }
  .hotkey-row {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }
  .device-row {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }
  .device-row select {
    flex: 1;
    padding: 0.5rem 0.6rem;
    border: 1px solid #d1d5db;
    border-radius: 6px;
    background: #fff;
  }
  .custom-list {
    list-style: none;
    margin: 0 0 0.5rem;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  .custom-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    padding: 0.3rem 0.5rem;
    border: 1px solid #e5e7eb;
    border-radius: 6px;
    background: #fff;
  }
  .custom-name {
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .custom-name-input,
  .custom-instruction-input {
    width: 100%;
    padding: 0.5rem 0.6rem;
    border: 1px solid #d1d5db;
    border-radius: 6px;
    background: #fff;
    margin-bottom: 0.4rem;
    box-sizing: border-box;
    font-family: inherit;
  }
  .custom-instruction-input {
    resize: vertical;
  }
  .tags-row {
    margin-top: 0.5rem;
  }
  .corrections {
    margin-top: 0.6rem;
    border: 1px solid #fde68a;
    background: #fffbeb;
    border-radius: 8px;
    padding: 0.5rem 0.6rem;
  }
  .corrections-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    font-size: 0.74rem;
    color: #92400e;
    margin-bottom: 0.4rem;
  }
  .correction-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }
  .correction-item {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    flex-wrap: wrap;
  }
  .correction-check {
    display: flex;
    align-items: center;
    gap: 0.3rem;
  }
  .correction-orig {
    text-decoration: line-through;
    color: #b91c1c;
  }
  .correction-arrow {
    color: #6b7280;
  }
  .correction-sugg {
    flex: 1;
    min-width: 8rem;
    padding: 0.3rem 0.45rem;
    border: 1px solid #d1d5db;
    border-radius: 6px;
    background: #fff;
    font-family: inherit;
  }
  .correction-reason {
    font-size: 0.7rem;
    color: #6b7280;
    max-width: 12rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .corrections-actions {
    display: flex;
    gap: 0.4rem;
    margin-top: 0.5rem;
  }
  /* ジャーナル(旧:保管庫)パネル: 一次情報の余白指針(幅~560/内24/8pxグリッド)を反映し呼吸感を持たせる。 */
  .vault-panel {
    max-width: 560px;
    padding: 1.5rem;
  }
  .vault-panel .settings-head {
    margin-bottom: 1rem;
  }
  .vault-panel .tags-input {
    height: 2.6rem;
    margin-bottom: 0.5rem;
  }
  .tag-filter {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
    margin: 0.75rem 0;
  }
  .entry-list {
    list-style: none;
    margin: 1rem 0 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  .entry-item {
    width: 100%;
    text-align: left;
    background: #fff;
    border: 1px solid #e5e7eb;
    border-radius: 10px;
    padding: 0.85rem 1rem;
    cursor: pointer;
    font-family: inherit;
  }
  .entry-item:hover {
    border-color: #c7d2fe;
    background: #f8faff;
  }
  .entry-meta {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    font-size: 0.72rem;
    color: #6b7280;
    margin-bottom: 0.3rem;
  }
  .entry-kind {
    background: #eef2ff;
    color: #4338ca;
    border-radius: 4px;
    padding: 0 0.3rem;
  }
  .entry-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
    margin: 0.15rem 0;
  }
  .entry-tag {
    font-size: 0.68rem;
    color: #2563eb;
  }
  .entry-preview {
    font-size: 0.82rem;
    color: #374151;
    line-height: 1.5;
  }
  .entry-view {
    white-space: pre-wrap;
    word-break: break-word;
    background: #f9fafb;
    border: 1px solid #e5e7eb;
    border-radius: 8px;
    padding: 0.6rem;
    font-family: inherit;
    font-size: 0.85rem;
    line-height: 1.6;
    max-height: 60vh;
    overflow: auto;
    margin-top: 0.5rem;
  }
  .tags-input {
    width: 100%;
    padding: 0.45rem 0.6rem;
    border: 1px solid #d1d5db;
    border-radius: 6px;
    background: #fff;
    box-sizing: border-box;
    font-family: inherit;
  }
  .hotkey-capture {
    flex: 1;
    text-align: left;
    padding: 0.5rem 0.6rem;
    border: 1px solid #d1d5db;
    border-radius: 8px;
    background: #fff;
    font-size: 0.85rem;
    color: #1f2330;
    cursor: pointer;
  }
  .hotkey-capture:hover {
    border-color: #a5b4fc;
  }
  .hotkey-capture.capturing {
    border-color: #4f46e5;
    background: #eef2ff;
    color: #4338ca;
    font-weight: 600;
  }
  .check {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.84rem;
    color: #1f2330;
    margin-bottom: 0.3rem;
  }
  .check input {
    width: auto;
    margin: 0;
  }
  .tip {
    font-size: 0.7rem;
    color: #6b7280;
    margin: 0.2rem 0 0;
    line-height: 1.5;
  }
  .tip.warn {
    color: #92400e;
    background: #fffbeb;
    border: 1px solid #fde68a;
    border-radius: 6px;
    padding: 0.4rem 0.5rem;
  }
  .dir-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    margin-top: 0.5rem;
  }
  .dir-row .tip {
    word-break: break-all;
  }

  .update-banner {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.6rem;
    flex-wrap: wrap;
    background: #eef2ff;
    border: 1px solid #c7d2fe;
    color: #3730a3;
    border-radius: 12px;
    padding: 0.6rem 0.9rem;
    font-size: 0.82rem;
    margin-bottom: 1rem;
  }
  .update-banner.ready {
    background: #ecfdf5;
    border-color: #a7f3d0;
    color: #065f46;
  }
  .btn-restart {
    border: none;
    background: #059669;
    color: #fff;
    font-weight: 600;
    font-size: 0.8rem;
    padding: 0.4rem 0.8rem;
    border-radius: 8px;
    cursor: pointer;
  }
  .btn-restart:hover {
    background: #047857;
  }

  .actions {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    align-items: stretch;
  }

  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 0.55rem;
    font-size: 1rem;
    font-weight: 600;
    padding: 0.85rem 1.25rem;
    border-radius: 14px;
    border: none;
    cursor: pointer;
    transition:
      background 0.15s ease,
      box-shadow 0.15s ease,
      transform 0.05s ease;
  }
  .btn:active {
    transform: translateY(1px);
  }
  .btn:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }
  .btn.primary {
    background: #4f46e5;
    color: #fff;
    box-shadow: 0 2px 8px rgba(79, 70, 229, 0.35);
  }
  .btn.primary:hover {
    background: #4338ca;
  }
  .btn.primary.recording {
    background: #dc2626;
    box-shadow: 0 2px 8px rgba(220, 38, 38, 0.35);
  }
  .btn.secondary {
    background: #fff;
    color: #4f46e5;
    border: 1.5px solid #c7d2fe;
    box-shadow: 0 1px 4px rgba(17, 24, 39, 0.06);
  }
  .btn.secondary:hover:not(:disabled) {
    background: #eef2ff;
    border-color: #a5b4fc;
  }
  .btn.small {
    font-size: 0.8rem;
    padding: 0.45rem 0.85rem;
    border-radius: 10px;
    background: #4f46e5;
    color: #fff;
  }
  .btn.small.ghost {
    background: #fff;
    color: #4f46e5;
    border: 1px solid #c7d2fe;
  }
  .dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.9);
  }
  .dot.on {
    background: #fff;
    box-shadow: 0 0 0 4px rgba(255, 255, 255, 0.3);
  }

  .hint {
    color: #6b7280;
    font-size: 0.72rem;
    text-align: center;
    margin: 0.7rem 0 1.1rem;
  }

  .panel {
    background: #fff;
    border: 1px solid #e5e7eb;
    border-radius: 14px;
    padding: 0.9rem 1rem;
    box-shadow: 0 1px 4px rgba(17, 24, 39, 0.05);
  }
  .status-row {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    font-size: 0.85rem;
    color: #374151;
  }
  .spinner {
    width: 15px;
    height: 15px;
    border: 2px solid #c7d2fe;
    border-top-color: #4f46e5;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
    flex: none;
    display: inline-block;
    vertical-align: middle;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
  .progress {
    margin-top: 0.7rem;
    height: 8px;
    background: #e5e7eb;
    border-radius: 999px;
    overflow: hidden;
  }
  .bar {
    height: 100%;
    background: linear-gradient(90deg, #6366f1, #4f46e5);
    transition: width 0.2s ease;
  }
  .progress-meta {
    display: flex;
    justify-content: space-between;
    font-size: 0.72rem;
    color: #6b7280;
    margin-top: 0.3rem;
  }
  .progress-meta .pct {
    font-weight: 600;
    color: #4f46e5;
  }
  .segments {
    margin-top: 0.6rem;
    max-height: 120px;
    overflow-y: auto;
    font-size: 0.82rem;
    line-height: 1.6;
    color: #4b5563;
    text-align: left;
  }

  .card {
    margin-top: 1rem;
    background: #fff;
    border: 1px solid #e5e7eb;
    border-radius: 14px;
    padding: 0.9rem 1rem;
    text-align: left;
    box-shadow: 0 1px 4px rgba(17, 24, 39, 0.05);
  }
  .card.refined {
    border-color: #c7d2fe;
    background: #fbfbff;
  }
  /* 段階的深掘り(S3.5): 結果から別スタイルで整形し直すチップ列。控えめに。 */
  .restyle-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.35rem;
    margin-top: 0.6rem;
    padding-top: 0.5rem;
    border-top: 1px solid #eef2ff;
  }
  .restyle-label {
    font-size: 0.74rem;
    color: #6b7280;
  }
  .chip {
    font-size: 0.76rem;
    padding: 0.2rem 0.55rem;
    border: 1px solid #d1d5db;
    border-radius: 999px;
    background: #fff;
    color: #4b5563;
    cursor: pointer;
  }
  .chip:hover:not(:disabled) {
    border-color: #a5b4fc;
    color: #4338ca;
  }
  .chip.active {
    border-color: #6366f1;
    background: #eef2ff;
    color: #4338ca;
    font-weight: 600;
  }
  .chip:disabled {
    opacity: 0.5;
    cursor: default;
  }
  .chip.copy {
    margin-left: auto;
  }
  .card-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 0.5rem;
  }
  .refine-controls {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }
  /* 処理画面: 整形スタイルは表示のみ(選択は設定画面)。ホバーで解説を出す。 */
  .style-indicator {
    font-size: 0.78rem;
    color: #6b7280;
    cursor: help;
  }
  .style-indicator strong {
    color: #374151;
    font-weight: 600;
  }
  /* 設定画面: 選択中スタイルの解説。 */
  .style-desc {
    margin: 0.3rem 0 0;
    font-size: 0.78rem;
    color: #6b7280;
    line-height: 1.5;
  }
  .card h2 {
    margin: 0;
    font-size: 0.82rem;
    color: #6b7280;
    font-weight: 600;
  }
  /* 文字列エリアは固定高でスクロール。アプリ全体はスクロールせず保存先まで見える。 */
  .scroll {
    max-height: 220px;
    overflow-y: auto;
    line-height: 1.75;
    white-space: pre-wrap;
    font-size: 0.9rem;
    padding-right: 0.3rem;
  }

  .muted {
    color: #6b7280;
    font-size: 0.78rem;
  }
  .model-hint {
    margin: -0.2rem 0 0.8rem;
  }
  .model-hint code {
    background: #eef2ff;
    color: #4338ca;
    padding: 0.05rem 0.3rem;
    border-radius: 5px;
    font-size: 0.72rem;
  }
  .center {
    text-align: center;
  }
  .error {
    font-size: 0.78rem;
    color: #b91c1c;
    background: #fef2f2;
    border: 1px solid #fecaca;
    border-radius: 10px;
    padding: 0.6rem 0.8rem;
    margin-top: 0.9rem;
  }

  /* OSの「視差効果を減らす/動きを減らす」設定を尊重する (#395 / 各社指針)。 */
  @media (prefers-reduced-motion: reduce) {
    *,
    *::before,
    *::after {
      animation-duration: 0.001ms !important;
      animation-iteration-count: 1 !important;
      transition-duration: 0.001ms !important;
    }
  }
</style>
