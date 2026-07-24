// 整形の知性(FormattingEngine: BYO-Cloud / ADR-0005)。
// 文字起こしテキストを、ニュアンスを残しつつ思考整理・構造化する(コア価値=ADR-0004)。
//
// 設計(S3.1/S3.3):
// - 整形「スタイル」(構造化/逐語/要約/ブレスト)を `RefineStyle` で切り替え可能にする(S3.3)。
//   逐語⇄要約⇄ブレストを行き来できるのがコア価値(ADR-0004)。
// - プロバイダは `FormattingEngine` trait の実装として差し替え可能(S3.1: 戦略の差し替え=DIP境界)。
//   実装済み6種: Gemini / Anthropic / OpenAI / ローカルOllama(S3.4) /
//   AWS Bedrock / Claude Platform on AWS(ADR-0011)。
// - 鍵はフロントの設定から渡す(コードに埋め込まない/ADR-0005)。

use serde_json::json;

use crate::aws_sign::{sign_post, AwsCreds};

/// AWSプロバイダ(Bedrock / Claude Platform on AWS)の認証方式(両対応 / ADR-0011)。
#[derive(Clone)]
pub enum AwsAuth {
    /// APIキー認証(Claude Platform=x-api-key / Bedrock=Authorization Bearer)。
    ApiKey,
    /// AWS IAM(SigV4署名)。一時credのときのみ session_token を Some。
    SigV4 {
        access_key: String,
        secret_key: String,
        session_token: Option<String>,
    },
}

/// AWSプロバイダ固有の設定(region / Claude Platform の workspace_id / 認証)。
#[derive(Clone)]
pub struct AwsConfig {
    pub region: String,
    /// Claude Platform on AWS で必須(anthropic-workspace-id ヘッダ)。Bedrock では未使用。
    pub workspace_id: String,
    pub auth: AwsAuth,
}

/// 整形スタイル(コア価値「整形の知性」: 逐語⇄要約⇄ブレストを行き来する / ADR-0004, S3.3)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefineStyle {
    /// 既定: 見出し+箇条書きで構造化し、ニュアンスを残す。
    Structured,
    /// 逐語: 言い淀み・繰り返しも極力残し、最小限の整形のみ。
    Verbatim,
    /// 要約: 全体を短く要約し要点を絞る。
    Summary,
    /// ブレスト: 内容から問い・観点・次の一歩を広げる。
    Brainstorm,
}

impl RefineStyle {
    /// フロントから渡る文字列(英/日)を解釈する。未知・空は Structured(既定)。
    pub fn parse(s: &str) -> RefineStyle {
        match s.trim().to_ascii_lowercase().as_str() {
            "verbatim" | "逐語" => RefineStyle::Verbatim,
            "summary" | "要約" => RefineStyle::Summary,
            "brainstorm" | "ブレスト" => RefineStyle::Brainstorm,
            _ => RefineStyle::Structured,
        }
    }

    /// 全スタイル共通のシステム指示(捏造禁止・ニュアンス保持=コア価値)。
    /// 「本文だけ」を XML タグ境界で囲ませる(前置き=AIの挨拶／後書き=補足 の3層を排す)。
    /// ※本文をJSONに封入すると品質が劣化し壊れやすい(docs/research/sources/llm-output-control.md)。
    ///   自由生成のまま `<journal>…</journal>` で囲ませ、コード側でタグ内を決定的に抽出する。
    fn system_prompt(&self) -> &'static str {
        "あなたは話者本人の思考整理を助けるアシスタントです。事実を捏造せず、\
         書かれていないことは足さず、本人のニュアンス・迷い・語り口を残してください。\
         整形後の本文だけを <journal> と </journal> で囲んで出力し、\
         タグの外には前置き・挨拶・後書き・補足説明を一切書かないでください。"
    }

    /// スタイル別の指示ブロック(箇条書き)。カスタムパターンはここだけを差し替える。
    fn instruction(&self) -> &'static str {
        match self {
            RefineStyle::Structured => "\
                - 要点を見出しと箇条書きで構造化する\n\
                - 言い淀みや繰り返しは整理しつつ、本人のニュアンス・迷い・語り口は残す\n\
                - 最後に「ひとことまとめ」を1〜2文で添える",
            RefineStyle::Verbatim => "\
                - 逐語に近い形で、言い淀み・繰り返し・口癖も極力そのまま残す\n\
                - 改行・句読点・段落分けなど最小限の読みやすさ調整のみ行う\n\
                - 要約や再構成はしない",
            RefineStyle::Summary => "\
                - 全体を短く要約する\n\
                - 重要な要点だけを3〜5個の箇条書きにする\n\
                - 細部は削ぎ落としつつ、本人の結論・気持ちのニュアンスは残す",
            RefineStyle::Brainstorm => "\
                - 内容から派生する問い・観点・次の一歩をブレスト的に列挙して思考を広げる\n\
                - 本人が気づいていない切り口や深掘りポイントを提案する\n\
                - 元の発話の要点も簡潔に添える",
        }
    }

    /// スタイル別のユーザープロンプトを組み立てる(純粋関数・テスト対象)。
    fn user_prompt(&self, transcript: &str) -> String {
        build_user_prompt(self.instruction(), transcript)
    }
}

/// 指示ブロックと本文からユーザープロンプトを組み立てる(純粋関数)。
/// 共通の不変条件(捏造禁止・本人の思考整理補助)は指示の中身に関わらず常に付与する。
/// → カスタムパターンでも「事実を捏造しない」等のコア規律が外れない。
fn build_user_prompt(instruction: &str, transcript: &str) -> String {
    format!(
        "以下は音声の文字起こしです。話者本人の思考整理を助けてください。\n\
         {instruction}\n\
         - 事実を捏造せず、書かれていないことは足さない\n\n\
         ---\n{transcript}"
    )
}

/// 整形リクエスト(スタイル・鍵・モデル・本文)。FormattingEngine に渡す。
pub struct RefineRequest<'a> {
    pub style: RefineStyle,
    pub api_key: &'a str,
    pub model: &'a str,
    pub transcript: &'a str,
    /// AWSプロバイダのときのみ Some(region/workspace_id/認証)。それ以外は None。
    pub aws: Option<&'a AwsConfig>,
    /// ユーザー定義のカスタム整形指示(S3.3)。Some かつ非空なら style の指示の代わりに使う。
    /// システム指示(捏造禁止・<journal>タグ境界)は共通で不変＝カスタムでも品質ガードが効く。
    pub custom_instruction: Option<&'a str>,
    /// 整形出力言語(翻訳 / #453)。Some(英語名 例 "Vietnamese")なら、話者/原文の言語に
    /// 関わらず指定言語で出力するようシステム指示に追記する(1パス)。原語の文字起こしは別途保持。
    pub output_lang: Option<&'a str>,
    /// OpenAI互換エンドポイントの接続先(base_url / #593)。Some かつ非空なら、公式 api.openai.com の
    /// 代わりにこの URL へ送る(LiteLLM 等のゲートウェイ・self-host のローカルLLMを対象にできる)。
    /// OpenAI プロバイダのときのみ意味を持つ。空/未指定は従来どおり公式エンドポイント。
    pub base_url: Option<&'a str>,
}

impl RefineRequest<'_> {
    /// 全リクエスト共通のシステム指示(カスタムでも不変)。
    /// output_lang 指定時は、指定言語で出力する指示を追記する(ニュアンス保持＋タグ境界は不変)。
    fn system_prompt(&self) -> String {
        let base = self.style.system_prompt();
        match self.output_lang {
            Some(lang) if !lang.trim().is_empty() => format!(
                "{base}\n\nIMPORTANT: Write the entire refined output in {lang}, translating from \
                 the source language if needed, while preserving the speaker's nuance and tone. \
                 Keep the <journal> and </journal> tags exactly as instructed above."
            ),
            _ => base.to_string(),
        }
    }

    /// ユーザープロンプト。カスタム指示があればそれを、無ければスタイル既定の指示を使う。
    fn user_prompt(&self) -> String {
        match self.custom_instruction {
            Some(instr) if !instr.trim().is_empty() => build_user_prompt(instr, self.transcript),
            _ => self.style.user_prompt(self.transcript),
        }
    }
}

/// 整形エンジンの抽象(S3.1: プロバイダ/戦略を差し替え可能にする DIP 境界)。
pub trait FormattingEngine {
    fn refine(&self, req: &RefineRequest) -> Result<String, String>;
}

