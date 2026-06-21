// 整形の知性(FormattingEngine: BYO-Cloud / ADR-0005)。
// 文字起こしテキストを、ニュアンスを残しつつ思考整理・構造化する(コア価値=ADR-0004)。
//
// 設計(S3.1/S3.3):
// - 整形「スタイル」(構造化/逐語/要約/ブレスト)を `RefineStyle` で切り替え可能にする(S3.3)。
//   逐語⇄要約⇄ブレストを行き来できるのがコア価値(ADR-0004)。
// - プロバイダ(Gemini/Anthropic/OpenAI)は `FormattingEngine` trait の実装として差し替え可能にする
//   (S3.1: 戦略の差し替え=DIP 境界。将来のローカルLLM=Ollama 等もこの trait を実装すれば載る/S3.4)。
// - 鍵はフロントの設定から渡す(コードに埋め込まない/ADR-0005)。

use serde_json::json;

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

    /// スタイル別のユーザープロンプトを組み立てる(純粋関数・テスト対象)。
    fn user_prompt(&self, transcript: &str) -> String {
        let instruction = match self {
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
        };
        format!(
            "以下は音声の文字起こしです。話者本人の思考整理を助けてください。\n\
             {instruction}\n\
             - 事実を捏造せず、書かれていないことは足さない\n\n\
             ---\n{transcript}"
        )
    }
}

/// 整形リクエスト(スタイル・鍵・モデル・本文)。FormattingEngine に渡す。
pub struct RefineRequest<'a> {
    pub style: RefineStyle,
    pub api_key: &'a str,
    pub model: &'a str,
    pub transcript: &'a str,
}

/// 整形エンジンの抽象(S3.1: プロバイダ/戦略を差し替え可能にする DIP 境界)。
pub trait FormattingEngine {
    fn refine(&self, req: &RefineRequest) -> Result<String, String>;
}

/// プロバイダ名から整形エンジンを解決する。空/未知は Gemini にフォールバック。
pub fn engine_for(provider: &str) -> Box<dyn FormattingEngine> {
    match provider.trim().to_ascii_lowercase().as_str() {
        "anthropic" | "claude" => Box::new(AnthropicEngine),
        "openai" | "gpt" => Box::new(OpenAiEngine),
        "ollama" | "local" => Box::new(OllamaEngine),
        _ => Box::new(GeminiEngine),
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
pub fn refine(
    provider: &str,
    api_key: &str,
    model: &str,
    style: &str,
    transcript: &str,
) -> Result<String, String> {
    let req = RefineRequest {
        style: RefineStyle::parse(style),
        api_key,
        model,
        transcript,
    };
    engine_for(provider).refine(&req)
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

/// Gemini で整形する。
impl FormattingEngine for GeminiEngine {
    fn refine(&self, req: &RefineRequest) -> Result<String, String> {
        if req.api_key.trim().is_empty() {
            return Err("Gemini APIキーが未設定です（設定から入力してください）".into());
        }
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            req.model, req.api_key
        );
        // Gemini はシステムロールを使わないため、システム指示をユーザーテキストに前置する。
        let text = format!(
            "{}\n\n{}",
            req.style.system_prompt(),
            req.style.user_prompt(req.transcript)
        );
        let body = json!({ "contents": [{ "parts": [{ "text": text }] }] });

        let resp = ureq::post(&url)
            .set("Content-Type", "application/json")
            .send_json(body)
            .map_err(|e| format!("整形APIの呼び出しに失敗(Gemini): {e}"))?;
        let v: serde_json::Value = resp.into_json().map_err(|e| e.to_string())?;

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
            return Err(format!("整形結果が空でした（Gemini応答: {}）", v));
        }
        Ok(body)
    }
}

/// Anthropic(Claude) Messages API で整形する。
/// 必須ヘッダ: x-api-key, anthropic-version: 2023-06-01, content-type: application/json
impl FormattingEngine for AnthropicEngine {
    fn refine(&self, req: &RefineRequest) -> Result<String, String> {
        if req.api_key.trim().is_empty() {
            return Err("Anthropic APIキーが未設定です（設定から入力してください）".into());
        }
        let body = json!({
            "model": req.model,
            "max_tokens": 4096,
            "system": req.style.system_prompt(),
            "messages": [
                { "role": "user", "content": req.style.user_prompt(req.transcript) }
            ]
        });

        let resp = ureq::post("https://api.anthropic.com/v1/messages")
            .set("Content-Type", "application/json")
            .set("x-api-key", req.api_key)
            .set("anthropic-version", "2023-06-01")
            .send_json(body)
            .map_err(|e| format!("整形APIの呼び出しに失敗(Anthropic): {e}"))?;
        let v: serde_json::Value = resp.into_json().map_err(|e| e.to_string())?;

        // 応答は content[] のブロック配列。type=="text" の text を連結する。
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
        let body = extract_tagged_body(&out);
        if body.is_empty() {
            return Err(format!("整形結果が空でした（Anthropic応答: {}）", v));
        }
        Ok(body)
    }
}

/// OpenAI Chat Completions API で整形する。
/// 必須ヘッダ: Authorization: Bearer <key>, content-type: application/json
impl FormattingEngine for OpenAiEngine {
    fn refine(&self, req: &RefineRequest) -> Result<String, String> {
        if req.api_key.trim().is_empty() {
            return Err("OpenAI APIキーが未設定です（設定から入力してください）".into());
        }
        let body = json!({
            "model": req.model,
            "messages": [
                { "role": "system", "content": req.style.system_prompt() },
                { "role": "user", "content": req.style.user_prompt(req.transcript) }
            ]
        });

        let resp = ureq::post("https://api.openai.com/v1/chat/completions")
            .set("Content-Type", "application/json")
            .set("Authorization", &format!("Bearer {}", req.api_key))
            .send_json(body)
            .map_err(|e| format!("整形APIの呼び出しに失敗(OpenAI): {e}"))?;
        let v: serde_json::Value = resp.into_json().map_err(|e| e.to_string())?;

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
            return Err(format!("整形結果が空でした（OpenAI応答: {}）", v));
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
                { "role": "system", "content": req.style.system_prompt() },
                { "role": "user", "content": req.style.user_prompt(req.transcript) }
            ]
        });

        let url = format!("{}/api/chat", ollama_base());
        let resp = ureq::post(&url)
            .set("Content-Type", "application/json")
            .send_json(body)
            .map_err(|e| {
                format!("ローカルOllamaへの接続に失敗しました（ollamaが起動し、モデルが取得済みか確認してください）: {e}")
            })?;
        let v: serde_json::Value = resp.into_json().map_err(|e| e.to_string())?;

        let out = v
            .get("message")
            .and_then(|m| m.get("content"))
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();
        let body = extract_tagged_body(&out);
        if body.is_empty() {
            return Err(format!("整形結果が空でした（Ollama応答: {}）", v));
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
    match provider.trim().to_ascii_lowercase().as_str() {
        "anthropic" | "claude" => latest_anthropic(api_key),
        "openai" | "gpt" => latest_openai(api_key),
        "ollama" | "local" => latest_ollama(),
        _ => latest_gemini(api_key),
    }
}

