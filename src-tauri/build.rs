fn main() {
    // Windows: comctl32.dll を遅延ロード(delay-load)にする。
    // tauri の mock runtime(テスト用 feature "test"・dev依存)をリンクすると
    // TaskDialogIndirect(comctl32 v6) が静的インポートされ、SxSマニフェストを持たない
    // テストEXEは起動時解決に失敗して STATUS_ENTRYPOINT_NOT_FOUND で落ちる。
    // 遅延ロードなら解決は「最初の呼び出し時」に延びるため、テストEXEは起動でき、
    // アプリ本体は tauri-build 埋め込みマニフェスト(comctl32 v6)の下で従来どおり解決される
    // （公開挙動は不変）。/MANIFEST:EMBED 方式は tauri-build の RT_MANIFEST と重複し
    // CVT1100/LNK1123 になるため不可。rustc-link-arg-tests は lib のユニットテストに
    // 適用されない(cargo仕様)ため rustc-link-arg を使う。
    if std::env::var("CARGO_CFG_WINDOWS").is_ok() {
        println!("cargo::rustc-link-arg=/DELAYLOAD:comctl32.dll");
        println!("cargo::rustc-link-arg=delayimp.lib");
        // CUDA変種(ADR-0027): nvcuda.dll はNVIDIAドライバ由来で同梱できない。遅延ロードにより
        // 非搭載機でもEXEが起動できるようにする(use_gpu=false 判定時はCUDA APIを一切呼ばない設計と
        // 併せて、CUDA変種がGPU無し環境でもCPU実行で動作する)。cudart/cublas等はインストーラ同梱。
        if std::env::var("CARGO_FEATURE_CUDA").is_ok() {
            println!("cargo::rustc-link-arg=/DELAYLOAD:nvcuda.dll");
        }
    }
    tauri_build::build()
}
