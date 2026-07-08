import { describe, it, expect } from "vitest";
import {
  upsertCreated,
  setStatus,
  setProgress,
  addSegment,
  setDone,
  setError,
  activeCount,
  runningJob,
  newestDone,
  markOpened,
  unopenedDoneCount,
  pruneFinished,
  type Job,
} from "./jobs";

const created = (id: number) => ({
  id,
  createdAtMs: id * 1000,
  durationSecs: 10,
  status: "queued" as const,
  progress: 0,
});

describe("jobs reducers", () => {
  it("upsertCreated appends; duplicate id は複製せずメタデータのみ補完（status/segmentsは保持）", () => {
    let jobs: Job[] = [];
    jobs = upsertCreated(jobs, created(1));
    jobs = upsertCreated(jobs, created(2));
    expect(jobs.map((j) => j.id)).toEqual([1, 2]);
    // 進行中の状態を作る。
    jobs = setStatus(jobs, 1, "running");
    jobs = addSegment(jobs, 1, "seg");
    // 同一 id の再送は複製せず、メタデータ(duration)のみ更新。status/segments は保持。
    jobs = upsertCreated(jobs, { ...created(1), durationSecs: 99 });
    expect(jobs.map((j) => j.id)).toEqual([1, 2]);
    expect(jobs[0].status).toBe("running");
    expect(jobs[0].segments).toEqual(["seg"]);
    expect(jobs[0].durationSecs).toBe(99);
  });

  it("setStatus / setProgress / setDone / setError transition a job", () => {
    let jobs = upsertCreated([], created(1));
    jobs = setStatus(jobs, 1, "running");
    expect(jobs[0].status).toBe("running");
    jobs = setProgress(jobs, 1, 63);
    expect(jobs[0].progress).toBe(63);
    jobs = setDone(jobs, 1, "本文");
    expect(jobs[0].status).toBe("done");
    expect(jobs[0].progress).toBe(100);
    expect(jobs[0].text).toBe("本文");
  });

  it("setError records code", () => {
    let jobs = upsertCreated([], created(1));
    jobs = setStatus(jobs, 1, "running");
    jobs = setError(jobs, 1, "E_STT_FAILED");
    expect(jobs[0].status).toBe("error");
    expect(jobs[0].errorCode).toBe("E_STT_FAILED");
  });

  it("setProgress clamps to 0-100", () => {
    let jobs = upsertCreated([], created(1));
    jobs = setProgress(jobs, 1, 250);
    expect(jobs[0].progress).toBe(100);
    jobs = setProgress(jobs, 1, -5);
    expect(jobs[0].progress).toBe(0);
  });

  it("addSegment appends non-empty, skips blank", () => {
    let jobs = upsertCreated([], created(1));
    jobs = addSegment(jobs, 1, "  hello  ");
    jobs = addSegment(jobs, 1, "   ");
    jobs = addSegment(jobs, 1, "world");
    expect(jobs[0].segments).toEqual(["hello", "world"]);
  });

  it("状態イベントが job-created より先着しても stub を作り結果を失わない（順序非依存）", () => {
    // job-done が job-created より先に届く（クロススレッドのイベント順序）ケース。
    let jobs = setDone([], 7, "先着した本文");
    expect(jobs.length).toBe(1);
    expect(jobs[0].status).toBe("done");
    expect(jobs[0].text).toBe("先着した本文");
    // 後から job-created が来てもメタデータのみ補完し、done/text は保持する。
    jobs = upsertCreated(jobs, { ...created(7), createdAtMs: 7000, durationSecs: 12 });
    expect(jobs.length).toBe(1);
    expect(jobs[0].status).toBe("done");
    expect(jobs[0].text).toBe("先着した本文");
    expect(jobs[0].durationSecs).toBe(12);
  });

  it("markOpened / unopenedDoneCount で未読完了を数える", () => {
    let jobs = upsertCreated([], created(1));
    jobs = upsertCreated(jobs, created(2));
    jobs = setDone(jobs, 1, "a");
    jobs = setDone(jobs, 2, "b");
    expect(unopenedDoneCount(jobs)).toBe(2);
    jobs = markOpened(jobs, 1);
    expect(unopenedDoneCount(jobs)).toBe(1);
    expect(jobs.find((j) => j.id === 1)?.opened).toBe(true);
  });

  it("activeCount counts queued+running only", () => {
    let jobs = upsertCreated([], created(1));
    jobs = upsertCreated(jobs, created(2));
    jobs = upsertCreated(jobs, created(3));
    jobs = setStatus(jobs, 1, "running");
    jobs = setDone(jobs, 2, "x");
    // 1=running, 2=done, 3=queued → active=2
    expect(activeCount(jobs)).toBe(2);
    expect(runningJob(jobs)?.id).toBe(1);
  });

  it("newestDone returns latest done with text", () => {
    let jobs = upsertCreated([], created(1));
    jobs = upsertCreated(jobs, created(2));
    jobs = setDone(jobs, 1, "first");
    jobs = setDone(jobs, 2, "second");
    expect(newestDone(jobs)?.id).toBe(2);
    // done でも text 無しは対象外（自然な空発話の完了など）。
    const noText: Job[] = [
      { id: 9, createdAtMs: 0, durationSecs: 1, status: "done", progress: 100, segments: [] },
    ];
    expect(newestDone(noText)).toBeUndefined();
  });

  it("pruneFinished は開いた完了のみ掃除し、未読完了と未終了は必ず残す", () => {
    let jobs: Job[] = [];
    for (let i = 1; i <= 4; i++) {
      jobs = upsertCreated(jobs, created(i));
      jobs = setStatus(jobs, i, "running");
      jobs = setDone(jobs, i, `t${i}`);
      jobs = markOpened(jobs, i); // 開いた完了＝掃除対象
    }
    jobs = upsertCreated(jobs, created(5)); // queued (未終了)
    jobs = pruneFinished(jobs, 2);
    // 開いた完了は新しい2件(3,4)のみ、未終了5は残る。
    expect(jobs.map((j) => j.id).sort((a, b) => a - b)).toEqual([3, 4, 5]);
  });

  it("pruneFinished は未読の完了ジョブを keep 超過でも失わない（未見の結果を守る）", () => {
    let jobs: Job[] = [];
    for (let i = 1; i <= 4; i++) {
      jobs = upsertCreated(jobs, created(i));
      jobs = setStatus(jobs, i, "running");
      jobs = setDone(jobs, i, `t${i}`); // 未 open のまま＝未読
    }
    jobs = pruneFinished(jobs, 2);
    // 未読完了は保護され4件すべて残る。
    expect(jobs.length).toBe(4);
  });
});
