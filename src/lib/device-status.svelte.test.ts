// @vitest-environment jsdom
import { describe, it, expect, vi, beforeEach } from "vitest";
import { flushSync } from "svelte";

const invokeMock = vi.fn();
const enableMock = vi.fn();
const disableMock = vi.fn();
const isEnabledMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invokeMock(...a) }));
vi.mock("@tauri-apps/plugin-autostart", () => ({
  enable: (...a: unknown[]) => enableMock(...a),
  disable: (...a: unknown[]) => disableMock(...a),
  isEnabled: (...a: unknown[]) => isEnabledMock(...a),
}));

import { createDeviceStatus } from "./device-status.svelte";

const t = (key: string) => key;
const make = (onError = () => {}) => createDeviceStatus({ t, onError });

beforeEach(() => {
  invokeMock.mockReset();
  enableMock.mockReset();
  disableMock.mockReset();
  isEnabledMock.mockReset();
});

describe("createDeviceStatus", () => {
  it("loadAutoStart は OS の登録状態を反映する", async () => {
    isEnabledMock.mockResolvedValueOnce(true);
    const d = make();
    await d.loadAutoStart();
    flushSync();
    expect(d.autoStart).toBe(true);
  });

  it("onAutoStartChange: ON なら enable し状態同期", async () => {
    enableMock.mockResolvedValueOnce(undefined);
    isEnabledMock.mockResolvedValueOnce(true);
    const d = make();
    d.autoStart = true;
    await d.onAutoStartChange();
    flushSync();
    expect(enableMock).toHaveBeenCalledTimes(1);
    expect(d.autoStart).toBe(true);
  });

  it("onAutoStartChange: OFF なら disable する", async () => {
    disableMock.mockResolvedValueOnce(undefined);
    isEnabledMock.mockResolvedValueOnce(false);
    const d = make();
    d.autoStart = false;
    await d.onAutoStartChange();
    flushSync();
    expect(disableMock).toHaveBeenCalledTimes(1);
    expect(d.autoStart).toBe(false);
  });

  it("onAutoStartChange 失敗時は onError を呼ぶ", async () => {
    enableMock.mockRejectedValueOnce(new Error("x"));
    isEnabledMock.mockResolvedValue(false);
    const onError = vi.fn();
    const d = make(onError);
    d.autoStart = true;
    await d.onAutoStartChange();
    flushSync();
    expect(onError).toHaveBeenCalledTimes(1);
  });

  it("loadAudioSources は一覧を格納、失敗時は空", async () => {
    invokeMock.mockResolvedValueOnce([{ id: "a", label: "Mic", kind: "input" }]);
    const d = make();
    await d.loadAudioSources();
    flushSync();
    expect(d.audioSources).toHaveLength(1);
    invokeMock.mockRejectedValueOnce(new Error("x"));
    await d.loadAudioSources();
    flushSync();
    expect(d.audioSources).toEqual([]);
  });

  it("loadWhisperModels は一覧を格納する", async () => {
    invokeMock.mockResolvedValueOnce([{ id: "base", label: "Base" }]);
    const d = make();
    await d.loadWhisperModels();
    flushSync();
    expect(d.whisperModels).toEqual([{ id: "base", label: "Base" }]);
  });

  it("applyTaskbarWidget は enabled をバックエンドへ渡す", async () => {
    invokeMock.mockResolvedValueOnce(undefined);
    const d = make();
    await d.applyTaskbarWidget(false);
    expect(invokeMock).toHaveBeenCalledWith("set_taskbar_widget", { enabled: false });
  });
});
