<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/plugin-dialog";
  import { onMount } from "svelte";
  import { elapsedSeconds, buildNoteContent } from "./lib/note";

  let recording = $state(false);
  let lastSaved = $state<string | null>(null);
  let error = $state<string | null>(null);
  let startedAt = $state<number | null>(null);
  let status = $state<string>("");
  let transcript = $state<string | null>(null);
  let busy = $state(false);

  // 音声ファイル(mp3等)を選んで文字起こし→保存する(S1.6)。
  async function transcribeFromFile() {
    error = null;
    transcript = null;
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
        // Phase 1 はメモのプレースホルダを保存し、保存導線(フォルダ/権限)を確立する。
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
    // グローバルホットキー(Rust側で登録)からのトグルを受ける。
    const unToggle = listen("toggle-record", () => toggle());
    // 文字起こしの進捗ステータスを受ける。
    const unStatus = listen<string>("status", (e) => (status = e.payload));
    return () => {
      unToggle.then((f) => f());
      unStatus.then((f) => f());
    };
  });
</script>

<main>
  <h1>QuickScribe</h1>
  <p class="tagline">思考整理・自己理解のためのボイスジャーナル</p>

  <button class="record" class:recording onclick={toggle}>
    {recording ? "■ 停止" : "● 録音開始"}
  </button>

  <p class="hint">ホットキー: Ctrl/Cmd + Shift + R</p>

  <button class="file" onclick={transcribeFromFile} disabled={busy}>
    🎧 音声ファイルから文字起こし
  </button>

  {#if status}
    <p class="status">{status}</p>
  {/if}
  {#if transcript}
    <div class="transcript">
      <h2>文字起こし</h2>
      <p>{transcript}</p>
    </div>
  {/if}
  {#if lastSaved}
    <p class="saved">保存しました: <code>{lastSaved}</code></p>
  {/if}
  {#if error}
    <p class="error">エラー: {error}</p>
  {/if}
</main>

<style>
  main {
    font-family: system-ui, sans-serif;
    text-align: center;
    padding: 1.5rem 1rem;
    color: #1f2330;
  }
  h1 {
    margin: 0;
    font-size: 1.6rem;
    letter-spacing: 0.02em;
  }
  .tagline {
    color: #6b7280;
    font-size: 0.8rem;
    margin: 0.2rem 0 1.4rem;
  }
  .record {
    font-size: 1.05rem;
    padding: 0.8rem 1.6rem;
    border-radius: 999px;
    border: none;
    background: #4f46e5;
    color: white;
    cursor: pointer;
    transition: background 0.15s ease;
  }
  .record:hover {
    background: #4338ca;
  }
  .record.recording {
    background: #dc2626;
  }
  .hint {
    color: #9ca3af;
    font-size: 0.75rem;
    margin-top: 0.8rem;
  }
  .saved {
    font-size: 0.75rem;
    color: #047857;
    word-break: break-all;
  }
  .error {
    font-size: 0.75rem;
    color: #dc2626;
  }
</style>
