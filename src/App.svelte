<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/plugin-dialog";
  import { check } from "@tauri-apps/plugin-updater";
  import { relaunch } from "@tauri-apps/plugin-process";
  import { onMount } from "svelte";
  import { estimateRemaining, formatRemaining } from "./lib/note";

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
  let refining = $state(false);
  let busy = $state(false);
  // バックグラウンド文字起こし中（録音ボタンはブロックしない＝録音の非同期化）。
  let transcribing = $state(false);

  // 設定（localStorageに保存。秘密情報はローカル端末内のみ）。
  // 整形プロバイダは Gemini / Anthropic(Claude) / OpenAI の3種をサポート(BYO鍵 / ADR-0005)。
  type Provider = "gemini" | "anthropic" | "openai";
  const PROVIDER_LABELS: Record<Provider, string> = {
    gemini: "Gemini",
    anthropic: "Anthropic (Claude)",
    openai: "OpenAI",
  };
  // モデルは「実行時に各社のモデル一覧APIから最新ミドルレンジを解決」する（ビルド時固定にしない）。
  // 取得失敗時のフォールバック表示（ローリングlatestエイリアス優先 / ADR-0007 deep research）。
  const FALLBACK_MODELS: Record<Provider, string> = {
    gemini: "gemini-flash-latest",
    anthropic: "claude-sonnet-4-6",
    openai: "gpt-4o",
  };
  const KEY_PLACEHOLDERS: Record<Provider, string> = {
    gemini: "AIza...",
    anthropic: "sk-ant-...",
    openai: "sk-...",
  };
  const ALL_PROVIDERS: Provider[] = ["gemini", "anthropic", "openai"];
  // 解決済みモデルのキャッシュ寿命（24時間）。これを過ぎたら再取得する。
  const MODEL_TTL_MS = 24 * 60 * 60 * 1000;

  let showSettings = $state(false);
  let provider = $state<Provider>("gemini");
  // プロバイダごとに鍵を保持する（切替時に再入力不要）。
  let apiKeys = $state<Record<Provider, string>>({ gemini: "", anthropic: "", openai: "" });
  // 実行時に解決した最新モデルID（表示・整形に使用）。
  let resolvedModel = $state<Record<Provider, string>>({ gemini: "", anthropic: "", openai: "" });
  let resolvingModel = $state(false);
  let updateMsg = $state<string>("");

  // 録音トグルのグローバルホットキー。内部・保存・登録は Tauri アクセラレータ表記
  // ("CommandOrControl+Shift+R")だが、表示は実行環境に合わせて Ctrl/Cmd に変換する。
  const DEFAULT_SHORTCUT = "CommandOrControl+Shift+R";
  let recordShortcut = $state<string>(DEFAULT_SHORTCUT);
  let shortcutMsg = $state<string>("");
  // ホットキーのキャプチャモード（変更ボタン押下中＝キー入力待ち）。
  let capturing = $state<boolean>(false);

  // 実行環境(OS)を判定し、修飾キーを親しみやすい表記にする。
  const IS_MAC =
    typeof navigator !== "undefined" &&
    /mac/i.test(`${navigator.userAgent} ${navigator.platform ?? ""}`);
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

  function loadSettings() {
    provider = (localStorage.getItem("provider") as Provider) || "gemini";
    if (!(provider in PROVIDER_LABELS)) provider = "gemini";
    for (const p of ALL_PROVIDERS) {
      apiKeys[p] = localStorage.getItem(`apiKey:${p}`) ?? "";
      resolvedModel[p] = localStorage.getItem(`resolvedModel:${p}`) ?? "";
    }
    // 旧バージョン(geminiKey)からの移行。
    const legacyKey = localStorage.getItem("geminiKey");
    if (legacyKey && !apiKeys.gemini) apiKeys.gemini = legacyKey;
    recordShortcut = localStorage.getItem("recordShortcut") || DEFAULT_SHORTCUT;
    includeTimestamps = localStorage.getItem("includeTimestamps") !== "false";
    autoPipeline = localStorage.getItem("autoPipeline") === "true";
    keepText = localStorage.getItem("keepText") !== "false";
    saveAudio = localStorage.getItem("saveAudio") === "true";
    audioFormat = localStorage.getItem("audioFormat") || "opus";
    saveDir = localStorage.getItem("saveDir") || "";
  }

  // 保存設定をバックエンドへ反映する（保存系コマンドが参照）。
  async function syncSaveSettings() {
    try {
      await invoke("set_save_settings", {
        saveDir,
        saveAudio,
        audioFormat,
        keepText,
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
  function saveSettings() {
    localStorage.setItem("provider", provider);
    for (const p of ALL_PROVIDERS) {
      localStorage.setItem(`apiKey:${p}`, apiKeys[p]);
    }
    void applyShortcut();
    localStorage.setItem("includeTimestamps", String(includeTimestamps));
    localStorage.setItem("autoPipeline", String(autoPipeline));
    localStorage.setItem("keepText", String(keepText));
    localStorage.setItem("saveAudio", String(saveAudio));
    localStorage.setItem("audioFormat", audioFormat);
    localStorage.setItem("saveDir", saveDir);
    void syncSaveSettings();
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
    else if (k.startsWith("Arrow")) key = k.slice(5); // ArrowUp -> Up
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
      shortcutMsg = String(e);
    }
  }

  // 現在のプロバイダの最新ミドルレンジモデルを実行時に解決し、キャッシュする。
  // force=false かつキャッシュが新しければ何もしない。鍵未入力なら何もしない。
  async function resolveCurrentModel(force = false) {
    const p = provider;
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
      updateMsg = manual ? `更新確認に失敗: ${e}` : "";
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
      filters: [
        { name: "音声ファイル", extensions: ["mp3", "wav", "m4a", "flac", "ogg", "aac"] },
      ],
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
      error = String(e);
    } finally {
      busy = false;
      status = "";
    }
  }

  // 文字起こしを整形（思考整理・要約）する＝コア価値。選択中プロバイダの鍵が必要。
  async function refineNow() {
    if (!transcript) return;
    if (!apiKeys[provider].trim()) {
      showSettings = true;
      error = `整形には ${PROVIDER_LABELS[provider]} のAPIキーが必要です。設定から入力してください。`;
      return;
    }
    error = null;
    refining = true;
    refined = null;
    try {
      // 整形直前に最新モデルを確保（キャッシュが新しければ即返る）。
      await resolveCurrentModel();
      refined = await invoke<string>("refine_text", {
        text: transcript,
        provider,
        apiKey: apiKeys[provider],
        // 解決済みモデル（空ならバックエンドがフォールバック既定を補完）。
        model: resolvedModel[provider],
      });
    } catch (e) {
      error = String(e);
    } finally {
      refining = false;
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
      error = String(e);
    }
  }

  // 録音トグル（S1.1）。開始でマイク収集、停止で文字起こし→保存→表示まで貫通する。
  async function toggle() {
    error = null;
    if (!recording) {
      try {
        await invoke("start_recording");
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
        error = String(e);
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
        error = String(e);
      }
    }
  }

  onMount(() => {
    loadSettings();
    void applyShortcut();
    void syncSaveSettings();
    void resolveCurrentModel();
    void checkForUpdate();
    const unToggle = listen("toggle-record", () => toggle());
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
        // 一気通貫: 自動で整形まで実行（鍵がある時のみ）。
        if (autoPipeline && apiKeys[provider].trim()) void refineNow();
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
      unToggle.then((f) => f());
      unStatus.then((f) => f());
      unProgress.then((f) => f());
      unSegment.then((f) => f());
      unDone.then((f) => f());
      unErr.then((f) => f());
    };
  });
</script>

<main>
  <div class="content">
  <header>
    <div class="title-row">
      <h1>QuickScribe</h1>
      <button
        class="gear"
        data-testid="settings-btn"
        title="設定"
        aria-label="設定"
        onclick={() => (showSettings = !showSettings)}
      >
        ⚙
      </button>
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

  <p class="hint">録音ホットキー: <code>{displayShortcut(recordShortcut)}</code>（設定で変更可）</p>

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
        <button class="btn small" onclick={refineNow} disabled={refining}>
          {refining ? "整形中…" : "✨ 整形する"}
        </button>
      </div>
      <div class="scroll">{transcript}</div>
    </section>
  {/if}

  {#if refining}
    <p class="muted center"><span class="spinner" aria-hidden="true"></span> AIが思考を整理しています…</p>
  {/if}
  {#if refined}
    <section class="card refined">
      <h2>整形（思考整理）</h2>
      <div class="scroll">{refined}</div>
    </section>
  {/if}

  {#if error}
    <p class="error">{error}</p>
  {/if}
  </div>
</main>

{#if showSettings}
  <div
    class="settings-overlay"
    role="presentation"
    onclick={(e) => {
      if (e.target === e.currentTarget) showSettings = false;
    }}
  >
    <aside class="settings">
      <div class="settings-head">
        <h2>設定</h2>
        <button class="close" aria-label="閉じる" onclick={() => (showSettings = false)}
          >×</button
        >
      </div>
      <label>
        整形プロバイダ
        <select bind:value={provider} onchange={() => resolveCurrentModel()}>
          <option value="gemini">Gemini</option>
          <option value="anthropic">Anthropic (Claude)</option>
          <option value="openai">OpenAI</option>
        </select>
      </label>
      <label>
        {PROVIDER_LABELS[provider]} APIキー（整形に使用・端末内のみ保存）
        <input
          type="password"
          bind:value={apiKeys[provider]}
          placeholder={KEY_PLACEHOLDERS[provider]}
          autocomplete="off"
        />
      </label>
      <p class="muted model-hint">
        モデル: <code>{resolvedModel[provider] || FALLBACK_MODELS[provider]}</code>
        {#if resolvingModel}（取得中…）{:else if resolvedModel[provider]}（最新を自動取得）{:else}（最新ミドルレンジを自動選択）{/if}
      </p>
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
      <p class="tip">「{displayShortcut(recordShortcut)}」をクリックして、登録したいキーを押します。</p>
      {#if shortcutMsg}<p class="muted">{shortcutMsg}</p>{/if}

      <div class="meta-group">
        <span class="meta-title">文字起こしのメタデータ</span>
        <label class="check">
          <input type="checkbox" bind:checked={includeTimestamps} />
          タイムスタンプを含める
        </label>
        <p class="tip">
          いつ何を話したかの時刻を残し、AIが話の流れを踏まえて整理します。
        </p>
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
          <p class="tip">
            Opusは小容量でジャーナル向き。WAVは無圧縮で容量大ですが確実です。
          </p>
        {/if}
        <div class="dir-row">
          <span class="tip">保存先: {saveDir || "既定（ドキュメント/QuickScribe）"}</span>
          <button class="btn small ghost" onclick={pickSaveDir}>変更</button>
        </div>
      </div>

      <div class="settings-actions">
        <button class="btn small" onclick={saveSettings}>保存</button>
        <button class="btn small ghost" onclick={() => checkForUpdate(true)}>更新を確認</button>
      </div>
      {#if updateMsg}<p class="muted">{updateMsg}</p>{/if}
    </aside>
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
      "Segoe UI", system-ui, -apple-system, "Hiragino Kaku Gothic ProN", "Noto Sans JP",
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
  .gear {
    position: absolute;
    right: 0;
    background: none;
    border: none;
    font-size: 1.2rem;
    cursor: pointer;
    opacity: 0.6;
    line-height: 1;
  }
  .gear:hover {
    opacity: 1;
  }
  .tagline {
    color: #6b7280;
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
    color: #9ca3af;
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
  .card-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 0.5rem;
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
</style>
