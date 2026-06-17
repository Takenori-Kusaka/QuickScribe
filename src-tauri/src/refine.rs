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
        _ => latest_gemini(api_key),
    }
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
}
