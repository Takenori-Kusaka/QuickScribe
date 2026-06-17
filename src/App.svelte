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

  // 設定（localStorageに保存。秘密情報はローカル端末内のみ）。
  // 整形プロバイダは Gemini / Anthropic(Claude) / OpenAI の3種をサポート(BYO鍵 / ADR-0005)。
  type Provider = "gemini" | "anthropic" | "openai";
  const PROVIDER_LABELS: Record<Provider, string> = {
    gemini: "Gemini",
    anthropic: "Anthropic (Claude)",
    openai: "OpenAI",
  };
  // 各プロバイダの「最新ミドルレンジモデル」を自動選択する（モデルはバックエンドが補完。
  // ここは表示用。実際の選択は src-tauri/src/lib.rs の default_model_for と一致させる）。
  const AUTO_MODELS: Record<Provider, string> = {
    gemini: "gemini-2.5-flash",
    anthropic: "claude-sonnet-4-6",
    openai: "gpt-4o",
  };
  const KEY_PLACEHOLDERS: Record<Provider, string> = {
    gemini: "AIza...",
    anthropic: "sk-ant-...",
    openai: "sk-...",
  };

  let showSettings = $state(false);
  let provider = $state<Provider>("gemini");
  // プロバイダごとに鍵を保持する（切替時に再入力不要）。モデルは自動選択のため保持しない。
  let apiKeys = $state<Record<Provider, string>>({ gemini: "", anthropic: "", openai: "" });
  let updateMsg = $state<string>("");

  function loadSettings() {
    provider = (localStorage.getItem("provider") as Provider) || "gemini";
    if (!(provider in PROVIDER_LABELS)) provider = "gemini";
    for (const p of ["gemini", "anthropic", "openai"] as Provider[]) {
      apiKeys[p] = localStorage.getItem(`apiKey:${p}`) ?? "";
    }
    // 旧バージョン(geminiKey)からの移行。
    const legacyKey = localStorage.getItem("geminiKey");
    if (legacyKey && !apiKeys.gemini) apiKeys.gemini = legacyKey;
  }
  function saveSettings() {
    localStorage.setItem("provider", provider);
    for (const p of ["gemini", "anthropic", "openai"] as Provider[]) {
      localStorage.setItem(`apiKey:${p}`, apiKeys[p]);
    }
    showSettings = false;
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
      const text = await invoke<string>("transcribe_file", { path: selected });
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
      refined = await invoke<string>("refine_text", {
        text: transcript,
        provider,
        apiKey: apiKeys[provider],
        // モデルは空 → バックエンドが各プロバイダの最新ミドルレンジを自動選択する。
        model: "",
      });
    } catch (e) {
      error = String(e);
    } finally {
      refining = false;
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
      busy = true;
      try {
        const text = await invoke<string>("stop_recording");
        if (text) transcript = text;
      } catch (e) {
        error = String(e);
      } finally {
        busy = false;
        status = "";
      }
    }
  }

  onMount(() => {
    loadSettings();
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
    return () => {
      unToggle.then((f) => f());
      unStatus.then((f) => f());
      unProgress.then((f) => f());
      unSegment.then((f) => f());
    };
  });
</script>

<main>
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

  {#if showSettings}
    <section class="settings">
      <h2>設定</h2>
      <label>
        整形プロバイダ
        <select bind:value={provider}>
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
        モデル: <code>{AUTO_MODELS[provider]}</code>（最新ミドルレンジを自動選択）
      </p>
      <div class="settings-actions">
        <button class="btn small" onclick={saveSettings}>保存</button>
        <button class="btn small ghost" onclick={() => checkForUpdate(true)}>更新を確認</button>
      </div>
      {#if updateMsg}<p class="muted">{updateMsg}</p>{/if}
    </section>
  {/if}

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
  </div>

  <p class="hint">ホットキー: Ctrl/Cmd + Shift + R</p>

  {#if busy || status}
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
</main>

<style>
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
    padding: 1rem;
    margin-bottom: 1rem;
    box-shadow: 0 1px 4px rgba(17, 24, 39, 0.06);
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
