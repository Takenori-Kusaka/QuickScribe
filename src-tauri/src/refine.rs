// 整形の知性(FormattingEngine: BYO-Cloud / ADR-0005)。
// 文字起こしテキストを、ニュアンスを残しつつ思考整理・構造化する(コア価値=ADR-0004)。
// プロバイダは Gemini / Anthropic(Claude) / OpenAI の3種をサポート(BYO鍵)。
// 鍵はフロントの設定から渡す(コードに埋め込まない)。

use serde_json::json;

/// 整形用の共通プロンプト。話者本人の思考整理を助け、ニュアンスを残す(コア価値)。
fn build_prompt(transcript: &str) -> String {
    format!(
        "以下は音声の文字起こしです。話者本人の思考を整理する手助けをしてください。\n\
         - 要点を見出しと箇条書きで構造化する\n\
         - 言い淀みや繰り返しは整理しつつ、本人のニュアンス・迷い・語り口は残す\n\
         - 事実を捏造せず、書かれていないことは足さない\n\
         - 最後に「ひとことまとめ」を1〜2文で添える\n\n\
         ---\n{transcript}"
    )
}

/// プロバイダ名で整形APIを振り分ける。空/未知は Gemini にフォールバック。
pub fn refine(
    provider: &str,
    api_key: &str,
    model: &str,
    transcript: &str,
) -> Result<String, String> {
    match provider.trim().to_ascii_lowercase().as_str() {
        "anthropic" | "claude" => refine_with_anthropic(api_key, model, transcript),
        "openai" | "gpt" => refine_with_openai(api_key, model, transcript),
        _ => refine_with_gemini(api_key, model, transcript),
    }
}

/// Gemini で文字起こしを整形する。
pub fn refine_with_gemini(api_key: &str, model: &str, transcript: &str) -> Result<String, String> {
    if api_key.trim().is_empty() {
        return Err("Gemini APIキーが未設定です（設定から入力してください）".into());
    }
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={api_key}"
    );

    let body = json!({
        "contents": [{ "parts": [{ "text": build_prompt(transcript) }] }]
    });

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
    if out.trim().is_empty() {
        return Err(format!("整形結果が空でした（Gemini応答: {}）", v));
    }
    Ok(out.trim().to_string())
}

/// Anthropic(Claude) Messages API で文字起こしを整形する。
/// エンドポイント: POST https://api.anthropic.com/v1/messages
/// 必須ヘッダ: x-api-key, anthropic-version: 2023-06-01, content-type: application/json
pub fn refine_with_anthropic(
    api_key: &str,
    model: &str,
    transcript: &str,
) -> Result<String, String> {
    if api_key.trim().is_empty() {
        return Err("Anthropic APIキーが未設定です（設定から入力してください）".into());
    }

    let body = json!({
        "model": model,
        "max_tokens": 4096,
        "system": "あなたは話者本人の思考整理を助けるアシスタントです。事実を捏造せず、本人のニュアンス・迷い・語り口を残してください。",
        "messages": [
            { "role": "user", "content": build_prompt(transcript) }
        ]
    });

    let resp = ureq::post("https://api.anthropic.com/v1/messages")
        .set("Content-Type", "application/json")
        .set("x-api-key", api_key)
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
    if out.trim().is_empty() {
        return Err(format!("整形結果が空でした（Anthropic応答: {}）", v));
    }
    Ok(out.trim().to_string())
}

/// OpenAI Chat Completions API で文字起こしを整形する。
/// エンドポイント: POST https://api.openai.com/v1/chat/completions
/// 必須ヘッダ: Authorization: Bearer <key>, content-type: application/json
pub fn refine_with_openai(api_key: &str, model: &str, transcript: &str) -> Result<String, String> {
    if api_key.trim().is_empty() {
        return Err("OpenAI APIキーが未設定です（設定から入力してください）".into());
    }

    let body = json!({
        "model": model,
        "messages": [
            { "role": "system", "content": "あなたは話者本人の思考整理を助けるアシスタントです。事実を捏造せず、本人のニュアンス・迷い・語り口を残してください。" },
            { "role": "user", "content": build_prompt(transcript) }
        ]
    });

    let resp = ureq::post("https://api.openai.com/v1/chat/completions")
        .set("Content-Type", "application/json")
        .set("Authorization", &format!("Bearer {api_key}"))
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

    if out.trim().is_empty() {
        return Err(format!("整形結果が空でした（OpenAI応答: {}）", v));
    }
    Ok(out.trim().to_string())
}