/// 整形プロバイダ(#392: lib/refine に散在した provider 文字列マッチを単一ソース化 / OCP)。
/// 別名の解釈・AWS判定・既定モデル・エンジン生成をここに集約する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefineProvider {
    Gemini,
    Anthropic,
    OpenAi,
    Ollama,
    Bedrock,
    ClaudePlatformAws,
}

impl RefineProvider {
    /// フロントから渡る provider 文字列(別名込み)を解釈する。空/未知/"gemini" は Gemini(既定)。
    pub fn parse(provider: &str) -> Self {
        match provider.trim().to_ascii_lowercase().as_str() {
            "anthropic" | "claude" => Self::Anthropic,
            "openai" | "gpt" => Self::OpenAi,
            "ollama" | "local" => Self::Ollama,
            "bedrock" | "aws-bedrock" => Self::Bedrock,
            "claude-aws" | "claude-platform-aws" | "anthropic-aws" => Self::ClaudePlatformAws,
            _ => Self::Gemini,
        }
    }

    /// AWS系(Bedrock / Claude Platform on AWS)か。AwsConfig 組み立ての要否判定 / ADR-0011。
    pub fn is_aws(self) -> bool {
        matches!(self, Self::Bedrock | Self::ClaudePlatformAws)
    }

    /// 既定モデルID(実行時解決に失敗した際のフォールバック)。
    pub fn default_model(self) -> &'static str {
        match self {
            Self::Anthropic => "claude-sonnet-4-6",
            Self::OpenAi => "gpt-4o",
            Self::Ollama => "llama3.1",
            // AWS Bedrock のモデルIDは anthropic. プレフィックス(リージョン/アカウント依存。UIで上書き可)。
            Self::Bedrock => "anthropic.claude-sonnet-4-6",
            // Claude Platform on AWS は第一者と同じ bare ID。
            Self::ClaudePlatformAws => "claude-sonnet-4-6",
            Self::Gemini => "gemini-flash-latest",
        }
    }

    /// 対応する整形エンジンを生成する(S3.1: 戦略の差し替え=DIP 境界)。
    pub fn make_engine(self) -> Box<dyn FormattingEngine> {
        match self {
            Self::Anthropic => Box::new(AnthropicEngine),
            Self::OpenAi => Box::new(OpenAiEngine),
            Self::Ollama => Box::new(OllamaEngine),
            Self::Bedrock => Box::new(BedrockEngine),
            Self::ClaudePlatformAws => Box::new(ClaudePlatformAwsEngine),
            Self::Gemini => Box::new(GeminiEngine),
        }
    }
}

/// プロバイダ名から整形エンジンを解決する。空/未知は Gemini にフォールバック。
pub fn engine_for(provider: &str) -> Box<dyn FormattingEngine> {
    RefineProvider::parse(provider).make_engine()
}

/// OpenAI互換の接続先(base_url)を解決する(#593)。req 側で非空指定があればそれを
/// (末尾スラッシュを除去して)使い、LiteLLM 等のゲートウェイや self-host のローカルLLMへ
/// 送れるようにする。空/未指定は従来どおり公式 api.openai.com(テスト時のみ
/// QS_TEST_OPENAI_BASE で差し替え可)。純粋関数=接続先解決の分岐をテストで固定する。
fn openai_base_url(req_base: Option<&str>) -> String {
    match req_base {
        Some(b) if !b.trim().is_empty() => b.trim().trim_end_matches('/').to_string(),
        _ => crate::api_base("https://api.openai.com", "QS_TEST_OPENAI_BASE"),
    }
}

/// Ollama の既定エンドポイント(ローカル)。環境変数 OLLAMA_HOST があれば優先。
fn ollama_base() -> String {
    std::env::var("OLLAMA_HOST")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(|h| {
            let h = h.trim().trim_end_matches('/').to_string();
            if h.starts_with("http") { h } else { format!("http://{h}") }
        })
        .unwrap_or_else(|| "http://localhost:11434".to_string())
}

/// 整形のエントリポイント。プロバイダとスタイルを解決して整形する。
#[allow(clippy::too_many_arguments)]
pub fn refine(
    provider: &str,
    api_key: &str,
    model: &str,
    style: &str,
    transcript: &str,
    aws: Option<AwsConfig>,
    custom_instruction: Option<String>,
    output_lang: Option<String>,
    base_url: Option<String>,
) -> Result<String, String> {
    // 空・空白のみの入力はAPIを呼ばずに弾く(無駄なコスト・無意味な整形を防ぐ / #18)。
    if transcript.trim().is_empty() {
        return Err(crate::errcode::E_REFINE_EMPTY_INPUT.into());
    }
    let req = RefineRequest {
        style: RefineStyle::parse(style),
        api_key,
        model,
        transcript,
        aws: aws.as_ref(),
        custom_instruction: custom_instruction.as_deref(),
        output_lang: output_lang.as_deref(),
        base_url: base_url.as_deref(),
    };
    engine_for(provider).refine(&req)
}

/// タイトル生成に渡す本文の最大文字数(冒頭で内容は判別できるためトークンを節約する)。
const TITLE_INPUT_MAX_CHARS: usize = 1500;

/// モデル応答からタイトル1行を取り出す(純粋・テスト対象)。
/// 最初の非空行を採り、見出し記号(#)・引用符・かぎ括弧の装飾を剥がす。
fn title_from_response(raw: &str) -> String {
    let line = raw
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("");
    line.trim_start_matches('#')
        .trim()
        .trim_matches(['"', '\'', '「', '」', '『', '』'])
        .trim()
        .to_string()
}

/// 整形結果のファイル名に付けるタイトルを同一プロバイダで生成する(ADR-0032)。
/// 本文の冒頭のみを送り、タイトル1行だけを受け取る。失敗時は呼び出し側が
/// 本文冒頭ラベルへフォールバックする想定(タイトル生成の失敗で保存は止めない)。
pub fn generate_title(
    provider: &str,
    api_key: &str,
    model: &str,
    text: &str,
    aws: Option<&AwsConfig>,
    base_url: Option<&str>,
) -> Result<String, String> {
    if text.trim().is_empty() {
        return Err(crate::errcode::E_REFINE_EMPTY_INPUT.into());
    }
    let head: String = text.chars().take(TITLE_INPUT_MAX_CHARS).collect();
    let req = RefineRequest {
        style: RefineStyle::parse(""),
        api_key,
        model,
        transcript: &head,
        aws,
        custom_instruction: Some(
            "- この文章全体の内容を表す簡潔なタイトルを1つだけ考える\n\
             - 本文と同じ言語で、20文字以内\n\
             - タイトルの文字列のみを出力する(前置き・引用符・記号装飾なし)",
        ),
        output_lang: None,
        base_url,
    };
    let raw = engine_for(provider).refine(&req)?;
    let title = title_from_response(&raw);
    if title.is_empty() {
        Err(crate::errcode::E_REFINE_EMPTY_RESULT.into())
    } else {
        Ok(title)
    }
}

/// 本文を囲む XML タグ(自由生成のまま境界だけ作らせ、本文をJSONに入れない=品質劣化回避)。
const BODY_OPEN: &str = "<journal>";
const BODY_CLOSE: &str = "</journal>";

/// モデル応答から整形本文だけを取り出す(docs/research/sources/llm-output-control.md の確定方針)。
/// L1 : `<journal>…</journal>` のタグ内(最初の開始〜最後の終了=最外)を抽出。
/// L1': 終了タグ欠落(truncation等)は開始タグ以降を本文として救済。
/// L2 : タグ欠落時は定型前置きを保守的に1行だけ除去(過剰除去で本文を壊さない)。
/// どの経路でも本文を失わない(コア価値: 整形は付加価値、原文喪失は事故)。
fn extract_tagged_body(raw: &str) -> String {
    let trimmed = raw.trim();
    if let Some(start) = trimmed.find(BODY_OPEN) {
        let after = start + BODY_OPEN.len();
        if let Some(end_rel) = trimmed[after..].rfind(BODY_CLOSE) {
            let inner = trimmed[after..after + end_rel].trim();
            if !inner.is_empty() {
                return inner.to_string();
            }
        }
        // L1': 終了タグが無い → 開始タグ以降を救済。
        let inner = trimmed[after..].trim();
        if !inner.is_empty() {
            return inner.to_string();
        }
    }
    strip_preamble(trimmed)
}

