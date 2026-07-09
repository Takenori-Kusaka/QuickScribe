# GPU/実行オプション拡充 調査（あらゆるPC環境対応・CPU並列・配布両立）

> 対象: あらゆるPC環境（NVIDIA/AMD/Intel GPU〜CPUのみ）に合わせてバックエンドを選べる「GPU/実行オプション拡充」の実現方法＋CPUのみ環境の並列処理オプション。
> 方法: ADR-0007 問い設計 → deep research（106エージェント・一次ソース・3票敵対的検証、23クレーム確定）。
> 実行日: 2026-07-09。背景: 実機(Ryzen 7 3700X + RTX 4060 8GB)で turbo q5 CPU実行が RTF≈10-16。
> 関連: [効率化限界調査](transcription-efficiency-limits.md) / [turbo高速化調査](turbo-speedup-question-design.md) / ADR-0012。

## 実測の確定値（このマシン・60秒スライス統一ベンチ）

| モデル | エンコーダ | RTF（CPU実測） | 備考 |
|---|---|---|---|
| base | 6層 | 1.95 | 変動あり（全録音実測では ~0.5-0.9） |
| kotoba-q5 | 32層(large級) | **14.66** | ADR-0025 の「0.90」と約16倍乖離 |
| large-v3-turbo q5 | 32層(large級) | **16.32**（全録音では11.7） | ADR-0025 の「1.15」と約10-14倍乖離 |

**結論: 32層エンコーダ級（turbo/kotoba/large）はこの CPU で RTF≈11-16 が素の実力。ADR-0025 に記録された RTF 実測値は全て約10倍以上誤っており、要改訂。**

## D1: 「1配布物が実行時に自動選択」は現状の whisper-rs では不可

- ggml 本体には **GGML_BACKEND_DL**（バックエンドを DLL/so 化し `ggml_backend_load` で実行時ロード）が公式に存在（llama.cpp PR #10469, 2024-11, メンテナ slaren 作。docs/build.md「同一バイナリを異なるGPU構成で使い回せる」）。
- **しかし** vendored whisper-rs-sys 0.13.1 は GGML_BACKEND_DL に一切未対応（build.rs に参照なし・GGML_* env 注入不可・ggml-backend.h 非公開＝ロードAPIがRustに露出しない）。whisper.cpp README も言及なし。
- **「CPU自動フォールバック込みの全自動選択」クレームは 0-3 で棄却**＝動的DL方式でもアプリ側の明示的ロード/選択ロジックが必要。
- → 長期の「1配布物」化は**カスタム sys クレート開発**（独自 bindgen + cmake + ロードAPI公開）が要る。#569 とは独立の投資。

## D2/D5: 現実解は「ビルド変種の出し分け」（ADR-0012 Phase2 の延長）

- **vendored 0.13.1 に `cuda`/`vulkan` の Cargo feature が既に存在**し、GGML_CUDA/GGML_VULKAN の cmake define にマップされる（**#569 は必須でない**。upstream whisper-rs 0.16 の GPU feature を使う場合のみ #569 が前提）。
- **Vulkan 変種**: ベンダー横断（NVIDIA/AMD/Intel）・前提はグラフィックスドライバの Vulkan 対応のみ・**DLL同梱不要**。ビルドに VULKAN_SDK が必要。GPU無し環境でのフォールバック挙動は一次未確認（実機検証要）。
- **CUDA 変種**: cudart/cublas/cublasLt の**DLL同梱が必要**（ドライバのみでは動かない。Windows に static cublas が無いため静的リンクでも回避不可）。**CUDA EULA Attachment A で再配布可・MIT配布と両立**（条件: SDK単体配布禁止/実質機能/自アプリのみアクセス/NVIDIA通知文＝サードパーティ表記）。
- 「CUDA+Vulkan 同梱静的ビルド＋実行時 --device 選択」は whisper.cpp/whisper-rs 文脈で未確認（1-2 棄却）。

## D3: RTX 4060 の実測 RTF（2026-07-09 実機で確定）

**vendored 0.13.1 の `cuda` feature でエンドツーエンドにビルド・動作を実証**（#569 不要を実機確認）。
実録音 rec-20260708-184059.opus（16:44・日本語独り語り）・large-v3-turbo q5・#600チャンク化：

| 実行 | 所要 | RTF | 末尾到達 |
|---|---|---|---|
| CPU (Ryzen 3700X) | 196.3分 | 11.7 | [16:38] ✓ |
| **GPU (RTX 4060, CUDA)** | **5.48分** | **0.33** | [16:38] ✓ |

