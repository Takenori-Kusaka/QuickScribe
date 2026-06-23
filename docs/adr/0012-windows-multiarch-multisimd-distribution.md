# ADR-0012: Windows配布アーキテクチャ（マルチアーキ x64/ARM64 ＋ 複数SIMD ＋ CPUガード）

- Status: Accepted（段階実装中。Phase 0 実装済み / Phase 1・2 はゲート付き）
- Date: 2026-06-23
- Deciders: Takenori Kusaka
- 関連: [ADR-0006 スコープ規律], [ADR-0007 調査規律], [ADR-0008/0010 配布・法的MUST]
- 一次情報: docs/research/sources/whisper-cpu-portability.md ほかdeep research 4本

## 背景・課題

`whisper-rs = 0.14.4`（whisper-rs-sys 0.13.1, 同梱whisper.cpp/ggml）は **`GGML_NATIVE=ON`（=-march=native）** でビルドし、命令セットが**ビルドランナーのCPUに依存（非決定的）**。AVX512搭載/CPUID誤申告ランナーでビルドされると配布バイナリにAVX512が混入し、**非対応CPUの実ユーザーで実行時SIGILL（不正命令）**＝**配布バグ**になる（#349のCIで顕在化）。
あわせて、対象を **Windows x64＋ARM64** に広げ、SIMD最適化版を出し分け（**b2**: 複数SIMDビルド＋インストーラCPUガード）たい（ユーザー要望）。

## 決定

1. **whisper(ggml)は決定的なAVX2ベースラインでビルドする**（最優先・実装済み）。env では `GGML_*` を cmake に渡せない（build.rsが `WHISPER_*/CMAKE_*` のみ転送）ため、**published whisper-rs-sys 0.13.1 を `vendor/` に取り込み、build.rs に `config.define("GGML_NATIVE","OFF")` を注入し `[patch.crates-io]` で差し替える**。AVX2/FMA/F16Cは既定ON・AVX512はOFF（MSVCで `/arch:AVX2`＝zmm無しが構造保証）。`QS_SIMD=avx512` で `GGML_AVX512=ON` のSIMD最適化版も同1箇所で切替可能。
2. **AVX512回帰ゲート**をCI/releaseに常設。成果物に AVX512(zmm) が混入、または `GGML_NATIVE≠OFF` を検出したらビルドを失敗させ「壊れた配布物が黙って出る」のを防ぐ。
3. **マルチアーキ・複数SIMD・CPUガードは段階実装**（ADR-0006: 削らず段階で全部やりきる）。各Phaseに**ゲート（実験で正当化）**を設ける。

## 段階実装

- **Phase 0（実装済み / v0.2.3）**: 決定的AVX2（x64）。`vendor/whisper-rs-sys-0.13.1` + `GGML_NATIVE=OFF` patch + 回帰ゲート。**元のSIGILL非決定性の根本解**。
- **Phase 1（ARM64）**: x64ランナー(`windows-latest`にARM64 MSVCツール同梱)で `aarch64-pc-windows-msvc` をクロスビルド。updaterは `windows-aarch64` キー（tauri-actionが同一tagの latest.json に統合）。
  - **ゲート（スパイク）**: `whisper-rs` のWindows ARM64ビルドは**未確認・失敗濃厚**（issue#182でC1001 ICE、whisper-rsはarchived）。**まずクロスビルドのスパイクでICE可否を確認**。通れば実装、通らなければ維持されたバインディング移行/自前ARM64 DLLを評価。ring(rustls)のclang要件はLIBCLANG_PATH流用で対応。
- **Phase 2（AVX512版＋CPUガード）**: `QS_SIMD=avx512` でAVX512版を別ビルド。NSIS PREINSTALLフックで起動時にCPU判定→不適合なら**インストールせず警告・誘導して中止**（AVX2判定=CPUFeaturesプラグイン、AVX512判定=判定用ヘルパーexe同梱、ARM64=`IsNativeARM64`要確認）。updaterは**custom target**（`windows-x86_64-avx512`等）＋ビルド時SIMDタグで自分の版だけ更新取得。passive更新時は `${If} ${Silent}` でメッセージ抑制。
  - **ゲート（VOI）**: **AVX2 vs AVX512 のwall-clock実測**でゲインを確認（throttleで逆効果の例あり、コア価値は速度限界最適化でない）。有意なら実装、無ければ見送り。

## 結果・トレードオフ

- 同期ureb方針・既存NOTICE同梱設計は不変。Phase 0で配布安全性を即確保。
- `vendor/whisper-rs-sys-0.13.1`（7.1MB・自己完結）をリポジトリに取り込む（外部fork不要・0.13.1を正確再現・上流archivedで変更追従ほぼ不要）。ライセンスは Unlicense+MIT で不変（deny.toml許可リスト変更なし）。NOTICE/透明性として「whisper-rs-sysのbuild.rsを改変」明記が望ましい。
- ARM64/AVX512は実験ゲートで投資前に検証＝無駄打ち回避。

## 反証条件
- 上流whisper-rs後継が `GGML_*` env転送/SIMD featureを実装したら自前patchを破棄して乗り換え。
- ARM64クロスビルドのICEが現行VS2022ツールで解消していればPhase 1は即実装可。
- AVX512実測ゲインが無ければPhase 2のAVX512版は作らない（CPUガード機構はARM64/将来用に温存）。
