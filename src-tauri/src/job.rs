//! 文字起こしジョブの逐次キュー・ドメイン（ADR-0026 / #621 Phase1）。
//!
//! コア価値「思考を止めない・取りこぼさない」を満たすため、録音停止ごとに 1 ジョブを
//! 発番し、**並列度1（FIFO）の逐次キュー**で処理する。ここは副作用を持たない純粋な
//! 状態機械であり、実行（whisper 呼び出し・イベント発火）は lib.rs のワーカーが担う。
//!
//! 逐次不変条件: 同時に `Running` は高々 1 件。`next_queued` は Running が居る間 `None` を返す。
//! これにより「複数 running の並行進捗をどう見せるか」（原典事例が無い難所 / 調査 openQuestion）を
//! 構造的に回避する。将来 opt-in で並列度 N に拡張する余地は残す（ADR-0006 スコープ規律）。

use serde::Serialize;

/// ジョブ識別子。stop_recording ごとに単調増加で発番する。
pub type JobId = u64;

/// ジョブの状態（最小十分セット / Apple HIG・Buzz 準拠）。
/// フロントへは lowercase 文字列でシリアライズする（`queued` 等）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    /// 待機中（前のジョブ処理待ち）。
    Queued,
    /// 実行中（画面上は高々1件）。
    Running,
    /// 完了（結果あり）。
    Done,
    /// 失敗（error_code あり）。
    Error,
    /// キャンセル済み。
    Canceled,
}

/// UI に見せるジョブのメタデータ（結果テキスト本体は含めない＝イベントで運ぶ）。
/// フロントは camelCase で受ける（`createdAtMs` / `durationSecs` / `errorCode`）。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub id: JobId,
    /// 録音停止時刻（epoch ミリ秒）。行ラベル用。純粋性のため呼び出し側から渡す。
    pub created_at_ms: i64,
    /// 音声長（秒）。行ラベル・ETA の目安用。
    pub duration_secs: f64,
    pub status: JobStatus,
    /// 進捗 0-100。Running のときのみ更新される。
    pub progress: u8,
    /// 失敗時の安定エラーコード（E_XXX）。成功時 None。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
}

/// ジョブの逐次キュー（純粋な状態機械）。実行副作用は持たない。
#[derive(Default)]
pub struct JobQueue {
    next_id: JobId,
    jobs: Vec<Job>,
}

impl JobQueue {
    pub fn new() -> Self {
        Self::default()
    }

    /// 新規ジョブを `Queued` で登録し、発番した ID を返す。ID は 1 始まりの単調増加。
    pub fn enqueue(&mut self, created_at_ms: i64, duration_secs: f64) -> JobId {
        self.next_id += 1;
        let id = self.next_id;
        self.jobs.push(Job {
            id,
            created_at_ms,
            duration_secs,
            status: JobStatus::Queued,
            progress: 0,
            error_code: None,
        });
        id
    }

    /// 現在の全ジョブ（登録順＝FIFO）。UI 一覧用。
    pub fn jobs(&self) -> &[Job] {
        &self.jobs
    }

    fn find_mut(&mut self, id: JobId) -> Option<&mut Job> {
        self.jobs.iter_mut().find(|j| j.id == id)
    }

    pub fn get(&self, id: JobId) -> Option<&Job> {
        self.jobs.iter().find(|j| j.id == id)
    }

    /// いずれかのジョブが実行中か（逐次不変条件の判定に使う）。
    pub fn has_running(&self) -> bool {
        self.jobs.iter().any(|j| j.status == JobStatus::Running)
    }

    /// 次に実行すべきジョブ ID。**Running が居る間は None**（並列度1の逐次保証）。
    /// Running が居なければ最古の `Queued` を返す（FIFO）。
    pub fn next_queued(&self) -> Option<JobId> {
        if self.has_running() {
            return None;
        }
        self.jobs
            .iter()
            .find(|j| j.status == JobStatus::Queued)
            .map(|j| j.id)
    }

    /// 処理中（Queued＋Running）の件数。ヘッダの「処理中N件」バッジ用。
    pub fn active_count(&self) -> usize {
        self.jobs
            .iter()
            .filter(|j| matches!(j.status, JobStatus::Queued | JobStatus::Running))
            .count()
    }