/// 先頭の定型前置き行(挨拶/「以下が…」等)を保守的に1行だけ除去する。
/// タグ抽出が主軸なので稀なフォールバック。短い先頭行が前置き語で始まり、かつ
/// 後続に本文がある場合のみ落とす(本文を誤って削らないため最小限)。
fn strip_preamble(s: &str) -> String {
    let trimmed = s.trim();
    let first = trimmed.lines().next().unwrap_or("").trim();
    const PREFIXES: [&str; 12] = [
        "はい", "わかりました", "了解", "承知", "以下", "整形しました", "まとめると",
        "Here is", "Here's", "Sure", "Certainly", "Below is",
    ];
    let looks_preamble =
        first.len() < 60 && PREFIXES.iter().any(|p| first.starts_with(p));
    if looks_preamble {
        if let Some(rest) = trimmed.splitn(2, '\n').nth(1) {
            let rest = rest.trim();
            if !rest.is_empty() {
                return rest.to_string();
            }
        }
    }
    trimmed.to_string()
}

struct GeminiEngine;
struct AnthropicEngine;
struct OpenAiEngine;
struct OllamaEngine;
/// AWS Bedrock(InvokeModel)。APIキー(Bearer)/SigV4両対応 / ADR-0011。
struct BedrockEngine;
/// Claude Platform on AWS(aws-external-anthropic / Messages API互換)。x-api-key/SigV4両対応。
struct ClaudePlatformAwsEngine;

/// 整形API(JSON POST)の共通呼び出し(#392 重複排除)。Content-Type: application/json を付与し、
/// 追加ヘッダを載せて body を送信、応答JSONを返す。provider はエラーメッセージ用ラベル(例 "Gemini")。
/// AWS署名系(body_bytesを署名)と Ollama(独自エラーメッセージ)は対象外。
fn post_json_refine(
    url: &str,
    headers: &[(&str, &str)],
    body: &serde_json::Value,
    provider: &str,
) -> Result<serde_json::Value, String> {
    let mut request = ureq::post(url).header("Content-Type", "application/json");
    for (k, v) in headers {
        request = request.header(*k, *v);
    }
    let mut resp = request
        .send_json(body)
        .map_err(|e| crate::errcode::ec(crate::errcode::E_REFINE_HTTP, format!("{provider}: {e}")))?;
    resp.body_mut().read_json().map_err(|e| e.to_string())
}

/// Anthropic Messages 互換応答(content[] の type=="text" を連結)から本文を取り出す。
fn anthropic_text(v: &serde_json::Value) -> String {
    let mut out = String::new();
    if let Some(blocks) = v.get("content").and_then(|c| c.as_array()) {
        for block in blocks {
            if block.get("type").and_then(|t| t.as_str()) == Some("text") {
                if let Some(t) = block.get("text").and_then(|t| t.as_str()) {
                    out.push_str(t);
                }
            }
        }
    }
    out
}

/// Gemini で整形する。
impl FormattingEngine for GeminiEngine {
    fn refine(&self, req: &RefineRequest) -> Result<String, String> {
        if req.api_key.trim().is_empty() {
            return Err(crate::errcode::E_REFINE_GEMINI_NO_KEY.into());
        }
        let url = format!(
            "{}/v1beta/models/{}:generateContent?key={}",
            crate::api_base("https://generativelanguage.googleapis.com", "QS_TEST_GEMINI_BASE"),
            req.model,
            req.api_key
        );
        // Gemini はシステムロールを使わないため、システム指示をユーザーテキストに前置する。
        let text = format!(
            "{}\n\n{}",
            req.system_prompt(),
            req.user_prompt()
        );
        let body = json!({ "contents": [{ "parts": [{ "text": text }] }] });

        let v = post_json_refine(&url, &[], &body, "Gemini")?;

        let mut out = String::new();
        if let Some(parts) = v
            .get("candidates")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("content"))
            .and_then(|c| c.get("parts"))
            .and_then(|p| p.as_array())
        {
            for part in parts {
                if let Some(t) = part.get("text").and_then(|t| t.as_str()) {
                    out.push_str(t);
                }
            }
        }
        let body = extract_tagged_body(&out);
        if body.is_empty() {
            return Err(crate::errcode::ec(crate::errcode::E_REFINE_EMPTY_RESULT, format!("Gemini: {v}")));
        }
        Ok(body)
    }
}

/// Anthropic(Claude) Messages API で整形する。
/// 必須ヘッダ: x-api-key, anthropic-version: 2023-06-01, content-type: application/json
impl FormattingEngine for AnthropicEngine {
    fn refine(&self, req: &RefineRequest) -> Result<String, String> {
        if req.api_key.trim().is_empty() {
            return Err(crate::errcode::E_REFINE_ANTHROPIC_NO_KEY.into());
        }
        let body = json!({
            "model": req.model,
            "max_tokens": 4096,
            "system": req.system_prompt(),
            "messages": [
                { "role": "user", "content": req.user_prompt() }
            ]
        });

        let url = format!(
            "{}/v1/messages",
            crate::api_base("https://api.anthropic.com", "QS_TEST_ANTHROPIC_BASE")
        );
        let v = post_json_refine(
            &url,
            &[
                ("x-api-key", req.api_key),
                ("anthropic-version", "2023-06-01"),
            ],
            &body,
            "Anthropic",
        )?;

        // 応答は content[] のブロック配列。type=="text" の text を連結する(anthropic_text)。
        let out = anthropic_text(&v);
        let body = extract_tagged_body(&out);
        if body.is_empty() {
            return Err(crate::errcode::ec(crate::errcode::E_REFINE_EMPTY_RESULT, format!("Anthropic: {v}")));
        }
        Ok(body)
    }
}

/// OpenAI Chat Completions API で整形する。
/// 必須ヘッダ: Authorization: Bearer <key>, content-type: application/json
impl FormattingEngine for OpenAiEngine {
    fn refine(&self, req: &RefineRequest) -> Result<String, String> {
        if req.api_key.trim().is_empty() {
            return Err(crate::errcode::E_REFINE_OPENAI_NO_KEY.into());
        }
        let body = json!({
            "model": req.model,
            "messages": [
                { "role": "system", "content": req.system_prompt() },
                { "role": "user", "content": req.user_prompt() }
            ]
        });

        let bearer = format!("Bearer {}", req.api_key);
        let url = format!("{}/v1/chat/completions", openai_base_url(req.base_url));
        let v = post_json_refine(
            &url,
            &[("Authorization", &bearer)],
            &body,
            "OpenAI",
        )?;

        let out = v
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();
        let body = extract_tagged_body(&out);
        if body.is_empty() {
            return Err(crate::errcode::ec(crate::errcode::E_REFINE_EMPTY_RESULT, format!("OpenAI: {v}")));
        }
        Ok(body)
    }
}

/// ローカル Ollama で整形する(S3.4)。鍵不要・端末内完結＝差別化「ローカルプライバシー」の核。
/// API: POST {base}/api/chat {model, messages, stream:false} → {message:{content}}。
impl FormattingEngine for OllamaEngine {
    fn refine(&self, req: &RefineRequest) -> Result<String, String> {
        let model = if req.model.trim().is_empty() {
            "llama3.1"
        } else {
            req.model
        };
        let body = json!({
            "model": model,
            "stream": false,
            "messages": [
                { "role": "system", "content": req.system_prompt() },
                { "role": "user", "content": req.user_prompt() }
            ]
        });

        let url = format!("{}/api/chat", ollama_base());
        let mut resp = ureq::post(&url)
            .header("Content-Type", "application/json")
            .send_json(&body)
            .map_err(|e| crate::errcode::ec(crate::errcode::E_REFINE_OLLAMA_CONN, e))?;
        let v: serde_json::Value = resp.body_mut().read_json().map_err(|e| e.to_string())?;

        let out = v
            .get("message")
            .and_then(|m| m.get("content"))
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();
        let body = extract_tagged_body(&out);
        if body.is_empty() {
            return Err(crate::errcode::ec(crate::errcode::E_REFINE_EMPTY_RESULT, format!("Ollama: {v}")));
        }
        Ok(body)
    }
}

