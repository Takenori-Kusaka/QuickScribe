# Windows でのローカル Rust ビルド手順（#467）

> src-tauri（Rust バックエンド）を **CI と同条件でローカルビルド/検査**するための手順。
> 依存の移行（ureq/cpal/symphonia 等）を CI 待ちなしで検証するために使う。

## なぜ素の Windows では失敗するか（根本原因）

本プロジェクトは **whisper.cpp / libopus という C/C++ ライブラリをソースからビルドして静的梱包**する
（`whisper-rs-sys`＝whisper.cpp、`audiopus_sys`＝libopus）。そのため Rust だけでなく **C ツールチェーン**が要り、
素の環境では次の3点で失敗する（いずれも実測・[ci.yml](../../.github/workflows/ci.yml) が対処済み）:

1. **MSVC 未導入 → `linker link.exe not found`**。Windows の Rust は既定で `x86_64-pc-windows-msvc` を使い MSVC リンカが必須。
2. **CMake 4.x が古い CMakeLists を拒否 → `Compatibility with CMake < 3.5 has been removed`**。libopus/whisper.cpp の `cmake_minimum_required` が古い。→ `CMAKE_POLICY_VERSION_MINIMUM=3.5`（CMake 公式回避策）で解消。
3. **libclang が新しすぎると bindgen が壊れた bindings を生成**。`whisper-rs-sys 0.13.1` は `bindgen 0.71` を使い、bindgen はレイアウトを libclang の出力に依存する。ローカルの libclang が新版（例: 22）だと構造体を不完全型化し `size mismatch overflow` になる。→ **CI 相当の libclang 18** を使う。

> これは Rust 固有の弱点ではなく、C に FFI する言語共通の「言語パッケージマネージャはネイティブCツールチェーンまでは固定しない」という限界。CI(Linux/Windows ランナー)はこれらを環境として持つため通る。

## セットアップ（初回のみ）

1. **VS Build Tools 2022（C++ ワークロード）**
   ```
   winget install --id Microsoft.VisualStudio.2022.BuildTools -e --override "--passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
   ```
2. **CMake** / **Rust(rustup)** を導入（未導入なら）。
3. **bindgen 用 libclang 18（CI相当・非破壊）** — 既存の別 LLVM を壊さない:
   ```
   python -m pip install --target C:\libclang18 "libclang==18.1.1"
   ```
   → `C:\libclang18\clang\native\libclang.dll` が置かれる。

## 使い方

`scripts/cargo-win.ps1` が vcvars64（MSVC）＋ `LIBCLANG_PATH` ＋ `CMAKE_POLICY_VERSION_MINIMUM=3.5` を
自動設定して cargo を実行する:

```
powershell -File scripts/cargo-win.ps1 check
powershell -File scripts/cargo-win.ps1 test --lib
powershell -File scripts/cargo-win.ps1 test --test transcribe_integration
```

- `LIBCLANG_PATH` を別の場所に置いた場合は環境変数で上書き可（スクリプトが尊重する）。
- 実機依存の検証（録音=cpal のマイク入力、通知）は CI でも本ビルドでも判定できないため、実際にアプリを起動して確認する。

## 補足

- CI（`ci.yml`）は Ubuntu/Windows ランナーで同等の設定を持つ（`CMAKE_POLICY_VERSION_MINIMUM: "3.5"` env、`LIBCLANG_PATH` を runner の clang から設定、apt/VS の C ツールチェーン）。ローカルはそれを再現しているだけ。
- whisper-rs-sys は ADR-0012 で `vendor/whisper-rs-sys-0.13.1` に固定（AVX512 CPU 移植性制御）。近代化は AVX512 ゲート再検証を伴う別課題。
