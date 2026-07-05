# ADR-0013: システム音声ループバックと「録音ソース」抽象の統一（S1.3）

- Status: Proposed（段階実装。Windows ループバックは実装済み／Linux の PipeWire monitor 経路は未着手）
- Date: 2026-06-24
- Deciders: Takenori Kusaka
- 関連: [ADR-0006 スコープ規律], [ADR-0007 調査規律], [ADR-0008 ライセンス/配布], [ADR-0012 Windows配布]
- 一次情報: [docs/research/sources/system-audio-loopback.md](../research/sources/system-audio-loopback.md)（deep research 2本）／問い設計: [s1-3-loopback-question-design.md](../research/s1-3-loopback-question-design.md)
- 対象 Issue: #19（Epic E1 #8）

## 背景・課題

S1.3 は当初「仮想デバイス不要でシステム音声（既定スピーカー）を録る」だった。
**ユーザー再枠組み（2026-06-24）**: 録音対象を**マイクの一種として選べる**ようにし、**既定スピーカーだけでなく任意のスピーカー(出力)デバイスのループバックも選択肢に出す**。
→ 真に解く問題は「ループバック単機能」ではなく **「録音ソース（入力マイク／出力ループバック）の抽象・列挙・選択をS1.2のデバイス選択UIに統一する」**こと。実装は VC/録画OSS・競合の prior art に倣い、独自実装に陥らない（ユーザー指示）。

## 決定

1. **録音ソースを統一抽象にする。** `AudioCapture` のソースを種別付きで表現する:
   `SourceKind ∈ { Input(マイク), Loopback(出力デバイス) }`。列挙・選択・録音開始を同一インターフェイスで扱い、S1.2のデバイスドロップダウンに**種別ラベル付きで一列に**並べる（Audacity「(loopback)」モデル）。OS依存は実装の内側に隠す。

2. **Windows = レンダーエンドポイント・ループバック（方式a）を `wasapi` クレートで実装。**
   - 「任意の出力デバイスを選んでループバック」は方式(a)に直結。`cpal` は当該機能を提供できない（既定出力のみ・デバイス選択不可: cpal #476/#251）。
   - **手書きFFIは避け、`wasapi`(HEnquist, MIT, active, CamillaDSP採用)** を Windows限定依存として追加。`Direction::Render` 列挙→選択→`initialize_client(Capture, Shared)` でループバック。
   - **マイクは現行 `cpal` 0.15 のまま**（変更なし）。

3. **Linux = sink の monitor 取得。経路はコピーレフトフリーを優先。**
   - **`pipewire` クレート(MIT)** か **cpal ≥0.18 ネイティブPipeWireホスト**。`libpulse-binding`(LGPL-2.1+)は動的リンク＋表示同梱を確認しない限り回避（ADR-0008整合）。
   - **ゲート（スパイク・最重要）**: cpal の PipeWire/Pulse ホストが **sink毎の `.monitor` を個別列挙するか**を**実機検証**してから経路確定（cpal #1133＝既定のみの懸念。信用せず実証）。個別列挙できれば依存を増やさない。できなければ `pipewire` クレート直叩き。

4. **無音ハンドリングを設計に織り込む（ジャーナル用途の要）。**
   ループバックは無音時にパケットが来ず、`AUDCLNT_BUFFERFLAGS_SILENT` も発生する。**タイムラインは壁時計基準で管理し、ギャップは無音(ゼロ)パディング**。`GetNextPacketSize==0` で無限ブロックしない。

5. **ループバック出力（48kHz/float/stereo等）は既存リサンプラでマイク経路の形式へ揃え**、文字起こしパイプラインを単一化。

6. **スコープは削らず段階実装（ADR-0006）。** 最終ゴール（mic／任意出力ループバック／mic＋システム同時）は不変。リスクはフェーズとスパイクで吸収する。

## 段階実装

- **Phase 0（MVP / 単一ソース選択）**: ソース抽象 `SourceKind` 導入。**Windows: `wasapi` で出力デバイス選択ループバック**。Linux: monitor 取得経路をスパイクで確定し実装。UIは S1.2 ドロップダウンに「マイク／システム音(各出力)」を種別ラベル付きで統合。受入基準(BDD/EARS)→TDD→CI(E2E含む)。
  - **ゲート**: Linux の per-sink monitor 個別列挙スパイク（cpal #1133）。
- **Phase 1（mic＋システム音 同時取得）**: 2デバイス同時キャプチャ＋クロックドリフト処理（リサンプル/タイムスタンプ整合）。対話まるごと（自分＋相手）を1ジャーナルに。
- **Phase 2（プロセス単位ループバック）**: 特定アプリのみ取り込み（通知音除外等）。`wasapi` の `new_application_loopback_client` を利用。Windows floor が build 20348 に上がる点を考慮（方式b）。

## 結果・トレードオフ

- **Pro**: prior art（Audacity/OBS/miniaudio/wasapi/PipeWire）に整合。独自実装を回避し保守負荷を下げる。コア価値（対話の振り返り）を Phase 0 で貫通。
- **Con/リスク**: Windows依存が `cpal`＋`wasapi` の2系統になる（`windows-rs` バージョン整合に注意）。Linux経路は実機スパイク結果に依存。同時取得のドリフトは Phase 1 の技術リスク。
- **却下案**: ① cpal一本化 → 出力デバイス選択ループバック不可で却下（将来cpalが対応すれば再検討）。② miniaudio-rs → 放置・Loopback WASAPI限定で依存先に不適（設計参照のみ）。③ 仮想オーディオデバイス前提 → ユーザー要件（仮想デバイス不要）に反し却下。

## 「考えが変わる条件」（反証）

- cpal が**列挙した任意 eRender デバイスでの input(loopback) stream 構築**を安定提供したら → `wasapi` 依存を外し cpal 一本化を再評価（cpal #476/#251 を監視）。
- Linux スパイクで cpal が **per-sink monitor を個別列挙できない**と判明 → `pipewire` クレート直叩きへ。
- 製品要求が「特定アプリのみ取り込み」へ移れば → 主機構を方式(b)プロセスループバックへ繰り上げ。