    /// `Queued` → `Running`。遷移できたら true。存在しない/Queued でないなら false（不正遷移を弾く）。
    pub fn mark_running(&mut self, id: JobId) -> bool {
        match self.find_mut(id) {
            Some(j) if j.status == JobStatus::Queued => {
                j.status = JobStatus::Running;
                true
            }
            _ => false,
        }
    }

    /// 進捗を更新（0-100 にクランプ）。Running のときのみ反映し、遷移できたら true。
    pub fn set_progress(&mut self, id: JobId, pct: u8) -> bool {
        match self.find_mut(id) {
            Some(j) if j.status == JobStatus::Running => {
                j.progress = pct.min(100);
                true
            }
            _ => false,
        }
    }

    /// `Running` → `Done`（progress=100）。遷移できたら true。
    pub fn mark_done(&mut self, id: JobId) -> bool {
        match self.find_mut(id) {
            Some(j) if j.status == JobStatus::Running => {
                j.status = JobStatus::Done;
                j.progress = 100;
                true
            }
            _ => false,
        }
    }

    /// `Running` → `Error`（error_code 付与）。遷移できたら true。
    pub fn mark_error(&mut self, id: JobId, code: impl Into<String>) -> bool {
        match self.find_mut(id) {
            Some(j) if j.status == JobStatus::Running => {
                j.status = JobStatus::Error;
                j.error_code = Some(code.into());
                true
            }
            _ => false,
        }
    }

    /// `Queued` または `Running` → `Canceled`。遷移できたら true。
    /// （Running のキャンセルは協調的で、実処理の中断は Phase3。状態遷移のみ先に許可する。）
    pub fn cancel(&mut self, id: JobId) -> bool {
        match self.find_mut(id) {
            Some(j) if matches!(j.status, JobStatus::Queued | JobStatus::Running) => {
                j.status = JobStatus::Canceled;
                true
            }
            _ => false,
        }
    }

