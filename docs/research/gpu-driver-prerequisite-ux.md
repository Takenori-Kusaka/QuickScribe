# GPU変種のドライバ前提UX 調査（案内・自動導入の可否／最低ドライバ版／Vulkan代替）

> 対象: CUDA変種のGPU実行に必要な NVIDIA コンポーネント不足を、インストーラ/アップデータが検知して案内・導入できるか（.NET bootstrapper 相当が EULA上・技術上可能か）。
> 方法: ADR-0007 問い設計 → deep research（109エージェント・一次ソース26件・確定25クレーム。統合出力はシリアライズ不具合でプレースホルダ化したが、journal から実クレームを復元）。
> 実行日: 2026-07-09。関連: ADR-0027（CUDA変種）/ ADR-0012。

## 結論（ユーザーの3基準に対する回答）

ユーザーが挙げた受け入れ基準「①リンクを知らなくても誘導される ②なぜ必要か示せる ③不要ならスキップできる」は、**EULAの可否に関わらず「検出＋案内＋スキップ可」で完全に満たせる**（自動導入というEULAグレーな部分に踏み込む必要がない）。しかも研究で **NVIDIA自身の推奨が「ドライバDLページへ誘導」＝検出+案内**であることが確認できた。

### D1: ドライバの自動導入 — 技術的には可能だが、正道は「検出＋案内」
- **NVIDIA display driver は再配布不可**（CUDA EULA Attachment A に含まれない。cudart/cublas 等ランタイムのみ再配布可＝同梱済み）。→ **ドライバをアプリに同梱することはできない**。
- 技術的な連鎖起動自体は可能: Tauri NSIS は WebView2 bootstrapper を `ExecWait` で連鎖する実績があり、依存インストーラの `/passive /quiet /norestart` 同梱・サイレント実行パターンを docs が明示。WiX Burn も Chain/ExePackage で可能。
- **しかし**ドライバはハード固有で「単一の連鎖可能インストーラ」が無く、管理者権限・再起動を伴う。**NVIDIA の公式remedyは『ユーザーを公式ドライバDLページへ誘導して適切な署名ドライバを入れさせる』＝検出+案内**。→ **正道は検出+案内**（＝ユーザーの3基準そのもの）。

### D2: 最低ドライバ版 — 重要な訂正（570ではなく ≥528.33）
- CUDA 12.8 GA の**同梱ドライバ**は Windows ≥570.65。
- **しかし minor-version(前方)互換**により、CUDA 12.x でビルドしたアプリは **Windows ドライバ ≥528.33（12.xファミリ下限 ≥525系）** で動く。→ **QuickScribe のCUDA 12.8ビルドが要求する実下限は ≥528.33、570ではない**。
- forward-compatibility(cuda-compat) は **Linux＋データセンターGPU限定**で、Windows の consumer GeForce では使えない＝下限をこれ以上下げられない。
- **検出は nvcuda.dll の存在だけでは不十分**（古いドライバは存在チェックを通るが CUDA init で失敗）。`cudaDriverGetVersion()`（0=未導入 / それ以外=版）や NVML `nvmlSystemGetDriverVersion` / NVAPI で**版を取得**し、「未導入」「古い（更新して）」「OK」を出し分けるのが堅牢。※現状の CPU フォールバックで**動作の正しさは担保済み**。版チェックは**案内文の精度**の改善。

### D3: NSIS 実装 — Tauri のフックで実現可能
- Tauri v2 NSIS は `NSIS_HOOK_PREINSTALL/POSTINSTALL`（.nsh を src-tauri/windows に置き tauri.conf.json で参照）を提供。ここにドライバ検出＋案内ダイアログを実装できる。汎用の前提検知枠組みは無い（WebView2 のみ組込）ため**自作**する。ADR-0012 Phase2 の CPUガード設計と同じ土俵。

### D4: Vulkan 代替 — ★ドライバ問題を根本的に軽くする最有力候補
- whisper.cpp の **Vulkan バックエンドは CUDA と同等性能**（RTX 3060 で1時間音声を約2分）。**ベンダー横断**（NVIDIA/AMD/Intel）。**CUDA固有ランタイム不要・DLL同梱不要**。
- 必要なのは **Vulkan対応の一般グラフィックスドライバのみ**（現代のGPUドライバはほぼ全て Vulkan 対応）＝「CUDAドライバ ≥528.33」より**はるかに低い前提**。「専用ドライバ導入案内」の負担が最小化。
- Ollama も Vulkan バックエンドを提供（インストール時 既定ON）。
- トレードオフ: Vulkan変種ビルドは VULKAN_SDK が要る／GPU無し環境でのフォールバック挙動は要実機検証。

