# ADR-0027: GPUビルド変種（CUDA）の opt-in 配布と実行バックエンドの可視化

- Status: Accepted（Deciders 承認 2026-07-09「起案および実装、テスト、リリースを進めてください」）
- Date: 2026-07-09
- Deciders: Takenori Kusaka
- 関連: [ADR-0012 マルチアーキ/SIMD配布](0012-windows-multiarch-multisimd-distribution.md)（CPUガード/変種設計を流用）/ [ADR-0025 日本語既定モデル](0025-japanese-default-model-revision.md)（速度データ要訂正）/ #600 / #569
- 一次情報: [GPU/実行オプション調査](../research/gpu-backend-options.md)（deep research 106エージェント＋実機実証）/ [効率化限界調査](../research/transcription-efficiency-limits.md)

## 背景・課題

実測（Ryzen 7 3700X・AVX2ビルド確認済み・実録音16:44）で、**32層エンコーダ級モデル（large-v3-turbo/kotoba）の CPU 実行は RTF≈11-16**（16.7分録音に約3時間）と確定した。これは反復暴走でもチューニング不足でもなく素の計算量であり、CPU側の改善（entropy/VAD/スレッド/チャンク並列）では桁が変わらない。ADR-0025 に記録された RTF 値（base 0.13/kotoba 0.90/turbo 1.15）は約10倍以上乖離しており、速度面の意思決定基盤として誤りだった。

一方、**同一マシンの RTX 4060 で CUDA 変種を実ビルド・実測した結果、16.7分録音が 196分→5.48分（36倍・RTF 0.33、品質・末尾到達同等）**。vendored whisper-rs-sys 0.13.1 の `cuda` feature がそのまま動作する（#569 不要）ことも実証済み。コア価値「思考を止めない」にとって、GPU 搭載機での turbo 実用化は決定的な体験改善である。

調査の要点（[研究doc](../research/gpu-backend-options.md)）:
- ggml の実行時バックエンドロード（GGML_BACKEND_DL）は公式に存在するが whisper-rs が全バージョン未対応＝「1配布物の自動選択」はカスタム sys 投資が必要（長期）。
- 現実解は**ビルド変種の出し分け**。CUDA は cudart/cublas/cublasLt の DLL 同梱が必要だが **CUDA EULA Attachment A で再配布可・MIT 配布と両立**（NVIDIA 通知文が必要）。
- Vulkan 変種はベンダー横断・DLL 同梱不要だが、GPU 無し環境でのフォールバック挙動が未確認。

## 決定

1. **CUDA ビルド変種を opt-in 配布物として追加する**（Windows x64・`QuickScribe_<ver>_x64-cuda-setup.exe`）。既定配布は従来どおり CPU 版（全マシンで動く安全側）。
2. **Cargo feature `cuda`** を追加（`whisper-rs/cuda` へ委譲）。コードは共通・ビルドフラグのみで分岐（ADR-0012 の決定的ビルド思想を踏襲）。
3. **実行バックエンドの可視化**: `stt_backend` コマンド（`"cuda"|"cpu"`）を追加し、設定「このアプリについて」に表示。どの変種での実行結果かを共有可能にする（バージョン表示 #625 と同趣旨）。
4. **CUDA 版の同梱と法的対応**: cudart64/cublas64/cublasLt64 の DLL をインストールディレクトリ直下に同梱。THIRD-PARTY-NOTICES に NVIDIA CUDA Runtime の帰属節を追記（EULA 準拠: SDK単体配布禁止/実質機能/自アプリのみアクセス/通知文）。
5. **updater の分離（Phase1 は自動更新なし）**: CUDA 版は `createUpdaterArtifacts=false`＋updater エンドポイントを変種専用 URL（`latest-cuda.json`・当面未発行）に向け、**CPU 版へ誤って自動更新される事故を構造的に防ぐ**。CUDA 版の更新は当面手動 DL（リリースページ案内）。
6. **検証経路**: `test-build.yml` に `cuda` 入力を追加し、リリース前に CI で CUDA 変種のビルド成立を検証できるようにする。

## 段階実装（ADR-0006: 削らず段階で全部やる）

- **Phase 1（本ADR・実装）**: CUDA 変種の release ジョブ＋feature＋バックエンド可視化＋NVIDIA 通知＋test-build 検証入力。インストーラ命名で「NVIDIA GPU（ドライバ）必須」を明示（nvcuda.dll はドライバ由来のため、非搭載機では起動不可）。
- **Phase 2**: NSIS プレインストールフックで NVIDIA ドライバ（nvcuda.dll）検出→非搭載機に警告・中止（ADR-0012 Phase2 の CPUガード設計を流用）。updater custom target（`windows-x86_64-cuda`）で変種内自動更新を有効化。
- **Phase 3**: Vulkan 変種（ベンダー横断・DLL 不要）。GPU 無し環境でのフォールバック挙動の実機検証をゲートに。
- **Phase 4（長期・要ゲート）**: GGML_BACKEND_DL による1配布物化（カスタム sys クレート投資。Vulkan/CUDA 変種の運用負荷が顕在化したら着手判断）。

## 結果・トレードオフ

- **Pro**: GPU 搭載機で turbo が実用化（36倍実測）。コード共通・feature 分岐のみで保守負荷最小。EULA/MIT 両立を確認済み。既定（CPU版）は無変更＝リスクゼロ。
- **Con/リスク**: (1) CUDA 版は NVIDIA 専用（非搭載機で起動不可）→ 命名と案内で明示、Phase2 でガード。(2) リリース CI に CUDA Toolkit 導入＋カーネルコンパイル（+30〜50分）。(3) 配布サイズ増（CUDA DLL 数百MB級）。(4) Phase1 は CUDA 版の自動更新なし（手動 DL）。
- **却下案**: 実行時自動選択（whisper-rs 未対応・カスタム sys 投資は時期尚早）/ CUDA を既定（非搭載機で全滅）/ CPU チャンク並列での高速化（桁が変わらず、境界品質リスク）。

## 検証

- 実機（RTX 4060）: 実録音 16:44 を 5.48 分・末尾[16:38] 到達・反復なし（2026-07-09）。
- ビルド再現手順は[研究doc](../research/gpu-backend-options.md) D3 節（Ninja 生成系・vcvars・stale CMakeCache 削除）。
- CI: test-build(cuda) でビルド成立を、リリースで成果物添付を検証。

## 反証・見直し条件

- Vulkan 変種が GPU 無し環境で安全にフォールバックすると実証されたら、Vulkan を優先変種に昇格（DLL 同梱不要のため）。
- whisper-rs が GGML_BACKEND_DL / GPU feature の実行時選択に対応したら Phase 4 を前倒し。
- CUDA EULA 改定時は再確認。
