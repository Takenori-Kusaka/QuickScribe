// 整形の知性(FormattingEngine: BYO-Cloud / ADR-0005)。
// 文字起こしテキストを、ニュアンスを残しつつ思考整理・構造化する(コア価値=ADR-0004)。
// 既定は Gemini API(BYO鍵)。鍵はフロントの設定から渡す(コードに埋め込まない)。

use serde_json::json;

/// Gemini で文字起こしを整形(思考整理・要約・ニュアンス保持)する。
pub fn refine_with_gemini(api_key: &str, model: &str, transcript: &str) -> Result<String, String> {
    if api_key.trim().is_empty() {
        return Err("Gemini APIキーが未設定です（設定から入力してください）".into());
    }
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={api_key}"
    );

    let prompt = format!(
        "以下は音声の文字起こしです。話者本人の思考を整理する手助けをしてください。\n\
         - 要点を見出しと箇条書きで構造化する\n\
         - 言い淀みや繰り返しは整理しつつ、本人のニュアンス・迷い・語り口は残す\n\
         - 事実を捏造せず、書かれていないことは足さない\n\
         - 最後に「ひとことまとめ」を1〜2文で添える\n\n\
         ---\n{transcript}"
    );

    let body = json!({
        "contents": [{ "parts": [{ "text": prompt }] }]
    });

    let resp = ureq::post(&url)
        .set("Content-Type", "application/json")
        .send_json(body)
        .map_err(|e| format!("整形APIの呼び出しに失敗: {e}"))?;

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
        return Err(format!("整形結果が空でした（応答: {}）", v));
    }
    Ok(out.trim().to_string())
}