### D5: 実在アプリの前提UX
- **Ollama**: NVIDIA driver ≥550(一部570)を**要求として明示**、ドライバは同梱しない。Vulkanバックエンドも提供。
- **LM Studio 0.3.15**: ドライバ版を**検出**し、古ければ失敗でなく**CUDA 11へ優雅にダウングレード**（RTX50系の CUDA12.8 は driver ≥551.61）。
- 共通: **同梱せず・検出して要求を明示・不足は案内**。

## QuickScribe への実装含意（ユーザー3基準を満たす具体設計）

### 即実装（次リリース）— CUDA変種の案内強化
1. **検出を版対応に**: `nvidia_driver_present()` を、cudart 同梱の利点を活かし `cudaDriverGetVersion()` で版取得へ。0=未導入 / <12.x下限(≈CUDA 12010 相当) = 古い / OK の3値。※動作はCPUフォールバックで担保、これは案内精度向上。
2. **NSIS preinstall フック**（`src-tauri/windows/*.nsh`）: ドライバ未検出/古い時に **[ドライバを入手（公式DLページを開く）] / [このまま続行（CPUで動作）] / [キャンセル]** の3択ダイアログ。＝①誘導 ②理由(GPUで約○倍速/無いとCPU) ③スキップ可、を1画面で充足。
3. **アプリ内**（既に「GPU利用不可→CPU」表示あり）に **[ドライバの入手方法]リンク＋理由文**、古い場合は「更新してください」を出し分け。
- 誘導先URL: NVIDIA公式ドライバDLページ（`https://www.nvidia.com/Download/index.aspx`）。型番非依存の入口。

### 戦略判断（要ユーザー承認）— Vulkan を主GPU変種にするか
- Vulkan は「ドライバ案内の負担ほぼゼロ・ベンダー横断・DLL同梱不要・CUDA同等速度」で、**ユーザーの懸念(ドライバ導入の面倒)を最も根本的に解消**する。CUDA は NVIDIA 最速の可能性が残るが、案内・配布(DLL/EULA)の負担が大きい。
- 推奨: **Phase3 で Vulkan 変種を追加し、実機(RTX 4060)で CUDA と速度実測 → 同等なら Vulkan を主GPU変種へ**（CUDAは最速を求める上級者向けに残す）。

## caveats / 未確定
- Vulkan の RTX 4060 実測（CUDA比の速度）は本調査では未実施＝要自機ベンチ。
- 版下限 528.33 は minor-version 互換の一般値。実機で古ドライバ検証が理想。
- NSIS フックでのドライバ検出コード（NVAPI/NVML/レジストリのどれを使うか）は実装時に確定（cudaDriverGetVersion がアプリ側と一貫）。

## 一次ソース（主要）
- CUDA EULA(Attachment A / driver非再配布): https://docs.nvidia.com/cuda/eula/index.html
- CUDA 12.8 リリースノート(最低ドライバ): https://docs.nvidia.com/cuda/archive/12.8.0/cuda-toolkit-release-notes/
- CUDA 互換(minor/forward, ≥525/528.33): https://docs.nvidia.com/deploy/cuda-compatibility/minor-version-compatibility.html / https://docs.nvidia.com/deploy/cuda-compatibility/forward-compatibility.html
- Tauri NSIS フック: https://v2.tauri.app/distribute/windows-installer/ / https://github.com/tauri-apps/tauri/issues/9668
- ドライバ検出(NVML/NVAPI/DXGI/cudaDriverGetVersion): https://docs.nvidia.com/deploy/nvml-api/group__nvmlSystemQueries.html / https://asawicki.info/news_1773
- Ollama GPU要件/Vulkan: https://docs.ollama.com/gpu
- LM Studio 0.3.15(検出+ダウングレード): https://lmstudio.ai/blog/lmstudio-v0.3.15
- whisper.cpp Vulkan(横断・CUDA同等): https://deepwiki.com/ggml-org/whisper.cpp/2.4.1-vulkan-backend / https://www.phoronix.com/news/Whisper-cpp-1.8.3-12x-Perf
