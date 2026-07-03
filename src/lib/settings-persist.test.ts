// @vitest-environment jsdom
import { describe, it, expect, beforeEach } from "vitest";
import {
  readSettings,
  writeSettings,
  SETTINGS_VERSION,
  type AppSettings,
} from "./settings-persist";

beforeEach(() => localStorage.clear());

describe("readSettings", () => {
  it("空の localStorage では既定値を返す（ローカルファースト / ADR-0021）", () => {
    const s = readSettings("en");
    expect(s.provider).toBe("ollama");
    expect(s.recordMode).toBe("toggle");
    expect(s.includeTimestamps).toBe(true);
    expect(s.keepText).toBe(true);
    expect(s.saveAudio).toBe(false);
    expect(s.audioFormat).toBe("opus");
    expect(s.outputFormat).toBe("txt");
    expect(s.whisperModel).toBe("base");
    expect(s.awsRegion).toBe("us-east-1");
    expect(s.awsAuthMode).toBe("sigv4");
    expect(s.taskbarWidget).toBe(true);
    expect(s.inputDeviceKind).toBe("input");
    expect(s.customStyles).toEqual([]);
    // 出力言語の既定は引数（起動時UI言語）。
    expect(s.outputLang).toBe("en");
  });

  it("保存値を読み、settingsVersion を記録する", () => {
    localStorage.setItem("provider", "anthropic");
    localStorage.setItem("saveAudio", "true");
    localStorage.setItem("saveDir", "/x");
    const s = readSettings("ja");
    expect(s.provider).toBe("anthropic");
    expect(s.saveAudio).toBe(true);
    expect(s.saveDir).toBe("/x");
    expect(localStorage.getItem("settingsVersion")).toBe(String(SETTINGS_VERSION));
  });

  it("破損した enum 値は既定へクランプする", () => {
    localStorage.setItem("provider", "bogus");
    localStorage.setItem("audioFormat", "flac");
    localStorage.setItem("outputFormat", "xml");
    localStorage.setItem("awsAuthMode", "weird");
    const s = readSettings("ja");
    // 破損 provider はローカルファースト(ollama)へ寄せる（ADR-0021）。
    expect(s.provider).toBe("ollama");
    expect(s.audioFormat).toBe("opus");
    expect(s.outputFormat).toBe("txt");
    expect(s.awsAuthMode).toBe("sigv4");
  });

  it("whisperModel の既定は日本語UIで kotoba-q5、他は base（#511/ADR-0021）", () => {
    expect(readSettings("ja").whisperModel).toBe("kotoba-q5");
    expect(readSettings("en").whisperModel).toBe("base");
  });

  it("offlineMode=true はクラウド設定を無視しローカルへ固定する", () => {
    localStorage.setItem("offlineMode", "true");
    localStorage.setItem("provider", "openai");
    localStorage.setItem("sttProvider", "groq");
    const s = readSettings("ja");
    expect(s.offlineMode).toBe(true);
    expect(s.provider).toBe("ollama");
    expect(s.sttProvider).toBe("local");
  });

  it("customStyles を読み、破損 JSON は空配列", () => {
    localStorage.setItem(
      "customStyles",
      JSON.stringify([{ id: "a", label: "L", instruction: "I" }]),
    );
    expect(readSettings("ja").customStyles).toEqual([{ id: "a", label: "L", instruction: "I" }]);
    localStorage.setItem("customStyles", "{壊れ");
    expect(readSettings("ja").customStyles).toEqual([]);
  });

  it("未知の refineStyle は structured へフォールバック", () => {
    localStorage.setItem("refineStyle", "nonexistent");
    expect(readSettings("ja").refineStyle).toBe("structured");
  });
});

describe("writeSettings", () => {
  const base: AppSettings = {
    provider: "anthropic",
    resolvedModel: {
      gemini: "",
      anthropic: "",
      openai: "",
      ollama: "",
      bedrock: "",
      "claude-aws": "",
    },
    recordShortcut: "X",
    recordMode: "momentary",
    includeTimestamps: false,
    autoPipeline: true,
    keepText: false,
    saveAudio: true,
    audioFormat: "wav",
    saveDir: "/d",
    outputFormat: "md",
    refineStyle: "verbatim",
    translateOutput: true,
    outputLang: "es",
    sttProvider: "openai",
    offlineMode: false,
    sttModel: "m",
    sttAzureResource: "r",
    whisperModel: "kotoba",
    customStyles: [],
    awsRegion: "eu-west-1",
    awsWorkspaceId: "w",
    awsAuthMode: "apikey",
    bedrockModel: "b",
    taskbarWidget: false,
    inputDevice: "dev",
    inputDeviceKind: "loopback",
  };

  it("フォーム項目を localStorage へ書き戻す", () => {
    writeSettings(base);
    expect(localStorage.getItem("provider")).toBe("anthropic");
    expect(localStorage.getItem("audioFormat")).toBe("wav");
    expect(localStorage.getItem("recordMode")).toBe("momentary");
    expect(localStorage.getItem("taskbarWidget")).toBe("false");
    expect(localStorage.getItem("outputLang")).toBe("es");
  });

  it("write→read のラウンドトリップで主要値が保たれる", () => {
    writeSettings(base);
    const s = readSettings("ja");
    expect(s.provider).toBe("anthropic");
    expect(s.recordMode).toBe("momentary");
    expect(s.audioFormat).toBe("wav");
    expect(s.outputFormat).toBe("md");
    expect(s.sttProvider).toBe("openai");
    expect(s.awsAuthMode).toBe("apikey");
    expect(s.taskbarWidget).toBe(false);
  });
});
