# whisper-rs(ggml) のCPU命令セット移植性とSIGILL（一次情報・deep research）

調査日: 2026-06-23。ADR-0007準拠。
**重大**: 配布バイナリが特定CPUの実ユーザーで **SIGILL（不正命令）** を起こし得る配布バグ。

## 症状
`whisper-rs = "0.14.4"`（→ whisper-rs-sys 0.13.1, 同梱whisper.cpp/ggml）で、**新規ビルドしたバイナリが一部CPUで実行時SIGILL**（ggmlのSIMD命令）。#349のCI(transcribe)で顕在化。mainは過去ビルドのキャッシュに乗って隠蔽していた。

## 根本原因（一次情報で確定）

- **whisper-rs-sys 0.13.1 の build.rs は `GGML_NATIVE`/`GGML_CPU_ALL_VARIANTS` を一切 define していない**（GPU系 `GGML_CUDA`/`GGML_VULKAN` 等のみ）。CPU命令セットを制御する Cargo feature も無い。
  出典: <https://docs.rs/crate/whisper-rs-sys/0.13.1/source/build.rs> / <https://docs.rs/crate/whisper-rs-sys/0.13.1/source/Cargo.toml>
- よって ggml 上流の CMake 既定が支配: **`GGML_NATIVE` 既定 ON**（= 実質 `-march=native`、ビルドマシンCPU向け最適化）、**`GGML_CPU_ALL_VARIANTS` 既定 OFF**。
  > `option(GGML_NATIVE "..." ${GGML_NATIVE_DEFAULT})` / `GGML_NATIVE_DEFAULT` はクロスコンパイル or `SOURCE_DATE_EPOCH` 定義時のみ OFF、それ以外 ON。
  > `option(GGML_CPU_ALL_VARIANTS "... (requires GGML_BACKEND_DL)" OFF)`
  出典: <https://github.com/ggml-org/llama.cpp> `ggml/CMakeLists.txt`
- → **リポジトリのコメント（「whisper-rs-sysはGGML_CPU_ALL_VARIANTSで実行時選択」）は誤り**。実際は `-march=native`。
- CIランナー(AVX512搭載/仮想化でCPUID誤申告)でビルド → AVX512命令を埋め込む → AVX512非対応CPUの実ユーザー(またはCPUID誤申告環境)で **SIGILL**。
  > Sapphire Rapids: "report AVX-512 in CPUID but have it disabled at the hypervisor level" → SIGILL。`-DGGML_NATIVE=OFF -DGGML_AVX512=OFF` で回避、性能影響は negligible。
  出典: <https://haitmg.pl/blog/cloud-run-sigill-avx512-llama-cpp/>
  > whisper.cpp #2928「forces building with AVX512 and silently fails on older computers due to illegal instructions」。FindSIMD.cmake が AVX512 を強制することがある。
  出典: <https://github.com/ggml-org/whisper.cpp/issues/2928>

## 修正の制約（重要）
build.rs の env パススルーは **`WHISPER_*` と `CMAKE_*` のみ**を `config.define(key,value)` する:
> `let is_whisper_flag = key.starts_with("WHISPER_") && key != "WHISPER_DONT_GENERATE_BINDINGS"; let is_cmake_flag = key.starts_with("CMAKE_"); if is_whisper_flag || is_cmake_flag { config.define(&key,&value); }`
- **`GGML_*` env は無視される** → `GGML_NATIVE=OFF` を env で直接渡せない。`CMAKE_C_FLAGS` 経由でも `-march=native`(GGML_NATIVE由来)を打ち消せない(後勝ち)。
- `GGML_CPU_ALL_VARIANTS` は `GGML_BACKEND_DL`(共有ライブラリ)を要し、whisper-rs-sysの `BUILD_SHARED_LIBS=OFF`(静的)と衝突 → 現行では非現実的。

## 採る方針: 保守的AVX2ベースライン（NATIVE無効＋AVX512無効）

NATIVEを切りAVX2上限にすれば、2013年Haswell以降の実質全x86実機で安全に動き、AVX512由来SIGILLを根絶（性能影響は実用上わずか）。

### 手段の優先順位（env→fork の二段）
1. **`SOURCE_DATE_EPOCH` env**（最小・fork不要）: ggmlのCMakeListsが直接 `ENV{SOURCE_DATE_EPOCH}` を読み `GGML_NATIVE_DEFAULT=OFF` にする（build.rsのパススルー制約を回避できる唯一のenvレバー）。NATIVE=OFFで -march=native が消える。
   - リスク: FindSIMD.cmake がAVX512を強制する可能性(#2928)。**CIの cmake構成ログ/コンパイルフラグで -march=native と AVX512 が消えたか検証必須**。
2. **`[patch.crates-io]` で whisper-rs-sys を fork し build.rs に明示注入**（確実）:
   `config.define("GGML_NATIVE","OFF"); ("GGML_AVX","ON"); ("GGML_AVX2","ON"); ("GGML_FMA","ON"); ("GGML_F16C","ON"); ("GGML_AVX512","OFF");`
   - env策が不十分なら採用。fork保守コストあり。

### 検証
- CIの whisper ビルドログで `-march=native` が無く、AVX512命令を出していないことを確認（cmake configure出力＋コンパイルフラグ）。
- rust-cache をバストして**新規ビルド**で transcribe が安定passすること。
- ※実機の非AVX512 CPUでの動作確認は最終的にインストール後（CIでは保証不能）。

## 残課題
- whisper-rs 0.15+ で build.rs がCPU featureを扱う可能性（fork不要化）→ アップグレード可否は別途調査。
- ターゲットCPU分布次第ではベースラインを `x86-64-v2`(SSE4.2)まで下げる検討。

## 主要ソース
- <https://docs.rs/crate/whisper-rs-sys/0.13.1/source/build.rs>（最重要・envパススルー）
- <https://github.com/ggml-org/llama.cpp> `ggml/CMakeLists.txt`（NATIVE既定ON）
- <https://github.com/ggml-org/whisper.cpp/issues/2928>（AVX512強制でSIGILL）
- <https://haitmg.pl/blog/cloud-run-sigill-avx512-llama-cpp/>（CPUID誤申告・回避レシピ）
- <https://github.com/ggml-org/llama.cpp/issues/5782>（-march=nativeがAVX512を有効化）
