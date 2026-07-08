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
  /** 作業領域に読み込んで開いたか（未読の完了ジョブを prune で失わない／未読件数の判定用）。 */
  opened?: boolean;
}

/** job-created イベントのペイロード（バックエンド Job のメタデータ）。 */
export interface JobCreated {
  id: number;
  createdAtMs: number;
  durationSecs: number;
  status: JobStatus;
  progress: number;
}

/** 新規ジョブを登録する。既存 id（状態イベントが先着した場合の stub 含む）にはメタデータのみ補完し、
 *  status/progress/text/segments は保持する（イベント順序に非依存＝job-done が先着しても失わない）。 */
export function upsertCreated(jobs: Job[], j: JobCreated): Job[] {
  if (jobs.some((x) => x.id === j.id)) {
    return jobs.map((x) =>
      x.id === j.id ? { ...x, createdAtMs: j.createdAtMs, durationSecs: j.durationSecs } : x,
    );
  }
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

/** 状態イベント用の最小 stub（job-created 未着でも状態を失わないため）。 */
function stub(id: number): Job {
  return { id, createdAtMs: 0, durationSecs: 0, status: "queued", progress: 0, segments: [] };
}

/** 指定ジョブに patch を当てた新配列を返す。**該当が無ければ stub を作って適用する**
 *  （job-status/done/error が job-created より先着しても完了ジョブを失わない＝順序非依存）。 */
function patch(jobs: Job[], id: number, fn: (j: Job) => Job): Job[] {
  if (!jobs.some((j) => j.id === id)) {
    return [...jobs, fn(stub(id))];
  }
  return jobs.map((j) => (j.id === id ? fn(j) : j));
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

/** ジョブを「開いた（作業領域へ読み込んだ）」印にする。未読件数・prune 保護に使う。 */
export function markOpened(jobs: Job[], id: number): Job[] {
  return patch(jobs, id, (j) => ({ ...j, opened: true }));
}

/** 未読の完了ジョブ数（done かつ本文あり かつ未 open）。「完了 N 件」バッジ・自動展開の判定用。 */
export function unopenedDoneCount(jobs: Job[]): number {
  return jobs.filter((j) => j.status === "done" && j.text && !j.opened).length;
}

/** 終了済みジョブを新しい順に最大 keep 件残して掃除する（UI の肥大防止）。
 *  ただし **未読の完了ジョブ（未 open で本文あり）は決して落とさない**（取りこぼさない＝未見の結果を失わない）。
 *  掃除対象は「開いた完了 / error / canceled」のみ。未終了(queued/running)も残す。 */
export function pruneFinished(jobs: Job[], keep: number): Job[] {
  const isPrunable = (j: Job) =>
    (j.status === "done" && j.opened) || j.status === "error" || j.status === "canceled";
  const prunableTotal = jobs.filter(isPrunable).length;
  if (prunableTotal <= keep) return jobs;
  let drop = prunableTotal - keep;
  return jobs.filter((j) => {
    if (isPrunable(j) && drop > 0) {
      drop--;
      return false;
    }
    return true;
  });
}
