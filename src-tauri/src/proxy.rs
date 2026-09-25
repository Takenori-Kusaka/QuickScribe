// システムプロキシ設定の検出と ureq Agent の構築。
// Windows 11 等の企業ネットワーク環境において、OSのシステムプロキシ設定（WinINet / レジストリ）や
// 環境変数（HTTP_PROXY, HTTPS_PROXY 等）を自動認識し、外部通信（モデルDL等）を正常に行えるようにする。

/// URL からホスト名（ポート番号やパスを除く）を抽出する純粋関数。
pub fn extract_host(url: &str) -> Option<&str> {
    let after_scheme = if let Some(idx) = url.find("://") {
        &url[idx + 3..]
    } else {
        url
    };
    let host_and_port = after_scheme
        .split(['/', '?', '#'])
        .next()
        .filter(|s| !s.is_empty())?;

    // ユーザー情報 (user:pass@host) があればホスト側を抽出
    let host_part = if let Some(at_idx) = host_and_port.rfind('@') {
        &host_and_port[at_idx + 1..]
    } else {
        host_and_port
    };

    // IPv6 [::1]:8080 の場合
    if host_part.starts_with('[') {
        if let Some(bracket_end) = host_part.find(']') {
            return Some(&host_part[1..bracket_end]);
        }
    }

    // ポート番号 (:8080) を除去
    Some(host_part.split(':').next().unwrap_or(host_part))
}

/// 対象ホストがプロキシ除外リスト（ProxyOverride / NO_PROXY）に該当するか判定する純粋関数。
/// Windows の ProxyOverride 形式（セミコロン区切り、"<local>", "*.domain", "prefix*" 等）に対応。
pub fn is_proxy_bypassed(host: &str, bypass_list: &str) -> bool {
    let host = host.trim().to_ascii_lowercase();
    if host.is_empty() {
        return false;
    }

    for item in bypass_list.split([';', ',']) {
        let rule = item.trim().to_ascii_lowercase();
        if rule.is_empty() {
            continue;
        }

        // <local>: イントラネット（ドットを含まないホスト）および localhost / 127.0.0.1
        if rule == "<local>" {
            if !host.contains('.') || host == "localhost" || host == "127.0.0.1" || host == "::1" {
                return true;
            }
            continue;
        }

        // ワイルドカードサフィックス: *.example.com または .example.com
        if let Some(suffix) = rule.strip_prefix("*.") {
            if host == suffix || host.ends_with(&format!(".{suffix}")) {
                return true;
            }
            continue;
        }
        if let Some(suffix) = rule.strip_prefix('.') {
            if host == suffix || host.ends_with(&format!(".{suffix}")) {
                return true;
            }
            continue;
        }

        // ワイルドカードプレフィックス: 192.168.*
        if let Some(prefix) = rule.strip_suffix('*') {
            if host.starts_with(prefix) {
                return true;
            }
            continue;
        }

        // 完全一致
        if host == rule {
            return true;
        }
    }

    false
}

/// Windows レジストリの ProxyServer 文字列からプロキシ URL を抽出・正規化する純粋関数。
/// 単一形式: "proxy.corp.example.com:8080"
/// プロトコル別形式: "http=http-proxy:8080;https=https-proxy:8443;ftp=..."
pub fn parse_proxy_server(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }

    let target = if raw.contains('=') {
        let mut chosen = None;
        for part in raw.split(';') {
            let part = part.trim();
            if let Some(val) = part.strip_prefix("https=") {
                chosen = Some(val.trim());
                break;
            } else if let Some(val) = part.strip_prefix("http=") {
                if chosen.is_none() {
                    chosen = Some(val.trim());
                }
            }
        }
        chosen?
    } else {
        raw
    };

    if target.is_empty() {
        return None;
    }

    let proxy_url = if target.starts_with("http://")
        || target.starts_with("https://")
        || target.starts_with("socks5://")
        || target.starts_with("socks4://")
    {
        target.to_string()
    } else {
        format!("http://{target}")
    };

    Some(proxy_url)
}

/// Windows のレジストリ（Internet Settings）からシステムプロキシ設定を読み取る。
#[cfg(windows)]
fn read_windows_proxy_settings() -> Option<(String, Option<String>)> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let settings = hkcu
        .open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings")
        .ok()?;

    let proxy_enable: u32 = settings.get_value("ProxyEnable").unwrap_or(0);
    if proxy_enable != 1 {
        return None;
    }

    let proxy_server: String = settings.get_value("ProxyServer").ok()?;
    let proxy_override: Option<String> = settings.get_value("ProxyOverride").ok();

    Some((proxy_server, proxy_override))
}

#[cfg(not(windows))]
fn read_windows_proxy_settings() -> Option<(String, Option<String>)> {
    None
}

