# パフォーマンス・ベースライン

> Status: Living（2026-06-29 初版）。計測方法と基準値を記録する（#403）。実測は `perf-bench` ワークフローのアーティファクト（`perf-report.md`）が一次情報。

## 計測方法

- **ワークフロー**: `.github/workflows/perf.yml`（`workflow_dispatch` ＋ release 公開時）。
- **対象**: ローカル whisper（`ggml-tiny`、Linux x64、決定的 AVX2 ベースライン / [ADR-0012](../adr/0012-windows-multiarch-multisimd-distribution.md)）。
- **指標**:
  - **RTF（実時間比）** = 文字起こし経過秒 / 音源長秒。**< 1.0 が実時間以内**の目安。
  - **ピークメモリ（RSS）** = `/usr/bin/time -v` の Maximum resident set size。
- 固定音源（espeak-ng の既知発話・約十数秒）で計測。コンパイルは事前ビルド（`--no-run`）で計測区間から除外。

## 目標（暫定 / [NFR](../non-functional-requirements.md)）

| 指標 | 目標（暫定） |
|---|---|
| RTF（tiny / x64 AVX2） | ≤ 1.0（実時間以内） |
| ピークメモリ（文字起こし時） | ≤ 300 MB 目安 |

## ベースライン実測

| 日付 | バージョン | モデル | 音源長 | RTF | ピークRSS | 備考 |
|---|---|---|---|---|---|---|
| 2026-06-29 | v0.6.4時点 | ggml-tiny | 18.88s | **0.857** ✅ | 1518.9 MB※ | GitHub Actions ubuntu-22.04 x64・AVX2。RTF<1.0=実時間以内（目標達成） |

> ※ ピークRSSは **`cargo test` ハーネス全体**の値（テストプロセス＋ビルド成果物＋whisperコンテキスト）であり、**配布アプリのアイドル時メモリではない**。アプリ実体のメモリ計測（GUI起動時）は #403 後続で別途行う。RTF は実用域（実時間以内）を確認。

## 起動時間・アイドルメモリ（#554 / #403）

- `perf.yml` の `startup-time` ジョブが xvfb ヘッドレスで `run()` 入口→フロント `onMount` の経過をアプリ計装で記録（`QS_PERF_STARTUP=1`）。
- 同ジョブで ready 後に**アプリ実プロセス(quickscribe)のアイドル RSS** をサンプルしピーク値を記録（`cargo test` ハーネス値ではなく配布バイナリ実体）。値は `startup-report.md` アーティファクト＋ジョブサマリが一次情報。

## アイドル CPU 使用率（#664 Phase 1）

常駐アプリで最も効く資源指標だが、従来 NFR にも perf CI にも存在しなかった（アイドル RSS のみ計測していた）。

### 計測方法（Linux / `perf.yml` の `startup-time` ジョブ）

- ready（`perf-startup.txt` 生成）を確認したのち **3 秒待って**起動処理の残りを落ち着かせる。
- そこから **10 秒**の観測窓を取り、窓の前後で対象プロセス群の `/proc/<pid>/stat` の **`utime + stime`（tick）** を 2 点サンプルする。
- 対象は **`quickscribe`（アプリ実体）と `WebKit(WebProcess|NetworkProcess)`（WebView の子プロセス）** の合計。RSS 指標がアプリ実体のみなのに対し、CPU は WebView 側の常駐コストを取りこぼすと意味を成さないため対象を広げている。
- 使用率(%) = `(Δtick / CLK_TCK) / 経過実時間秒 × 100`。**1 コア基準**（100 % = 1 コア占有）。
- 出力先は `startup-report.md` アーティファクト＋ジョブサマリ（起動時間・アイドル RSS と同じ扱い）。

### この計測でカバーできない範囲（重要）

perf CI は **ubuntu-22.04 / xvfb** で走る。一方タスクバーウィジェット（#662）は **Windows 限定コード**（`src-tauri/src/taskbar_widget.rs`）であり、**Linux CI には原理的に現れない**。**Linux の数値だけを見て「アイドル CPU は問題ない」と判断してはならない。** Windows 側は次節の手順で計測する。

### 実測（Linux CI）

| 日付 | ブランチ/バージョン | アイドル CPU (1コア基準) | 備考 |
|---|---|---|---|
| （未取得） | — | — | Phase 1 マージ後の初回 `perf-bench`（`workflow_dispatch`）で記録する |

## アイドル CPU 使用率 — Windows 実機（#664 Phase 2）

### 計測手順

`scripts/perf/measure_idle_cpu.ps1` を使う。Linux ジョブと**同じ定義**（消費CPU秒 / 経過実時間、1コア基準）で算出する。

