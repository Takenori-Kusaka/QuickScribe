# ADR-0029: オファリングの簡素化 — CUDA変種と kotoba モデルの廃止

- Status: Accepted（Deciders 承認 2026-07-10。「CUDAを廃止し真の一本化」「kotobaをカタログから削除」を明示選択）
- Date: 2026-07-10
- Deciders: Takenori Kusaka
- 改訂/廃止対象: [ADR-0027 GPUビルド変種(CUDA)](0027-gpu-build-variants.md)（**Superseded**＝CUDA変種を廃止）/ [ADR-0028 単一Vulkan](0028-single-vulkan-build-startup-gpu-detection.md)（決定#5「CUDAを任意extraとして存置」を**取消**）/ [ADR-0025 日本語既定モデル](0025-japanese-default-model-revision.md)（決定#2「kotoba-q5を降格して残す」を**取消**）
- 関連原則: [ADR-0006 スコープ完全性](0006-scope-completeness-policy.md)（本ADRはコア機能集合を縮小しない＝任意extra/劣位選択肢の除去。Deciders承認済み）/ CLAUDE.md コア価値「簡便さ（リッチすぎると簡便でなくなる）」

## 背景・課題

ADR-0028 で単一Vulkanビルドへ一本化しつつ、CUDA変種は「NVIDIA最速の任意extra」として存置した（決定#5）。また ADR-0025 は日本語既定を kotoba→turbo へ変えつつ kotoba-q5 を「朗読in-domain向け」として降格存置した（決定#2）。しかし利用者（Decider）の問いで、いずれも「価値の薄い選択肢を残すことが、掲げた一本化・簡便さの思想を崩している」と判明した。

1. **CUDAの上積みは僅少**。実測（RTX 4060）で Vulkan 6.10分 vs CUDA 5.48分＝**約11%差**。Vulkan は CPU〜全GPUを単一インストーラでカバーし自動更新も効く。CUDA を残すことは **2つ目の別インストーラ**（＝一本化の未達）に加え、NVIDIAドライバ前提・DLL同梱・EULA対応・CI +40分・保守を背負う。11%のためにこれらは見合わない。

2. **kotoba は本プロダクトのコア用途と正面衝突**。QuickScribe の用途＝自発的な一人語り（会話寄り）で、kotoba は会話CER 0.495（turbo 0.184 の約2.7倍悪化）＋長尺末尾欠落＝崩壊する。優位なのはクリーン朗読のみで、利用者は台本を朗読しない。加えて開発元は OSS 重み更新を約1年9か月停止。カタログに2エントリ（q5/フル）残すことは選択肢肥大でコア価値「簡便さ」に逆行する。

## 決定

1. **CUDA変種を完全に廃止する**。`release-cuda` ジョブ・`test-build` の cuda 入力/手順・`cuda` Cargo feature・`build.rs` の nvcuda DELAYLOAD・`system32_dll_exists`・`gpu_backend_available`/`stt_backend` の cuda 分岐・cuda/vulkan 排他 `compile_error!`・`tauri.cuda.conf.json`・`nsis-hooks-cuda.nsh`・NVIDIA ドライバ案内UX（`open_external`/`get_driver`/`NVIDIA_DRIVER_URL`）・cuda_manual_update・backend_cuda を全撤去。GPU配布は**単一Vulkanビルドのみ**。

2. **kotoba モデル（kotoba-q5・kotoba）をカタログから撤去する**。`MODELS` から両エントリと i18n ラベルを削除。既存ユーザーが kotoba を選択していた場合、`model_for` の未知ID→`MODELS[0]`（base）フォールバックで**安全に既定へ移行**する（クラッシュや空文字化なし）。日本語推奨は large-v3-turbo、頑健フォールバックは base。

3. **配布は「単一Vulkanインストーラ（Windows x64）＋Linux」に集約**。実行時にVulkanデバイスを検出し GPU/CPU を自動選択（ADR-0028）。インストール時の変種分岐なし・更新は通常の latest.json 一本。

## 結果・トレードオフ

- **Pro**: 真の一本化（1インストーラ・1更新経路）。NVIDIAドライバ案内/DLL同梱/EULA/CUDA CI/2つ目インストーラの保守が消滅。モデル一覧が用途に素直（base/turbo/英語向け）でコア価値「簡便さ」に合致。停滞OSS(kotoba)依存を解消。
- **Con/リスク**: (1) NVIDIA機で最速の約11%を諦める（大多数に無影響。Vulkanで実用速度は達成済み）。(2) クリーン朗読でのkotoba微優位を失う（本用途にほぼ非該当）。(3) kotoba選択中の既存ユーザーは次回起動で base 実行になる（末尾欠落なし・安全側。設定で turbo を選び直せる）。
- **却下案**: CUDAをextra存置（一本化未達・保守負担）／kotobaを降格存置（コア用途と矛盾・選択肢肥大）。

## 検証

- Rust: `stt_backend`＝"vulkan"|"cpu" のみ、`model_for("kotoba"/"kotoba-q5")`→"base" フォールバックを単体テストで固定。カタログに kotoba が無いこと・id一意を検証。
- フロント: GPUトグルは Vulkan変種のみ表示、GPU不可時はCPU案内のみ（入手ボタン無し）、About は "GPU版 Vulkan"|"CPU版"。i18n 4ロケール整合（backend_cuda/get_driver/cuda_manual_update/kotoba ラベル除去）。
- errcode: `open_external` 撤去後も E_UNSUPPORTED_FORMAT（record.rs）/E_FILE_MANAGER（open_vault等）は他所で使用継続＝孤児化なし。

## 反証・見直し条件

- Vulkan が特定GPU/ドライバで実用に耐えない事例が頻発し、かつ CUDA が有意に安定・高速と実証されたら、CUDA を限定再導入（要新ADR）。
- 会話でも崩れない日本語特化モデル（活発に保守されるもの）が現れたら、ADR-0024 の CER ベンチで評価しカタログ復帰を検討。
