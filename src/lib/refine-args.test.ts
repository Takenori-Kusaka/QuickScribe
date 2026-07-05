import { describe, it, expect } from "vitest";
import { buildRefineArgs, type RefineArgsInput } from "./refine-args";

const base: RefineArgsInput = {
  transcript: "本文",
  provider: "gemini",
  apiKey: "k",
  bedrockModel: "",
  resolvedModel: "gemini-flash-latest",
  style: "structured",
  customStyles: [],
  entryTags: "",
  awsRegion: "",
  awsWorkspaceId: "",
  awsAuthMode: "bearer",
  awsAccessKey: "",
  awsSecretKey: "",
  awsSessionToken: "",
  translateOutput: false,
  outputLang: "ja",
  openaiBaseUrl: "",
};

describe("buildRefineArgs", () => {
  it("基本: text/provider/apiKey/model/style", () => {
    const a = buildRefineArgs(base);
    expect(a).toMatchObject({
      text: "本文",
      provider: "gemini",
      apiKey: "k",
      model: "gemini-flash-latest",
      style: "structured",
    });
    expect(a.tags).toBeUndefined();
    expect(a.customInstruction).toBeUndefined();
  });

  it("bedrock は手入力モデルを使う", () => {
    const a = buildRefineArgs({
      ...base,
      provider: "bedrock",
      bedrockModel: "my-model",
      awsRegion: "us-east-1",
    });
    expect(a.model).toBe("my-model");
  });

  it("カスタムスタイルは指示文を渡す", () => {
    const a = buildRefineArgs({
      ...base,
      style: "custom:x",
      customStyles: [{ id: "x", label: "X", instruction: "指示" }],
    });
    expect(a.customInstruction).toBe("指示");
  });

  it("未知のカスタムは null", () => {
    const a = buildRefineArgs({ ...base, style: "custom:none", customStyles: [] });
    expect(a.customInstruction).toBeNull();
  });

  it("タグはパースして配列で付与（空なら付かない）", () => {
    expect(buildRefineArgs({ ...base, entryTags: "a, b" }).tags).toEqual(["a", "b"]);
    expect(buildRefineArgs({ ...base, entryTags: "  " }).tags).toBeUndefined();
  });

  it("翻訳: ON かつ既知言語のとき英語名を outputLang で渡す（#453）", () => {
    expect(buildRefineArgs({ ...base, translateOutput: true, outputLang: "vi" }).outputLang).toBe(
      "Vietnamese",
    );
    expect(buildRefineArgs({ ...base, translateOutput: true, outputLang: "en" }).outputLang).toBe(
      "English",
    );
  });

  it("翻訳: OFF または未知言語では outputLang を渡さない（原語のまま）", () => {
    expect(
      buildRefineArgs({ ...base, translateOutput: false, outputLang: "vi" }).outputLang,
    ).toBeUndefined();
    expect(
      buildRefineArgs({ ...base, translateOutput: true, outputLang: "zzz" }).outputLang,
    ).toBeUndefined();
  });

  it("OpenAI互換 base_url: OpenAIプロバイダで非空のときだけ baseUrl を渡す（#593）", () => {
    // OpenAI かつ非空 → trim して baseUrl を送る。
    expect(
      buildRefineArgs({ ...base, provider: "openai", openaiBaseUrl: " http://localhost:4000 " })
        .baseUrl,
    ).toBe("http://localhost:4000");
    // OpenAI でも空/空白なら送らない（既定=公式）。
    expect(
      buildRefineArgs({ ...base, provider: "openai", openaiBaseUrl: "  " }).baseUrl,
    ).toBeUndefined();
    // OpenAI 以外は base_url を無視する（値があっても送らない）。
    expect(
      buildRefineArgs({ ...base, provider: "gemini", openaiBaseUrl: "http://localhost:4000" })
        .baseUrl,
    ).toBeUndefined();
  });

  it("AWS SigV4 は資格情報を付与、session 空は null", () => {
    const a = buildRefineArgs({
      ...base,
      provider: "claude-aws",
      awsRegion: "us-east-1 ",
      awsWorkspaceId: "ws ",
      awsAuthMode: "sigv4",
      awsAccessKey: "AKIA ",
      awsSecretKey: "secret ",
      awsSessionToken: "",
    });
    expect(a).toMatchObject({
      region: "us-east-1",
      workspaceId: "ws",
      authMode: "sigv4",
      awsAccessKey: "AKIA",
      awsSecretKey: "secret",
      awsSessionToken: null,
    });
  });

  it("AWS Bearer はキーを付けない", () => {
    const a = buildRefineArgs({
      ...base,
      provider: "bedrock",
      awsRegion: "us",
      awsAuthMode: "bearer",
    });
    expect(a.awsAccessKey).toBeUndefined();
    expect(a.authMode).toBe("bearer");
  });
});