```powershell
# 0. 診断ログを有効にしてから起動する（タイマーが実際に回っていることの確認手段）
#    #667 以降 taskbar-diag.log は既定 OFF。これを立てないと show/hide 遷移を観測できない。
$env:QS_TASKBAR_DIAG = "1"

# 1. QuickScribe を起動し、操作せず放置する（＝アイドル状態にする）
# 2. 観測窓 120 秒で計測する
powershell -NoProfile -ExecutionPolicy Bypass `
  -File scripts/perf/measure_idle_cpu.ps1 -Label widget-on -WindowSeconds 120 `
  -JsonPath idle-cpu-widget-on.json

# 3. 設定でタスクバーウィジェットを OFF にし、同じ長さで計測して差分を取る
powershell -NoProfile -ExecutionPolicy Bypass `
  -File scripts/perf/measure_idle_cpu.ps1 -Label widget-off -WindowSeconds 120 `
  -JsonPath idle-cpu-widget-off.json
```

- **`QS_TASKBAR_DIAG=1` を立てずに計測してはならない。** ウィジェットのタイマーが回っていることの唯一の確認手段が `taskbar-diag.log` の `show`/`hide` 遷移であり、#667 でこのログは既定 OFF になった（[troubleshooting.md](../guide/troubleshooting.md)）。立て忘れると「タイマーが動いているつもりで動いていない」数値を掴む。
- 計測対象は `quickscribe.exe` **とその子孫の `msedgewebview2.exe` 群**（WebView2）。`per_process` にプロセス別内訳が出るので、**タイマーが回るアプリ実体単独の消費**を WebView2 と切り離して読める。
- ウィジェット ON/OFF の差分が意味を持つのは、設定 OFF 時に `WM_TIMER` ハンドラが `ENABLED` の atomic 読みだけで早期 return するため（`taskbar_widget.rs:475-483`）。**高価な Win32 呼び出し群（`GetForegroundWindow` / `FindWindowW` / `SHAppBarMessage` / `SetWindowPos`）は OFF 時には走らない**＝差分がウィジェットの寄与になる。
- **CI 化はしていない。** `windows-latest` ランナーで `Shell_TrayWnd`（explorer のタスクバー）が成立する保証がなく、成立しなければウィジェットは配置されず寄与がゼロになって計測の意味が消える。ヘッドレス Windows での成立可否が確認できるまでは、本手順による**実機での手動計測**を一次情報とする。

### 実測（Windows 実機）

| 日付 | ビルド | 観測窓 | アイドル CPU 合計 (1コア基準) | アプリ実体単独 | WebView2 合計 |
|---|---|---|---|---|---|
| 2026-07-26 | main 相当のローカル release ビルド | 30 s | 0.052 % | — | — |
| 2026-07-26 | 同上 | 120 s | **0.117 %** | **0.091 %** | 0.026 % |

計測環境: Windows 11 Pro 26200 / ウィジェット有効（`taskbar-diag.log` に `show`/`hide` 遷移あり＝300ms タイマー稼働中）/ プロセス7件（実体1 + WebView2 6）。

**目標 ≤ 1 % に対して 1桁小さい。** タスクバーウィジェットのタイマーが動いている状態でこの値なので、**#662（300ms ポーリングのイベント駆動化）の CPU 面での上限効果は 0.09 %/1コア 未満**である。静的なコード読解では「毎秒3.3回の Win32 呼び出し」に見えたが、実測では 1 tick あたり 1 ms 未満だった。

> 注: `TotalProcessorTime` の分解能は約 15.6 ms のため、120 秒窓での 0.109 秒は約 7 量子分。絶対値の精度は粗いが、**「1 % を大きく下回る」という桁の判断には十分**。

### 参考: アイドル時メモリ（同一環境・同時計測）

| プロセス | WorkingSet |
|---|---|
| `quickscribe.exe`（アプリ実体） | 29.8 MB |
| `msedgewebview2.exe` × 6（WebView2） | 165.5 MB |
| **合計** | **195.3 MB** |

NFR 目標 300 MB 以内。**常駐メモリの約 85 % は WebView2 側**で、アプリ実体は 30 MB 程度。トレイ常駐時に WebView2 を保持し続ける設計トレードオフ（`lib.rs` の close→hide）は、この内訳が根拠になる。

## エントリ一覧の描画コスト（#666）

