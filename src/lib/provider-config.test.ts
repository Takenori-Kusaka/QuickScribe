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

  it("クラウドは APIキー必須（コード＋provider params）", () => {
    expect(validateRefineConfig({ ...base, provider: "openai", apiKey: "" })).toEqual({
      code: "errors.cfg_api_key",
      params: { provider: "OpenAI" },
    });
    expect(validateRefineConfig({ ...base, provider: "openai", apiKey: "sk-x" })).toBeNull();
  });

  it("AWSは region 必須", () => {
    expect(validateRefineConfig({ ...base, provider: "bedrock" })).toEqual({
      code: "errors.cfg_aws_region",
    });
  });

  it("claude-aws は workspace_id 必須", () => {
    expect(
      validateRefineConfig({ ...base, provider: "claude-aws", awsRegion: "us-east-1" }),
    ).toEqual({ code: "errors.cfg_workspace_id" });
  });

  it("AWS SigV4 は アクセスキー/シークレット必須", () => {
    const c: RefineConfig = {
      ...base,
      provider: "bedrock",
      awsRegion: "us-east-1",
      awsAuthMode: "sigv4",
    };
    expect(validateRefineConfig(c)).toEqual({ code: "errors.cfg_aws_keys" });
    expect(validateRefineConfig({ ...c, awsAccessKey: "AKIA", awsSecretKey: "secret" })).toBeNull();
  });

  it("AWS Bearer は APIキー必須（コード＋provider params）", () => {
    const c: RefineConfig = {
      ...base,
      provider: "bedrock",
      awsRegion: "us-east-1",
      awsAuthMode: "bearer",
    };
    expect(validateRefineConfig(c)).toEqual({
      code: "errors.cfg_api_key_aws",
      params: { provider: "AWS Bedrock" },
    });
    expect(validateRefineConfig({ ...c, apiKey: "key" })).toBeNull();
  });
});
