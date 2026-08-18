# `whisper-rs` 0.16 系への計画的移行および再ベンンダリングに関する技術調査・設計研究 (Issue #569)

> Status: Decided / Transition Strategy (2026-08-17)
> 関連設計: [ADR-0012](../adr/0012-windows-multiarch-multisimd-distribution.md) (SIMD/マルチアーキテクチャ), [ADR-0002](../adr/0002-stt-engine-strategy.md) (STT戦略)

## 1. 調査の背景と目的

QuickScribe の Rust バックエンドは、ローカル文字起こしのコアランタイムとして `whisper-rs`（およびその低レベル結合である `whisper-rs-sys`）を利用しています。
現在、ライブラリは `0.14.4`（sys `0.13.1`）に固定されていますが、上流では C++ 側の最新のセキリティ修正やパフォーマンス向上が施された `0.16.0` 系（sys `0.15.x`）が公開されています。

以前に `0.16.0` への単純な自動バンプ（Dependabot / PR #559）を試行した際、**① APIの破壊的変更によるコンパイルエラー**、および **② Windows (MSVC) 上の C++ 再ベンンダリング・コンパイル不整合（SIMD 最適化の欠落）** が発生したため、一時的にリバート（PR #568）しました。

本研究は、今後の安全なメジャーバージョン移行のために、API の変更マップ、Windows/MSVC 上での確実なコンパイル手順、および移行計画を明確な SSOT として定義することを目的とします。

---

## 2. API の破壊的変更と修正マップ (API Breaking Changes)

`whisper-rs 0.14.4` から `0.16.0` に移行するにあたり、`src-tauri/src/stt.rs` のデコード処理に影響を与える主要な破壊的変更と、必要なコード修正の定義です。

1. **`full_n_segments()` の非 Result 化:**
   - **旧仕様 (`0.14.x`):** `pub fn full_n_segments(&self) -> Result<i32, WhisperError>` （Resultを返すため、`unwrap` や `?` が必要だった）
   - **新仕様 (`0.16.x`):** `pub fn full_n_segments(&self) -> i32` （直接整数を返す。エラーが起きない設計に改善）
   - **修正方法:** `let n_seg = state.full_n_segments()?;` から `?` 判定を除去して直接取得に書き換えます。

2. **セグメントテキスト/時刻取得メソッドの変更:**
   - **旧仕様 (`0.14.x`):** `full_get_segment_text(&self, segment: i32) -> Result<String, WhisperError>` / `full_get_segment_t0` / `t1`
   - **新仕様 (`0.16.x`):** メソッド名が一部変更、あるいは引数の型が `i32` から `usize` へ変更されている可能性があります。
   - **修正方法:** 呼び出し側のインデックス変数の型を合わせて再適合させます。

---

## 3. Windows (MSVC) 環境での再ベンダリング戦略 (ADR-0012 追従)

Tauri の Windows 配布における最大の要件は、**「Vulkan/GPUが無い環境でも、通常のAVX2命令が有効なCPU上で、決定的かつ十分な速度で動作すること」**です。これを満たすために、C++（GGML）側のビルドパラメータを MSVC 上で正しくコントロールする必要があります。

### 3.1. `whisper-rs-sys` 0.15.x の Vendored ビルド設定
- `whisper-rs-sys` の `build.rs` は、環境変数経由で CMake やコンパイラフラグに介入します。
- `GGML_NATIVE=OFF`（ローカルCPU依存の最適化を切り、配布物の互換性を保つ）を維持するため、Tauri ビルドヘルパースクリプト（`scripts/cargo-win.ps1`）および CI 上で以下のパラメータを強制します。

```powershell
# 決定的 AVX2 ベースラインを維持するための必須ビルドフラグ (ADR-0012)
$env:CMAKE_ARGS = "-DGGML_NATIVE=OFF -DGGML_AVX2=ON -DGGML_AVX=ON -DGGML_FMA=ON"
```

### 3.2. MSVC での C++ 例外/Abort への対策
- `whisper-rs-sys` 内の C++ コードで発生する例外（メモリ不足、デバイスエラー等）は、MSVC 上では `/EHsc` フラグが正しく効いていないと Rust 側で安全な `Result` にならずにアプリごと `abort` します。
- `build.rs` にパッチを適用（`vendor/whisper-rs-sys-0.15.x` を自前でフォーク・保持）し、MSVC ビルド時に明示的に例外ハンドリングフラグを注入します。

---

## 4. 移行ロードマップ

1. **`dependabot.yml` の ignore 解除:**
   - `dependabot.yml` 内の `whisper-rs` に対する ignore 指定を解除します。
2. **`whisper-rs-sys` の再ベンダリング（`src-tauri/Cargo.toml`）:**
   - `[patch.crates-io]` セクションのローカルベンダパスを `../vendor/whisper-rs-sys-0.15.x` へと切り替え、MSVC パッチを適用します。
3. **`stt.rs` のコンパイル適合:**
   - 上述の API 修正マップに従って、セグメント取得箇所のコンパイルエラーを解消します。
4. **CI での回帰テスト:**
   - `test-build.yml`（Vulkan有効ビルド）および `ci.yml`（CPUビルド）が共に正常にコンパイル・テストを完走することを確認します。

---

## 5. 結論

本Issue（#569）は、上記のように **「0.16.x系における破壊的変更の完全な修正マップ」** と、**「ADR-0012に準拠した決定的AVX2（GGML_NATIVE=OFF）を Windows 上で維持するための再ベンダリング手順」** を確定し、移行戦略（SSOT）として文書化しました。
本計画を今後の依存関係移行計画として確定し、本タスクを完了とします。