    /// 終了済み（Done/Error/Canceled）ジョブを古い順に最大 `keep` 件だけ残して掃除する。
    /// キュー無限成長を防ぐ（UI は「最近N件＋履歴」/ 履歴永続化は Phase3）。
    /// 未終了（Queued/Running）は決して削除しない。
    pub fn prune_finished(&mut self, keep: usize) {
        let is_finished = |j: &Job| {
            matches!(
                j.status,
                JobStatus::Done | JobStatus::Error | JobStatus::Canceled
            )
        };
        let finished_total = self.jobs.iter().filter(|j| is_finished(j)).count();
        if finished_total <= keep {
            return;
        }
        // 古い順（jobs は登録順）に (finished_total - keep) 件だけ終了済みを落とす。
        let mut drop_remaining = finished_total - keep;
        self.jobs.retain(|j| {
            if is_finished(j) && drop_remaining > 0 {
                drop_remaining -= 1;
                false
            } else {
                true
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enqueue_assigns_monotonic_ids_from_one() {
        let mut q = JobQueue::new();
        assert_eq!(q.enqueue(1000, 10.0), 1);
        assert_eq!(q.enqueue(2000, 20.0), 2);
        assert_eq!(q.enqueue(3000, 30.0), 3);
        assert_eq!(q.jobs().len(), 3);
        assert_eq!(q.get(1).unwrap().status, JobStatus::Queued);
        assert_eq!(q.get(2).unwrap().duration_secs, 20.0);
    }

    #[test]
    fn sequential_invariant_only_one_running_at_a_time() {
        let mut q = JobQueue::new();
        let a = q.enqueue(0, 1.0);
        let b = q.enqueue(0, 1.0);
        let c = q.enqueue(0, 1.0);
        // 最初は最古(a)が次に実行される。
        assert_eq!(q.next_queued(), Some(a));
        assert!(q.mark_running(a));
        // a が Running の間は next_queued は None（並列度1）。
        assert!(q.has_running());
        assert_eq!(q.next_queued(), None);
        // a 完了で次に b。
        assert!(q.mark_done(a));
        assert_eq!(q.next_queued(), Some(b));
        assert!(q.mark_running(b));
        assert_eq!(q.next_queued(), None);
        assert!(q.mark_done(b));
        assert_eq!(q.next_queued(), Some(c));
    }

    #[test]
    fn invalid_transitions_are_rejected() {
        let mut q = JobQueue::new();
        let a = q.enqueue(0, 1.0);
        // Queued からいきなり done/error/progress は不可。
        assert!(!q.mark_done(a));
        assert!(!q.mark_error(a, "E_X"));
        assert!(!q.set_progress(a, 50));
        // running にした後は可能。
        assert!(q.mark_running(a));
        assert!(!q.mark_running(a)); // 二重 running は不可。
        assert!(q.set_progress(a, 50));
        assert_eq!(q.get(a).unwrap().progress, 50);
        assert!(q.mark_done(a));
        // 終了後は再遷移不可。
        assert!(!q.set_progress(a, 60));
        assert!(!q.cancel(a));
        // 存在しない ID。
        assert!(!q.mark_running(999));
    }

    #[test]
    fn progress_is_clamped_and_done_sets_100() {
        let mut q = JobQueue::new();
        let a = q.enqueue(0, 1.0);
        q.mark_running(a);
        q.set_progress(a, 250);
        assert_eq!(q.get(a).unwrap().progress, 100);
        q.set_progress(a, 30);
        assert_eq!(q.get(a).unwrap().progress, 30);
        q.mark_done(a);
        assert_eq!(q.get(a).unwrap().progress, 100);
        assert_eq!(q.get(a).unwrap().status, JobStatus::Done);
    }

    #[test]
    fn error_records_code() {
        let mut q = JobQueue::new();
        let a = q.enqueue(0, 1.0);
        q.mark_running(a);
        assert!(q.mark_error(a, "E_STT_FAILED"));
        assert_eq!(q.get(a).unwrap().status, JobStatus::Error);
        assert_eq!(q.get(a).unwrap().error_code.as_deref(), Some("E_STT_FAILED"));
    }

    #[test]
    fn cancel_from_queued_and_running() {
        let mut q = JobQueue::new();
        let a = q.enqueue(0, 1.0);
        let b = q.enqueue(0, 1.0);
        // Queued をキャンセル → 実行対象から外れる。
        assert!(q.cancel(b));
        assert_eq!(q.get(b).unwrap().status, JobStatus::Canceled);
        // Running をキャンセル。
        q.mark_running(a);
        assert!(q.cancel(a));
        assert_eq!(q.get(a).unwrap().status, JobStatus::Canceled);
        // 全て終了済み → 次は無い。
        assert_eq!(q.next_queued(), None);
    }

    #[test]
    fn active_count_counts_queued_and_running_only() {
        let mut q = JobQueue::new();
        let a = q.enqueue(0, 1.0);
        let _b = q.enqueue(0, 1.0);
        assert_eq!(q.active_count(), 2);
        q.mark_running(a);
        assert_eq!(q.active_count(), 2); // running + queued
        q.mark_done(a);
        assert_eq!(q.active_count(), 1); // queued のみ
    }

    #[test]
    fn canceled_job_does_not_block_queue() {
        // Running を持たず Canceled/Done が混じっても、最古の Queued が選ばれる。
        let mut q = JobQueue::new();
        let a = q.enqueue(0, 1.0);
        let b = q.enqueue(0, 1.0);
        q.cancel(a);
        assert_eq!(q.next_queued(), Some(b));
    }

    #[test]
    fn prune_keeps_unfinished_and_recent_finished() {
        let mut q = JobQueue::new();
        // 終了済み4件 + 未終了1件。
        for _ in 0..4 {
            let id = q.enqueue(0, 1.0);
            q.mark_running(id);
            q.mark_done(id);
        }
        let live = q.enqueue(0, 1.0); // Queued（未終了）
        q.prune_finished(2);
        // 終了済みは最新2件のみ残る（id 3,4）。未終了 live は必ず残る。
        assert!(q.get(live).is_some());
        assert!(q.get(1).is_none());
        assert!(q.get(2).is_none());
        assert!(q.get(3).is_some());
        assert!(q.get(4).is_some());
        assert_eq!(q.jobs().len(), 3);
    }
}
