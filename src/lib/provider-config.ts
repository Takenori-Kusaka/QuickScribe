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
 * 整形を実行できる設定かを検証する。問題があればユーザー向けの理由を返し、無ければ null。
 * - ローカル(Ollama): 鍵不要。
 * - AWS(Bedrock/Claude Platform): region 必須。claude-aws は workspace_id 必須。
 *   SigV4 は アクセスキー/シークレット、それ以外は APIキー。
 * - その他クラウド(Gemini/Anthropic/OpenAI): APIキー必須。
 */
export function validateRefineConfig(c: RefineConfig): string | null {
  if (LOCAL_PROVIDERS.includes(c.provider)) return null;
  if (AWS_PROVIDERS.includes(c.provider)) {
    if (!c.awsRegion.trim()) return "AWSリージョンを設定してください。";
    if (c.provider === "claude-aws" && !c.awsWorkspaceId.trim())
      return "Claude Platform on AWS には workspace_id が必要です。";
    if (c.awsAuthMode === "sigv4") {
      if (!c.awsAccessKey.trim() || !c.awsSecretKey.trim())
        return "AWSアクセスキー/シークレットを設定してください。";
    } else if (!c.apiKey.trim()) {
      return `${PROVIDER_LABELS[c.provider]} のAPIキーが必要です。`;
    }
    return null;
  }
  return c.apiKey.trim() ? null : `整形には ${PROVIDER_LABELS[c.provider]} のAPIキーが必要です。`;
}
