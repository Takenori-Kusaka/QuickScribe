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
  it("upsertCreated appends and is idempotent on duplicate id", () => {
    let jobs: Job[] = [];
    jobs = upsertCreated(jobs, created(1));
    jobs = upsertCreated(jobs, created(2));
    expect(jobs.map((j) => j.id)).toEqual([1, 2]);
    // 同一 id の再送は無視（冪等）。
    const same = upsertCreated(jobs, created(1));
    expect(same).toBe(jobs);
    expect(jobs[0].segments).toEqual([]);
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

  it("reducers on unknown id are no-ops returning same ref", () => {
    const jobs = upsertCreated([], created(1));
    expect(setStatus(jobs, 999, "running")).toBe(jobs);
    expect(setProgress(jobs, 999, 50)).toBe(jobs);
    expect(setDone(jobs, 999, "x")).toBe(jobs);
    expect(setError(jobs, 999, "E")).toBe(jobs);
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

  it("pruneFinished keeps unfinished and newest N finished", () => {
    let jobs: Job[] = [];
    for (let i = 1; i <= 4; i++) {
      jobs = upsertCreated(jobs, created(i));
      jobs = setStatus(jobs, i, "running");
      jobs = setDone(jobs, i, `t${i}`);
    }
    jobs = upsertCreated(jobs, created(5)); // queued (unfinished)
    jobs = pruneFinished(jobs, 2);
    // 終了済みは新しい2件(3,4)のみ、未終了5は必ず残る。
    expect(jobs.map((j) => j.id).sort((a, b) => a - b)).toEqual([3, 4, 5]);
  });
});
