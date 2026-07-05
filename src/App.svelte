<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/plugin-dialog";
  import { createUpdater } from "./lib/update.svelte";
  import { createDeviceStatus } from "./lib/device-status.svelte";
  import { createCustomStyles } from "./lib/custom-styles.svelte";
  import { createPrivacy } from "./lib/privacy.svelte";
  import { onMount } from "svelte";
  import { estimateRemaining, formatRemaining } from "./lib/note";
  import { parseCorrections, applyCorrections, type Correction } from "./lib/corrections";
  import { errorText } from "./lib/errors";
  import { modal } from "./lib/a11y";
  import { kindLabel } from "./lib/entry";
  import { createVaultView } from "./lib/vault-view.svelte";
  import { createShortcut } from "./lib/shortcut-store.svelte";
  import { getSecret, setSecret, loadSecretMigrating } from "./lib/secrets";
  import { parseRecordSource } from "./lib/record-source";
  import { validateRefineConfig, type RefineConfigError } from "./lib/provider-config";
  import { readSettings, writeSettings, type AppSettings } from "./lib/settings-persist";
  import { maybeNudge, requestNudgePermission } from "./lib/nudge";
  import { buildRefineArgs } from "./lib/refine-args";
  import { isModelCacheFresh } from "./lib/model-cache";
  import { selectDiscoveryTargets, buildDiscoveryText } from "./lib/discovery";
  import { _, locale } from "svelte-i18n";
  import { LOCALE_STORAGE_KEY } from "./lib/i18n";
  import { statusText } from "./lib/status";
  import {
    type Provider,
    type SttProvider,
    ALL_PROVIDERS,
    PROVIDER_LABEL_KEYS,
    LOCAL_PROVIDERS,
    AWS_PROVIDERS,
    FALLBACK_MODELS,
    KEY_PLACEHOLDER_KEYS,
    STT_PROVIDERS,
    STT_LABEL_KEYS,
    STT_CLOUD,
    STT_KEY_PLACEHOLDER_KEYS,
    STT_MODEL_PLACEHOLDER_KEYS,
    MODEL_TTL_MS,
    DISCOVERY_MAX,
    MAX_INPUT_MB,
    SUPPORTED_AUDIO_EXTS,
    LANGUAGES,
    languageEnglishName,
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

  // 初回オンボーディング（#397）。初回起動時にコア体験3ステップとローカル完結を案内。
  // 表示済みフラグは localStorage に持ち、以後は出さない。
  const ONBOARDED_KEY = "onboarded";
  let showOnboarding = $state(false);
  function dismissOnboarding() {
    showOnboarding = false;
    try {
      localStorage.setItem(ONBOARDED_KEY, "1");
    } catch {
      /* localStorage 不可環境では単に閉じる */
    }
  }
  // 初回の aha 体験（#57 S9.3）: マイク無しでもサンプル文でコアループ(文字起こし→整形)を
  // 即座に体験させる。整形は本人が「整形する」で実行(プロバイダ未設定なら設定へ誘導)。
  function trySample() {
    transcript = $_("onboarding.sample_text");
    refined = null;
    segments = [];
    error = null;
    dismissOnboarding();
  }

  // 保管庫エントリの横断（S4.3）。一覧＋タグ/全文絞り込み＋閲覧の状態・ロジックは
  // lib/vault-view.svelte.ts へ集約(#392)。App は error への書き戻しのみ受け持つ。
  // 横断発見(discovery)は整形設定に依存するため下記で App 側に残す。
  const vault = createVaultView({ t: $_, onError: (m) => (error = m) });

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
      vault.showEntries = false;
      openSettingsForConfig(cfgErr);
      return;
    }
    const { targets, truncated } = selectDiscoveryTargets(vault.filteredEntries, DISCOVERY_MAX);
    discoveryTruncated = truncated;
    if (targets.length < 2) {
      error = $_("errors.discover_need_two");
      return;
    }
    error = null;
    discovering = true;
    discoveryResult = null;
    try {
      await resolveCurrentModel();
      const items = [];
      for (const e of targets) {
        const content = await invoke<string>("read_text_file", { path: e.path });
        items.push({ created: e.created, tags: e.tags, content });
      }
      const args = refineArgs();
      args.text = buildDiscoveryText(items);
      args.customInstruction = DISCOVERY_INSTRUCTION;
      args.save = false; // 発見結果は一時表示（保管庫を汚さない）。
      delete args.tags;
      discoveryResult = await invoke<string>("refine_text", { params: args });
    } catch (e) {
      error = $_("errors.discover_failed", { values: { detail: errorText(e, $_) } });
    } finally {
      discovering = false;
    }
  }

  let provider = $state<Provider>("ollama"); // 既定=ローカルファースト(#465/ADR-0021)。loadSettingsが上書き
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

  // 録音トグルのグローバルホットキー（状態＋キャプチャ／登録は lib/shortcut-store へ集約 / #392）。
  const shortcut = createShortcut($_);
  // 録音モード（S1.5 / ADR-0014）: toggle=1押しで開始/停止 / momentary=押している間だけ録音。
  let recordMode = $state<"toggle" | "momentary">("toggle");

  // タスクバーウィジェットは Windows 専用機能。設定UIの出し分けに使う。
  const IS_WINDOWS =
    typeof navigator !== "undefined" &&
    /win/i.test(`${navigator.userAgent} ${navigator.platform ?? ""}`);
  // タスクバー上のウィジェット表示（Windows）。既定ON。設定でON/OFF可能。
  let taskbarWidget = $state<boolean>(true);
  // 習慣ナッジ（S9.4 #58）: 起動時に継続中ストリークが未記録なら通知で促す。既定OFF(opt-in)。
  let nudgeEnabled = $state<boolean>(false);

  // OS/デバイス連携（自動起動・録音ソース列挙・whisperモデル一覧）は lib/device-status へ集約(#392)。
  const device = createDeviceStatus({ t: $_, onError: (m) => (error = m) });

  // 録音ソース選択（S1.2/S1.3 / #18 #19）。マイク入力＋出力デバイスのループバックを統一。
  // inputDevice: 入力=デバイス名 / ループバック=レンダーデバイスID（空=OS既定）。永続化する。
  let inputDevice = $state<string>("");
  let inputDeviceKind = $state<string>("input");

  // プルダウンは "kind|id" を値に使う（id がレンダーデバイスIDでも安全に分解できる）。
  function onSourceChange(e: Event) {
    const { kind, id } = parseRecordSource((e.currentTarget as HTMLSelectElement).value);
    inputDeviceKind = kind;
    inputDevice = id;
  }

  // タスクバーウィジェットの表示有効/無効をバックエンドへ反映する（Windowsのみ実体動作）。
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
  // 文字起こし(STT)プロバイダ（S2.4 / ADR-0016）。既定はローカルwhisper（プライバシー）。
  // クラウドは音声を端末外へ送信＝明示選択時のみ。鍵はkeyring保管。
  // STTプロバイダ定義は src/lib/constants.ts に集約(SSOT / #401 Phase0)。
  let sttProvider = $state<SttProvider>("local");
  let sttModel = $state<string>(""); // クラウドのモデルID（空=プロバイダ既定）
  let sttAzureResource = $state<string>(""); // Azureのリソース名（azure時のみ）
  // ローカル whisper のモデル選択（S2.2）。クラウドの sttModel とは分離。永続化する。
  // 選択可能なモデル一覧(whisperModels)は device.loadWhisperModels() で列挙する。
  let whisperModel = $state<string>(""); // 空=既定 base
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
  // 整形出力言語(翻訳 / #453)。ON時、整形結果を outputLang の言語で生成する(1パス・原語の
  // 文字起こしは常に保持)。ある作業期間の「期待する出力状態」として永続保持。既定OFF。
  // 既定言語は起動時のUI言語。UI言語に限らず任意言語(Vietnamese等)を選べる(constants LANGUAGES)。
  let translateOutput = $state<boolean>(false);
  let outputLang = $state<string>("ja");
  // OpenAI互換の接続先(base_url / #593)。空=公式 api.openai.com。上級者が LiteLLM 等のゲートウェイや
  // self-host のローカルLLM(loopbackなら端末内完結扱い)を指定できる。OpenAIプロバイダのときのみ有効。
  let openaiBaseUrl = $state<string>("");
  // カスタム整形スタイル(#392): 状態・統合リスト(allStyles)・追加/削除は lib/custom-styles へ集約。
  // 選択値 refineStyle は設定 state に残すため、削除時のフォールバックはコールバックで委譲する。
  const customStyleStore = createCustomStyles({
    t: $_,
    onError: (m) => (error = m),
    isActive: (v) => refineStyle === v,
    onRemovedActive: () => (refineStyle = "structured"),
  });
  // 現在選択中のスタイル(処理画面の表示・解説に使う)。未知値は既定にフォールバック。
  const currentStyle = $derived(
    customStyleStore.allStyles.find((s) => s.value === refineStyle) ??
      customStyleStore.allStyles[0],
  );

  // ローカル完結（オフライン/プライバシー）モード(#465)は lib/privacy へ集約。provider/sttProvider は設定 state に残すため注入する(#392)。
  const privacy = createPrivacy({
    getProvider: () => provider,
    setProvider: (v) => (provider = v as Provider),
    getSttProvider: () => sttProvider,
    setSttProvider: (v) => (sttProvider = v as SttProvider),
    getBaseUrl: () => openaiBaseUrl,
    syncStt: () => void syncSttSettings(),
  });

  // 設定の読み込み。localStorage 読み書き＋検証は lib/settings-persist.ts に集約(#392)。
  function loadSettings() {
    const s = readSettings(($locale ?? "ja").split("-")[0] || "ja");
    provider = s.provider;
    resolvedModel = s.resolvedModel;
    shortcut.recordShortcut = s.recordShortcut;
    recordMode = s.recordMode;
    includeTimestamps = s.includeTimestamps;
    autoPipeline = s.autoPipeline;
    keepText = s.keepText;
    saveAudio = s.saveAudio;
    audioFormat = s.audioFormat;
    saveDir = s.saveDir;
    outputFormat = s.outputFormat;
    refineStyle = s.refineStyle;
    translateOutput = s.translateOutput;
    outputLang = s.outputLang;
    openaiBaseUrl = s.openaiBaseUrl;
    sttProvider = s.sttProvider;
    privacy.offlineMode = s.offlineMode;
    sttModel = s.sttModel;
    sttAzureResource = s.sttAzureResource;
    whisperModel = s.whisperModel;
    customStyleStore.customStyles = s.customStyles;
    awsRegion = s.awsRegion;
    awsWorkspaceId = s.awsWorkspaceId;
    awsAuthMode = s.awsAuthMode;
    bedrockModel = s.bedrockModel;
    taskbarWidget = s.taskbarWidget;
    inputDevice = s.inputDevice;
    inputDeviceKind = s.inputDeviceKind;
    nudgeEnabled = s.nudgeEnabled;
    // 秘密情報(API鍵/AWS鍵)は keyring から非同期で読む(S3.2)。
    void loadSecrets();
  }

  // 現在の設定状態から永続化用スナップショットを組み立てる（writeSettings / 設定検証で共用）。
  function settingsSnapshot(): AppSettings {
    return {
      provider,
      resolvedModel,
      recordShortcut: shortcut.recordShortcut,
      recordMode,
      includeTimestamps,
      autoPipeline,
      keepText,
      saveAudio,
      audioFormat,
      saveDir,
      outputFormat,
      refineStyle,
      translateOutput,
      outputLang,
      openaiBaseUrl,
      sttProvider,
      offlineMode: privacy.offlineMode,
      sttModel,
      sttAzureResource,
      whisperModel,
      customStyles: customStyleStore.customStyles,
      awsRegion,
      awsWorkspaceId,
      awsAuthMode,
      bedrockModel,
      taskbarWidget,
      inputDevice,
      inputDeviceKind,
      nudgeEnabled,
    };
  }

  // 習慣ナッジ（#58）: 起動時に記録日一覧を取り、継続中で今日未記録なら1回だけ通知で促す。
  // 通知失敗やエラーは致命でない（静かに諦める）。
  async function maybeNudgeOnStartup() {
    if (!nudgeEnabled) return;
    try {
      const entries = await invoke<{ created: string }[]>("list_entries");
      await maybeNudge({
        enabled: nudgeEnabled,
        dates: entries.map((e) => e.created),
        today: new Date().toISOString(),
        title: $_("nudge.title"),
        body: $_("nudge.body"),
      });
    } catch {
      /* 通知は補助機能。失敗しても本体機能に影響させない */
    }
  }

  // 設定でナッジを ON にした瞬間に通知権限を要求する（拒否されたら OFF へ戻す）。
  async function onToggleNudge() {
    if (nudgeEnabled) {
      const granted = await requestNudgePermission().catch(() => false);
      if (!granted) nudgeEnabled = false;
    }
    saveSettings();
  }

  // 秘密情報ブリッジ(keyring / S3.2)は lib/secrets.ts へ抽出(#392)。App は鍵の反映のみ担う。
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
      error = $_("errors.open_output_failed", { values: { detail: errorText(e, $_) } });
    }
  }
  function saveSettings() {
    // 未設定でも保存する(文字起こしのみで使う人のため / #603)。整形に必要な設定(選択中クラウド
    // プロバイダのAPIキー/AWS資格情報)が不足していてもブロックせず、警告だけ出す。従来#516は保存を
    // ガードしていたが、文字起こし完結ペルソナのため緩和。鍵が空でも整形時の「鍵なしエラー」で
    // 二重に安全(保存＝設定の永続化であって整形の実行ではない)。
    const cfgErr = refineConfigError();
    settingsError = "";
    settingsWarning = "";
    // 鍵(API鍵/AWS鍵)は keyring に保存する(localStorageには置かない / S3.2)。
    void saveSecrets();
    void shortcut.applyShortcut();
    // 設定フォームの値は lib/settings-persist の writeSettings で localStorage へ書き戻す。
    writeSettings(settingsSnapshot());
    void device.applyTaskbarWidget(taskbarWidget);
    void syncSaveSettings();
    void syncSttSettings();
    // 鍵が入っていれば現在のプロバイダの最新モデルを取得（強制更新）。
    void resolveCurrentModel(true);
    if (cfgErr) {
      // 保存はしたが整形設定が不足。警告を出し、設定パネルは開いたまま(気づけるように)。
      settingsWarning = $_("settings.save_warning_incomplete");
    } else {
      showSettings = false;
    }
  }

  // キー入力イベントから Tauri アクセラレータ表記を組み立てる（修飾キー＋1キー）。
  // 現在のプロバイダの最新ミドルレンジモデルを実行時に解決し、キャッシュする。
  // force=false かつキャッシュが新しければ何もしない。鍵未入力なら何もしない。
  async function resolveCurrentModel(force = false) {
    const p = provider;
    // AWS系はモデル一覧APIが別系統のため自動解決しない(フォールバック/手入力)。
    if (AWS_PROVIDERS.includes(p)) return;
    if (!apiKeys[p].trim()) return;
    const at = Number(localStorage.getItem(`resolvedModelAt:${p}`) || 0);
    if (!force && isModelCacheFresh(resolvedModel[p], at, Date.now(), MODEL_TTL_MS)) return;
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

  // 自動アップデート（状態と操作は lib/update.svelte.ts へ集約 / #392）。
  const updater = createUpdater($_);

  // ローカル whisper モデルの表示名。カタログ(catalog.whisper_models.<id>)を優先し、
  // 未収載の id はバックエンドのラベルへフォールバックする（新モデル追加時も壊れない）。
  function whisperModelLabel(m: { id: string; label: string }): string {
    const key = `catalog.whisper_models.${m.id}`;
    const v = $_(key);
    return v === key ? m.label : v;
  }

  // トレイのメニュー/ツールチップ文言をバックエンドへ反映する（#462: Rust側の日本語固定を排除）。
  // トレイ文字列は Rust 側で必要なため、フロントが現在のUI言語で解決して渡す。
  async function syncTrayTexts() {
    try {
      await invoke("set_tray_texts", {
        record: $_("tray.record"),
        show: $_("tray.show"),
        quit: $_("tray.quit"),
        tooltipRecording: $_("tray.tooltip_recording"),
        tooltipIdle: $_("tray.tooltip_idle"),
      });
    } catch (e) {
      console.error("set_tray_texts failed", e);
    }
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
      filters: [{ name: $_("dialog.audio_files"), extensions: SUPPORTED_AUDIO_EXTS }],
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
      error = $_("errors.transcribe_failed", { values: { detail: errorText(e, $_) } });
    } finally {
      busy = false;
      status = "";
    }
  }

  // 文字起こしを整形（思考整理・要約）する＝コア価値。選択中プロバイダの鍵が必要。
  // プロバイダが整形可能な設定になっているか（鍵/AWS資格情報）。未設定なら i18n コードを返す。
  function refineConfigError(): RefineConfigError | null {
    return validateRefineConfig({
      provider,
      apiKey: apiKeys[provider],
      awsRegion,
      awsWorkspaceId,
      awsAuthMode,
      awsAccessKey,
      awsSecretKey,
    });
  }

  // 設定のタブ(#512): 一般/録音/文字起こし/整形/出力で目的の設定に素早く到達する。
  let settingsTab = $state<"general" | "recording" | "transcription" | "refine" | "output">(
    "general",
  );
  // 未設定時の動線(#516): 設定パネル内に表示する検証エラーと、不足項目へのフォーカス。
  let settingsError = $state<string>("");
  // 非ブロックの警告(#603): 未設定でも保存はするが「この設定では整形が失敗する」と知らせる。
  let settingsWarning = $state<string>("");
  // エラーコード→フォーカス対象の要素id。該当項目まで誘導する。
  function configFieldId(code: string): string | null {
    if (code === "errors.cfg_api_key" || code === "errors.cfg_api_key_aws") return "cfg-api-key";
    if (code === "errors.cfg_aws_region") return "cfg-aws-region";
    if (code === "errors.cfg_workspace_id") return "cfg-aws-workspace";
    if (code === "errors.cfg_aws_keys") return "cfg-aws-access";
    return null;
  }
  // 設定を開いて不足項目を明示＋フォーカスする(#516)。整形導線から設定不足時に呼ぶ。
  function openSettingsForConfig(err: RefineConfigError) {
    showSettings = true;
    settingsTab = "refine"; // 整形の鍵/AWS設定は整形タブにあるため、そのタブへ切替えて誘導。
    // params.provider は表示ラベルの i18n キー(catalog.providers.*) → 先に翻訳して補間する。
    const values = err.params?.provider
      ? { ...err.params, provider: $_(err.params.provider) }
      : err.params;
    settingsError = $_(err.code, { values });
    settingsWarning = ""; // ブロックのエラー表示時は非ブロック警告を消す(二重表示回避)。
    const id = configFieldId(err.code);
    if (id) {
      queueMicrotask(() => {
        const el = document.getElementById(id);
        el?.scrollIntoView?.({ block: "center" });
        (el as HTMLElement | null)?.focus?.();
      });
    }
  }

  // refine_text に渡す追加引数（AWS時のみ資格情報。非AWSは undefined＝後方互換）。
  // styleOverride: 結果から別スタイルで整形し直す時に既定スタイルを上書きする(S3.5・行き来)。
  function refineArgs(styleOverride?: string) {
    return buildRefineArgs({
      transcript,
      provider,
      apiKey: apiKeys[provider],
      bedrockModel,
      resolvedModel: resolvedModel[provider],
      style: styleOverride ?? refineStyle,
      customStyles: customStyleStore.customStyles,
      entryTags,
      awsRegion,
      awsWorkspaceId,
      awsAuthMode,
      awsAccessKey,
      awsSecretKey,
      awsSessionToken,
      translateOutput,
      outputLang,
      openaiBaseUrl,
    });
  }

  async function refineNow(styleOverride?: string) {
    if (!transcript) return;
    const cfgErr = refineConfigError();
    if (cfgErr) {
      openSettingsForConfig(cfgErr);
      return;
    }
    error = null;
    refining = true;
    refined = null;
    try {
      // 整形直前に最新モデルを確保（キャッシュが新しければ即返る。AWSはスキップ）。
      await resolveCurrentModel();
      refined = await invoke<string>("refine_text", { params: refineArgs(styleOverride) });
      refinedStyle = styleOverride ?? refineStyle; // どのスタイルで整形したか(再整形チップの強調用)。
    } catch (e) {
      error = $_("errors.refine_failed", { values: { detail: errorText(e, $_) } });
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
      openSettingsForConfig(cfgErr);
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
      const raw = await invoke<string>("refine_text", { params: args });
      corrections = parseCorrections(raw);
    } catch (e) {
      error = $_("errors.term_check_failed", { values: { detail: errorText(e, $_) } });
    } finally {
      checkingTerms = false;
    }
  }
  // 選択された候補を文字起こしに適用（全置換）し、確認パネルを閉じる。
  async function applyCorrectionsToTranscript() {
    if (!corrections || !transcript) return;
    const { text, applied } = applyCorrections(transcript, corrections);
    transcript = text;
    corrections = null;
    if (applied <= 0) {
      status = "";
      return;
    }
    // 補正済みの文字起こしを別ファイルで残す(#599)。原本は書き換えない=非破壊(ADR-0017)。
    // テキスト保存を切っている(keepText=false)人の意思は尊重し、その場合は保存しない。
    if (keepText) {
      try {
        await invoke("save_note", { content: text, kind: "transcript" });
        status = $_("corrections.replaced_saved", { values: { n: applied } });
      } catch (e) {
        status = $_("corrections.replaced", { values: { n: applied } });
        error = $_("errors.save_corrected_failed", { values: { detail: errorText(e, $_) } });
      }
    } else {
      status = $_("corrections.replaced", { values: { n: applied } });
    }
  }

  // 整形結果をクリップボードへコピー(S3.5・最小操作の利便)。
  let copyMsg = $state<string>("");
  async function copyRefined() {
    if (!refined) return;
    try {
      await navigator.clipboard.writeText(refined);
      copyMsg = $_("results.copied");
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
      filters: [{ name: $_("dialog.text_memo"), extensions: ["txt", "md", "markdown", "text"] }],
    });
    if (typeof selected !== "string") return;
    try {
      const text = await invoke<string>("read_text_file", { path: selected });
      // 空・空白のみのファイルは整形しても意味が無く、無駄なAPI呼び出しになる(#18)。
      if (!text.trim()) {
        error = $_("errors.empty_text");
        return;
      }
      transcript = text;
      refined = null;
      segments = [];
      await refineNow();
    } catch (e) {
      error = $_("errors.memo_refine_failed", { values: { detail: errorText(e, $_) } });
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
        error = $_("errors.record_start_failed", { values: { detail: errorText(e, $_) } });
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
        error = $_("errors.record_stop_failed", { values: { detail: errorText(e, $_) } });
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
    // 起動時間ベンチ(#403): UI 準備完了を通知（QS_PERF_STARTUP 設定時のみ Rust 側が計測）。
    void invoke("report_startup").catch(() => {});
    loadSettings();
    // 初回起動（未表示）ならオンボーディングを出す（#397）。
    try {
      if (localStorage.getItem(ONBOARDED_KEY) !== "1") showOnboarding = true;
    } catch {
      /* localStorage 不可環境では出さない */
    }
    void shortcut.applyShortcut();
    void device.applyTaskbarWidget(taskbarWidget);
    void device.loadAutoStart();
    void device.loadAudioSources();
    void device.loadWhisperModels();
    void syncSaveSettings();
    void syncTrayTexts();
    void resolveCurrentModel();
    void updater.checkForUpdate();
    // 習慣ナッジ（#58）: 起動をアンカーに、継続中ストリークが今日未記録なら1回だけ促す（opt-in）。
    void maybeNudgeOnStartup();
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
    // status は Rust から安定コード(S_XXX)で届く → カタログでローカライズ(#462)。
    const unStatus = listen<string>("status", (e) => (status = statusText(e.payload, $_)));
    const unProgress = listen<number>("progress", (e) => {
      progress = e.payload;
      if (progress > 0 && transcribeStartMs === null) transcribeStartMs = Date.now();
      if (transcribeStartMs && progress > 0 && progress < 100) {
        const elapsed = (Date.now() - transcribeStartMs) / 1000;
        eta = formatRemaining(estimateRemaining(elapsed, progress), $_);
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
        error = $_("errors.no_audio");
      }
    });
    const unErr = listen<string>("transcribe-error", (e) => {
      transcribing = false;
      status = "";
      // 安定エラーコード(E_XXX)をローカライズして表示（生コードを露出させない / #462）。
      error = errorText(e.payload, $_);
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

<main inert={showSettings || vault.showEntries}>
  <div class="content">
    <header>
      <div class="title-row">
        <h1>QuickScribe</h1>
        <div class="header-actions">
          <button
            class="gear"
            title={$_("header.journal_title")}
            aria-label={$_("header.journal")}
            onclick={vault.openPanel}
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
          </button>
          <button
            class="gear"
            data-testid="settings-btn"
            title={$_("header.settings")}
            aria-label={$_("header.settings")}
            onclick={() => {
              settingsError = "";
              showSettings = !showSettings;
            }}
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
      <p class="tagline">{$_("main.tagline")}</p>
    </header>

    {#if updater.updateState === "downloading"}
      <div class="update-banner">
        <span class="spinner" aria-hidden="true"></span>
        {$_("update.downloading", {
          values: { version: updater.updateVersion, pct: updater.updatePct },
        })}
      </div>
    {:else if updater.updateState === "ready"}
      <div class="update-banner ready">
        <span>{$_("update.ready", { values: { version: updater.updateVersion } })}</span>
        <button class="btn-restart" onclick={updater.restartNow}>{$_("update.restart")}</button>
      </div>
    {/if}

    <!-- オンボーディングは操作ボタンより上部に置き、初回に最初に認識させる(#510)。 -->
    {#if showOnboarding && !recording && !busy && !transcribing && !status && !transcript && !refined}
      <section class="onboarding-card" aria-labelledby="onboarding-title">
        <div class="onboarding-head">
          <h2 id="onboarding-title">{$_("onboarding.title")}</h2>
          <button
            class="close"
            aria-label={$_("onboarding.skip")}
            onclick={dismissOnboarding}
            onkeydown={(e) => {
              if (e.key === "Escape") dismissOnboarding();
            }}>×</button
          >
        </div>
        <p class="onboarding-local">{$_("onboarding.local_note")}</p>
        <ol class="onboarding-steps">
          <li>
            <span class="onboarding-step-n" aria-hidden="true">1</span>
            <div>
              <strong>{$_("onboarding.step1_title")}</strong>
              <p>{$_("onboarding.step1_desc")}</p>
            </div>
          </li>
          <li>
            <span class="onboarding-step-n" aria-hidden="true">2</span>
            <div>
              <strong>{$_("onboarding.step2_title")}</strong>
              <p>{$_("onboarding.step2_desc")}</p>
            </div>
          </li>
          <li>
            <span class="onboarding-step-n" aria-hidden="true">3</span>
            <div>
              <strong>{$_("onboarding.step3_title")}</strong>
              <p>{$_("onboarding.step3_desc")}</p>
            </div>
          </li>
        </ol>
        <div class="onboarding-actions">
          <button class="btn small" onclick={trySample}>{$_("onboarding.try_sample")}</button>
          <button class="btn small ghost" onclick={dismissOnboarding}
            >{$_("onboarding.start")}</button
          >
        </div>
      </section>
    {/if}

    <div class="actions">
      <button
        class="btn primary"
        class:recording
        data-testid="record-btn"
        aria-pressed={recording}
        onclick={toggle}
      >
        <span class="dot" class:on={recording}></span>
        {recording ? $_("main.record_stop") : $_("main.record_start")}
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
        {$_("main.from_file")}
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
        {$_("main.from_memo")}
      </button>
    </div>

    <p class="hint">
      {$_("main.formats_hint", {
        values: { exts: SUPPORTED_AUDIO_EXTS.join(" / "), max: MAX_INPUT_MB },
      })}
    </p>
    <p class="hint">
      {$_("main.hotkey_hint", { values: { key: shortcut.display() } })}
    </p>

    <!-- 初回オンボーディング（#397）: 初回はコア体験3ステップ＋ローカル完結を非ブロッキングの
         インラインカードで案内（リッチすぎて簡便さを損なわないよう、操作を妨げない）。
         以降の空状態では軽量な「まず録音」導線のみ。 -->
    {#if !showOnboarding && !recording && !busy && !transcribing && !status && !transcript && !refined}
      <p class="empty-cta">
        {$_("main.empty_cta", { values: { key: shortcut.display() } })}
      </p>
    {/if}

    {#if busy || transcribing || status}
      <div class="panel" role="status" aria-live="polite">
        <div class="status-row">
          <span class="spinner" aria-hidden="true"></span>
          <span class="status-text">{status || $_("results.processing")}</span>
        </div>
        {#if progress > 0}
          <div
            class="progress"
            role="progressbar"
            aria-label={$_("results.processing")}
            aria-valuenow={progress}
            aria-valuemin="0"
            aria-valuemax="100"
          >
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
          <h2>{$_("results.transcript")}</h2>
          <div class="refine-controls">
            <!-- スタイルは設定画面で選ぶ。ここでは「どのスタイルで整形するか」の表示のみ
               (マウスオーバーでモードの解説を表示)。 -->
            <span class="style-indicator" title={currentStyle.desc}>
              {$_("results.style_label")} <strong>{currentStyle.label}</strong>
            </span>
            <button
              class="btn small ghost"
              title={$_("results.term_check_title")}
              onclick={suggestCorrections}
              disabled={checkingTerms || refining}
            >
              {checkingTerms ? $_("results.term_checking") : $_("results.term_check")}
            </button>
            <button class="btn small" onclick={() => refineNow()} disabled={refining}>
              {refining ? $_("results.refining") : $_("results.refine")}
            </button>
            <!-- 文字起こしのみで完結する人向け: 保存先フォルダをここからも開ける(#603)。 -->
            <button
              type="button"
              class="btn small ghost"
              onclick={openVault}
              title={$_("results.open_output_title")}
            >
              {$_("results.open_output")}
            </button>
          </div>
        </div>
        <div class="scroll">{transcript}</div>

        <!-- 用語補正フェーズ: 誤変換疑いの候補を確認→置換（置換しない選択肢付き）。 -->
        {#if corrections !== null}
          {#if corrections.length === 0}
            <p class="tip">{$_("corrections.none_found")}</p>
          {:else}
            <div class="corrections">
              <div class="corrections-head">
                <span>{$_("corrections.head", { values: { n: corrections.length } })}</span>
                <button
                  type="button"
                  class="btn small ghost"
                  onclick={() => corrections && corrections.forEach((c) => (c.replace = false))}
                >
                  {$_("corrections.clear_all")}
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
                    <input
                      class="correction-sugg"
                      type="text"
                      bind:value={c.suggestion}
                      aria-label={$_("corrections.suggestion_label")}
                    />
                    {#if c.reason}<span class="correction-reason" title={c.reason}>{c.reason}</span
                      >{/if}
                  </li>
                {/each}
              </ul>
              <div class="corrections-actions">
                <button class="btn small" onclick={applyCorrectionsToTranscript}
                  >{$_("corrections.apply")}</button
                >
                <button class="btn small ghost" onclick={() => (corrections = null)}
                  >{$_("corrections.close")}</button
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
            aria-label={$_("results.tags_placeholder")}
            placeholder={$_("results.tags_placeholder")}
          />
        </div>
      </section>
    {/if}

    {#if refining}
      <p class="muted center">
        <span class="spinner" aria-hidden="true"></span>
        {$_("results.refining_status")}
      </p>
    {/if}
    {#if refined}
      <section class="card refined">
        <h2>{$_("results.refined_title")}</h2>
        <div class="scroll">{refined}</div>
        <!-- 段階的深掘り(S3.5): 結果から別スタイルで整形し直す(再文字起こし不要＝逐語⇄要約⇄ブレストを行き来)。 -->
        <div class="restyle-row">
          <span class="restyle-label">{$_("results.restyle_label")}</span>
          {#each customStyleStore.allStyles as s}
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
            {copyMsg || $_("results.copy")}
          </button>
          <button
            type="button"
            class="chip"
            onclick={openVault}
            title={$_("results.open_output_title")}
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
            {$_("results.open_output")}
          </button>
        </div>
      </section>
    {/if}

    {#if error}
      <p class="error" role="alert">{error}</p>
    {/if}
  </div>
</main>

{#if vault.showEntries}
  <div
    class="settings-overlay"
    role="presentation"
    onclick={(e) => {
      if (e.target === e.currentTarget) vault.showEntries = false;
    }}
  >
    <div
      class="settings vault-panel"
      role="dialog"
      aria-modal="true"
      aria-labelledby="vault-title"
      tabindex="-1"
      use:modal={{ onClose: () => (vault.showEntries = false) }}
    >
      <div class="settings-head">
        <h2 id="vault-title">
          {vault.viewingEntry
            ? $_("vault.title_viewing", { values: { name: vault.viewingEntry.name } })
            : $_("header.journal")}
        </h2>
        {#if !vault.viewingEntry && vault.journalStreak > 0}
          <span class="streak-badge" title={$_("vault.streak_title")}
            >{$_("vault.streak", { values: { n: vault.journalStreak } })}</span
          >
        {/if}
        <button
          class="close"
          aria-label={$_("header.close")}
          onclick={() => (vault.showEntries = false)}>×</button
        >
      </div>

      {#if discoveryResult !== null}
        <button class="btn small ghost" onclick={() => (discoveryResult = null)}
          >{$_("vault.back_to_list")}</button
        >
        <p class="tip">
          {$_("vault.discovery_result_tip", {
            values: {
              n: Math.min(vault.filteredEntries.length, 30),
              detail: discoveryTruncated
                ? $_("vault.discovery_truncated", {
                    values: { total: vault.filteredEntries.length },
                  })
                : "",
            },
          })}
        </p>
        <pre class="entry-view">{discoveryResult}</pre>
      {:else if vault.viewingEntry}
        <button class="btn small ghost" onclick={() => (vault.viewingEntry = null)}
          >{$_("vault.back_to_list")}</button
        >
        <pre class="entry-view">{vault.viewingEntry.content}</pre>
      {:else}
        <input
          class="tags-input"
          type="text"
          bind:value={vault.entrySearch}
          aria-label={$_("vault.search_placeholder")}
          placeholder={$_("vault.search_placeholder")}
        />
        {#if vault.allTags.length > 0}
          <div class="tag-filter">
            {#each vault.allTags as t}
              <button
                type="button"
                class="chip"
                class:active={vault.selectedTags.includes(t)}
                onclick={() => vault.toggleTag(t)}>#{t}</button
              >
            {/each}
          </div>
        {/if}
        {#if vault.filteredEntries.length >= 2}
          <button type="button" class="btn small" disabled={discovering} onclick={discoverAcross}>
            {discovering
              ? $_("vault.discovering")
              : $_("vault.discover", { values: { n: vault.filteredEntries.length } })}
          </button>
          <p class="tip">{$_("vault.discover_tip")}</p>
        {/if}
        {#if vault.entriesLoading}
          <p class="muted center">
            <span class="spinner" aria-hidden="true"></span>
            {$_("vault.loading")}
          </p>
        {:else if vault.filteredEntries.length === 0}
          <p class="tip">
            {vault.entries.length === 0 ? $_("vault.empty") : $_("vault.no_match")}
          </p>
        {:else}
          <ul class="entry-list">
            {#each vault.filteredEntries as e}
              <li>
                <button type="button" class="entry-item" onclick={() => vault.openEntry(e)}>
                  <div class="entry-meta">
                    <span class="entry-date">{e.created.replace("T", " ")}</span>
                    {#if e.kind}<span class="entry-kind">{$_(kindLabel(e.kind))}</span>{/if}
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
        <h2 id="settings-title">{$_("settings.title")}</h2>
        <button class="close" aria-label={$_("header.close")} onclick={() => (showSettings = false)}
          >×</button
        >
      </div>

      <!-- 不足設定の明示(#516): 保存不可/整形不可の原因を設定内で表示し、該当項目へ誘導。 -->
      {#if settingsError}
        <p class="settings-error" role="alert">{settingsError}{$_("errors.config_suffix")}</p>
      {/if}
      <!-- 非ブロックの警告(#603): 未設定でも保存はしたが整形は失敗する旨を知らせる。 -->
      {#if settingsWarning}
        <p class="settings-warning" role="status">{settingsWarning}</p>
      {/if}

      <!-- 設定タブ(#512): 5分類で目的の設定へ素早く到達する。 -->
      <div class="settings-tabs" role="tablist">
        {#each [["general", "tab_general"], ["recording", "tab_recording"], ["transcription", "tab_transcription"], ["refine", "tab_refine"], ["output", "tab_output"]] as [id, key] (id)}
          <button
            type="button"
            role="tab"
            class="settings-tab"
            class:active={settingsTab === id}
            aria-selected={settingsTab === id}
            onclick={() => (settingsTab = id as typeof settingsTab)}>{$_("settings." + key)}</button
          >
        {/each}
      </div>

      {#if settingsTab === "general"}
        <!-- プライバシー状態インジケータ(#465): 現在の構成が完全オンデバイスか、クラウド送信を
           伴うかを常時可視化。クラウド時はワンクリックでローカルAIへ切り替えられる。 -->
        <div
          class="privacy-status"
          class:local={privacy.isFullyLocal}
          class:cloud={!privacy.isFullyLocal}
        >
          <span class="privacy-dot" aria-hidden="true"></span>
          <div class="privacy-text">
            <strong>{privacy.isFullyLocal ? $_("privacy.on_device") : $_("privacy.cloud")}</strong>
            <p>{privacy.isFullyLocal ? $_("privacy.on_device_desc") : $_("privacy.cloud_desc")}</p>
          </div>
          {#if !privacy.isFullyLocal && !privacy.offlineMode}
            <button type="button" class="btn small ghost" onclick={privacy.makeOffline}
              >{$_("privacy.make_offline")}</button
            >
          {/if}
        </div>
        <label class="check">
          <input
            type="checkbox"
            checked={privacy.offlineMode}
            onchange={(e) => privacy.setOfflineMode(e.currentTarget.checked)}
          />
          {$_("privacy.offline_mode")}
        </label>
        <p class="tip">{$_("privacy.offline_mode_desc")}</p>
        <!-- 一気通貫(停止→文字起こし→整形の自動実行)はアプリ全体の挙動＝一般タブに置く(#512改善)。 -->
        <label class="check">
          <input type="checkbox" bind:checked={autoPipeline} />
          {$_("settings.auto_pipeline")}
        </label>
        <p class="tip">{$_("settings.tip_auto_pipeline")}</p>
      {/if}

      {#if settingsTab === "recording"}
        <span class="meta-title">{$_("settings.group_hotkey")}</span>
        <div class="hotkey-row">
          <button
            type="button"
            class="hotkey-capture"
            class:capturing={shortcut.capturing}
            onclick={shortcut.startCapture}
            onkeydown={shortcut.onCaptureKeydown}
            onblur={shortcut.cancelCapture}
          >
            {#if shortcut.capturing}
              {$_("settings.press_key")}
            {:else}
              {shortcut.display()}
            {/if}
          </button>
          <button type="button" class="btn small ghost" onclick={shortcut.resetShortcut}
            >{$_("settings.reset_default")}</button
          >
        </div>
        <p class="tip">
          {$_("settings.tip_hotkey", { values: { key: shortcut.display() } })}
        </p>
        {#if shortcut.shortcutMsg}<p class="muted" role="status" aria-live="polite">
            {shortcut.shortcutMsg}
          </p>{/if}

        <details class="meta-group" open>
          <summary class="meta-title">{$_("settings.group_record_mode")}</summary>
          <div class="device-row">
            <select bind:value={recordMode} aria-label={$_("settings.group_record_mode")}>
              <option value="toggle">{$_("settings.mode_toggle")}</option>
              <option value="momentary">{$_("settings.mode_momentary")}</option>
            </select>
          </div>
          <p class="tip">
            {$_("settings.tip_momentary_1")}<strong>{$_("settings.tip_momentary_strong")}</strong
            >{$_("settings.tip_momentary_2")}
          </p>
        </details>

        <details class="meta-group">
          <summary class="meta-title">{$_("settings.group_record_source")}</summary>
          <div class="device-row">
            <select
              value={`${inputDeviceKind}|${inputDevice}`}
              onchange={onSourceChange}
              aria-label={$_("settings.group_record_source")}
            >
              <option value="input|">{$_("settings.source_default_mic")}</option>
              {#if IS_WINDOWS}
                <option value="mix|">{$_("settings.source_mix")}</option>
              {/if}
              {#each device.audioSources as s}
                <option value={`${s.kind}|${s.id}`}>{s.label}</option>
              {/each}
            </select>
            <button
              type="button"
              class="btn small ghost"
              onclick={() => void device.loadAudioSources()}
            >
              {$_("settings.reload")}
            </button>
          </div>
          <p class="tip">{$_("settings.tip_record_source")}</p>
        </details>
      {/if}

      {#if settingsTab === "transcription"}
        <details class="meta-group" open>
          <summary class="meta-title">{$_("settings.group_stt")}</summary>
          <div class="device-row">
            <select
              bind:value={sttProvider}
              aria-label={$_("settings.group_stt")}
              disabled={privacy.offlineMode}
            >
              {#each STT_PROVIDERS as p (p)}
                <option value={p}>{$_(STT_LABEL_KEYS[p])}</option>
              {/each}
            </select>
          </div>
          {#if STT_CLOUD.includes(sttProvider)}
            <label>
              {$_("settings.stt_api_key", {
                values: { provider: $_(STT_LABEL_KEYS[sttProvider]) },
              })}
              <input
                type="password"
                bind:value={sttKeys[sttProvider]}
                placeholder={$_(STT_KEY_PLACEHOLDER_KEYS[sttProvider])}
              />
            </label>
            {#if sttProvider === "azure"}
              <label>
                {$_("settings.azure_resource")}
                <input
                  type="text"
                  bind:value={sttAzureResource}
                  placeholder={$_("settings.azure_resource_ph")}
                />
              </label>
            {/if}
            {#if sttProvider !== "azure"}
              <label>
                {$_("settings.stt_model_optional")}
                <input
                  type="text"
                  bind:value={sttModel}
                  placeholder={$_(STT_MODEL_PLACEHOLDER_KEYS[sttProvider])}
                />
              </label>
            {/if}
            <p class="tip warn">
              {$_("settings.tip_stt_warn_1")}<strong
                >{$_("settings.tip_stt_warn_strong", {
                  values: { provider: $_(STT_LABEL_KEYS[sttProvider]) },
                })}</strong
              >{$_("settings.tip_stt_warn_2")}
            </p>
          {:else}
            <label>
              {$_("settings.stt_model")}
              <select bind:value={whisperModel}>
                {#each device.whisperModels as m}
                  <option value={m.id}>{whisperModelLabel(m)}</option>
                {/each}
              </select>
            </label>
            <p class="tip">
              {$_("settings.tip_whisper_1")}<strong>kotoba-whisper</strong>{$_(
                "settings.tip_whisper_2",
              )}
            </p>
            <p class="tip">{$_("settings.tip_model_download")}</p>
          {/if}
        </details>

        <details class="meta-group">
          <summary class="meta-title">{$_("settings.group_transcribe_meta")}</summary>
          <label class="check">
            <input type="checkbox" bind:checked={includeTimestamps} />
            {$_("settings.include_timestamps")}
          </label>
          <p class="tip">{$_("settings.tip_timestamps")}</p>
        </details>
      {/if}

      {#if settingsTab === "refine"}
        <details class="meta-group" open>
          <summary class="meta-title">{$_("settings.group_refine")}</summary>
          <label>
            {$_("settings.label_refine_provider")}
            <select
              bind:value={provider}
              onchange={() => resolveCurrentModel()}
              disabled={privacy.offlineMode}
            >
              {#each ALL_PROVIDERS as p (p)}
                <option value={p}>{$_(PROVIDER_LABEL_KEYS[p])}</option>
              {/each}
            </select>
          </label>
          {#if !LOCAL_PROVIDERS.includes(provider)}
            <!-- クラウド整形の送信同意(#465): 何が端末外へ出るかを明示（#397 STT警告の整形版）。 -->
            <p class="tip warn">
              {$_("settings.tip_refine_warn_1")}<strong
                >{$_("settings.tip_refine_warn_strong", {
                  values: { provider: $_(PROVIDER_LABEL_KEYS[provider]) },
                })}</strong
              >{$_("settings.tip_refine_warn_2")}
            </p>
          {/if}
          {#if LOCAL_PROVIDERS.includes(provider)}
            <p class="muted">
              {$_("settings.ollama_note_1")}<code>ollama serve</code>{$_(
                "settings.ollama_note_2",
              )}<code>ollama pull llama3.1</code>{$_("settings.ollama_note_3")}
            </p>
          {:else if AWS_PROVIDERS.includes(provider)}
            <!-- AWSプロバイダ(Bedrock / Claude Platform on AWS) / ADR-0011。SigV4 or APIキー。 -->
            <label>
              {$_("settings.label_aws_region")}
              <input
                id="cfg-aws-region"
                type="text"
                bind:value={awsRegion}
                placeholder="us-east-1"
                autocomplete="off"
              />
            </label>
            {#if provider === "claude-aws"}
              <label>
                {$_("settings.label_workspace_id")}
                <input
                  type="text"
                  id="cfg-aws-workspace"
                  bind:value={awsWorkspaceId}
                  placeholder="wrkspc_..."
                  autocomplete="off"
                />
              </label>
            {/if}
            {#if provider === "bedrock"}
              <label>
                {$_("settings.label_bedrock_model")}
                <input
                  type="text"
                  bind:value={bedrockModel}
                  placeholder="anthropic.claude-sonnet-4-6"
                  autocomplete="off"
                />
              </label>
            {/if}
            <label>
              {$_("settings.label_auth_mode")}
              <select bind:value={awsAuthMode}>
                <option value="sigv4">{$_("settings.auth_sigv4")}</option>
                <option value="apikey">{$_("settings.auth_apikey")}</option>
              </select>
            </label>
            {#if awsAuthMode === "sigv4"}
              <label>
                {$_("settings.label_aws_access_key")}
                <input
                  type="password"
                  id="cfg-aws-access"
                  bind:value={awsAccessKey}
                  placeholder="AKIA..."
                  autocomplete="off"
                />
              </label>
              <label>
                {$_("settings.label_aws_secret_key")}
                <input
                  type="password"
                  bind:value={awsSecretKey}
                  placeholder="..."
                  autocomplete="off"
                />
              </label>
              <label>
                {$_("settings.label_aws_session")}
                <input
                  type="password"
                  bind:value={awsSessionToken}
                  placeholder={$_("settings.session_optional_ph")}
                  autocomplete="off"
                />
              </label>
            {:else}
              <label>
                {$_("settings.api_key_save", {
                  values: { provider: $_(PROVIDER_LABEL_KEYS[provider]) },
                })}
                <input
                  type="password"
                  bind:value={apiKeys[provider]}
                  placeholder={$_(KEY_PLACEHOLDER_KEYS[provider])}
                  autocomplete="off"
                />
              </label>
            {/if}
            <p class="muted">{$_("settings.secret_local_note")}</p>
          {:else}
            <label>
              {$_("settings.api_key_refine", {
                values: { provider: $_(PROVIDER_LABEL_KEYS[provider]) },
              })}
              <input
                id="cfg-api-key"
                type="password"
                bind:value={apiKeys[provider]}
                placeholder={$_(KEY_PLACEHOLDER_KEYS[provider])}
                autocomplete="off"
              />
            </label>
            {#if provider === "openai"}
              <label>
                {$_("settings.openai_base_url")}
                <input
                  id="cfg-openai-base-url"
                  type="url"
                  bind:value={openaiBaseUrl}
                  placeholder={$_("settings.openai_base_url_placeholder")}
                  autocomplete="off"
                />
              </label>
              <p class="muted model-hint">{$_("settings.openai_base_url_hint")}</p>
            {/if}
          {/if}
          <p class="muted model-hint">
            {#if provider === "bedrock"}
              {$_("settings.model_label")}<code>{bedrockModel || FALLBACK_MODELS[provider]}</code
              >{$_("settings.model_bedrock_suffix")}
            {:else if provider === "claude-aws"}
              {$_("settings.model_label")}<code>{FALLBACK_MODELS[provider]}</code>{$_(
                "settings.model_claude_aws_suffix",
              )}
            {:else}
              {$_("settings.model_label")}<code
                >{resolvedModel[provider] || FALLBACK_MODELS[provider]}</code
              >
              {#if resolvingModel}{$_(
                  "settings.model_resolving",
                )}{:else if resolvedModel[provider]}{$_("settings.model_latest_auto")}{:else}{$_(
                  "settings.model_latest_midrange",
                )}{/if}
            {/if}
          </p>
          <label>
            {$_("settings.label_refine_style")}
            <select bind:value={refineStyle}>
              {#each customStyleStore.allStyles as s}
                <option value={s.value} title={s.desc}>{s.label}</option>
              {/each}
            </select>
          </label>
          <!-- 選択中スタイルの解説を常時表示(マウスオーバーに頼らず各モードの違いを示す)。 -->
          <p class="style-desc">{currentStyle.desc}</p>

          <!-- 整形出力言語(翻訳 / #453)。既定OFF。ONにすると出力言語を選べる(progressive disclosure)。
           原語の文字起こしは常に保持し、翻訳は整形結果にのみ適用する。 -->
          <label class="check">
            <input type="checkbox" bind:checked={translateOutput} />
            {$_("settings.translate_output")}
          </label>
          {#if translateOutput}
            <label>
              {$_("settings.output_language")}
              <select bind:value={outputLang}>
                {#each LANGUAGES as l}
                  <option value={l.code}>{l.label}</option>
                {/each}
              </select>
            </label>
          {/if}
          <p class="tip">{$_("settings.tip_translate")}</p>
        </details>

        <!-- カスタム整形パターン(S3.3): ユーザー定義の指示を追加・管理する。 -->
        <details class="meta-group">
          <summary class="meta-title">{$_("settings.group_custom_style")}</summary>
          {#if customStyleStore.customStyles.length > 0}
            <ul class="custom-list">
              {#each customStyleStore.customStyles as c}
                <li class="custom-item">
                  <span class="custom-name">{c.label}</span>
                  <button
                    type="button"
                    class="btn small ghost"
                    onclick={() => customStyleStore.removeCustomStyle(c.id)}
                    >{$_("settings.delete")}</button
                  >
                </li>
              {/each}
            </ul>
          {/if}
          <input
            class="custom-name-input"
            type="text"
            bind:value={customStyleStore.newCustomLabel}
            placeholder={$_("settings.custom_name_ph")}
          />
          <textarea
            class="custom-instruction-input"
            bind:value={customStyleStore.newCustomInstruction}
            rows="4"
            placeholder={$_("settings.custom_instruction_ph")}></textarea>
          <button type="button" class="btn small" onclick={customStyleStore.addCustomStyle}>
            {$_("settings.add_custom")}
          </button>
          <p class="tip">{$_("settings.tip_custom")}</p>
        </details>
      {/if}

      {#if settingsTab === "output"}
        <details class="meta-group" open>
          <summary class="meta-title">{$_("settings.group_save")}</summary>
          <label class="check">
            <input type="checkbox" bind:checked={keepText} />
            {$_("settings.keep_text")}
          </label>
          <label class="check">
            <input type="checkbox" bind:checked={saveAudio} />
            {$_("settings.save_audio")}
          </label>
          {#if saveAudio}
            <label>
              {$_("settings.audio_format")}
              <select bind:value={audioFormat}>
                <option value="opus">{$_("settings.fmt_opus")}</option>
                <option value="wav">{$_("settings.fmt_wav")}</option>
              </select>
            </label>
            <p class="tip">{$_("settings.tip_audio_format")}</p>
          {/if}
          <div class="dir-row">
            <span class="tip"
              >{$_("settings.save_dir", {
                values: { dir: saveDir || $_("settings.save_dir_default") },
              })}</span
            >
            <button class="btn small ghost" onclick={pickSaveDir}>{$_("settings.change")}</button>
            <button class="btn small ghost" onclick={openVault}>{$_("settings.open_output")}</button
            >
          </div>
          <label>
            {$_("settings.output_format")}
            <select bind:value={outputFormat}>
              <option value="txt">{$_("settings.out_txt")}</option>
              <option value="md">{$_("settings.out_md")}</option>
            </select>
          </label>
          <p class="tip">
            {$_("settings.tip_output_1")}<strong>{$_("settings.tip_output_strong")}</strong>{$_(
              "settings.tip_output_2",
            )}<code>transcript-…</code>{$_("settings.tip_output_3")}<code>refined-…</code>{$_(
              "settings.tip_output_4",
            )}
          </p>
        </details>
      {/if}

      {#if settingsTab === "general"}
        <details class="meta-group">
          <summary class="meta-title">{$_("settings.group_app")}</summary>
          <label>
            {$_("settings.language")}
            <select
              value={($locale ?? "ja").split("-")[0]}
              onchange={(e) => {
                locale.set(e.currentTarget.value);
                localStorage.setItem(LOCALE_STORAGE_KEY, e.currentTarget.value);
                // トレイ文言はRust側保持のため、言語切替時に再送する。
                setTimeout(() => void syncTrayTexts(), 0);
              }}
            >
              {#each LANGUAGES.filter((l) => l.ui) as l}
                <option value={l.code}>{l.label}</option>
              {/each}
            </select>
          </label>
          {#if IS_WINDOWS}
            <label class="check">
              <input type="checkbox" bind:checked={taskbarWidget} />
              {$_("settings.taskbar_widget")}
            </label>
            <p class="tip">{$_("settings.tip_taskbar")}</p>
          {/if}
          <label class="check">
            <input
              type="checkbox"
              bind:checked={device.autoStart}
              onchange={() => void device.onAutoStartChange()}
            />
            {$_("settings.autostart")}
          </label>
          <p class="tip">{$_("settings.tip_autostart")}</p>
          <!-- 習慣ナッジ（#58）: opt-in。ONにした瞬間に通知権限を要求し、拒否ならOFFへ戻す。 -->
          <label class="check">
            <input
              type="checkbox"
              bind:checked={nudgeEnabled}
              onchange={() => void onToggleNudge()}
            />
            {$_("settings.nudge")}
          </label>
          <p class="tip">{$_("settings.tip_nudge")}</p>
          <!-- オンボーディングを再表示する導線(#397)。一度閉じると出なくなるため、後から見返せるように。 -->
          <button
            type="button"
            class="btn small ghost"
            onclick={() => {
              showSettings = false;
              showOnboarding = true;
            }}>{$_("settings.show_onboarding")}</button
          >
          <p class="tip">{$_("settings.tip_show_onboarding")}</p>
        </details>

        <!-- このアプリについて（ライセンス表示 / #394 監査項目5）。OSS帰属をアプリ内で明示。 -->
        <details class="meta-group">
          <summary class="meta-title">{$_("settings.group_about")}</summary>
          <p class="tip">QuickScribe — {$_("settings.about_license")}</p>
          <p class="tip">{$_("settings.about_oss")}</p>
          <p class="tip">{$_("settings.about_repo")}: github.com/Takenori-Kusaka/QuickScribe</p>
        </details>
      {/if}

      <div class="settings-actions">
        <button class="btn small" onclick={saveSettings}>{$_("settings.save")}</button>
        <button class="btn small ghost" onclick={() => updater.checkForUpdate(true)}
          >{$_("settings.check_update")}</button
        >
      </div>
      {#if updater.updateMsg}<p class="muted" role="status" aria-live="polite">
          {updater.updateMsg}
        </p>{/if}
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
    background: var(--color-bg-muted);
  }
  main {
    font-family:
      "Segoe UI",
      system-ui,
      -apple-system,
      "Hiragino Kaku Gothic ProN",
      "Noto Sans JP",
      sans-serif;
    color: var(--color-text);
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
  .settings-tabs {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
    margin-bottom: 1rem;
    border-bottom: 1px solid color-mix(in srgb, currentColor 15%, transparent);
    padding-bottom: 0.5rem;
  }
  .settings-tab {
    padding: 0.35rem 0.8rem;
    border: none;
    border-radius: 8px 8px 0 0;
    background: transparent;
    color: inherit;
    font-size: 0.9rem;
    font-weight: 600;
    cursor: pointer;
    opacity: 0.6;
  }
  .settings-tab.active {
    opacity: 1;
    background: color-mix(in srgb, currentColor 10%, transparent);
  }
  .settings-error {
    margin: 0 0 0.8rem;
    padding: 0.6rem 0.8rem;
    border-radius: 8px;
    background: color-mix(in srgb, crimson 14%, transparent);
    color: color-mix(in srgb, crimson 75%, black);
    font-size: 0.9rem;
    font-weight: 600;
  }
  /* 非ブロックの警告(#603): エラー(赤)より穏当な琥珀色。保存はできたが整形は失敗する旨。 */
  .settings-warning {
    margin: 0 0 0.8rem;
    padding: 0.6rem 0.8rem;
    border-radius: 8px;
    background: color-mix(in srgb, darkorange 16%, transparent);
    color: color-mix(in srgb, darkorange 78%, black);
    font-size: 0.9rem;
    font-weight: 600;
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
  .streak-badge {
    margin-left: auto;
    margin-right: 0.6rem;
    padding: 0.15rem 0.55rem;
    border-radius: 999px;
    background: color-mix(in srgb, orange 18%, transparent);
    font-size: 0.85rem;
    font-weight: 600;
    white-space: nowrap;
  }
  .close {
    background: none;
    border: none;
    font-size: 1.4rem;
    line-height: 1;
    color: var(--color-text-faint);
    cursor: pointer;
    padding: 0 0.2rem;
  }
  .close:hover {
    color: var(--color-text);
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
  /* ヘッダのアイコンボタン（ジャーナル・設定）はアイコンのみ＋ツールチップ(title)。
     狭いウィンドウでタイトルと重ならないよう、ラベルは表示しない。 */
  .gear {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    cursor: pointer;
    /* WCAG AA 非テキストコントラスト(#395): 旧 opacity 0.55 は実効 2.58:1。0.8 で 3:1 を満たす。 */
    opacity: 0.8;
    color: var(--color-text-muted);
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
    background: var(--color-bg-muted);
  }
  .ic-sm {
    width: 0.9rem;
    height: 0.9rem;
    vertical-align: -0.13rem;
    margin-right: 0.15rem;
  }
  .tagline {
    /* 薄いグレー背景上でも AA(4.5:1) を満たす濃さ (#395)。 */
    color: var(--color-text-muted);
    font-size: 0.8rem;
    margin: 0.25rem 0 0;
    text-align: center;
  }

  .settings {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
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
    color: var(--color-text-muted);
    margin-bottom: 0.7rem;
  }
  .settings input,
  .settings select {
    width: 100%;
    box-sizing: border-box;
    margin-top: 0.25rem;
    padding: 0.5rem 0.6rem;
    border: 1px solid var(--color-border-strong);
    border-radius: 8px;
    font-size: 0.85rem;
    background: var(--color-surface);
  }
  .settings-actions {
    display: flex;
    gap: 0.5rem;
  }
  .meta-group {
    border-top: 1px solid var(--color-border-faint);
    padding-top: 0.7rem;
    margin-bottom: 0.8rem;
  }
  .meta-title {
    display: block;
    font-size: 0.76rem;
    color: var(--color-text-muted);
    font-weight: 600;
    margin-bottom: 0.45rem;
  }
  /* 設定グループを折りたたみ可能なアコーディオンに（#404）。
     native <details>/<summary> で開閉＋キーボード/SR対応が標準で得られる。 */
  details.meta-group > summary.meta-title {
    display: list-item;
    list-style-position: inside;
    cursor: pointer;
    user-select: none;
  }
  details.meta-group > summary.meta-title:hover {
    color: var(--color-accent-strong);
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
    border: 1px solid var(--color-border-strong);
    border-radius: 6px;
    background: var(--color-surface);
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
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-surface);
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
    border: 1px solid var(--color-border-strong);
    border-radius: 6px;
    background: var(--color-surface);
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
    border: 1px solid var(--color-warning-border);
    background: var(--color-warning-bg);
    border-radius: 8px;
    padding: 0.5rem 0.6rem;
  }
  .corrections-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    font-size: 0.74rem;
    color: var(--color-warning-text);
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
    color: var(--color-danger);
  }
  .correction-arrow {
    color: var(--color-text-faint);
  }
  .correction-sugg {
    flex: 1;
    min-width: 8rem;
    padding: 0.3rem 0.45rem;
    border: 1px solid var(--color-border-strong);
    border-radius: 6px;
    background: var(--color-surface);
    font-family: inherit;
  }
  .correction-reason {
    font-size: 0.7rem;
    color: var(--color-text-faint);
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
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 10px;
    padding: 0.85rem 1rem;
    cursor: pointer;
    font-family: inherit;
  }
  .entry-item:hover {
    border-color: var(--color-accent-border);
    background: var(--color-accent-bg-hover);
  }
  .entry-meta {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    font-size: 0.72rem;
    color: var(--color-text-faint);
    margin-bottom: 0.3rem;
  }
  .entry-kind {
    background: var(--color-accent-bg);
    color: var(--color-accent-strong);
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
    color: var(--color-link);
  }
  .entry-preview {
    font-size: 0.82rem;
    color: var(--color-text-secondary);
    line-height: 1.5;
  }
  .entry-view {
    white-space: pre-wrap;
    word-break: break-word;
    background: var(--color-bg-subtle);
    border: 1px solid var(--color-border);
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
    border: 1px solid var(--color-border-strong);
    border-radius: 6px;
    background: var(--color-surface);
    box-sizing: border-box;
    font-family: inherit;
  }
  .hotkey-capture {
    flex: 1;
    text-align: left;
    padding: 0.5rem 0.6rem;
    border: 1px solid var(--color-border-strong);
    border-radius: 8px;
    background: var(--color-surface);
    font-size: 0.85rem;
    color: var(--color-text);
    cursor: pointer;
  }
  .hotkey-capture:hover {
    border-color: var(--color-accent-border-strong);
  }
  .hotkey-capture.capturing {
    border-color: var(--color-accent);
    background: var(--color-accent-bg);
    color: var(--color-accent-strong);
    font-weight: 600;
  }
  .check {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.84rem;
    color: var(--color-text);
    margin-bottom: 0.3rem;
  }
  .check input {
    width: auto;
    margin: 0;
  }
  .tip {
    font-size: 0.7rem;
    color: var(--color-text-faint);
    margin: 0.2rem 0 0;
    line-height: 1.5;
  }
  .tip.warn {
    color: var(--color-warning-text);
    background: var(--color-warning-bg);
    border: 1px solid var(--color-warning-border);
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
    background: var(--color-accent-bg);
    border: 1px solid var(--color-accent-border);
    color: var(--color-accent-deep);
    border-radius: 12px;
    padding: 0.6rem 0.9rem;
    font-size: 0.82rem;
    margin-bottom: 1rem;
  }
  .update-banner.ready {
    background: var(--color-success-bg);
    border-color: var(--color-success-border);
    color: var(--color-success-text);
  }
  .btn-restart {
    border: none;
    background: var(--color-success-strong);
    color: var(--color-surface);
    font-weight: 600;
    font-size: 0.8rem;
    padding: 0.4rem 0.8rem;
    border-radius: 8px;
    cursor: pointer;
  }
  .btn-restart:hover {
    background: var(--color-success-hover);
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
    background: var(--color-accent);
    color: var(--color-surface);
    box-shadow: 0 2px 8px rgba(79, 70, 229, 0.35);
  }
  .btn.primary:hover {
    background: var(--color-accent-strong);
  }
  .btn.primary.recording {
    background: var(--color-danger-bright);
    box-shadow: 0 2px 8px rgba(220, 38, 38, 0.35);
  }
  .btn.secondary {
    background: var(--color-surface);
    color: var(--color-accent);
    border: 1.5px solid var(--color-accent-border);
    box-shadow: 0 1px 4px rgba(17, 24, 39, 0.06);
  }
  .btn.secondary:hover:not(:disabled) {
    background: var(--color-accent-bg);
    border-color: var(--color-accent-border-strong);
  }
  .btn.small {
    font-size: 0.8rem;
    padding: 0.45rem 0.85rem;
    border-radius: 10px;
    background: var(--color-accent);
    color: var(--color-surface);
  }
  .btn.small.ghost {
    background: var(--color-surface);
    color: var(--color-accent);
    border: 1px solid var(--color-accent-border);
  }
  .dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.9);
  }
  .dot.on {
    background: var(--color-surface);
    box-shadow: 0 0 0 4px rgba(255, 255, 255, 0.3);
  }

  .hint {
    /* WCAG AA(#395): body背景 #f3f4f6 上でも 4.5:1 を満たす濃さにする(旧 #6b7280 は 4.34:1)。 */
    color: var(--color-text-muted);
    font-size: 0.72rem;
    text-align: center;
    margin: 0.7rem 0 1.1rem;
  }

  .empty-cta {
    color: var(--color-text-muted);
    font-size: 0.82rem;
    text-align: center;
    background: var(--color-bg-subtle);
    border: 1px dashed var(--color-border-strong);
    border-radius: 8px;
    padding: 0.7rem 0.9rem;
    margin: 0.2rem 0 1rem;
  }

  /* プライバシー状態インジケータ(#465)。オンデバイス=緑 / クラウド送信あり=琥珀。 */
  .privacy-status {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    border-radius: 10px;
    padding: 0.6rem 0.8rem;
    margin: 0 0 1rem;
    border: 1px solid transparent;
  }
  .privacy-status.local {
    background: var(--color-success-bg);
    border-color: var(--color-success-border);
  }
  .privacy-status.cloud {
    background: var(--color-warning-bg);
    border-color: var(--color-warning-border);
  }
  .privacy-dot {
    flex: 0 0 auto;
    width: 0.7rem;
    height: 0.7rem;
    border-radius: 50%;
  }
  .privacy-status.local .privacy-dot {
    background: var(--color-success);
  }
  .privacy-status.cloud .privacy-dot {
    background: var(--color-warning);
  }
  .privacy-text {
    flex: 1 1 auto;
    min-width: 0;
  }
  .privacy-text strong {
    display: block;
    font-size: 0.86rem;
  }
  .privacy-text p {
    margin: 0.1rem 0 0;
    font-size: 0.76rem;
    color: var(--color-text-muted);
    line-height: 1.4;
  }

  /* 初回オンボーディング（#397）。操作を妨げない非ブロッキングのインラインカード。 */
  .onboarding-card {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 12px;
    padding: 1.1rem 1.2rem;
    margin: 0.2rem 0 1.1rem;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.06);
    text-align: left;
  }
  .onboarding-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .onboarding-card h2 {
    margin: 0;
    font-size: 1.05rem;
  }
  .onboarding-local {
    color: var(--color-text-faint);
    font-size: 0.8rem;
    margin: 0.35rem 0 1rem;
  }
  .onboarding-steps {
    list-style: none;
    margin: 0 0 1rem;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
  }
  .onboarding-steps li {
    display: flex;
    gap: 0.75rem;
    align-items: flex-start;
  }
  .onboarding-steps strong {
    display: block;
    font-size: 0.92rem;
    margin-bottom: 0.15rem;
  }
  .onboarding-steps p {
    margin: 0;
    color: var(--color-text-muted);
    font-size: 0.82rem;
    line-height: 1.45;
  }
  .onboarding-step-n {
    flex: 0 0 auto;
    width: 1.6rem;
    height: 1.6rem;
    border-radius: 50%;
    background: var(--color-accent-bg);
    color: var(--color-accent-strong);
    font-weight: 700;
    font-size: 0.85rem;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .onboarding-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.6rem;
  }

  .panel {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 14px;
    padding: 0.9rem 1rem;
    box-shadow: 0 1px 4px rgba(17, 24, 39, 0.05);
  }
  .status-row {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    font-size: 0.85rem;
    color: var(--color-text-secondary);
  }
  .spinner {
    width: 15px;
    height: 15px;
    border: 2px solid var(--color-accent-border);
    border-top-color: var(--color-accent);
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
    background: var(--color-border);
    border-radius: 999px;
    overflow: hidden;
  }
  .bar {
    height: 100%;
    background: linear-gradient(90deg, var(--color-accent-bright), var(--color-accent));
    transition: width 0.2s ease;
  }
  .progress-meta {
    display: flex;
    justify-content: space-between;
    font-size: 0.72rem;
    color: var(--color-text-faint);
    margin-top: 0.3rem;
  }
  .progress-meta .pct {
    font-weight: 600;
    color: var(--color-accent);
  }
  .segments {
    margin-top: 0.6rem;
    max-height: 120px;
    overflow-y: auto;
    font-size: 0.82rem;
    line-height: 1.6;
    color: var(--color-text-muted);
    text-align: left;
  }

  .card {
    margin-top: 1rem;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 14px;
    padding: 0.9rem 1rem;
    text-align: left;
    box-shadow: 0 1px 4px rgba(17, 24, 39, 0.05);
  }
  .card.refined {
    border-color: var(--color-accent-border);
    background: var(--color-accent-bg-tint);
  }
  /* 段階的深掘り(S3.5): 結果から別スタイルで整形し直すチップ列。控えめに。 */
  .restyle-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.35rem;
    margin-top: 0.6rem;
    padding-top: 0.5rem;
    border-top: 1px solid var(--color-accent-bg);
  }
  .restyle-label {
    font-size: 0.74rem;
    color: var(--color-text-faint);
  }
  .chip {
    font-size: 0.76rem;
    padding: 0.2rem 0.55rem;
    border: 1px solid var(--color-border-strong);
    border-radius: 999px;
    background: var(--color-surface);
    color: var(--color-text-muted);
    cursor: pointer;
  }
  .chip:hover:not(:disabled) {
    border-color: var(--color-accent-border-strong);
    color: var(--color-accent-strong);
  }
  .chip.active {
    border-color: var(--color-accent-bright);
    background: var(--color-accent-bg);
    color: var(--color-accent-strong);
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
    /* コンパクト幅(スマホ的Quick比)を保つため、詰まらせず折り返す(#513再設計)。 */
    flex-wrap: wrap;
    gap: 0.4rem 0.6rem;
    margin-bottom: 0.5rem;
  }
  .refine-controls {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.4rem;
  }
  /* 処理画面: 整形スタイルは表示のみ(選択は設定画面)。ホバーで解説を出す。 */
  .style-indicator {
    font-size: 0.78rem;
    color: var(--color-text-faint);
    cursor: help;
  }
  .style-indicator strong {
    color: var(--color-text-secondary);
    font-weight: 600;
  }
  /* 設定画面: 選択中スタイルの解説。 */
  .style-desc {
    margin: 0.3rem 0 0;
    font-size: 0.78rem;
    color: var(--color-text-faint);
    line-height: 1.5;
  }
  .card h2 {
    margin: 0;
    font-size: 0.82rem;
    color: var(--color-text-faint);
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
    color: var(--color-text-faint);
    font-size: 0.78rem;
  }
  .model-hint {
    margin: -0.2rem 0 0.8rem;
  }
  .model-hint code {
    background: var(--color-accent-bg);
    color: var(--color-accent-strong);
    padding: 0.05rem 0.3rem;
    border-radius: 5px;
    font-size: 0.72rem;
  }
  .center {
    text-align: center;
  }
  .error {
    font-size: 0.78rem;
    color: var(--color-danger);
    background: var(--color-danger-bg);
    border: 1px solid var(--color-danger-border);
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
