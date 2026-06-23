# システム音声ループバック — 一次情報リサーチ（S1.3 #19）

> 調査日: 2026-06-24 / 問い設計: [s1-3-loopback-question-design.md](../s1-3-loopback-question-design.md)（[ADR-0007](../../adr/0007-research-question-framing-method.md)）
> deep research 2本（Windows班 / Linux＋横断班）の統合。全クレームに一次情報URL。

## Q1. Windows: WASAPIループバックの2方式

| 方式 | 取得単位 | 最低Win | 無音時 | 採用OSS |
|---|---|---|---|---|
| **(a) レンダーエンドポイント・ループバック**<br>`IAudioClient`(eRender)+`AUDCLNT_STREAMFLAGS_LOOPBACK` | **出力デバイス全体**（系統ミックス） | Vista（イベント駆動は1703+） | **パケット来ない**＋`AUDCLNT_BUFFERFLAGS_SILENT` | OBS "Audio Output Capture", Audacity(PortAudio WASAPI), miniaudio |
| **(b) プロセスループバック**<br>`ActivateAudioInterfaceAsync`+`AUDIOCLIENT_ACTIVATION_TYPE_PROCESS_LOOPBACK` | **プロセス(PID)単位** | Win10 build 20348 | 同様 | OBS "Application Audio Capture"(PR#5218), bozbez/win-capture-audio |

- Shared-modeのみ（exclusiveはループバック不可）。
- **ユーザー要件「任意のスピーカーを選んでループバック」は (a) に直結**。(b)はPID単位で用途違い。
- 出典: [MS Learn Loopback Recording](https://learn.microsoft.com/en-us/windows/win32/coreaudio/loopback-recording) / [Capturing a Stream（SILENTフラグ）](https://learn.microsoft.com/en-us/windows/win32/coreaudio/capturing-a-stream) / [AUDIOCLIENT_ACTIVATION_TYPE（min build 20348）](https://learn.microsoft.com/en-us/windows/win32/api/audioclientactivationparams/ne-audioclientactivationparams-audioclient_activation_type) / [OBS PR#5218](https://github.com/obsproject/obs-studio/pull/5218)

## Q2. Windows: 特定出力デバイスの列挙・選択

- `IMMDeviceEnumerator::EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)` で出力列挙 → 選択デバイスで `Activate(IAudioClient)` → `Initialize(SHARED, LOOPBACK, ...)`。**既定取得を `Item(i)` に変えるだけ**でデバイス選択が成立（ループバックは初期化したeRenderデバイスに紐づく）。
- 競合UX:
  - **Audacity（PortAudio WASAPIホスト）**: 録音デバイスのドロップダウンに各出力を **「Speakers (Realtek) (loopback)」** として列挙。**各スピーカーが録音ソースとして並ぶ**＝QuickScribeが倣うべきモデル。
  - **OBS**: "Audio Output Capture" がデバイスドロップダウン（既定＋各出力）。
- 出典: [OBS win-wasapi.cpp](https://github.com/obsproject/obs-studio/blob/master/plugins/win-wasapi/win-wasapi.cpp) / [CamillaDSP WASAPI backend（PortAudio流デバイス命名）](https://github.com/HEnquist/camilladsp/blob/master/backend_wasapi.md)

## Q3. Rust実装手段（独自実装回避の核心）

| クレート | ループバック | **特定出力デバイス選択** | メンテ | ライセンス |
|---|---|---|---|---|
| `cpal` 0.15（現行・mic用） | 既定出力のみ・実質不可 | **不可** | Active | **Apache-2.0（単一）** |
| `cpal` master | eRender時にフラグ付与のみ・列挙デバイスでのinput streamは非対応 | **不可**（#476/#251） | Active | Apache-2.0 |
| **`wasapi`(HEnquist) 0.23.0** | **可** | **可** | **Active（2026-04, 25 releases, CamillaDSP採用）** | **MIT** |
| `windows-rs` 生 | 可（自前実装） | 可 | Active(MS) | MIT/Apache-2.0 |

- **cpalは「任意の出力デバイス＋ループバック」を今は提供できない**（#476/#251）。最近 "WASAPI advanced interop"（`IMMDevice`露出）が入ったが、結局ループバックFFIは自前になる。
- **`wasapi`クレートが欠けたピースをそのまま安全Rustで提供**: `get_device_collection(&Direction::Render)`→`get_device_at_index`→`get_friendlyname`、`initialize_client(fmt, &Direction::Capture, Shared)` が内部で `AUDCLNT_STREAMFLAGS_LOOPBACK` を設定。将来の `new_application_loopback_client(pid, tree)`（=方式b）も用意。MIT・`windows=0.62`依存。
- **手書きFFIは不要**。
- 出典: [cpal #476](https://github.com/RustAudio/cpal/issues/476) / [#251](https://github.com/RustAudio/cpal/issues/251) / [cpal device.rs](https://github.com/RustAudio/cpal/blob/master/src/host/wasapi/device.rs) / [wasapi-rs](https://github.com/HEnquist/wasapi-rs) / [docs.rs/wasapi](https://docs.rs/wasapi) / [crates.io/wasapi](https://crates.io/crates/wasapi)

## Q4. Linux: PipeWire / PulseAudio monitor

- sink の **monitor ソース**取得が現行標準（PipeWire）。PulseAudio `.monitor` は旧ディストロ用フォールバック。
- **コピーレフト回避の推奨経路**: **`pipewire`クレート（MIT, libpipewire自体MIT）** ＝完全コピーレフトフリー。`libpulse-binding` は **LGPL-2.1+** を引き込むため、動的リンク＋LGPL表示同梱を確認しない限り避ける（[deny.toml](../../adr/0008-licensing-and-distribution.md)整合）。
- **要実証（最重要オープン項目）**: cpal の PipeWire/Pulse ホストが **sink毎の `.monitor` を個別に列挙するか**（既定のみの懸念 [cpal #1133](https://github.com/RustAudio/cpal/issues/1133)）。**これは信用せず実機検証してから決める**。cpal ≥0.18 のネイティブPipeWireホストで個別列挙できれば依存を増やさず済む。
- 出典: [cpal #1133](https://github.com/RustAudio/cpal/issues/1133) / [pipewire-rs](https://gitlab.freedesktop.org/pipewire/pipewire-rs) / PipeWire公式

## Q5. 横断: 「入力 vs ループバック」を1つの抽象/UIで扱う設計

- **miniaudio**: デバイス種別に `ma_device_type_loopback` を持ち、WASAPI/CoreAudio/Pulseで「ループバックデバイス」を列挙の一種として扱う＝**デバイス種別フラグで統一**するのが定石（**設計リファレンス**）。ただし Rustバインディング `miniaudio-rs` は **2020年以降放置・Loopbackは WASAPI限定**なので依存先にはしない。
- **UX定石**: Audacity流に **同一の録音ソースドロップダウンに、種別ラベル/バッジ付きで** マイクと出力ループバックを並べる（「System audio — Speakers (Realtek)」等）。OBSは別ソース型だが、ジャーナルアプリでは選択を1箇所に集約する方がコア価値（簡便さ）に合う。
- 出典: [miniaudio](https://miniaud.io/docs/manual/index.html)（loopbackデバイス種別） / Audacity マニュアル(WASAPI loopback)

## Q6. mic＋システム音の同時取得・クロックずれ

- 2デバイス同時取得は**別クロック**＝サンプルレート差・ドリフトを**リサンプル/タイムスタンプ整合**で吸収（OBSの音声ミキシング、miniaudio）。
- **MVP判断**: コアジョブ「対話の振り返り」は**単一ソース選択でも成立**（相手の声=ループバック、自分の声=マイク、を回ごとに選ぶ）。**同時取得はスコープから削らず後続フェーズへ分割**（ADR-0006: 段階実装、最終ゴール不変）。

## ライセンス補足（deny.toml整合）
- `cpal` = **Apache-2.0（単一。MIT/Apacheデュアルではない）** → allow-list確認。
- `wasapi` = MIT、`pipewire` = MIT。`libpulse-binding`(LGPL)は回避優先。

## macOS（将来・現対象外）
- システム音声タップは **ScreenCaptureKit / Core Audio taps（macOS 14.4+）** が必要（旧来の仮想デバイス不要化）。現対象はWin/Linuxだが、抽象設計時に種別フラグで拡張余地を残す。
