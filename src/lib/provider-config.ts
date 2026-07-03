// 整形プロバイダの設定検証（鍵/AWS資格情報の充足チェック）の純粋関数（#402）。
// App.svelte の refineConfigError から抽出してユニットテスト可能化。
import { LOCAL_PROVIDERS, AWS_PROVIDERS, PROVIDER_LABELS, type Provider } from "./constants";

export interface RefineConfig {
  provider: Provider;
  /** apiKeys[provider]（非AWS、または AWS の Bearer 認証で使用）。 */
  apiKey: string;
  awsRegion: string;
  awsWorkspaceId: string;
  /** "sigv4" のとき IAM 資格情報、それ以外は APIキー(Bearer) を要求。 */
  awsAuthMode: string;
  awsAccessKey: string;
  awsSecretKey: string;
}

/**
 * 検証エラー（i18n対応 / #401 Phase2）。`code` は errors.* カタログのキー、
 * `params` は ICU 補間値。呼び出し側で `$_(code, { values: params })` で翻訳する。
 * これにより lib は言語に依存せず、多言語(ja/en/zh/es)で表示できる。
 */
export interface RefineConfigError {
  code: string;
  params?: Record<string, string>;
}

/**
 * 整形を実行できる設定かを検証する。
 * - ローカル(Ollama): 鍵不要。
 * - AWS(Bedrock/Claude Platform): region 必須。claude-aws は workspace_id 必須。
 *   SigV4 は アクセスキー/シークレット、それ以外は APIキー。
 * - その他クラウド(Gemini/Anthropic/OpenAI): APIキー必須。
 * @param c 検証する整形設定（プロバイダ・鍵・AWS 資格情報）。
 * @returns 不足があれば i18n コード付きの {@link RefineConfigError}、問題なければ null。
 */
export function validateRefineConfig(c: RefineConfig): RefineConfigError | null {
  if (LOCAL_PROVIDERS.includes(c.provider)) return null;
  if (AWS_PROVIDERS.includes(c.provider)) {
    if (!c.awsRegion.trim()) return { code: "errors.cfg_aws_region" };
    if (c.provider === "claude-aws" && !c.awsWorkspaceId.trim())
      return { code: "errors.cfg_workspace_id" };
    if (c.awsAuthMode === "sigv4") {
      if (!c.awsAccessKey.trim() || !c.awsSecretKey.trim()) return { code: "errors.cfg_aws_keys" };
    } else if (!c.apiKey.trim()) {
      return { code: "errors.cfg_api_key_aws", params: { provider: PROVIDER_LABELS[c.provider] } };
    }
    return null;
  }
  return c.apiKey.trim()
    ? null
    : { code: "errors.cfg_api_key", params: { provider: PROVIDER_LABELS[c.provider] } };
}
