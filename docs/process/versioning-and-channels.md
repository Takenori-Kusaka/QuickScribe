# バージョニングとリリースチャネル方針（S8.5）

> Status: Living。SemVer と release-please を前提にした版・チャネル・鍵分離の方針。
> 関連: [ADR-0008 ライセンス/配布](../adr/0008-licensing-and-distribution.md)、[ADR-0010 v0.1.0ゲート](../adr/0010-v0.1.0-gate-legal-must-only.md)、[distribution-and-signing](distribution-and-signing.md)。

## 1. バージョニング（SemVer）

- **SemVer 2.0** に従う。**0.x 系（現行）はプレ1.0** とみなし、破壊的変更を minor で許容する（ユーザーへは CHANGELOG で明示）。
- **版の単一情報源(SSOT)は release-please**（`.release-please-manifest.json`）。Conventional Commits から次版を決定し Release PR を生成する。
- 配布物の版は **タグ由来**で設定する（`release.yml` の "Set version from tag" が `Cargo.toml` を上書き）。手動で版を散在管理しない。

## 2. 1.0.0 を切る基準（v1.0 Readiness）

1.0.0 は「公開APIの安定」ではなく**プロダクトとしての完成度**で切る。判断基準:

- Epic「v1.0.0 Readiness」(#389) と最終監査(#481) の残 issue が解消（**未クローズissue 0 / 未処理PR 0** をゲートとする）。
- 品質ゲートが常時グリーン（build/lint/e2e/transcribe/CodeQL/a11y、カバレッジ80%）。
- コア価値（ニュアンス保持整形・ローカル完結・物理トリガー統合）が実機で通しで成立。
- **未署名でも出荷可**とする（コード署名は認知拡大後に再申請 = Post-v1.0。SmartScreen 回避手順を明示済み）。

## 3. リリースチャネル

- **stable（既定）**: `v*` タグ = release-please のリリース。GitHub Releases + アプリ内 updater（`latest.json`）。
- **nightly（opt-in・#52 で導入）**: 早期検証・自己ドッグフード用のローリング `nightly` タグ（prerelease）。updater 鍵は stable と分離（`TAURI_SIGNING_PRIVATE_KEY_NIGHTLY`）。鍵 secret 未設定の間はワークフローがガードで no-op となり inert。運用手順は [release-channels](release-channels.md) を正とする。

## 4. 鍵の分離

- **updater 署名鍵（minisign）**: 自動更新の配布物完全性用。**現在も独立**して管理（コード署名とは別物）。秘密鍵は CI Secrets、公開鍵は `tauri.conf` に埋め込み。
- **コード署名証明書（OS向け）**: 現在**未導入**（SignPath 審査は認知不足で非承認 → 再申請は Post-v1.0）。導入時も updater 鍵とは**用途・保管を分離**する（署名証明書=OS信頼、updater鍵=配布完全性）。
- stable/nightly を将来分ける場合も updater 鍵をチャネル間で分離する（片方の失効が他方に波及しないように）。

## 5. まとめ

- 版は release-please が単一管理、配布はタグ駆動。0.x はプレ1.0。
- 1.0 は「issue/PR 0＋品質ゲート＋コア価値の実機成立」で切る（未署名可）。
- チャネルは stable（既定）＋ nightly（opt-in・鍵セットアップ後に有効）。鍵は用途・チャネル別に分離する（[release-channels](release-channels.md)）。