/// Ollama: GET {base}/api/tags からインストール済みモデルの先頭を返す（鍵不要）。
/// 取得不可（未起動/未取得）なら呼び出し側が既定モデルにフォールバックする。
fn latest_ollama() -> Result<String, String> {
    let url = format!("{}/api/tags", ollama_base());
    let resp = ureq::get(&url)
        .call()
        .map_err(|e| format!("Ollamaモデル一覧の取得に失敗: {e}"))?;
    let v: serde_json::Value = resp.into_json().map_err(|e| e.to_string())?;
    v.get("models")
        .and_then(|m| m.as_array())
        .and_then(|a| a.first())
        .and_then(|m| m.get("name"))
        .and_then(|n| n.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "インストール済みOllamaモデルが見つかりません".into())
}

/// Anthropic: GET /v1/models（新しい順）から最新の Sonnet(=ミドルレンジ) を選ぶ。
fn latest_anthropic(api_key: &str) -> Result<String, String> {
    if api_key.trim().is_empty() {
        return Err("Anthropic APIキーが未設定です".into());
    }
    let resp = ureq::get("https://api.anthropic.com/v1/models?limit=1000")
        .set("x-api-key", api_key)
        .set("anthropic-version", "2023-06-01")
        .call()
        .map_err(|e| format!("モデル一覧の取得に失敗(Anthropic): {e}"))?;
    let v: serde_json::Value = resp.into_json().map_err(|e| e.to_string())?;
    let data = v
        .get("data")
        .and_then(|d| d.as_array())
        .ok_or("Anthropicのモデル一覧応答が不正です")?;
    // data[] は作成日の新しい順。最初に見つかった sonnet が最新。
    for m in data {
        if let Some(id) = m.get("id").and_then(|i| i.as_str()) {
            if id.contains("sonnet") {
                return Ok(id.to_string());
            }
        }
    }
    Err("Sonnet系モデルが見つかりませんでした".into())
}

/// OpenAI: GET /v1/models からミドルレンジ汎用チャットの最新を発見的に選ぶ。
fn latest_openai(api_key: &str) -> Result<String, String> {
    if api_key.trim().is_empty() {
        return Err("OpenAI APIキーが未設定です".into());
    }
    let resp = ureq::get("https://api.openai.com/v1/models")
        .set("Authorization", &format!("Bearer {api_key}"))
        .call()
        .map_err(|e| format!("モデル一覧の取得に失敗(OpenAI): {e}"))?;
    let v: serde_json::Value = resp.into_json().map_err(|e| e.to_string())?;
    let data = v
        .get("data")
        .and_then(|d| d.as_array())
        .ok_or("OpenAIのモデル一覧応答が不正です")?;
    let ids: Vec<&str> = data
        .iter()
        .filter_map(|m| m.get("id").and_then(|i| i.as_str()))
        .collect();
    pick_openai_mid(&ids).ok_or_else(|| "適切なミドルレンジ(gpt-4o/4.1系)が見つかりません".into())
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
        return Err("Gemini APIキーが未設定です".into());
    }
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models?pageSize=1000&key={api_key}"
    );
    let resp = ureq::get(&url)
        .call()
        .map_err(|e| format!("モデル一覧の取得に失敗(Gemini): {e}"))?;
    let v: serde_json::Value = resp.into_json().map_err(|e| e.to_string())?;
    let models = v
        .get("models")
        .and_then(|m| m.as_array())
        .ok_or("Geminiのモデル一覧応答が不正です")?;

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
        .ok_or_else(|| "flash系モデルが見つかりませんでした".into())
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
