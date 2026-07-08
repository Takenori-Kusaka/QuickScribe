# turbo 文字起こし高速化 調査（問い設計＋deep research 結果）

> 対象: large-v3-turbo(q5) の文字起こしが実測で **16.7分の録音に約180分（RTF≈10.7）** かかった事象の原因切り分けと、現実的な高速化レバー。
> 方法: ADR-0007 の[問い設計メソッド](question-framing-method.md)で問いを設計 → deep research（一次ソース中心・3票敵対的検証）。
> 実行日: 2026-07-08。前提: ローカル完結・プライバシー厳守・ADR-0012（単一バイナリ・MIT・多環境CPU配布）。

## 1. 主因の確定（D1）

**180分/RTF≈10.7 の桁違いの遅さの主因は「turbo が遅い」ではなく、Whisper の seq2seq 構造に内在する反復ループ/ハルシネーション暴走。**

- 長い無音・非発話区間で誤フレーズが decoder の prompt history（`condition_on_previous_text`）に入り**自己強化**、反復検出 → temperature fallback の再試行が積み重なって処理時間を桁違いに膨張させる。
- large-v3 系はこの反復ループ不具合が**既知**で、whisper.cpp maintainer(ggerganov) も v3 large の欠陥を認め large-v2 を推奨（Discussion #1490）。ユーザー報告では約1時間音声のうち**最大20分が反復に消費**された例あり（無音区間に紐づく）。
- ユーザー環境の実測（v1.2.0＝#600 チャンク化マージ**前**）と整合。

**→ #600 の `no_context` チャンク化（各24s窓・前区間コンテキストを断つ）は、暴走の主因である「前区間コンテキスト持ち越し」を断つ直接的対策**（whisper.cpp `--context 0` / `condition_on_previous_text=False` の等価物）。180分問題の解決策そのもの。ただし**単一窓内の自己反復は残る**ため完全解消ではない（追加レバーで補強）。

- 一次: https://huggingface.co/openai/whisper-large-v3-turbo / https://github.com/ggml-org/whisper.cpp/discussions/1490 / https://github.com/ggml-org/whisper.cpp/discussions/2286 / https://github.com/openai/whisper/discussions/679 / https://github.com/SYSTRAN/faster-whisper

## 2. 今すぐ効く高速化レバー（効く順）

### レバー1: 反復抑制（実効時間の桁違い短縮に直結）★最優先
- **`no_context` チャンク化（#600・実装済/main）** — 主因を断つ。
- **`entropy_thold` 引き上げ（既定2.40→2.6程度）**: 低エントロピー（反復）配列をより多く「失敗」判定し temperature fallback を誘発、暴走反復を減らす。
  - **前提**: temperature fallback が有効なときのみ効く（`--no-fallback` や temperature=0 固定 greedy では無効）。**QuickScribe の whisper-rs decode 設定が fallback を許すか要確認**（現状 `Greedy{best_of:1}`。whisper.cpp 既定は temperature_inc=0.2 で fallback 有効）。
- **VAD で無音/非発話をスキップ**: 独り語りジャーナルは沈黙が多く、処理量削減＋反復抑制の双方に効く。査読論文で Silero VAD が WER・ハルシネーション率を有意に低下と報告。
  - **制約**: whisper.cpp の `--vad`(Silero) は **whisper-rs 0.14.4＋vendor sys 0.13.1 に無い**（#569 の更新が要る）。

### レバー2: CPU 計算の素の高速化（Encoder のみ・180分問題は解けない）
- スレッド数 `-t N`（既定4）を実機コア数へ（QuickScribe は既に `num_cpus::get_physical()` を設定済）。
- SIMD（AVX/AVX2/AVX512, ARM NEON/FMA）が release/正しいビルドで有効か **`system_info` 行で確認**（debug/SIMD未活用は桁違い遅延の一因になりうる → **実測の切り分けに system_info を出す**）。
- OpenBLAS（`-DGGML_BLAS=1` / whisper-rs `openblas`・`openmp` feature, opt-in）。
- **重要な限界**: BLAS は **Encoder の行列積のみ**高速化し、**Decoder（反復暴走の場所）には無効**。180分問題は BLAS では解けない。

### レバー3: GPU バックエンド（opt-in・要新ADR）
- whisper-rs は `cuda/metal/vulkan/hipblas/intel-sycl` を**既定OFF の Cargo feature** で露出。compile 時有効化で hidden GPU flag が runtime で立つ。
- **Vulkan** はベンダ非依存の単一コードパスで NVIDIA/AMD/Intel/ARM を横断でき、Win/Linux 多環境の opt-in GPU に最適（Phoronix: AMD 680M/Intel Arc に ~12x、GPU非搭載時 CPU フォールバック）。
- **制約**: (1) feature 有効化は compile-time 選択でビルド変種が分岐（単一ビルドの自動検出とは別問題）、(2) Vulkan は SDK/loader リンクが必要、(3) ADR-0012 は sys 0.13.1 ピン。**逸脱には opt-in 化方針の新ADRが要**。

## 3. エンジン/モデル代替（whisper.cpp 外・ADR-0012 見直し要）（D5）
- **faster-whisper(CTranslate2 int8)**: openai/whisper 比 最大4x・低メモリ・同精度・Silero VAD 統合。ただし 4x は Python openai/whisper 基準で、**whisper.cpp は既に最適化済みのため相対利得は縮む**。CTranslate2 は whisper.cpp/whisper-rs 外。
- **distil-whisper**: 5.8x/51%小/WER<1% だが **英語のみ**。QuickScribe は日本語＝turbo 継続。
- → whisper.cpp 外へ出るため **ADR-0012 の見直しが必要**。

## 4. QuickScribe への実装含意（今すぐ効く順）

1. **#600 no_context チャンク化（実装済/main）** = 180分問題の主因対策。**まず実録音の turbo で実処理時間が短縮したか実測**（本セッションで検証中）。
2. **decode 設定の確認・調整**（ADR-0012 内で可・低リスク）: temperature fallback が有効か確認し、`entropy_thold` を安全側に。効果は日本語CERベンチ(ADR-0024)で回帰監視。
3. **system_info の可視化**（切り分け用）: SIMD/BLAS/スレッドが効いているかをログ/設定に出す。
4. **VAD 無音スキップ**（#569 の whisper-rs 更新とセット）。
5. **opt-in GPU（Vulkan 優先）**（新ADR＋ビルド変種）。
6. **エンジン代替**は最終手段（ADR-0012 見直し）。

## 5. caveats / 反証・未解決
- `entropy_thold` は fallback 有効時のみ効く。QuickScribe の実 decode 設定（fallback 有効か）を要確認。
- BLAS/SIMD は Encoder のみ＝Decoder 反復には無効（180分問題の本命は反復抑制側）。
- distil は英語のみ＝日本語用途では turbo 継続。
- GPU feature 有効化は compile-time でビルド分岐＝「単一ビルドの自動検出」とは別問題。
- **未解決**: #600 チャンク化後の turbo 実処理時間の実測（180分→どこまで短縮したか）。VAD の日本語ジャーナルでの実効。opt-in GPU の配布・署名(#50)への影響。
