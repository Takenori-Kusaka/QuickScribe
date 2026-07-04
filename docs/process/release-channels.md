# リリースチャネルと updater 鍵分離 Runbook（S8.5 #52）

QuickScribe の配布チャネル運用と、自動アップデート署名鍵の分離手順。
バージョニングは **0.x 系を維持**（1.0 到達条件は [ADR-0009]）。

## チャネル

| チャネル | 目的 | タグ | prerelease | updater 署名鍵 | updater エンドポイント |
|---|---|---|---|---|---|
| **stable** | 一般配布 | `vX.Y.Z` | false | `TAURI_SIGNING_PRIVATE_KEY` | `releases/latest/download/latest.json` |
| **nightly** | 早期検証・自己ドッグフード | `nightly`（ローリング） | true | `TAURI_SIGNING_PRIVATE_KEY_NIGHTLY` | `releases/download/nightly/latest.json` |

- **stable**: `release-please.yml` → `release.yml`（タグ push / PAT不要 #427）。既存フロー。
- **nightly**: `nightly.yml`（`workflow_dispatch` ＋ 週次 cron / prerelease）。main HEAD をビルドし `nightly` タグへ上書き公開。

## なぜ鍵を分けるのか（updater 鍵分離）

Tauri updater はアプリに埋め込んだ **pubkey** で更新アーティファクトの署名を検証する。
stable と nightly で鍵を分けると:

1. **越境更新を防ぐ**: stable インスタンス（stable pubkey 埋込）は nightly 署名を検証できず、nightly を誤って掴まない。逆も同様。チャネルの隔離が暗号的に保証される。
2. **被害範囲を限定**: nightly 鍵が漏れても stable 配布は無傷（信頼の分離）。
3. **秘密鍵の露出面を最小化**: nightly は実験的でCI露出が多いぶん、stable 鍵と物理的に別管理にする。

## セットアップ（初回のみ・要手動＝鍵は秘密のため代行不可）

nightly を有効化するには、リポジトリ管理者が以下を実施する（1回だけ）:

```bash
# 1) nightly 用の updater 署名鍵ペアを生成（stable とは別物）。
npm exec -- tauri signer generate -w ~/.tauri/quickscribe-nightly.key
#   → 出力される「公開鍵(base64)」を控える。秘密鍵ファイルとパスワードも保管。

# 2) GitHub の Secrets に登録（Settings → Secrets and variables → Actions）。
#    - TAURI_SIGNING_PRIVATE_KEY_NIGHTLY          = 秘密鍵ファイルの中身
#    - TAURI_SIGNING_PRIVATE_KEY_NIGHTLY_PASSWORD = 生成時のパスワード（未設定なら空）

# 3) 公開鍵を nightly 設定へ埋め込む。
#    src-tauri/tauri.nightly.conf.json の plugins.updater.pubkey を
#    プレースホルダ "REPLACE_WITH_NIGHTLY_PUBKEY" から控えた公開鍵に置換してコミット。
```

セットアップ前は `nightly.yml` はガードで**自動スキップ**する（secret 未設定 or pubkey 未置換なら no-op で成功）。
＝誤って壊れた nightly を公開しない。stable 配布には一切影響しない。

## 公開 Runbook（nightly）

- **自動**: 週次 cron（`nightly.yml`）。main HEAD をビルドし `nightly` タグを上書き→prerelease 更新。
- **手動**: Actions → Nightly → Run workflow。
- **ロールバック**: 問題があれば prerelease `nightly` を Draft/削除、または直前の nightly アセットへ差し替え。stable は独立のため巻き込まれない。

## 公開 Runbook（stable / 既存）

- `release-please` の Release PR をマージ → タグ生成 → `release.yml` が同一ラン内でアセット添付（#427）。
- 詳細は [.github/workflows/release.yml] と [ADR-0009]。

## バージョニング（0.x）

- 破壊的変更でも 0.x の間は minor を上げる（SemVer 0.x 慣行）。1.0 は [ADR-0009] のゲート充足後。
- 版の単一ソースは `src-tauri/Cargo.toml` の `[package].version`（release.yml がタグから設定）。
- nightly は版に `-nightly.<日付/SHA>` を付さず、ローリング `nightly` タグで最新1つのみ提供（履歴を残さず軽量に）。

## 残（要ユーザー操作 / secret）

- nightly 鍵の生成・Secrets 登録・pubkey 置換（上記セットアップ）。これが済むまで nightly は inert。
  鍵は秘密情報のため自動化・代行の対象外（[signpath-signing-status] のコード署名審査待ちとは別物）。

[ADR-0009]: ../adr/0009-release-versioning-and-1.0-scope.md
