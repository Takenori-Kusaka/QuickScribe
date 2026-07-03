// @vitest-environment jsdom
import { describe, it, expect, vi, beforeEach } from "vitest";

const invokeMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invokeMock(...a) }));

import { getSecret, setSecret, loadSecretMigrating } from "./secrets";

beforeEach(() => {
  invokeMock.mockReset();
  localStorage.clear();
});

describe("getSecret", () => {
  it("値を返す", async () => {
    invokeMock.mockResolvedValueOnce("v");
    expect(await getSecret("k")).toBe("v");
  });
  it("null は空文字に正規化", async () => {
    invokeMock.mockResolvedValueOnce(null);
    expect(await getSecret("k")).toBe("");
  });
  it("失敗時は空文字（例外を投げない）", async () => {
    invokeMock.mockRejectedValueOnce(new Error("x"));
    expect(await getSecret("k")).toBe("");
  });
});

describe("setSecret", () => {
  it("成功時 true", async () => {
    invokeMock.mockResolvedValueOnce(undefined);
    expect(await setSecret("k", "v")).toBe(true);
  });
  it("失敗時 false（鍵を失わないため）", async () => {
    invokeMock.mockRejectedValueOnce(new Error("x"));
    expect(await setSecret("k", "v")).toBe(false);
  });
});

describe("loadSecretMigrating", () => {
  it("keyring に値があればそれを返し、平文は触らない", async () => {
    invokeMock.mockResolvedValueOnce("secure"); // getSecret
    localStorage.setItem("legacy", "old");
    expect(await loadSecretMigrating("k", "legacy")).toBe("secure");
    expect(localStorage.getItem("legacy")).toBe("old");
  });

  it("keyring 空なら平文から移行し、書き込み成功で平文を削除", async () => {
    invokeMock
      .mockResolvedValueOnce("") // getSecret → 空
      .mockResolvedValueOnce(undefined); // setSecret → 成功
    localStorage.setItem("legacy", "old");
    expect(await loadSecretMigrating("k", "legacy")).toBe("old");
    expect(localStorage.getItem("legacy")).toBeNull();
  });

  it("書き込み失敗時は平文を残す（損失防止）", async () => {
    invokeMock
      .mockResolvedValueOnce("") // getSecret → 空
      .mockRejectedValueOnce(new Error("x")); // setSecret → 失敗
    localStorage.setItem("legacy", "old");
    expect(await loadSecretMigrating("k", "legacy")).toBe("old");
    expect(localStorage.getItem("legacy")).toBe("old");
  });

  it("keyring も平文も無ければ空文字", async () => {
    invokeMock.mockResolvedValueOnce(""); // getSecret
    expect(await loadSecretMigrating("k", "legacy")).toBe("");
  });
});