- **約36倍の短縮**。品質同等（transcript 行数・末尾到達とも同等）。VRAM 8GB で問題なし（`using device CUDA0` 確認）。
- 60秒スライス単体は173.8s＝**固定オーバーヘッド（モデルGPU転送+CUDA初期化）が約2.5分**を占め、チャンクあたりの限界速度は ~3.4s/20sストライド（**限界RTF≈0.17**）。短い録音ほど固定費が支配的、長い録音ほど RTF→0.17 に漸近。
- ビルド手順（Windows）: CUDA Toolkit 12.8 + **Ninja 生成系**（`CMAKE_GENERATOR=Ninja`・vcvars64 環境・`CMAKE_GENERATOR_INSTANCE` 除去）で VS統合ファイル不要。stale CMakeCache（VS生成系の残骸）は削除が必要。ビルド約20分。
- Vulkan 変種の実測は未実施（VULKAN_SDK 未導入）。CUDA 実証により「GPU で桁が変わる」は確定。

## D4: CPU並列処理オプション

- whisper.cpp には**組込チャンク並列**（CLI `-p` / C API `whisper_full_parallel`）が存在し、**vendored 0.13.1 の bindings にも露出**（FFI で今日呼べる）。
- **重大な限定**:
  1. メンテナ ggerganov 自身が「**分割点の文字起こしは部分語・文脈喪失で悪化しやすい**」と明言（discussion #403）＝ニュアンス保持のコア価値と対立。※QuickScribe の #600 チャンク化は overlap＋担当区間で境界品質を守っており、素朴分割の `whisper_full_parallel` よりこの点で優位。並列化するなら **#600 の自前チャンクを複数 state で並列**が筋。
  2. whisper-rs 安全ラッパーは `whisper_full_parallel` を非公開（unsafe 直呼びか自前実装）。
  3. 同一 context からの並列実行はスレッド安全性に注意（whisper.h 注記）。
- **チューニング規則**: `並列数 × スレッド数 ≤ 物理コア数`（メンテナ提示）。
- **スレッド数最適点**: 同一CPU(Ryzen 3700X)の一次計測で **6-7スレッドがピーク、それ以上（SMT域）は劣化**（issue #200・2022計測のため現行ggmlでは要再計測。方向性「物理コア以下」は頑健）。現行実装の `num_cpus::get_physical()`=8 はほぼ適正、6-7 の再計測余地あり。
- 「2チャンク×4スレッド > 1チャンク×8スレッド」の一次ベンチは**未発見**＝実装するなら自前ベンチ（品質劣化の定量込み）が必要。
- **ただし**: CPU並列で稼げてもせいぜい数十%〜2倍で、**RTF≈15 → 実用域(≤1)への桁の改善は不可能**。桁を変えるのは GPU のみ。

## 推奨（統合）

1. **短期: Vulkan/CUDA のビルド変種を opt-in で出す**（ADR-0012 Phase2 延長・新ADR起案）。まず実機(RTX 4060)で CUDA 実測 → 桁が変わるか確定（実測中）。
2. **配布設計**: CPU版（現行・既定）＋ Vulkan版（ベンダー横断・DLL不要）＋ 必要なら CUDA版（NVIDIA最速・DLL同梱+EULA表記）。インストーラ/updater は ADR-0012 Phase2 の CPUガード設計（custom target）を流用。
3. **CPU並列オプション**: 桁が変わらないため優先度低。実装するなら #600 チャンクの並列度N（opt-in・品質検証込み）として Phase4（#621）に統合。
4. **長期**: GGML_BACKEND_DL による1配布物化はカスタム sys 投資と引き換えに可能。Vulkan/CUDA 変種の運用負荷が問題化したら再検討。
5. スレッド数の 6-7 最適点は現行 ggml で再計測の価値あり（低コスト・数%〜十数%）。

## caveats

- 最大の欠落は D3（RTX 4060 実測）→ 実機実測で確定させる（進行中）。
- Vulkan 版の GPU無し環境での挙動（クラッシュ/エラー/フォールバック）は実機検証必要。
- vendored 0.13.1 の cuda/vulkan feature は「存在しcmakeにマップ」までが確認済み。QuickScribe の vendor構成（path patch＋whisper.cpp 1.7.4）でエンドツーエンドにビルド・動作するかは実測中。
- whisper-rs の正規リポジトリは **Codeberg に移転済み**（GitHub master は 0.14.3 で凍結ミラー）。今後の一次情報は Codeberg 側。
- CUDA EULA は改定されうるため、CUDA 変種リリース時に再確認。

## 一次ソース（主要）

- GGML_BACKEND_DL: https://github.com/ggml-org/llama.cpp/pull/10469 / https://github.com/ggml-org/llama.cpp/discussions/12821
- whisper.cpp Vulkan（Cross-vendor）: whisper.cpp README (GGML_VULKAN=1)
- whisper-rs cuda/vulkan feature: https://docs.rs/crate/whisper-rs-sys/0.13.1/source/build.rs / vendor/whisper-rs-sys-0.13.1/
- CUDA EULA Attachment A（cudart/cublas 再配布可）: NVIDIA CUDA Toolkit EULA
- whisper_full_parallel と品質劣化: https://github.com/ggml-org/whisper.cpp/discussions/403
- Ryzen 3700X スレッドスケーリング: https://github.com/ggml-org/whisper.cpp/issues/200
