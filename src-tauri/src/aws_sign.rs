// AWS SigV4署名(同期・tokio非依存 / ADR-0011, docs/research/sources/aws-providers.md)。
// http::Request を SigV4署名し、ureq に載せるヘッダ列を返す純関数。
// aws-sdk-bedrockruntime は async専用のため使わず、aws-sigv4(署名計算のみ)で同期署名する。

use std::time::SystemTime;

/// AWS資格情報(UIから渡す。コードに埋めない / ADR-0005)。
pub struct AwsCreds {
    pub access_key: String,
    pub secret_key: String,
    /// STS一時credのときのみ Some。
    pub session_token: Option<String>,
    pub region: String,
}

/// URL からホスト名(`host:port` のうちホスト部)を取り出す。SigV4 は host ヘッダの署名が必須。
fn host_of(url: &str) -> String {
    url.split("://")
        .nth(1)
        .unwrap_or(url)
        .split('/')
        .next()
        .unwrap_or("")
        .to_string()
}

/// POST リクエストを SigV4署名し、付与すべきヘッダ(name,value)を返す。
/// service: Amazon Bedrock="bedrock" / Claude Platform on AWS="aws-external-anthropic"。
/// (注: Bedrock のホスト名は bedrock-runtime だが署名サービス名は "bedrock"。)
/// content-type は application/json 固定(整形APIのJSON送受信)。
pub fn sign_post(
    url: &str,
    body: &[u8],
    service: &str,
    creds: &AwsCreds,
) -> Result<Vec<(String, String)>, String> {
    use aws_credential_types::Credentials;
    use aws_sigv4::http_request::{sign, SignableBody, SignableRequest, SigningSettings};
    use aws_sigv4::sign::v4;

    let identity = Credentials::new(
        creds.access_key.clone(),
        creds.secret_key.clone(),
        creds.session_token.clone(),
        None,
        "quickscribe-user",
    )
    .into();

    let params = v4::SigningParams::builder()
        .identity(&identity)
        .region(&creds.region)
        .name(service)
        .time(SystemTime::now())
        .settings(SigningSettings::default())
        .build()
        .map_err(|e| format!("SigV4パラメータ構築に失敗: {e}"))?
        .into();

    // host は SigV4 の必須署名ヘッダ。ureq が送る Host と一致させる(URL由来で同一)。
    let host = host_of(url);
    let signed_headers = [("host", host.as_str()), ("content-type", "application/json")];
    let signable = SignableRequest::new(
        "POST",
        url,
        signed_headers.iter().map(|(k, v)| (*k, *v)),
        SignableBody::Bytes(body),
    )
    .map_err(|e| format!("署名対象の構築に失敗: {e}"))?;

    let signing_output = sign(signable, &params).map_err(|e| format!("SigV4署名に失敗: {e}"))?;
    let (instructions, _signature) = signing_output.into_parts();

    // instructions(Authorization / X-Amz-Date / X-Amz-Security-Token 等)を http::Request に適用し、
    // 付与されたヘッダを取り出す。host/content-length は ureq が自前で付けるため除外。
    let mut req = http::Request::builder()
        .method("POST")
        .uri(url)
        .header("host", host)
        .header("content-type", "application/json")
        .body(())
        .map_err(|e| format!("httpリクエスト構築に失敗: {e}"))?;
    instructions.apply_to_request_http1x(&mut req);

    let mut out = Vec::new();
    for (name, value) in req.headers() {
        let n = name.as_str();
        if n.eq_ignore_ascii_case("host") || n.eq_ignore_ascii_case("content-length") {
            continue;
        }
        if let Ok(v) = value.to_str() {
            out.push((n.to_string(), v.to_string()));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_of_extracts_hostname() {
        assert_eq!(
            host_of("https://bedrock-runtime.us-east-1.amazonaws.com/model/x/invoke"),
            "bedrock-runtime.us-east-1.amazonaws.com"
        );
        assert_eq!(
            host_of("https://aws-external-anthropic.us-west-2.api.aws/v1/messages"),
            "aws-external-anthropic.us-west-2.api.aws"
        );
    }

    #[test]
    fn sign_post_produces_auth_headers() {
        // ダミー資格情報でも署名計算は成立し、Authorization / X-Amz-Date を付与する(決定性検証)。
        let creds = AwsCreds {
            access_key: "AKIAEXAMPLE".into(),
            secret_key: "secretexamplekey".into(),
            session_token: None,
            region: "us-east-1".into(),
        };
        let headers = sign_post(
            "https://bedrock-runtime.us-east-1.amazonaws.com/model/test/invoke",
            b"{\"x\":1}",
            "bedrock",
            &creds,
        )
        .expect("署名は成功するはず");
        let names: Vec<String> = headers.iter().map(|(n, _)| n.to_ascii_lowercase()).collect();
        assert!(names.iter().any(|n| n == "authorization"), "Authorization必須");
        assert!(names.iter().any(|n| n == "x-amz-date"), "X-Amz-Date必須");
    }

    #[test]
    fn sign_post_includes_session_token_when_present() {
        let creds = AwsCreds {
            access_key: "AKIAEXAMPLE".into(),
            secret_key: "secretexamplekey".into(),
            session_token: Some("FwoGtoken".into()),
            region: "us-east-1".into(),
        };
        let headers = sign_post(
            "https://bedrock-runtime.us-east-1.amazonaws.com/model/test/invoke",
            b"{}",
            "bedrock",
            &creds,
        )
        .expect("署名は成功するはず");
        let names: Vec<String> = headers.iter().map(|(n, _)| n.to_ascii_lowercase()).collect();
        assert!(
            names.iter().any(|n| n == "x-amz-security-token"),
            "session_token があれば X-Amz-Security-Token を付与"
        );
    }
}
