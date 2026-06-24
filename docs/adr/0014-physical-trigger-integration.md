# ADR-0014: 物理トリガー連携（ホットキー橋渡し＋録音API start/stop 分離＋モーメンタリ）

- Status: Proposed（段階実装。Phase 0=ドキュメント未着手）
- Date: 2026-06-24
- Deciders: Takenori Kusaka
- 関連: [ADR-0006 スコープ規律], [ADR-0007 調査規律]
- 一次情報: [docs/research/sources/physical-trigger-integration.md](../research/sources/physical-trigger-integration.md)
- 対象 Issue: #21（Epic E1 #8）。差別化「物理ボタン統合体験」。

## 背景・課題

S1.5 は「マウス/Stream Deck 等の物理トリガーで録音を起動」。録音トグルの起動点は既に (a) 設定可能なグローバルホットキー (b) CLI `quickscribe --toggle-record`（single-instance転送）で公開済み。
deep research（一次情報）の結論: **物理デバイスは各エコシステムの公式手段（Stream Deck組み込みHotkey / Logi Options+ / Razer Synapse / プログラマブルHIDフットスイッチ）で、既存ホットキーへコード変更ゼロで橋渡しできる**。独自デバイスAPIは作らない（ユーザー方針＝変な独自実装を避ける）。

唯一の設計判断は **トグルしか持たない現状が Loom型アンチパターン**で、**モーメンタリ（押下中のみ録音＝hold-to-record。Dragon/OBS/Discordの標準UX）を後付けできない**こと。

## 決定

1. **トグルは「ドキュメントのみ」で全デバイス対応**（コード変更ゼロ）。各デバイスを QuickScribe のグローバルホットキー（**F13–F24 推奨**＝衝突回避）へ割り当てる手順を提供。Logitech/Razer は CLI `--toggle-record` 直接起動も選べる。
2. **録音APIを start/stop に分離**（小改修・高ROI）。内部に明示的な開始/停止を持ち、CLI に `--start-record`/`--stop-record` を追加。トグルもモーメンタリも上位で構成可能にする。ADR-0006: 「削る」のでなく**分割・抽象化**で最終機能集合（toggle＋momentary）を保つ。
3. **モーメンタリ（hold-to-record）モードを提供**。`tauri-plugin-global-shortcut` の `ShortcutState::Pressed`/`Released` を用い、設定で「録音モード: トグル / 押している間」を選択。停止に短い **release delay**（末尾切れ防止。OBS/Discord定石）。
4. **カスタム Stream Deck プラグインは現時点で作らない**（過剰）。組み込みHotkeyで十分。視覚フィードバック/Marketplace露出/SD上momentaryが要件化したら再検討（反証条件）。

## 段階実装（ADR-0006: 最終ゴール不変）

- **Phase 0（ドキュメント / 即時）**: `docs/guide/physical-triggers.md` — Stream Deck・Logi・Razer・フットスイッチの割当手順。トグルは全デバイス即対応。
- **Phase 1（start/stop分離＋モーメンタリ）**: 内部 start/stop 明示化、CLI `--start-record`/`--stop-record`、録音モード設定（トグル/モーメンタリ）、Pressed/Releasedハンドリング、release delay。受入基準(BDD/EARS)→TDD→CI。
- **Phase 2（後続・任意）**: カスタム Stream Deck プラグイン（視覚FB/Marketplace/SD上momentaryが必要になった場合のみ）。Linux のマウスremap（libratbag/Piper/solaar/OpenRazer）案内強化。

## 結果・トレードオフ

- **Pro**: 独自実装ゼロで全デバイス対応。差別化UX（モーメンタリ）を既存基盤の小改修で開ける。prior art（Dragon/OBS/Discord）に整合。
- **Con/リスク**: モーメンタリは録音モードという設定面が増える（核心課題「リッチすぎず簡便」に配慮し、既定はトグルのまま明示選択に留める）。release delay の値は実機調整要。
- **却下案**: ① いまカスタムSDプラグインを作る → 単機能トグルに過剰、保守増。② start/stop分離を見送る → momentaryを恒久放棄しない限り後付け高コスト。③ デバイス固有コードを書く → 公式手段で不要、移植性悪化。

## 「考えが変わる条件」（反証）

- 主要物理デバイスのどれかが公式手段でホットキー/起動を割り当てられないと判明 → そのデバイス用の橋渡しを個別検討。
- ボタン面の録音状態視覚FB / Marketplace露出 / SD上momentary が製品要件化 → Phase 2 のカスタムプラグインに着手。
- Deciders がモーメンタリをロードマップから恒久除外 → Phase 1 を start/stop 分離（CLI拡充）までに留める。
