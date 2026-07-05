// @vitest-environment jsdom
import { describe, it, expect, vi, beforeEach } from "vitest";
import { flushSync } from "svelte";
import { createPrivacy, isLoopbackUrl } from "./privacy.svelte";

// provider/sttProvider を保持する簡易 App スタブ（getter/setter を DI する）。
function harness(initProvider = "gemini", initStt = "local", initBaseUrl = "") {
  let provider = initProvider;
  let stt = initStt;
  let baseUrl = initBaseUrl;
  const syncStt = vi.fn();
  const p = createPrivacy({
    getProvider: () => provider,
    setProvider: (v) => (provider = v),
    getSttProvider: () => stt,
    setSttProvider: (v) => (stt = v),
    getBaseUrl: () => baseUrl,
    syncStt,
  });
  return {
    p,
    syncStt,
    setBaseUrl(v: string) {
      baseUrl = v;
    },
    get provider() {
      return provider;
    },
    get stt() {
      return stt;
    },
  };
}

beforeEach(() => localStorage.clear());

describe("createPrivacy", () => {
  it("isFullyLocal: ローカル整形＋ローカルSTTのときだけ true", () => {
    expect(harness("ollama", "local").p.isFullyLocal).toBe(true);
    expect(harness("gemini", "local").p.isFullyLocal).toBe(false);
    expect(harness("ollama", "groq").p.isFullyLocal).toBe(false);
  });

  it("isFullyLocal: OpenAI互換でも base_url が loopback なら端末内完結（#593）", () => {
    // OpenAI + localhost base_url + STT local → オンデバイス完結。
    expect(harness("openai", "local", "http://localhost:4000").p.isFullyLocal).toBe(true);
    expect(harness("openai", "local", "http://127.0.0.1:4000").p.isFullyLocal).toBe(true);
    // リモート/未指定は端末内完結にしない（安全側）。
    expect(harness("openai", "local", "https://api.openai.com").p.isFullyLocal).toBe(false);
    expect(harness("openai", "local", "").p.isFullyLocal).toBe(false);
    // localhost でも STT がクラウドなら false。
    expect(harness("openai", "groq", "http://localhost:4000").p.isFullyLocal).toBe(false);
  });

  it("isLoopbackUrl: loopback だけ true、LAN/リモート/不正は false（#593）", () => {
    expect(isLoopbackUrl("http://localhost:4000")).toBe(true);
    expect(isLoopbackUrl("http://127.0.0.1:11434")).toBe(true);
    expect(isLoopbackUrl("http://[::1]:8080")).toBe(true);
    expect(isLoopbackUrl("http://0.0.0.0:4000")).toBe(true);
    expect(isLoopbackUrl("https://api.openai.com")).toBe(false);
    expect(isLoopbackUrl("http://192.168.1.10:4000")).toBe(false);
    expect(isLoopbackUrl("localhost:4000")).toBe(false); // スキーム無しはパース不能=安全側false
    expect(isLoopbackUrl("")).toBe(false);
  });

  it("makeOffline はローカルへ切替え STT 同期を呼ぶ", () => {
    const h = harness("gemini", "groq");
    h.p.makeOffline();
    expect(h.provider).toBe("ollama");
    expect(h.stt).toBe("local");
    expect(h.syncStt).toHaveBeenCalledTimes(1);
  });

  it("setOfflineMode(true) は永続化しローカル固定する", () => {
    const h = harness("gemini", "groq");
    h.p.setOfflineMode(true);
    flushSync();
    expect(h.p.offlineMode).toBe(true);
    expect(localStorage.getItem("offlineMode")).toBe("true");
    expect(h.provider).toBe("ollama");
  });

  it("setOfflineMode(false) は固定解除・切替えしない", () => {
    const h = harness("gemini", "groq");
    h.p.setOfflineMode(false);
    flushSync();
    expect(h.p.offlineMode).toBe(false);
    expect(h.provider).toBe("gemini");
  });
});
