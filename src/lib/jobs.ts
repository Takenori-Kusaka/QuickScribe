// マルチジョブ・キューのフロント状態モデル（ADR-0026 / #621 Phase2）。
// バックエンド(job.rs / lib.rs)の job-* イベントを、UIが描画する jobs[] へ畳み込む純粋関数群。
// 副作用を持たないためユニットテスト可能（App.svelte は薄く保つ / coverage ゲート対策）。

/** ジョブ状態（バックエンド job::JobStatus と一致 / lowercase シリアライズ）。 */
export type JobStatus = "queued" | "running" | "done" | "error" | "canceled";

/** UI が保持する1ジョブ。バックエンドの Job メタデータ＋結果本文/セグメント。 */
export interface Job {
  id: number;
  /** 録音停止時刻(epoch ミリ秒)。行ラベル用。 */
  createdAtMs: number;
  /** 音声長(秒)。行ラベル・ETA目安。 */
  durationSecs: number;
  status: JobStatus;
  /** 進捗 0-100（running のとき更新）。 */
  progress: number;
  /** 完了時の文字起こし本文。 */
  text?: string;
  /** 失敗時の安定エラーコード(E_XXX)。 */
  errorCode?: string;
  /** 逐次通知された確定セグメント（running中のライブプレビュー）。 */
  segments: string[];
}

/** job-created イベントのペイロード（バックエンド Job のメタデータ）。 */
export interface JobCreated {
  id: number;
  createdAtMs: number;
  durationSecs: number;
  status: JobStatus;
  progress: number;
}

/** 新規ジョブを追加する（既存 id は上書きしない＝重複イベントに冪等）。末尾(新しい)に積む。 */
export function upsertCreated(jobs: Job[], j: JobCreated): Job[] {
  if (jobs.some((x) => x.id === j.id)) return jobs;
  return [
    ...jobs,
    {
      id: j.id,
      createdAtMs: j.createdAtMs,
      durationSecs: j.durationSecs,
      status: j.status,
      progress: j.progress,
      segments: [],
    },
  ];
}

/** 指定ジョブに patch を当てた新配列を返す（該当なしは無変化）。 */
function patch(jobs: Job[], id: number, fn: (j: Job) => Job): Job[] {
  let changed = false;
  const next = jobs.map((j) => {
    if (j.id !== id) return j;
    changed = true;
    return fn(j);
  });
  return changed ? next : jobs;
}

export function setStatus(jobs: Job[], id: number, status: JobStatus): Job[] {
  return patch(jobs, id, (j) => ({ ...j, status }));
}

/** 進捗を 0-100 にクランプして反映。 */
export function setProgress(jobs: Job[], id: number, progress: number): Job[] {
  const p = Math.max(0, Math.min(100, Math.round(progress)));
  return patch(jobs, id, (j) => ({ ...j, progress: p }));
}

/** 確定セグメントを追記（空はスキップ）。 */
export function addSegment(jobs: Job[], id: number, text: string): Job[] {
  const t = text.trim();
  if (!t) return jobs;
  return patch(jobs, id, (j) => ({ ...j, segments: [...j.segments, t] }));
}

/** 完了: 本文を格納し status=done・progress=100。 */
export function setDone(jobs: Job[], id: number, text: string): Job[] {
  return patch(jobs, id, (j) => ({ ...j, status: "done", progress: 100, text }));
}

/** 失敗: エラーコードを格納し status=error。 */
export function setError(jobs: Job[], id: number, code: string): Job[] {
  return patch(jobs, id, (j) => ({ ...j, status: "error", errorCode: code }));
}

/** 処理中(queued または running)の件数。ヘッダの「処理中N件」バッジ用。 */
export function activeCount(jobs: Job[]): number {
  return jobs.filter((j) => j.status === "queued" || j.status === "running").length;
}

/** 実行中のジョブ（逐次処理のため高々1件）。進捗バーの対象。 */
export function runningJob(jobs: Job[]): Job | undefined {
  return jobs.find((j) => j.status === "running");
}

/** 最新の完了ジョブ（本文あり）。作業領域への自動読み込み対象。 */
export function newestDone(jobs: Job[]): Job | undefined {
  for (let i = jobs.length - 1; i >= 0; i--) {
    if (jobs[i].status === "done" && jobs[i].text) return jobs[i];
  }
  return undefined;
}

/** 終了済み(done/error/canceled)を新しい順に最大 keep 件残して掃除する（UI の肥大防止）。 */
export function pruneFinished(jobs: Job[], keep: number): Job[] {
  const isFinished = (j: Job) =>
    j.status === "done" || j.status === "error" || j.status === "canceled";
  const finishedTotal = jobs.filter(isFinished).length;
  if (finishedTotal <= keep) return jobs;
  let drop = finishedTotal - keep;
  return jobs.filter((j) => {
    if (isFinished(j) && drop > 0) {
      drop--;
      return false;
    }
    return true;
  });
}
