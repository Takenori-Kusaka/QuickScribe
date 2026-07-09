# ADR-0028: 単一Vulkanビルドへの一本化と、起動時GPUデバイス検出

- Status: Accepted（Deciders 承認 2026-07-09「単一Vulkanビルドに一本化(推奨)」／配布一本化・実行時オプション化の明示指示）
- Date: 2026-07-09
- Deciders: Takenori Kusaka
- 関連/一部改訂: [ADR-0027 GPUビルド変種(CUDA)](0027-gpu-build-variants.md)（Phase3の昇格条件を発火・CPU既定/CUDA opt-in の配布方針を差し替え）/ [ADR-0012 単一バイナリ配布](0012-windows-multiarch-multisimd-distribution.md)（決定的CPUベースラインは維持）
- 一次情報: [GPU/実行オプション調査](../research/gpu-backend-options.md) / 本ADRの「検証」節（RTX 4060 実測・空ICD安全性テスト）

## 背景・課題

ADR-0027 は GPU を **opt-in のビルド変種（CUDA）** として足し、既定は CPU 版とした。その後の実測で方針を見直す2つの事実が出た。

1. **Vulkan は CUDA とほぼ同速で、はるかに軽い**。RTX 4060 実測で 16.7分録音が Vulkan 6.10分（RTF 0.364）／CUDA 5.48分（RTF 0.33）＝差 約11%。Vulkan は**ベンダー横断（NVIDIA/AMD/Intel）・専用DLL同梱不要・普通のGPUドライバのみ**で、CUDA の重荷（専用ドライバ≥528.33前提・数百MBのcudart/cublas同梱・EULA対応・別インストーラ）を負わない。

2. **「GPUを試して失敗したらCPUへ」という実行時フォールバックは原理的に不可能**。使えるGPUデバイスが無い状態で whisper.cpp に `use_gpu=true` を渡すと、GPU初期化が **C++例外で abort** し（実測 `fatal runtime error: Rust cannot catch foreign exceptions` / STATUS_STACK_BUFFER_OVERRUN 0xC0000409）、Rust側では捕捉できない。ADR-0027 が前提にした「DELAYLOAD＋実行時にGPU初期化失敗→CPU自動リトライ」は**成立しない**（`Err`が返らずプロセスごと落ちる）。

さらに利用者から**明確な要求**が出た ―「インストーラ/アップデータは同一であるべき。CPU/GPUのビルド差はインストール時にユーザへ分岐させるものではなく、技術事情のUX押し付けだ。選択は実行時オプションにすべき」。

## 決定

1. **既定配布を「単一の Vulkan ビルド」に一本化する**（Windows x64）。1ビルド＝1インストーラ＝1アップデータで、**CPU からあらゆるGPUまでを1つがカバー**する。CPU専用ビルドは独立配布物として廃止し、**Vulkanビルド内の自動フォールバック**に降格する。

2. **起動時にGPUデバイスを安全に検出し、有る時だけGPUを使う**（abort構造回避の要）。GPUを触る *前* に、Vulkanローダの安全な C API（`vkCreateInstance` + `vkEnumeratePhysicalDevices`・戻り値は `VkResult` で例外を投げない）で物理デバイス数を数える。**1台以上ある時だけ** `use_gpu=true`。デバイス0・ローダ不在・ドライバ不在・インスタンス生成失敗はすべて `use_gpu=false`（CPU実行）。実装は `ash` クレート（実行時dlopen）。既存の起動時環境認識フェーズ（`stt_backend` コマンド）に載せる。

3. **`vulkan-1.dll` を遅延ロード（DELAYLOAD）する**。whisper-rs-sys が静的インポートするため、ローダ未導入の最小Windowsでは起動時解決に失敗してEXEが起動不能になる。遅延ロードで起動を可能にし、デバイス検出（ash動的ロード）でデバイス0なら Vulkan APIを一切呼ばずCPU実行、デバイス有りの時だけGPU経路に入る（その時は vulkan-1.dll が在るので遅延解決が成功）。

4. **CPU/GPU の選択は実行時**：既定はGPU（起動時にデバイスがあれば自動選択）、設定トグルで明示OFF可（`use_gpu`）。インストール時に分岐させない。実行バックエンドは「このアプリについて」に `GPU版 Vulkan` / `CPU版` と可視化。

