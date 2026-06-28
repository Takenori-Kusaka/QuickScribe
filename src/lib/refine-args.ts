// 整形コマンド(refine_text)へ渡す引数オブジェクトの組み立て（#402）。
// App.svelte の refineArgs から純粋ロジックを抽出してユニットテスト可能化。
import { AWS_PROVIDERS, type Provider, type CustomStyle } from "./constants";
import { parseTags } from "./entry";

export interface RefineArgsInput {
  transcript: string | null;
  provider: Provider;
  apiKey: string;
  /** Bedrock は手入力モデル、それ以外は解決済みモデル（空ならバックエンドが既定補完）。 */
  bedrockModel: string;
  resolvedModel: string;
  /** styleOverride ?? refineStyle（"custom:<id>" を含む）。 */
  style: string;
  customStyles: CustomStyle[];
  entryTags: string;
  awsRegion: string;
  awsWorkspaceId: string;
  awsAuthMode: string;
  awsAccessKey: string;
  awsSecretKey: string;
  awsSessionToken: string;
}

/** refine_text コマンドの引数を組み立てる。カスタム整形・内省タグ・AWS認証を反映する。 */
export function buildRefineArgs(i: RefineArgsInput): Record<string, unknown> {
  const base: Record<string, unknown> = {
    text: i.transcript,
    provider: i.provider,
    apiKey: i.apiKey,
    model: i.provider === "bedrock" ? i.bedrockModel : i.resolvedModel,
    style: i.style,
  };
  // カスタムパターン選択時は指示文をバックエンドへ渡す（style既定指示の代わりに使われる）。
  if (i.style.startsWith("custom:")) {
    const cs = i.customStyles.find((c) => `custom:${c.id}` === i.style);
    base.customInstruction = cs?.instruction ?? null;
  }
  // 内省タグ（S4.3）。保存時にメタデータとして付与。
  const tags = parseTags(i.entryTags);
  if (tags.length > 0) base.tags = tags;
  if (AWS_PROVIDERS.includes(i.provider)) {
    base.region = i.awsRegion.trim();
    base.workspaceId = i.awsWorkspaceId.trim();
    base.authMode = i.awsAuthMode;
    if (i.awsAuthMode === "sigv4") {
      base.awsAccessKey = i.awsAccessKey.trim();
      base.awsSecretKey = i.awsSecretKey.trim();
      base.awsSessionToken = i.awsSessionToken.trim() || null;
    }
  }
  return base;
}