保管庫のエントリは**単調増加する**（削除しない限り減らない）。従来は一覧を開いた瞬間に全件を DOM 化していたため、蓄積が進むほど開封が遅くなる構造だった。段階表示（既定 `ENTRY_VISIBLE = 50` 件）＋検索デバウンス（`ENTRY_SEARCH_DEBOUNCE_MS = 150` ms）で初期描画の DOM 量を定数で頭打ちにした（`src/lib/constants.ts`）。

### 計測方法（手動 / CI 対象外）

```bash
npx playwright test --config e2e/playwright.perf.config.ts
```

- `e2e/perf-entry-list.spec.ts`。Tauri API はモック（`e2e/mocks/core.ts`）で、`?entries=N` により `list_entries` が N 件を返す。クエリ無し（スクショ/a11y の既定経路）は従来どおり 4 件。
- **CI からは意図的に外している**（`playwright.config.ts` の `testMatch` に一致しない）。時間依存の数値であり、環境が変われば絶対値が変わるため。
- **計測はページ内（`page.evaluate`）で完結させる。** Playwright 越しに `click` して待つと、ドライバの往復（数百 ms）に描画コストが埋もれて差が出ない（実際に最初の計測はこれで失敗し、staged 253.8 ms / full 254.0 ms と差が消えていた）。
- 比較の取り方: 「開いて 50 行が出るまで（staged）」と「そこから残り 950 行を展開して 1000 行になるまで（full）」を分けて測る。この2つは**同一の開封動作を分割したもの**なので、**staged + full が従来実装（開いた瞬間に全件を DOM 化）の所要時間に相当**する。同一ビルド内で測るためビルド差・機種差が入らない。

### 実測（Chromium / エントリ1000件 / 3回）

| 区間 | 1回目 | 2回目 | 3回目 |
|---|---|---|---|
| 開いて 50 行が出るまで（staged＝現在の初期描画） | 92.3 ms | 116.2 ms | 84.5 ms |
| そこから残り 950 行を展開（full） | 159.3 ms | 185.1 ms | 153.2 ms |
| **合計（＝従来の初期描画に相当）** | **251.6 ms** | **301.3 ms** | **237.7 ms** |

初期描画は **約 238–301 ms → 約 85–116 ms（約 2.6–2.9 倍）**。計測環境: Windows 11 Pro 26200 / Chromium（Playwright 同梱）/ ローカル vite dev サーバ。

### 回帰ゲートに使う事実（時間に依らない）

時間の数値はゲートにしていない。アサーションにしているのは**初期描画の行数が 1000 件でも 50 行で頭打ちになる**という構造的事実（`expect(rows).toBe(50)`）と、`staged < full` の順序のみ。時間依存の値をゲートにすると環境差で壊れる。

> 未実装: 仮想スクロール（#666 の提案4）。50 件で頭打ちになる以上、全件展開時のコストだけが残る問題であり、そこに到達するユーザー操作が明示的（「他 N 件を表示」）なので優先度は低い。

## 日本語精度（CER / #26）

- **方法**: 本人音読のパブリックドメイン作品3点（`src-tauri/tests/fixtures/ja-accuracy`）を `QS_LANG=ja` で認識し、`scripts/cer_ja.py`（NFKC・約物空白除去・文字単位 Levenshtein / 参照長）で CER を算出。`perf.yml` の「日本語精度 CER」ジョブが計測。
- **注**: 原文へのルビ混入で絶対CERは悲観側。N=3。**相対/回帰指標**として扱う（絶対精度の主張には使わない）。

### ベースライン（平均CER・CI実測 ubuntu-22.04 / opus / ja）

| モデル | 平均CER | 位置づけ |
|---|---|---|
| ggml-tiny | 56.9% | 日本語で base に完全劣位（[ADR-0022]） |
| ggml-base | 44.0% | 頑健な既定 |
| kotoba-whisper v2.0 q5 | **38.3%** | 日本語推奨（素材により幻覚で悪化しうる。蜘蛛の糸で base 超過） |

- サンプル別（CI / run 28699724095）: 銀河鉄道 tiny45.1/base38.0/**kotoba26.1** ・ 蜘蛛の糸 tiny58.6/**base47.9**/kotoba57.1 ・ 吾輩 tiny67.1/base46.1/**kotoba31.6**（%）。
- 回帰ゲートの基準値は `docs/perf/ja-cer-baseline.json`（margin=5pt）。上表はCI実測で確定済み。

## 残（#403 後続 / 小粒）
- **録音時**メモリ（ヘッドレスで録音を駆動する必要があり別途）。アイドル時RSSは計測済み。
- RTF/メモリの CI ハード変動を吸収する統計的な回帰ゲート（精度CERは閾値ゲート実装済み）。

[ADR-0022]: ../adr/0022-model-catalog-curation.md
