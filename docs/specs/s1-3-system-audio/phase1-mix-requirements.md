# S1.3 Phase 1 マイク＋システム音 同時取得 — requirements

> Status: Draft (2026-06-24) / 対象 Issue: #19（Epic E1 #8）/ [ADR-0013](../../adr/0013-system-audio-loopback-and-source-unification.md) Phase 1
> 記法: 軽量BDD主 ＋ EARS。前提: Phase 0（任意出力デバイスのループバック）は実装・実機検証済み。

## ユビキタス言語

- **ミックス録音（mix）**: マイク入力とシステム音（出力デバイスのループバック）を**同時に取得**し、1つの音声に合成して文字起こしする録音ソース種別。会議・通話で「自分の声＋相手の声」をまるごと残す。
- **共通タイムベース**: 文字起こし用の 16kHz mono。各ソースを独立にこの形式へ変換してから合成する。

## ユーザーストーリー

When 会議・通話を録音したい時、I want to 自分のマイクとPCが再生する相手の声を同時に1本のジャーナルへ残したい、so I can 対話の全体（自分＋相手）を後から振り返って整理できる。

## 設計方針（クロックドリフト対策）

マイク（cpal）とシステム音（wasapi）は**別クロック**。サンプル単位の厳密同期は行わず、**各ソースを独立に 16kHz mono へリサンプルしてから加算合成**する（OBSの音声ミキシングと同系統の簡易方式）。録音中の微小ドリフトは文字起こし品質に実害が小さく、実装が堅牢。合成は加算＋クリップ（[-1,1]）。

## 受入基準（EARS）

- **R1（event）**: When the user selects the "マイク＋システム音" source and records, the system shall capture microphone input and default system-audio output simultaneously.
- **R2（ubiquitous）**: The system shall convert each captured source to 16kHz mono independently, then mix them by summation with clipping to [-1, 1].
- **R3（unwanted）**: If one source yields no samples (例: 無音/取得失敗), then the system shall still produce the other source's audio (合成は欠損側を無音として扱う).
- **R4（state）**: While not on Windows, the mix source shall be unavailable (Phase 0と同じ制約。Linux同時取得は後続)。
- **R5（ubiquitous・回帰不変）**: The system shall keep single-source (mic-only / loopback-only) recording behavior unchanged.

## BDD 例

```gherkin
Feature: マイク＋システム音の同時取得

  Scenario: 同時取得して合成する (R1,R2)
    Given 録音ソースに「マイク＋システム音」を選択
    And マイクに発話があり、システム音(既定出力)に相手の声が再生されている
    When 録音→停止する
    Then 両者を16k monoへ各々変換し加算合成した音声が文字起こしされる

  Scenario: 片側無音でも他方は残る (R3)
    Given システム音が無音
    When ミックス録音する
    Then マイク音声だけの文字起こしが得られる（エラーにしない）

  Scenario: 単一ソースは従来どおり (R5)
    Given 録音ソースに「OS既定のマイク」を選択
    When 録音→停止する
    Then Phase 0以前と同一の挙動（マイクのみ）で文字起こしされる
```

## テストリスト（Canon TDD）

- [ ] `mix_16k(a, b)` 同長加算＋クリップ（純粋）
- [ ] `mix_16k` 長さ違い→長い方に合わせ、短い側は無音扱い（純粋・三角測量）
- [ ] `mix_16k` 加算がクリップ境界を超えない（[-1,1]）（純粋）
- [ ] 片側空→他方をそのまま返す（R3）（純粋）
- [ ] `Recording` が複数 `Capture` を保持し finish で全停止・合成（結合・Windows）

## 範囲・MVP境界

- MVP: ミックスは **既定マイク ＋ 指定/既定の出力デバイス**。プルダウンに1項目「マイク＋システム音」を追加（核心課題「リッチすぎず簡便」に配慮し最小限）。
- 後続: 任意マイク×任意出力の組合せ選択 / Linux同時取得 / 個別音量調整。**削除ではなく段階**（ADR-0006）。
