// @vitest-environment jsdom
import { describe, it, expect, vi, beforeEach } from "vitest";
import { flushSync } from "svelte";
import { createPrivacy } from "./privacy.svelte";

// provider/sttProvider を保持する簡易 App スタブ（getter/setter を DI する）。
function harness(initProvider = "gemini", initStt = "local") {
  let provider = initProvider;
  let stt = initStt;
  const syncStt = vi.fn();
  const p = createPrivacy({
    getProvider: () => provider,
    setProvider: (v) => (provider = v),
    getSttProvider: () => stt,
    setSttProvider: (v) => (stt = v),
    syncStt,
  });
  return {
    p,
    syncStt,
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