5. ~~**CUDA は任意 extra として温存（凍結）**。NVIDIAで最後の約1割の速度を絞りたい上級者向けの opt-in 配布物として残すが、既定ではない。~~ → **取消（[ADR-0029](0029-simplify-offering-drop-cuda-and-kotoba.md)）**。11%差では2つ目の別インストーラ＋保守が見合わず、CUDA変種を完全廃止し**単一Vulkanのみ**に集約した。

6. **CPU移植性は不変**（ADR-0012）。Vulkanビルドの CPU 実行経路も vendored sys の `GGML_NATIVE=OFF`＝決定的AVX2ベースラインのまま（`GGML_VULKAN=ON` は追加されるだけ）。

## 段階実装（ADR-0006: 削らず段階で全部やる）

- **P1（本ADR・実装）**: `ash` による起動時デバイス検出（`gpu_backend_available` を DLL存在チェック→実デバイス列挙へ）／`vulkan-1.dll` DELAYLOAD／フロント一般化（GPUトグルをvulkan対応・NVIDIA固有導線はCUDA限定・About表示）／i18n汎用化＋`backend_vulkan`／空ICDでの安全fallback統合テスト。
- **P2（CI・リリース）**: `test-build.yml` に vulkan 入力＋Vulkan SDK導入で検証グリーン化→ `release.yml` の Windows x64 を `--features vulkan`＋Vulkan SDK導入に切替（既定インストーラ＝Vulkan）。Linux/ARM64 の GPU 対応は後続（当面CPU）。
- **P3（運用）**: AMD/Intel/Linux 実機での Vulkan 実測、RTF統一計測基盤（ADR-0025 の10倍ズレ再発防止）。CUDA extra の CI 維持要否を運用負荷で判断。

## 結果・トレードオフ

- **Pro**: 配布が単一化（1インストーラ・1アップデータ）。CUDA の同梱DLL/EULA/専用ドライバ案内が既定から消える。GPU搭載機は自動で実用速度（RTF≈0.36）、非搭載機は自動でCPU。技術事情をUXに漏らさない。
- **Con/リスク**: (1) Vulkanは初回シェーダ実行時コンパイルの固定コストが大きい（短尺で顕著・実利用では償却）。(2) CIに Vulkan SDK 導入＋シェーダコンパイルでビルド時間増。(3) 事前検出が通っても、壊れたドライバが `vkCreateInstance` 自体で落ちる残存リスクはゼロにできない（プロセス分離は将来課題）。(4) バイナリサイズ微増（Vulkanバックエンド分・ただし外部DLLは無し）。
- **却下案**: 両ビルド同梱＋実行時選択（サイズ倍・Tauri非標準のNSIS自作・Vulkanの利点を捨てる）／GGML_BACKEND_DLの単一バイナリ動的ロード（whisper-rs未対応＝カスタムsys投資で時期尚早・ADR-0027 Phase4のまま）／CUDAを既定維持（非横断・重荷）。

## 検証

- 速度（RTX 4060）: フル16.7分録音 Vulkan 6.10分/RTF 0.364（CUDA 5.48分/CPU 196.3分）。デバイスは fp16・matrix cores 付きで認識。
- **安全fallback**: 空ICD（VK_DRIVER_FILES/VK_ICD_FILENAMES を存在しないパスへ）でデバイス0を擬似し、`gpu_backend_available()` が **abort せず false を返す** ことを実プロセスの統合テスト（`tests/gpu_detect_integration.rs`）で固定。回帰すればテストプロセスが STATUS_STACK_BUFFER_OVERRUN で落ちて検出される。
- 反例の確認: `use_gpu=true` を直接渡す旧経路はデバイス0で abort する（＝事前検出が必須である根拠）。

## 反証・見直し条件

- Vulkanドライバのバグで `vkCreateInstance` 段階の abort/クラッシュが実利用で頻発するなら、GPU検出・実行をサブプロセス分離してプロセスクラッシュを隔離する（ADR-0027 Phase4隣接）。
- whisper-rs が GGML_BACKEND_DL / 実行時バックエンド選択に対応したら、単一バイナリ動的ロードへ前倒し（CUDA/Vulkan/CPUを1配布物で実行時選択）。
- Linux/ARM64 で Vulkan の利得・安定性が確認できたら既定を拡張。