/// AWSプロバイダに付与する認証ヘッダ(APIキー or SigV4署名)を計算して返す。
/// SigV4 は body_bytes を署名するため、呼び出し側は同一バイト列を send すること。
/// service: Bedrock="bedrock" / Claude Platform="aws-external-anthropic"。
fn aws_auth_headers(
    url: &str,
    body_bytes: &[u8],
    service: &str,
    aws: &AwsConfig,
    api_key: &str,
    bearer: bool,
) -> Result<Vec<(String, String)>, String> {
    let mut headers: Vec<(String, String)> = Vec::new();
    match &aws.auth {
        AwsAuth::ApiKey => {
            if api_key.trim().is_empty() {
                return Err(crate::errcode::E_REFINE_NO_KEY.into());
            }
            if bearer {
                headers.push(("Authorization".into(), format!("Bearer {}", api_key.trim())));
            } else {
                headers.push(("x-api-key".into(), api_key.trim().into()));
            }
        }
        AwsAuth::SigV4 {
            access_key,
            secret_key,
            session_token,
        } => {
            if access_key.trim().is_empty() || secret_key.trim().is_empty() {
                return Err(crate::errcode::E_REFINE_AWS_NO_KEYS.into());
            }
            let creds = AwsCreds {
                access_key: access_key.clone(),
                secret_key: secret_key.clone(),
                session_token: session_token.clone().filter(|s| !s.trim().is_empty()),
                region: aws.region.clone(),
            };
            headers.extend(sign_post(url, body_bytes, service, &creds)?);
        }
    }
    Ok(headers)
}

/// AWS Bedrock(InvokeModel)で整形する / ADR-0011。
/// エンドポイント: bedrock-runtime.{region}.amazonaws.com/model/{modelId}/invoke。
/// body は Anthropic on Bedrock 形式(model不要・anthropic_version必須)。署名名は "bedrock"。
impl FormattingEngine for BedrockEngine {
    fn refine(&self, req: &RefineRequest) -> Result<String, String> {
        let aws = req
            .aws
            .ok_or_else(|| crate::errcode::ec(crate::errcode::E_REFINE_AWS_CONFIG, "Bedrock"))?;
        if aws.region.trim().is_empty() {
            return Err(crate::errcode::E_REFINE_AWS_NO_REGION.into());
        }
        // Bedrock のモデルIDはリージョン/アカウント依存。未指定時は anthropic. プレフィックス既定。
        let model = if req.model.trim().is_empty() {
            "anthropic.claude-sonnet-4-6"
        } else {
            req.model.trim()
        };
        let url = format!(
            "{}/model/{}/invoke",
            crate::api_base(
                &format!("https://bedrock-runtime.{}.amazonaws.com", aws.region.trim()),
                "QS_TEST_BEDROCK_BASE"
            ),
            model
        );
        let body = json!({
            "anthropic_version": "bedrock-2023-05-31",
            "max_tokens": 4096,
            "system": req.system_prompt(),
            "messages": [
                { "role": "user", "content": req.user_prompt() }
            ]
        });
        let body_bytes = serde_json::to_vec(&body).map_err(|e| e.to_string())?;

        let mut request = ureq::post(&url).header("Content-Type", "application/json");
        for (k, v) in aws_auth_headers(&url, &body_bytes, "bedrock", aws, req.api_key, true)? {
            request = request.header(k.as_str(), v.as_str());
        }
        let mut resp = request
            .send(&body_bytes[..])
            .map_err(|e| crate::errcode::ec(crate::errcode::E_REFINE_HTTP, format!("Bedrock: {e}")))?;
        let v: serde_json::Value = resp.body_mut().read_json().map_err(|e| e.to_string())?;

        let out = anthropic_text(&v);
        let body = extract_tagged_body(&out);
        if body.is_empty() {
            return Err(crate::errcode::ec(crate::errcode::E_REFINE_EMPTY_RESULT, format!("Bedrock: {v}")));
        }
        Ok(body)
    }
}

