# 物理トリガー連携 — 一次情報リサーチ（S1.5 #21）

> 調査日: 2026-06-24 / 問い設計: [s1-5-physical-trigger-question-design.md](../s1-5-physical-trigger-question-design.md)（[ADR-0007](../../adr/0007-research-question-framing-method.md)）
> deep research（Stream Deck / マウス・ペダル / 類似アプリUX）の統合。全クレームに一次情報URL。

## 結論（先出し）

録音**トグル**は、QuickScribe が既に持つ**システム全体のグローバルホットキー**1本で、Stream Deck・Logitech・Razer・フットスイッチの**全デバイスにコード変更ゼロ**で対応できる。唯一価値ある先行投資は**録音APIの start/stop 分離**で、これが**モーメンタリ（押下中のみ録音＝hold-to-record）**という差別化UXの扉を開く。カスタム Stream Deck プラグインは現時点では過剰。

## Q1. Stream Deck

- **組み込み Hotkey アクション**で任意ショートカットを送出可能。ただし「Some apps also need to be in focus before they will respond to hotkeys」＝配送はアプリのグローバルホットキー登録に依存（QuickScribeは登録済み）。([how-to-set-up-stream-deck-hotkeys](https://www.elgato.com/us/en/explorer/products/stream-deck/how-to-set-up-stream-deck-hotkeys/))
- **F13–F24 推奨**: 「no program will set up hotkeys with them」＝衝突回避。([System Actions KB](https://help.elgato.com/hc/en-us/articles/360028234471))
- **Open アクションは引数なし**（`exe --toggle-record` は公式非対応）。引数を渡すならラッパー(.bat)かカスタムプラグイン。([同KB](https://help.elgato.com/hc/en-us/articles/360028234471))
- カスタムプラグインの現行公式SDK = **`@elgato/streamdeck`（Node/TS）+ `@elgato/cli`**（Node24+/SD7.1+）。`streamdeck create`→`manifest.json`(Actions/CodePath/Nodejs/OS/SDKVersion)→`streamdeck pack`→`.streamDeckPlugin`（**ローカルはダブルクリック導入可、Marketplace必須でない**）。`keyDown`/`keyUp`を別イベントで露出＝**プラグインならモーメンタリ可**。([github.com/elgatosf/streamdeck](https://github.com/elgatosf/streamdeck) / [Manifest](https://docs.elgato.com/streamdeck/sdk/references/manifest/) / [Keys guide](https://docs.elgato.com/streamdeck/sdk/guides/keys/) / [Distribution](https://docs.elgato.com/streamdeck/sdk/introduction/distribution/))
- 実アプリ: **OBSは公式プラグイン出荷**（豊富な状態API）／単純なstart/stop系はホットキー文書化のみ。**単機能トグルには組み込みHotkeyで十分**。([OBS Studio Plugin](https://www.elgato.com/ww/en/s/obs-studio-plugin-for-stream-deck))

## Q2. マウス／フットスイッチ（アプリ側コード不要）

| デバイス | ホットキー送出 | exe起動 | 実行時依存 | アプリ側コード |
|---|---|---|---|---|
| Logi Options+ | ✅ | ✅ Smart Action→Application(.exe) | Options+常駐 | 不要 |
| Razer Synapse | ✅(修飾可) | ✅ "LAUNCH PROGRAM" | Synapse常駐 | 不要 |
| プログラマブルUSBフットスイッチ | ✅(オンボード保存) | ❌(HIDキーのみ) | **なし(OS非依存)** | 不要 |

- Logi: [programming-buttons](https://hub.sync.logitech.com/options/post/programming-buttons-and-keys-in-options-ntwM6VsAEKhHACY) / [Application action](https://support.logi.com/hc/en-gb/articles/14308014808215)
- Razer: [remap](https://mysupport.razer.com/app/answers/detail/a_id/5569) / [launch program](https://mysupport.razer.com/app/answers/detail/a_id/1710) / [常駐要](https://mysupport.razer.com/app/answers/detail/a_id/1711)
- フットスイッチ: 標準HIDキーボードとして列挙、設定をデバイスに保存、Linuxもソフト不要で最も堅牢。固定機能版でなく**プログラマブル版**推奨。([PCsensor](https://pcsensor.com/product/pcsensor-usb-foot-pedal-keyboard-single-switch-programmable-pc-keyboard-mouse-customized-hot-key-shortcut-combination-key-for-page-turner-gaming-office-hid-mechanical-switch/))
- Linux注: Logi/Razerのremapは Options+/Synapse非対応→libratbag/Piper/solaar/OpenRazer依存。**Linuxはフットスイッチ経路が最堅牢**。

## Q3. TOGGLE vs MOMENTARY（差別化の核心）

| アプリ | toggle | momentary(hold) | 公式SDプラグイン |
|---|---|---|---|
| Dragon | ✅ | ✅ **Press-to-talk** | — |
| OBS | ✅ | ✅ **Push-to-talk/mute** | ✅ |
| Discord | ✅ | ✅ **PTT(既定が押下保持)** | — |
| Loom | ✅単一トグルのみ | ❌ | 不明 |

- Dragonが手本: press-to-talk(離すと停止)とmic toggleを**別アクションで併存**。([Dragon Hot keys](https://www.nuance.com/products/help/dragon15/dragon-for-pc/enx/dmpe_sub/Content/DialogBoxes/options/options_dialog_hotkeys_tab.htm))
- OBS/Discordは**release delay**（離してから停止までの遅延）で末尾切れ防止が定石。([OBS](https://obsproject.com/forum/threads/delaying-push-to-mute-delays-release-only-not-attack.97167/) / [Discord PTT](https://support.discord.com/hc/en-us/articles/211376518))
- Loomは単一トグルでmomentary不可＝**避けるべきアンチパターン**。([Loom shortcuts](https://support.atlassian.com/loom/docs/use-looms-keyboard-shortcuts/))
- **技術要件**: momentaryには key-down/key-up ペア、または分離した start/stop が必須（単一トグルでは原理的に不可）。
  - QuickScribe の `tauri-plugin-global-shortcut` ハンドラは既に `ShortcutState::Pressed`/`Released` を取得可能（現在はPressedのみ使用）。→ **モーメンタリは既存基盤で実装可能**。

## QuickScribe が BUILD すべきもの

1. **【最優先・コード0】ドキュメント**: 各デバイスを QuickScribe のグローバルホットキー（F13–F24推奨）へ割り当てる手順。トグルは全デバイス即対応。
2. **【小改修・高ROI】録音APIの start/stop 分離**: `--start-record`/`--stop-record` CLI＋内部の明示 start/stop。トグルもモーメンタリも上位で構成可能に（ADR-0006: 削るのでなく分割で対処）。
3. **【任意・後続】モーメンタリ（hold-to-record）モード**: Dragon/OBS/Discord標準の実績UX。停止に短い release delay。Stream Deckでのmomentaryのみカスタムプラグインが要る（#2が済めば薄いラッパー）。

### 何が判断を覆すか
- カスタムSDプラグインを作るべき条件: ボタン面に録音状態の視覚FB / Marketplace露出をマーケ資産化 / SD上でmomentary提供。単機能トグルを超える要求が出た時のみ。
- start/stop分離を見送ってよい条件: momentaryを恒久的にロードマップ除外とDecidersが決めた場合のみ（後付けは高コスト＝今が安い）。
