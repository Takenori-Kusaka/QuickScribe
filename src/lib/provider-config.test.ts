import { describe, it, expect } from "vitest";
import { validateRefineConfig, type RefineConfig } from "./provider-config";

const base: RefineConfig = {
  provider: "gemini",
  apiKey: "",
  awsRegion: "",
  awsWorkspaceId: "",
  awsAuthMode: "bearer",
  awsAccessKey: "",
  awsSecretKey: "",
};

describe("validateRefineConfig", () => {
  it("ローカル(ollama)は鍵不要でOK", () => {
    expect(validateRefineConfig({ ...base, provider: "ollama" })).toBeNull();
  });

  it("クラウドは APIキー必須", () => {
    expect(validateRefineConfig({ ...base, provider: "openai", apiKey: "" })).toContain(
      "APIキーが必要",
    );
    expect(validateRefineConfig({ ...base, provider: "openai", apiKey: "sk-x" })).toBeNull();
  });

  it("AWSは region 必須", () => {
    expect(validateRefineConfig({ ...base, provider: "bedrock" })).toContain("リージョン");
  });

  it("claude-aws は workspace_id 必須", () => {
    expect(
      validateRefineConfig({ ...base, provider: "claude-aws", awsRegion: "us-east-1" }),
    ).toContain("workspace_id");
  });

  it("AWS SigV4 は アクセスキー/シークレット必須", () => {
    const c: RefineConfig = {
      ...base,
      provider: "bedrock",
      awsRegion: "us-east-1",
      awsAuthMode: "sigv4",
    };
    expect(validateRefineConfig(c)).toContain("アクセスキー");
    expect(validateRefineConfig({ ...c, awsAccessKey: "AKIA", awsSecretKey: "secret" })).toBeNull();
  });

  it("AWS Bearer は APIキー必須", () => {
    const c: RefineConfig = {
      ...base,
      provider: "bedrock",
      awsRegion: "us-east-1",
      awsAuthMode: "bearer",
    };
    expect(validateRefineConfig(c)).toContain("APIキー");
    expect(validateRefineConfig({ ...c, apiKey: "key" })).toBeNull();
  });
});