/// Claude Platform on AWS(aws-external-anthropic)で整形する / ADR-0011。
/// 実体は Anthropic Messages API。差分は base_url と anthropic-workspace-id ヘッダ。
/// 署名名は "aws-external-anthropic"。APIキー時は x-api-key。
impl FormattingEngine for ClaudePlatformAwsEngine {
    fn refine(&self, req: &RefineRequest) -> Result<String, String> {
        let aws = req.aws.ok_or_else(|| {
            crate::errcode::ec(crate::errcode::E_REFINE_AWS_CONFIG, "Claude Platform on AWS")
        })?;
        if aws.region.trim().is_empty() {
            return Err(crate::errcode::E_REFINE_AWS_NO_REGION.into());
        }
        let url = format!(
            "{}/v1/messages",
            crate::api_base(
                &format!("https://aws-external-anthropic.{}.api.aws", aws.region.trim()),
                "QS_TEST_CLAUDE_AWS_BASE"
            )
        );
        let body = json!({
            "model": req.model,
            "max_tokens": 4096,
            "system": req.system_prompt(),
            "messages": [
                { "role": "user", "content": req.user_prompt() }
            ]
        });
        let body_bytes = serde_json::to_vec(&body).map_err(|e| e.to_string())?;

        let mut request = ureq::post(&url)
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01");
        if !aws.workspace_id.trim().is_empty() {
            request = request.header("anthropic-workspace-id", aws.workspace_id.trim());
        }
        for (k, v) in
            aws_auth_headers(&url, &body_bytes, "aws-external-anthropic", aws, req.api_key, false)?
        {
            request = request.header(k.as_str(), v.as_str());
        }
        let mut resp = request
            .send(&body_bytes[..])
            .map_err(|e| crate::errcode::ec(crate::errcode::E_REFINE_HTTP, format!("Claude Platform on AWS: {e}")))?;
        let v: serde_json::Value = resp.body_mut().read_json().map_err(|e| e.to_string())?;

        let out = anthropic_text(&v);
        let body = extract_tagged_body(&out);
        if body.is_empty() {
            // 他エンジンと同じ安定コード(E_REFINE_EMPTY_RESULT)へ統一（#462）。
            return Err(crate::errcode::ec(
                crate::errcode::E_REFINE_EMPTY_RESULT,
                format!("Claude Platform on AWS: {v}"),
            ));
        }
        Ok(body)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 実行時のモデル解決（「最新」はビルド時固定でなく実行時に各社のモデル一覧APIから選ぶ）。
// 各社APIに「ティア」表記は無いため、ミドルレンジは発見的に選択する。失敗時は呼び出し側が
// 既定モデルへフォールバックする（lib.rs::default_model_for）。
// ─────────────────────────────────────────────────────────────────────────────

/// プロバイダ名で「最新ミドルレンジモデル」を実行時に解決する。
pub fn resolve_latest_model(provider: &str, api_key: &str) -> Result<String, String> {
    match RefineProvider::parse(provider) {
        RefineProvider::Anthropic => latest_anthropic(api_key),
        RefineProvider::OpenAi => latest_openai(api_key),
        RefineProvider::Ollama => latest_ollama(),
        // Gemini・AWS系(Bedrock/ClaudePlatform)・未知は Gemini の最新解決へフォールバック(従来同値)。
        RefineProvider::Gemini | RefineProvider::Bedrock | RefineProvider::ClaudePlatformAws => {
            latest_gemini(api_key)
        }
    }
}

/// Ollama: GET {base}/api/tags からインストール済みモデルの先頭を返す（鍵不要）。
/// 取得不可（未起動/未取得）なら呼び出し側が既定モデルにフォールバックする。
fn latest_ollama() -> Result<String, String> {
    let url = format!("{}/api/tags", ollama_base());
    let mut resp = ureq::get(&url)
        .call()
        .map_err(|e| crate::errcode::ec(crate::errcode::E_REFINE_MODELS_HTTP, format!("Ollama: {e}")))?;
    let v: serde_json::Value = resp.body_mut().read_json().map_err(|e| e.to_string())?;
    v.get("models")
        .and_then(|m| m.as_array())
        .and_then(|a| a.first())
        .and_then(|m| m.get("name"))
        .and_then(|n| n.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| crate::errcode::E_REFINE_NO_OLLAMA_MODEL.into())
}

/// Anthropic: GET /v1/models（新しい順）から最新の Sonnet(=ミドルレンジ) を選ぶ。
fn latest_anthropic(api_key: &str) -> Result<String, String> {
    if api_key.trim().is_empty() {
        return Err(crate::errcode::E_REFINE_ANTHROPIC_NO_KEY.into());
    }
    let url = format!(
        "{}/v1/models?limit=1000",
        crate::api_base("https://api.anthropic.com", "QS_TEST_ANTHROPIC_BASE")
    );
    let mut resp = ureq::get(&url)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .call()
        .map_err(|e| crate::errcode::ec(crate::errcode::E_REFINE_MODELS_HTTP, format!("Anthropic: {e}")))?;
    let v: serde_json::Value = resp.body_mut().read_json().map_err(|e| e.to_string())?;
    let data = v
        .get("data")
        .and_then(|d| d.as_array())
        .ok_or_else(|| crate::errcode::ec(crate::errcode::E_REFINE_MODELS_PARSE, "Anthropic"))?;
    // data[] は作成日の新しい順。最初に見つかった sonnet が最新。
    for m in data {
        if let Some(id) = m.get("id").and_then(|i| i.as_str()) {
            if id.contains("sonnet") {
                return Ok(id.to_string());
            }
        }
    }
    Err(crate::errcode::E_REFINE_NO_SONNET.into())
}

/// OpenAI: GET /v1/models からミドルレンジ汎用チャットの最新を発見的に選ぶ。
fn latest_openai(api_key: &str) -> Result<String, String> {
    if api_key.trim().is_empty() {
        return Err(crate::errcode::E_REFINE_OPENAI_NO_KEY.into());
    }
    let url = format!(
        "{}/v1/models",
        crate::api_base("https://api.openai.com", "QS_TEST_OPENAI_BASE")
    );
    let mut resp = ureq::get(&url)
        .header("Authorization", &format!("Bearer {api_key}"))
        .call()
        .map_err(|e| crate::errcode::ec(crate::errcode::E_REFINE_MODELS_HTTP, format!("OpenAI: {e}")))?;
    let v: serde_json::Value = resp.body_mut().read_json().map_err(|e| e.to_string())?;
    let data = v
        .get("data")
        .and_then(|d| d.as_array())
        .ok_or_else(|| crate::errcode::ec(crate::errcode::E_REFINE_MODELS_PARSE, "OpenAI"))?;
    let ids: Vec<&str> = data
        .iter()
        .filter_map(|m| m.get("id").and_then(|i| i.as_str()))
        .collect();
    pick_openai_mid(&ids).ok_or_else(|| crate::errcode::E_REFINE_NO_OPENAI_MID.into())
}

/// OpenAIのモデルID群からミドルレンジ汎用チャットの最新を選ぶ（純粋関数・テスト対象）。
/// ローリングエイリアス(常に最新を指す)を最優先し、無ければスナップショット最新を選ぶ。
fn pick_openai_mid(ids: &[&str]) -> Option<String> {
    let is_excluded = |id: &str| {
        [
            "mini",
            "nano",
            "audio",
            "realtime",
            "search",
            "transcribe",
            "tts",
            "image",
            "instruct",
            "vision",
            "preview",
        ]
        .iter()
        .any(|w| id.contains(w))
    };
    // 1) ローリングエイリアス（常に最新を指す）を最優先: gpt-4.1 > gpt-4o。
    for alias in ["gpt-4.1", "gpt-4o"] {
        if ids.iter().any(|&i| i == alias) {
            return Some(alias.to_string());
        }
    }
    // 2) スナップショットから最新(辞書順最大=日付が新しい)を選ぶ。
    let mut cands: Vec<&str> = ids
        .iter()
        .copied()
        .filter(|i| (i.starts_with("gpt-4.1") || i.starts_with("gpt-4o")) && !is_excluded(i))
        .collect();
    cands.sort_unstable();
    cands.last().map(|s| s.to_string())
}

/// Gemini: GET /v1beta/models から generateContent 対応の flash(=ミドルレンジ) 最新を選ぶ。
/// `*flash-latest` ローリングエイリアスがあれば最優先、無ければ最大バージョンの `gemini-<ver>-flash`。
fn latest_gemini(api_key: &str) -> Result<String, String> {
    if api_key.trim().is_empty() {
        return Err(crate::errcode::E_REFINE_GEMINI_NO_KEY.into());
    }
    let url = format!(
        "{}/v1beta/models?pageSize=1000&key={api_key}",
        crate::api_base("https://generativelanguage.googleapis.com", "QS_TEST_GEMINI_BASE")
    );
    let mut resp = ureq::get(&url)
        .call()
        .map_err(|e| crate::errcode::ec(crate::errcode::E_REFINE_MODELS_HTTP, format!("Gemini: {e}")))?;
    let v: serde_json::Value = resp.body_mut().read_json().map_err(|e| e.to_string())?;
    let models = v
        .get("models")
        .and_then(|m| m.as_array())
        .ok_or_else(|| crate::errcode::ec(crate::errcode::E_REFINE_MODELS_PARSE, "Gemini"))?;

    let mut alias: Option<String> = None;
    let mut best: Option<(f64, String)> = None;
    for m in models {
        let name = m.get("name").and_then(|n| n.as_str()).unwrap_or("");
        let id = name.strip_prefix("models/").unwrap_or(name);
        let supports = m
            .get("supportedGenerationMethods")
            .and_then(|s| s.as_array())
            .map(|a| a.iter().any(|x| x.as_str() == Some("generateContent")))
            .unwrap_or(false);
        if !supports {
            continue;
        }
        if id.ends_with("flash-latest") {
            alias = Some(id.to_string());
        }
        // gemini-<ver>-flash（末尾が -flash。-lite/-8b 等の追加サフィックスは除外）。
        if let Some(ver) = id.strip_prefix("gemini-").and_then(|s| s.strip_suffix("-flash")) {
            if let Ok(num) = ver.parse::<f64>() {
                if best.as_ref().map_or(true, |(b, _)| num > *b) {
                    best = Some((num, id.to_string()));
                }
            }
        }
    }
    if let Some(a) = alias {
        return Ok(a);
    }
    best.map(|(_, id)| id)
        .ok_or_else(|| crate::errcode::E_REFINE_NO_FLASH.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refine_provider_parse_aliases_and_defaults() {
        // 別名解釈の単一ソース(#392)。既存の engine_for/default_model/is_aws の分岐を保存。
        assert_eq!(RefineProvider::parse("claude"), RefineProvider::Anthropic);
        assert_eq!(RefineProvider::parse("gpt"), RefineProvider::OpenAi);
        assert_eq!(RefineProvider::parse("local"), RefineProvider::Ollama);
        assert_eq!(RefineProvider::parse("aws-bedrock"), RefineProvider::Bedrock);
        assert_eq!(
            RefineProvider::parse("anthropic-aws"),
            RefineProvider::ClaudePlatformAws
        );
        // 空/未知は既定 Gemini。
        assert_eq!(RefineProvider::parse(""), RefineProvider::Gemini);
        assert_eq!(RefineProvider::parse("unknown"), RefineProvider::Gemini);
        // is_aws は AWS系のみ true。
        assert!(RefineProvider::parse("bedrock").is_aws());
        assert!(RefineProvider::parse("claude-platform-aws").is_aws());
        assert!(!RefineProvider::parse("anthropic").is_aws());
        assert!(!RefineProvider::parse("gemini").is_aws());
        // 既定モデル。
        assert_eq!(RefineProvider::Bedrock.default_model(), "anthropic.claude-sonnet-4-6");
        assert_eq!(RefineProvider::Gemini.default_model(), "gemini-flash-latest");
    }

    #[test]
    fn title_from_response_takes_first_line_and_strips_decorations() {
        // 見出し記号・かぎ括弧・引用符の装飾を剥がし、1行のタイトルにする。
        assert_eq!(title_from_response("# 今日の振り返り"), "今日の振り返り");
        assert_eq!(title_from_response("「仕事の不安」\n補足行"), "仕事の不安");
        assert_eq!(title_from_response("\n\n  \"Plan for tomorrow\"  \n"), "Plan for tomorrow");
        assert_eq!(title_from_response("   "), "");
    }

    #[test]
    fn generate_title_rejects_empty_input_without_api_call() {
        // 空入力はAPIを呼ばずに弾く(refine と同じ規律)。
        assert!(generate_title("gemini", "k", "m", "  ", None, None).is_err());
    }

    #[test]
    fn style_parse_maps_known_and_defaults_unknown() {
        assert_eq!(RefineStyle::parse("verbatim"), RefineStyle::Verbatim);
        assert_eq!(RefineStyle::parse("逐語"), RefineStyle::Verbatim);
        assert_eq!(RefineStyle::parse("Summary"), RefineStyle::Summary);
        assert_eq!(RefineStyle::parse("brainstorm"), RefineStyle::Brainstorm);
        // 未知・空は既定(Structured)。
        assert_eq!(RefineStyle::parse(""), RefineStyle::Structured);
        assert_eq!(RefineStyle::parse("zzz"), RefineStyle::Structured);
        assert_eq!(RefineStyle::parse("structured"), RefineStyle::Structured);
    }

    #[test]
    fn user_prompt_contains_transcript_and_no_fabrication_rule() {
        // どのスタイルでも本文と「捏造しない」制約を必ず含む(コア価値: ニュアンス保持/誠実)。
        for style in [
            RefineStyle::Structured,
            RefineStyle::Verbatim,
            RefineStyle::Summary,
            RefineStyle::Brainstorm,
        ] {
            let p = style.user_prompt("テスト本文XYZ");
            assert!(p.contains("テスト本文XYZ"), "{:?} は本文を含むべき", style);
            assert!(p.contains("捏造"), "{:?} は捏造禁止を含むべき", style);
        }
    }

    #[test]
    fn user_prompt_is_style_specific() {
        // スタイル固有の指示語が入っている(=スタイルが効いている)。
        assert!(RefineStyle::Structured.user_prompt("x").contains("見出し"));
        assert!(RefineStyle::Verbatim.user_prompt("x").contains("逐語"));
        assert!(RefineStyle::Summary.user_prompt("x").contains("要約"));
        assert!(RefineStyle::Brainstorm.user_prompt("x").contains("ブレスト"));
    }

    fn req_with<'a>(custom: Option<&'a str>, transcript: &'a str) -> RefineRequest<'a> {
        RefineRequest {
            style: RefineStyle::Structured,
            api_key: "",
            model: "",
            transcript,
            aws: None,
            custom_instruction: custom,
            output_lang: None,
            base_url: None,
        }
    }

    #[test]
    fn openai_base_url_prefers_custom_then_defaults() {
        // 非空指定はそのまま(末尾スラッシュ除去)。空/未指定は公式エンドポイントへフォールバック。
        assert_eq!(
            openai_base_url(Some("http://localhost:4000")),
            "http://localhost:4000"
        );
        assert_eq!(
            openai_base_url(Some("https://gw.example.com/")),
            "https://gw.example.com"
        );
        // 空白のみ/未指定は同じ既定へフォールバックする(env非依存で等価性を確認)。
        let fallback = openai_base_url(None);
        assert_eq!(openai_base_url(Some("   ")), fallback);
        assert!(!fallback.is_empty());
    }

    #[test]
    fn custom_instruction_overrides_style_but_keeps_core_rules() {
        // カスタム指示は本文に反映され、かつ「捏造禁止」等のコア規律は外れない(S3.3)。
        let p = req_with(Some("- 箇条書きせず一段落で書く"), "本文ABC").user_prompt();
        assert!(p.contains("- 箇条書きせず一段落で書く"), "カスタム指示を含む");
        assert!(p.contains("本文ABC"), "本文を含む");
        assert!(p.contains("捏造"), "コア規律(捏造禁止)は不変");
        // 既定スタイル(Structured)の指示語には引きずられない。
        assert!(!p.contains("ひとことまとめ"), "既定指示は使われない");
    }

    #[test]
    fn refine_rejects_empty_input_without_calling_provider() {
        // 空・空白のみは HTTP を呼ばず即エラー（無駄コスト防止 / #18）。
        let err = refine("gemini", "k", "m", "structured", "   \n", None, None, None, None).unwrap_err();
        assert_eq!(err, crate::errcode::E_REFINE_EMPTY_INPUT, "空入力は安定コードを返す: {err}");
    }

    #[test]
    fn output_lang_appends_language_directive_and_keeps_tags() {
        // 出力言語指定時: 指定言語(英語名)の出力指示が追記され、タグ境界の規律は保たれる(#453)。
        let mut r = req_with(None, "x");
        r.output_lang = Some("Vietnamese");
        let sp = r.system_prompt();
        assert!(sp.contains("Vietnamese"), "指定言語名を含む");
        assert!(sp.contains("<journal>"), "タグ境界の規律は不変");
        // 未指定/空は追記されない(原語のまま)。
        assert!(!req_with(None, "x").system_prompt().contains("Vietnamese"));
        let mut r_empty = req_with(None, "x");
        r_empty.output_lang = Some("  ");
        assert!(!r_empty.system_prompt().contains("IMPORTANT: Write the entire"));
    }

    #[test]
    fn empty_custom_instruction_falls_back_to_style() {
        // 空白のみのカスタムは未指定扱い → スタイル既定の指示を使う。
        let p = req_with(Some("   "), "x").user_prompt();
        assert!(p.contains("見出し"), "空カスタムは既定(Structured)へフォールバック");
        let p_none = req_with(None, "x").user_prompt();
        assert!(p_none.contains("見出し"));
    }

    #[test]
    fn custom_system_prompt_is_unchanged() {
        // システム指示(journalタグ境界・捏造禁止)はカスタムでも共通で不変。
        let p = req_with(Some("好きに書いて"), "x");
        assert!(p.system_prompt().contains("<journal>"));
    }

    #[test]
    fn openai_prefers_rolling_alias() {
        let ids = ["gpt-4o-2024-08-06", "gpt-4o", "gpt-4o-mini"];
        assert_eq!(pick_openai_mid(&ids).as_deref(), Some("gpt-4o"));
    }

    #[test]
    fn openai_prefers_41_over_4o() {
        let ids = ["gpt-4o", "gpt-4.1", "gpt-3.5-turbo"];
        assert_eq!(pick_openai_mid(&ids).as_deref(), Some("gpt-4.1"));
    }

    #[test]
    fn openai_excludes_mini_and_nano() {
        let ids = ["gpt-4o-mini", "gpt-4.1-nano", "gpt-4o-realtime-preview"];
        assert_eq!(pick_openai_mid(&ids), None);
    }

    #[test]
    fn openai_picks_latest_snapshot_when_no_alias() {
        let ids = ["gpt-4o-2024-05-13", "gpt-4o-2024-08-06"];
        assert_eq!(pick_openai_mid(&ids).as_deref(), Some("gpt-4o-2024-08-06"));
    }

    #[test]
    fn extract_strips_journal_tags_and_surrounding_noise() {
        // 前置き＋タグ本文＋後書き の3層から、タグ内本文だけを取り出す。
        let raw = "はい、以下が整形結果です:\n<journal>## 見出し\n本文の中身</journal>\n何か補足";
        assert_eq!(extract_tagged_body(raw), "## 見出し\n本文の中身");
    }

    #[test]
    fn extract_preserves_markdown_inside_tags_unescaped() {
        // markdown(改行/引用符/コードフェンス)を一切壊さない(JSON封入を捨てた理由)。
        let raw = "<journal>- 「迷い」も残す\n```\ncode\n```</journal>";
        assert_eq!(extract_tagged_body(raw), "- 「迷い」も残す\n```\ncode\n```");
    }

    #[test]
    fn extract_rescues_when_close_tag_missing() {
        // 終了タグ欠落(truncation等)でも開始タグ以降を救済する。
        let raw = "<journal>途中で切れた本文";
        assert_eq!(extract_tagged_body(raw), "途中で切れた本文");
    }

    #[test]
    fn extract_strips_leading_preamble_when_no_tags() {
        // タグ欠落時は定型前置きを1行だけ保守的に除去する。
        let raw = "はい、整形しました。\n本文だけ残る";
        assert_eq!(extract_tagged_body(raw), "本文だけ残る");
    }

    #[test]
    fn extract_keeps_body_without_tags_or_preamble() {
        // タグも前置きも無ければ本文をそのまま返す(本文を失わない)。
        let raw = "ふつうの本文\n2行目";
        assert_eq!(extract_tagged_body(raw), "ふつうの本文\n2行目");
    }

    #[test]
    fn extract_takes_outermost_span_with_inner_tag_like_text() {
        // 本文中にタグ様の語があっても最外(最初の開始〜最後の終了)で囲む。
        let raw = "<journal>A <journal> B</journal>";
        assert_eq!(extract_tagged_body(raw), "A <journal> B");
    }

    // ─── ここから HTTP 経路のテスト（ローカルテストサーバ / 監査項目12） ───
    use crate::testhttp::{serve, set_envs, Route};

    /// 標準の refine 呼び出し（スタイル固定・AWSなし）。
    fn call(provider: &str, key: &str) -> Result<String, String> {
        refine(provider, key, "test-model", "structured", "本文X", None, None, None, None)
    }

    #[test]
    fn gemini_engine_http_paths() {
        // 鍵なしは即エラー（HTTPを呼ばない）。
        assert_eq!(call("gemini", " ").unwrap_err(), crate::errcode::E_REFINE_GEMINI_NO_KEY);
        // 成功: candidates[].content.parts[].text を連結しタグ内を抽出。
        let (base, seen) = serve(vec![Route::json(
            ":generateContent",
            200,
            r#"{"candidates":[{"content":{"parts":[{"text":"<journal>G本文</journal>"}]}}]}"#,
        )]);
        {
            let _g = set_envs(&[("QS_TEST_GEMINI_BASE", &base)]);
            assert_eq!(call("gemini", "k").unwrap(), "G本文");
        }
        assert!(
            seen.lock().unwrap()[0].contains("generateContent"),
            "generateContent エンドポイントへPOSTする"
        );
        // 空応答は安定コード E_REFINE_EMPTY_RESULT。
        let (base2, _) = serve(vec![Route::json(":generateContent", 200, r#"{"candidates":[]}"#)]);
        {
            let _g = set_envs(&[("QS_TEST_GEMINI_BASE", &base2)]);
            let err = call("gemini", "k").unwrap_err();
            assert!(err.starts_with(crate::errcode::E_REFINE_EMPTY_RESULT), "{err}");
        }
        // 非2xxは E_REFINE_HTTP。
        let (base3, _) = serve(vec![Route::json(":generateContent", 500, "{}")]);
        {
            let _g = set_envs(&[("QS_TEST_GEMINI_BASE", &base3)]);
            let err = call("gemini", "k").unwrap_err();
            assert!(err.starts_with(crate::errcode::E_REFINE_HTTP), "{err}");
        }
    }

    #[test]
    fn anthropic_engine_http_paths() {
        assert_eq!(call("anthropic", "").unwrap_err(), crate::errcode::E_REFINE_ANTHROPIC_NO_KEY);
        // 成功: content[] の type=text のみ連結（非textブロックは無視）。
        let (base, seen) = serve(vec![Route::json(
            "/v1/messages",
            200,
            r#"{"content":[{"type":"text","text":"<journal>A本文</journal>"},{"type":"tool_use","id":"x"}]}"#,
        )]);
        {
            let _g = set_envs(&[("QS_TEST_ANTHROPIC_BASE", &base)]);
            assert_eq!(call("claude", "sk").unwrap(), "A本文");
        }
        let req = seen.lock().unwrap()[0].clone();
        assert!(req.contains("x-api-key: sk"), "x-api-key ヘッダで認証: {req}");
        assert!(req.contains("anthropic-version"), "versionヘッダ必須");
        // 空応答。
        let (base2, _) = serve(vec![Route::json("/v1/messages", 200, r#"{"content":[]}"#)]);
        {
            let _g = set_envs(&[("QS_TEST_ANTHROPIC_BASE", &base2)]);
            let err = call("anthropic", "sk").unwrap_err();
            assert!(err.starts_with(crate::errcode::E_REFINE_EMPTY_RESULT), "{err}");
        }
    }

    #[test]
    fn anthropic_latest_model_resolution() {
        assert_eq!(
            resolve_latest_model("anthropic", "").unwrap_err(),
            crate::errcode::E_REFINE_ANTHROPIC_NO_KEY
        );
        // 新しい順の data[] から最初の sonnet を選ぶ。
        let (base, _) = serve(vec![Route::json(
            "/v1/models",
            200,
            r#"{"data":[{"id":"claude-opus-9"},{"id":"claude-sonnet-9"},{"id":"claude-sonnet-8"}]}"#,
        )]);
        {
            let _g = set_envs(&[("QS_TEST_ANTHROPIC_BASE", &base)]);
            assert_eq!(resolve_latest_model("anthropic", "k").unwrap(), "claude-sonnet-9");
        }
        // sonnet 不在は安定コード。
        let (base2, _) = serve(vec![Route::json("/v1/models", 200, r#"{"data":[{"id":"claude-opus-9"}]}"#)]);
        {
            let _g = set_envs(&[("QS_TEST_ANTHROPIC_BASE", &base2)]);
            assert_eq!(
                resolve_latest_model("anthropic", "k").unwrap_err(),
                crate::errcode::E_REFINE_NO_SONNET
            );
        }
        // data 欠落は解析エラー。
        let (base3, _) = serve(vec![Route::json("/v1/models", 200, "{}")]);
        {
            let _g = set_envs(&[("QS_TEST_ANTHROPIC_BASE", &base3)]);
            let err = resolve_latest_model("anthropic", "k").unwrap_err();
            assert!(err.starts_with(crate::errcode::E_REFINE_MODELS_PARSE), "{err}");
        }
    }

    #[test]
    fn openai_engine_and_latest_http_paths() {
        assert_eq!(call("openai", "").unwrap_err(), crate::errcode::E_REFINE_OPENAI_NO_KEY);
        assert_eq!(
            resolve_latest_model("openai", "").unwrap_err(),
            crate::errcode::E_REFINE_OPENAI_NO_KEY
        );
        // 整形成功（Bearer 認証）。
        let (base, seen) = serve(vec![
            Route::json(
                "/v1/chat/completions",
                200,
                r#"{"choices":[{"message":{"content":"<journal>O本文</journal>"}}]}"#,
            ),
            Route::json("/v1/models", 200, r#"{"data":[{"id":"gpt-4.1"},{"id":"gpt-4o"}]}"#),
        ]);
        {
            let _g = set_envs(&[("QS_TEST_OPENAI_BASE", &base)]);
            assert_eq!(call("gpt", "sk").unwrap(), "O本文");
            // 最新解決はローリングエイリアス gpt-4.1 を優先。
            assert_eq!(resolve_latest_model("openai", "k").unwrap(), "gpt-4.1");
        }
        assert!(seen.lock().unwrap()[0].to_ascii_lowercase().contains("authorization: bearer sk"));
        // content 欠落は空結果エラー。
        let (base2, _) = serve(vec![Route::json("/v1/chat/completions", 200, r#"{"choices":[]}"#)]);
        {
            let _g = set_envs(&[("QS_TEST_OPENAI_BASE", &base2)]);
            let err = call("openai", "sk").unwrap_err();
            assert!(err.starts_with(crate::errcode::E_REFINE_EMPTY_RESULT), "{err}");
        }
    }

    #[test]
    fn ollama_engine_and_latest_http_paths() {
        // 成功: OLLAMA_HOST(スキーム無し表記) → http:// を前置して解決する。
        let (base, seen) = serve(vec![
            Route::json("/api/chat", 200, r#"{"message":{"content":"<journal>L本文</journal>"}}"#),
            Route::json("/api/tags", 200, r#"{"models":[{"name":"llama3.2:latest"}]}"#),
        ]);
        let hostport = base.strip_prefix("http://").unwrap().to_string();
        {
            let _g = set_envs(&[("OLLAMA_HOST", &hostport)]);
            // model 空は既定 llama3.1 を送る。
            let out = refine("ollama", "", "", "structured", "本文", None, None, None, None).unwrap();
            assert_eq!(out, "L本文");
            assert!(seen.lock().unwrap()[0].contains("llama3.1"), "既定モデルを送信");
            // インストール済み一覧の先頭を最新として返す。
            assert_eq!(resolve_latest_model("local", "").unwrap(), "llama3.2:latest");
        }
        // モデル未取得（models空）は安定コード。
        let (base2, _) = serve(vec![Route::json("/api/tags", 200, r#"{"models":[]}"#)]);
        {
            let _g = set_envs(&[("OLLAMA_HOST", &base2)]);
            assert_eq!(
                resolve_latest_model("ollama", "").unwrap_err(),
                crate::errcode::E_REFINE_NO_OLLAMA_MODEL
            );
        }
        // 接続不能（未起動）は E_REFINE_OLLAMA_CONN / E_REFINE_MODELS_HTTP。
        {
            let _g = set_envs(&[("OLLAMA_HOST", "http://127.0.0.1:9")]);
            let err = refine("ollama", "", "m", "structured", "x", None, None, None, None).unwrap_err();
            assert!(err.starts_with(crate::errcode::E_REFINE_OLLAMA_CONN), "{err}");
            let err = resolve_latest_model("ollama", "").unwrap_err();
            assert!(err.starts_with(crate::errcode::E_REFINE_MODELS_HTTP), "{err}");
        }
        // OLLAMA_HOST 空は既定 localhost:11434。
        {
            let _g = set_envs(&[("OLLAMA_HOST", "  ")]);
            assert_eq!(ollama_base(), "http://localhost:11434");
        }
    }

    #[test]
    fn gemini_latest_model_resolution() {
        assert_eq!(
            resolve_latest_model("gemini", "").unwrap_err(),
            crate::errcode::E_REFINE_GEMINI_NO_KEY
        );
        // flash-latest ローリングエイリアスを最優先（generateContent 非対応は無視）。
        let (base, _) = serve(vec![Route::json(
            "/v1beta/models",
            200,
            r#"{"models":[
                {"name":"models/gemini-2.5-flash","supportedGenerationMethods":["generateContent"]},
                {"name":"models/gemini-flash-latest","supportedGenerationMethods":["generateContent"]},
                {"name":"models/embedding-001","supportedGenerationMethods":["embedContent"]}
            ]}"#,
        )]);
        {
            let _g = set_envs(&[("QS_TEST_GEMINI_BASE", &base)]);
            assert_eq!(resolve_latest_model("gemini", "k").unwrap(), "gemini-flash-latest");
            // 未知プロバイダ/AWS系も gemini の解決へフォールバックする。
            assert_eq!(resolve_latest_model("bedrock", "k").unwrap(), "gemini-flash-latest");
        }
        // エイリアス不在は gemini-<ver>-flash の最大バージョン。
        let (base2, _) = serve(vec![Route::json(
            "/v1beta/models",
            200,
            r#"{"models":[
                {"name":"models/gemini-2.0-flash","supportedGenerationMethods":["generateContent"]},
                {"name":"models/gemini-2.5-flash","supportedGenerationMethods":["generateContent"]},
                {"name":"models/gemini-2.5-flash-lite","supportedGenerationMethods":["generateContent"]}
            ]}"#,
        )]);
        {
            let _g = set_envs(&[("QS_TEST_GEMINI_BASE", &base2)]);
            assert_eq!(resolve_latest_model("gemini", "k").unwrap(), "gemini-2.5-flash");
        }
        // flash 候補ゼロは安定コード。
        let (base3, _) = serve(vec![Route::json("/v1beta/models", 200, r#"{"models":[]}"#)]);
        {
            let _g = set_envs(&[("QS_TEST_GEMINI_BASE", &base3)]);
            assert_eq!(
                resolve_latest_model("gemini", "k").unwrap_err(),
                crate::errcode::E_REFINE_NO_FLASH
            );
        }
        // models 欠落は解析エラー。
        let (base4, _) = serve(vec![Route::json("/v1beta/models", 200, "{}")]);
        {
            let _g = set_envs(&[("QS_TEST_GEMINI_BASE", &base4)]);
            let err = resolve_latest_model("gemini", "k").unwrap_err();
            assert!(err.starts_with(crate::errcode::E_REFINE_MODELS_PARSE), "{err}");
        }
    }

    fn aws_cfg(auth: AwsAuth) -> AwsConfig {
        AwsConfig {
            region: "us-east-1".into(),
            workspace_id: "ws-1".into(),
            auth,
        }
    }

    #[test]
    fn aws_auth_headers_apikey_and_sigv4() {
        let cfg = aws_cfg(AwsAuth::ApiKey);
        // APIキー空はエラー。
        assert_eq!(
            aws_auth_headers("http://x", b"{}", "bedrock", &cfg, " ", true).unwrap_err(),
            crate::errcode::E_REFINE_NO_KEY
        );
        // bearer=true は Authorization、false は x-api-key。
        let h = aws_auth_headers("http://x", b"{}", "bedrock", &cfg, "k1", true).unwrap();
        assert_eq!(h, vec![("Authorization".to_string(), "Bearer k1".to_string())]);
        let h = aws_auth_headers("http://x", b"{}", "aws-external-anthropic", &cfg, "k1", false).unwrap();
        assert_eq!(h, vec![("x-api-key".to_string(), "k1".to_string())]);
        // SigV4: 鍵欠落はエラー、揃えば authorization/x-amz-date が付く。
        let empty = aws_cfg(AwsAuth::SigV4 {
            access_key: "".into(),
            secret_key: "".into(),
            session_token: None,
        });
        assert_eq!(
            aws_auth_headers("http://x", b"{}", "bedrock", &empty, "", true).unwrap_err(),
            crate::errcode::E_REFINE_AWS_NO_KEYS
        );
        let sig = aws_cfg(AwsAuth::SigV4 {
            access_key: "AKIA123".into(),
            secret_key: "secret".into(),
            session_token: Some("tok".into()),
        });
        let h = aws_auth_headers("http://x/model/m/invoke", b"{}", "bedrock", &sig, "", true).unwrap();
        let names: Vec<&str> = h.iter().map(|(k, _)| k.as_str()).collect();
        assert!(names.iter().any(|n| n.eq_ignore_ascii_case("authorization")), "{names:?}");
        assert!(names.iter().any(|n| n.eq_ignore_ascii_case("x-amz-date")), "{names:?}");
    }

    #[test]
    fn bedrock_engine_http_paths() {
        // AwsConfig 無しは設定エラー。
        let err = call("bedrock", "k").unwrap_err();
        assert!(err.starts_with(crate::errcode::E_REFINE_AWS_CONFIG), "{err}");
        // region 空はエラー。
        let mut cfg = aws_cfg(AwsAuth::ApiKey);
        cfg.region = " ".into();
        let err = refine("bedrock", "k", "", "structured", "x", Some(cfg), None, None, None).unwrap_err();
        assert_eq!(err, crate::errcode::E_REFINE_AWS_NO_REGION);
        // 成功（APIキー=Bearer / model 空は既定IDへ）。
        let (base, seen) = serve(vec![Route::json(
            "/invoke",
            200,
            r#"{"content":[{"type":"text","text":"<journal>B本文</journal>"}]}"#,
        )]);
        {
            let _g = set_envs(&[("QS_TEST_BEDROCK_BASE", &base)]);
            let out = refine(
                "bedrock", "bk", "", "structured", "x",
                Some(aws_cfg(AwsAuth::ApiKey)), None, None, None,
            )
            .unwrap();
            assert_eq!(out, "B本文");
            let req = seen.lock().unwrap()[0].clone();
            assert!(req.contains("/model/anthropic.claude-sonnet-4-6/invoke"), "既定モデルID: {req}");
            assert!(req.to_ascii_lowercase().contains("authorization: bearer bk"), "{req}");
            // SigV4 でも送信できる（署名ヘッダ付与）。
            let sig = aws_cfg(AwsAuth::SigV4 {
                access_key: "AKIA123".into(),
                secret_key: "secret".into(),
                session_token: None,
            });
            let out = refine("aws-bedrock", "", "m1", "structured", "x", Some(sig), None, None, None).unwrap();
            assert_eq!(out, "B本文");
        }
        // 空応答。
        let (base2, _) = serve(vec![Route::json("/invoke", 200, r#"{"content":[]}"#)]);
        {
            let _g = set_envs(&[("QS_TEST_BEDROCK_BASE", &base2)]);
            let err = refine(
                "bedrock", "bk", "", "structured", "x",
                Some(aws_cfg(AwsAuth::ApiKey)), None, None, None,
            )
            .unwrap_err();
            assert!(err.starts_with(crate::errcode::E_REFINE_EMPTY_RESULT), "{err}");
        }
    }

    #[test]
    fn claude_platform_aws_engine_http_paths() {
        // AwsConfig 無しは設定エラー。
        let err = call("claude-aws", "k").unwrap_err();
        assert!(err.starts_with(crate::errcode::E_REFINE_AWS_CONFIG), "{err}");
        // region 空はエラー。
        let mut cfg = aws_cfg(AwsAuth::ApiKey);
        cfg.region = "".into();
        let err = refine("claude-aws", "k", "m", "structured", "x", Some(cfg), None, None, None).unwrap_err();
        assert_eq!(err, crate::errcode::E_REFINE_AWS_NO_REGION);
        // 成功（x-api-key + anthropic-workspace-id）。
        let (base, seen) = serve(vec![Route::json(
            "/v1/messages",
            200,
            r#"{"content":[{"type":"text","text":"<journal>C本文</journal>"}]}"#,
        )]);
        {
            let _g = set_envs(&[("QS_TEST_CLAUDE_AWS_BASE", &base)]);
            let out = refine(
                "claude-platform-aws", "pk", "m", "structured", "x",
                Some(aws_cfg(AwsAuth::ApiKey)), None, None, None,
            )
            .unwrap();
            assert_eq!(out, "C本文");
            let req = seen.lock().unwrap()[0].clone();
            assert!(req.contains("anthropic-workspace-id: ws-1"), "{req}");
            assert!(req.contains("x-api-key: pk"), "{req}");
        }
        // 空応答は他エンジンと同じ安定コード（#462）。
        let (base2, _) = serve(vec![Route::json("/v1/messages", 200, r#"{"content":[]}"#)]);
        {
            let _g = set_envs(&[("QS_TEST_CLAUDE_AWS_BASE", &base2)]);
            let err = refine(
                "anthropic-aws", "pk", "m", "structured", "x",
                Some(aws_cfg(AwsAuth::ApiKey)), None, None, None,
            )
            .unwrap_err();
            assert!(err.starts_with(crate::errcode::E_REFINE_EMPTY_RESULT), "{err}");
        }
    }
}
