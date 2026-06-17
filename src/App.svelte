<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/plugin-dialog";
  import { check } from "@tauri-apps/plugin-updater";
  import { relaunch } from "@tauri-apps/plugin-process";
  import { onMount } from "svelte";
  import {
    elapsedSeconds,
    buildNoteContent,
    estimateRemaining,
    formatRemaining,
  } from "./lib/note";

  let recording = $state(false);
  let lastSaved = $state<string | null>(null);
  let error = $state<string | null>(null);
  let startedAt = $state<number | null>(null);
  let status = $state<string>("");
  let progress = $state<number>(0);
  let eta = $state<string>("");
  let transcribeStartMs = $state<number | null>(null);
  let segments = $state<string[]>([]);
  let transcript = $state<string | null>(null);
  let busy = $state(false);

  // 自動アップデート(起動時に非同期チェック→背景DL→完了後に再起動を確認)。
  type UpdateState = "idle" | "downloading" | "ready";
  let updateState = $state<UpdateState>("idle");
  let updateVersion = $state<string>("");
  let updatePct = $state<number>(0);

  async function checkForUpdate() {
    try {
      const update = await check();
      if (!update) return;
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
      // ダウンロード+インストール完了。ユーザーに再起動を確認する。
      updateState = "ready";
    } catch (e) {
      // 自動更新の失敗は致命的でない（手動DL可）。静かに無視しつつログ。
      console.error("update check failed", e);
    }
  }

  async function restartNow() {
    await relaunch();
  }

  // 音声ファイル(mp3等)を選んで文字起こし→保存する(S1.6)。非同期で実行しUIを固めない。
  async function transcribeFromFile() {
    error = null;
    transcript = null;
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

  async function toggle() {
    error = null;
    if (!recording) {
      recording = true;
      startedAt = Date.now();
    } else {
      recording = false;
      const seconds = startedAt ? elapsedSeconds(startedAt, Date.now()) : 0;
      startedAt = null;
      try {
        const path = await invoke<string>("save_note", {
          content: buildNoteContent(seconds),
        });
        lastSaved = path;
      } catch (e) {
        error = String(e);
      }
    }
  }

  onMount(() => {
    // 起動時に非同期でアップデート確認（UIをブロックしない）。
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
    <h1>QuickScribe</h1>
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
    <section class="transcript">
      <h2>文字起こし</h2>
      <p>{transcript}</p>
    </section>
  {/if}
  {#if lastSaved}
    <p class="saved">保存しました: <code>{lastSaved}</code></p>
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
    padding: 1.5rem 1.25rem 2rem;
    max-width: 560px;
    margin: 0 auto;
  }
  header {
    text-align: center;
    margin-bottom: 1.4rem;
  }
  h1 {
    margin: 0;
    font-size: 1.55rem;
    font-weight: 700;
    letter-spacing: 0.01em;
  }
  .tagline {
    color: #6b7280;
    font-size: 0.8rem;
    margin: 0.25rem 0 0;
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

  /* 統一されたボタン設計（Material風: 角丸・elevation・状態遷移） */
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
    max-height: 160px;
    overflow-y: auto;
    font-size: 0.82rem;
    line-height: 1.6;
    color: #4b5563;
    text-align: left;
  }

  .transcript {
    margin-top: 1.1rem;
    background: #fff;
    border: 1px solid #e5e7eb;
    border-radius: 14px;
    padding: 1rem 1.1rem;
    text-align: left;
    box-shadow: 0 1px 4px rgba(17, 24, 39, 0.05);
  }
  .transcript h2 {
    margin: 0 0 0.5rem;
    font-size: 0.8rem;
    color: #6b7280;
    font-weight: 600;
  }
  .transcript p {
    margin: 0;
    line-height: 1.75;
    white-space: pre-wrap;
  }

  .saved {
    font-size: 0.74rem;
    color: #047857;
    word-break: break-all;
    text-align: center;
    margin-top: 0.9rem;
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
