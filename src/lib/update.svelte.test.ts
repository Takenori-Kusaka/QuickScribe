import { describe, it, expect, vi, beforeEach } from "vitest";
import { flushSync } from "svelte";

const checkMock = vi.fn();
const relaunchMock = vi.fn();
vi.mock("@tauri-apps/plugin-updater", () => ({ check: (...a: unknown[]) => checkMock(...a) }));
vi.mock("@tauri-apps/plugin-process", () => ({
  relaunch: (...a: unknown[]) => relaunchMock(...a),
}));

import { createUpdater } from "./update.svelte";

// key をそのまま返す翻訳器（メッセージ内容は関心外）。
const t = (key: string) => key;

beforeEach(() => {
  checkMock.mockReset();
  relaunchMock.mockReset();
});

describe("createUpdater", () => {
  it("初期状態は idle", () => {
    const u = createUpdater(t);
    expect(u.updateState).toBe("idle");
    expect(u.updateMsg).toBe("");
  });

  it("更新なし(manual)は最新メッセージを出す", async () => {
    checkMock.mockResolvedValueOnce(null);
    const u = createUpdater(t);
    await u.checkForUpdate(true);
    flushSync();
    expect(u.updateMsg).toBe("update.latest");
    expect(u.updateState).toBe("idle");
  });

  it("更新なし(自動)はメッセージを出さない", async () => {
    checkMock.mockResolvedValueOnce(null);
    const u = createUpdater(t);
    await u.checkForUpdate(false);
    flushSync();
    expect(u.updateMsg).toBe("");
  });

  it("更新ありは DL 進捗→ready、version を設定する", async () => {
    const update = {
      version: "1.2.3",
      downloadAndInstall: async (cb: (e: unknown) => void) => {
        cb({ event: "Started", data: { contentLength: 100 } });
        cb({ event: "Progress", data: { chunkLength: 50 } });
      },
    };
    checkMock.mockResolvedValueOnce(update);
    const u = createUpdater(t);
    await u.checkForUpdate();
    flushSync();
    expect(u.updateVersion).toBe("1.2.3");
    expect(u.updatePct).toBe(50);
    expect(u.updateState).toBe("ready");
  });

  it("失敗(manual)は check_failed を出す", async () => {
    checkMock.mockRejectedValueOnce(new Error("net"));
    const u = createUpdater(t);
    await u.checkForUpdate(true);
    flushSync();
    expect(u.updateMsg).toBe("update.check_failed");
  });

  it("restartNow は relaunch を呼ぶ", async () => {
    relaunchMock.mockResolvedValueOnce(undefined);
    const u = createUpdater(t);
    await u.restartNow();
    expect(relaunchMock).toHaveBeenCalledTimes(1);
  });
});
