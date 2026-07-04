// テスト専用の極小HTTPサーバ（新規依存なし / std::net::TcpListener）。
// ureq系エンジン(refine/stt/model)の送受信をローカルで決定論的に検証するために使う。
// cfg(test) でのみコンパイルされる（lib.rs 側の mod 宣言参照）＝リリース挙動は不変。

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex, MutexGuard};

/// パス部分一致でレスポンスを返すルート定義。
pub struct Route {
    /// リクエストラインのパスに含まれていればマッチ（先頭一致でなく部分一致で緩く）。
    pub path_contains: &'static str,
    pub status: u16,
    pub body: Vec<u8>,
}

impl Route {
    pub fn json(path_contains: &'static str, status: u16, body: &str) -> Self {
        Route {
            path_contains,
            status,
            body: body.as_bytes().to_vec(),
        }
    }
}

/// 受信したリクエスト（リクエストライン＋ヘッダ＋本文の一部）を記録する。
pub type SeenRequests = Arc<Mutex<Vec<String>>>;

/// ルート群を配信するテストサーバを起動し、(base_url, 受信記録) を返す。
/// スレッドはプロセス終了まで生きる（テストプロセス内のリークは許容）。
pub fn serve(routes: Vec<Route>) -> (String, SeenRequests) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
    let base = format!("http://{}", listener.local_addr().unwrap());
    let seen: SeenRequests = Arc::new(Mutex::new(Vec::new()));
    let seen_t = seen.clone();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut s) = stream else { continue };
            let _ = s.set_read_timeout(Some(std::time::Duration::from_secs(5)));
            // ヘッダ末尾(\r\n\r\n)まで読み、Content-Length があれば本文も読み切る。
            let mut buf: Vec<u8> = Vec::new();
            let mut tmp = [0u8; 4096];
            let header_end = loop {
                match s.read(&mut tmp) {
                    Ok(0) => break None,
                    Ok(n) => {
                        buf.extend_from_slice(&tmp[..n]);
                        if let Some(pos) = find_subsequence(&buf, b"\r\n\r\n") {
                            break Some(pos + 4);
                        }
                        if buf.len() > 1_048_576 {
                            break None;
                        }
                    }
                    Err(_) => break None,
                }
            };
            let Some(header_end) = header_end else { continue };
            let head = String::from_utf8_lossy(&buf[..header_end]).to_string();
            let content_length: usize = head
                .lines()
                .find_map(|l| {
                    let (k, v) = l.split_once(':')?;
                    if k.trim().eq_ignore_ascii_case("content-length") {
                        v.trim().parse().ok()
                    } else {
                        None
                    }
                })
                .unwrap_or(0);
            while buf.len() < header_end + content_length {
                match s.read(&mut tmp) {
                    Ok(0) => break,
                    Ok(n) => buf.extend_from_slice(&tmp[..n]),
                    Err(_) => break,
                }
            }
            let body_snippet =
                String::from_utf8_lossy(&buf[header_end..buf.len().min(header_end + 65536)])
                    .to_string();
            let request_line = head.lines().next().unwrap_or("").to_string();
            if let Ok(mut g) = seen_t.lock() {
                g.push(format!("{request_line}\n{head}\n{body_snippet}"));
            }
            // リクエストラインのパスでルートを選ぶ。無ければ404。
            let path = request_line.split_whitespace().nth(1).unwrap_or("");
            let (status, body): (u16, &[u8]) = routes
                .iter()
                .find(|r| path.contains(r.path_contains))
                .map(|r| (r.status, r.body.as_slice()))
                .unwrap_or((404, b"{\"error\":\"no route\"}"));
            let resp = format!(
                "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            let _ = s.write_all(resp.as_bytes());
            let _ = s.write_all(body);
            let _ = s.flush();
        }
    });
    (base, seen)
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

/// 環境変数を設定するテストは process-global のためロックで直列化する。
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// 設定した環境変数を drop 時に復元するガード（グローバルロック保持で並行テストと衝突しない）。
pub struct EnvGuard {
    saved: Vec<(String, Option<String>)>,
    _lock: MutexGuard<'static, ()>,
}

/// 環境変数をまとめて設定し、ガードを返す。ガードが生きている間はロックを保持する。
pub fn set_envs(pairs: &[(&str, &str)]) -> EnvGuard {
    env_scope(pairs, &[])
}

/// 環境変数を設定(set)＋除去(unset)し、drop 時に元へ戻すガードを返す。
pub fn env_scope(set: &[(&str, &str)], unset: &[&str]) -> EnvGuard {
    let lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut saved = Vec::new();
    for (k, v) in set {
        saved.push((k.to_string(), std::env::var(k).ok()));
        std::env::set_var(k, v);
    }
    for k in unset {
        saved.push((k.to_string(), std::env::var(k).ok()));
        std::env::remove_var(k);
    }
    EnvGuard {
        saved,
        _lock: lock,
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (k, prev) in &self.saved {
            match prev {
                Some(v) => std::env::set_var(k, v),
                None => std::env::remove_var(k),
            }
        }
    }
}