/// 指定された URL に対して適用すべき Proxy 設定を取得する。
/// 1. 環境変数 (HTTPS_PROXY, HTTP_PROXY, ALL_PROXY 等)
/// 2. Windows システムプロキシ (ProxyEnable, ProxyServer, ProxyOverride)
pub fn detect_proxy_for_url(url: &str) -> Option<ureq::Proxy> {
    // 1. 環境変数の設定を最優先
    if let Some(proxy) = ureq::Proxy::try_from_env() {
        return Some(proxy);
    }

    // 2. Windows システムプロキシの検出
    if let Some((proxy_server, proxy_override)) = read_windows_proxy_settings() {
        if let Some(host) = extract_host(url) {
            if let Some(bypass) = proxy_override.as_deref() {
                if is_proxy_bypassed(host, bypass) {
                    return None;
                }
            }
        }

        if let Some(proxy_url) = parse_proxy_server(&proxy_server) {
            match ureq::Proxy::new(&proxy_url) {
                Ok(proxy) => return Some(proxy),
                Err(e) => {
                    eprintln!("Failed to parse system proxy URL '{proxy_url}': {e}");
                }
            }
        }
    }

    None
}

/// 指定 URL 宛てのリクエスト用にプロキシ設定を適用した ureq::Agent を構築する。
pub fn build_agent_for_url(url: &str) -> ureq::Agent {
    let mut builder = ureq::Agent::config_builder();
    if let Some(proxy) = detect_proxy_for_url(url) {
        builder = builder.proxy(Some(proxy));
    }
    ureq::Agent::from(builder.build())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_host_parses_various_urls() {
        assert_eq!(
            extract_host("https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin"),
            Some("huggingface.co")
        );
        assert_eq!(
            extract_host("http://internal-server:8080/path?query=1"),
            Some("internal-server")
        );
        assert_eq!(
            extract_host("https://user:pass@sub.example.com:443/file"),
            Some("sub.example.com")
        );
        assert_eq!(extract_host("http://[::1]:8080/"), Some("::1"));
        assert_eq!(extract_host("invalid"), Some("invalid"));
    }

    #[test]
    fn parse_proxy_server_formats() {
        // 単純なホスト:ポート
        assert_eq!(
            parse_proxy_server("proxy.corp.example.com:8080"),
            Some("http://proxy.corp.example.com:8080".to_string())
        );
        // 既にスキーマがある
        assert_eq!(
            parse_proxy_server("http://proxy.corp.example.com:8080"),
            Some("http://proxy.corp.example.com:8080".to_string())
        );
        // プロトコル別（https優先）
        assert_eq!(
            parse_proxy_server("http=proxy1:8080;https=proxy2:8443;ftp=proxy3:21"),
            Some("http://proxy2:8443".to_string())
        );
        // プロトコル別（httpのみ）
        assert_eq!(
            parse_proxy_server("http=proxy1:8080;ftp=proxy3:21"),
            Some("http://proxy1:8080".to_string())
        );
        // 空文字
        assert_eq!(parse_proxy_server("   "), None);
    }

    #[test]
    fn is_proxy_bypassed_evaluates_rules() {
        let bypass = "<local>;*.internal.corp;192.168.*;api.special.com";

        // <local> 判定
        assert!(is_proxy_bypassed("intranet", bypass));
        assert!(is_proxy_bypassed("localhost", bypass));
        assert!(is_proxy_bypassed("127.0.0.1", bypass));

        // ワイルドカードサフィックス
        assert!(is_proxy_bypassed("service.internal.corp", bypass));
        assert!(is_proxy_bypassed("internal.corp", bypass));
        assert!(!is_proxy_bypassed("other-corp.com", bypass));

        // プレフィックス
        assert!(is_proxy_bypassed("192.168.1.100", bypass));
        assert!(!is_proxy_bypassed("10.0.0.1", bypass));

        // 完全一致
        assert!(is_proxy_bypassed("api.special.com", bypass));
        assert!(!is_proxy_bypassed("huggingface.co", bypass));
    }

    #[test]
    fn build_agent_for_url_constructs_agent_safely() {
        // 例外なく Agent が構築できること
        let agent = build_agent_for_url("https://huggingface.co/models");
        drop(agent);
    }

    #[test]
    fn detect_proxy_picks_up_env_var() {
        let _g = crate::testhttp::env_scope(
            &[("HTTPS_PROXY", "http://127.0.0.1:9999")],
            &["NO_PROXY", "no_proxy", "HTTP_PROXY", "http_proxy"],
        );
        let proxy = detect_proxy_for_url("https://huggingface.co/model.bin");
        assert!(proxy.is_some(), "HTTPS_PROXY 環境変数が認識される");
    }
}
