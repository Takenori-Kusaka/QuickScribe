fn main() {
    // Windows: テストバイナリにも Common-Controls v6 のSxSマニフェストを埋め込む。
    // tauri の mock runtime(テスト専用feature・dev依存)が TaskDialogIndirect(comctl32 v6) を
    // 静的インポートするため、マニフェスト無しのテストEXEは STATUS_ENTRYPOINT_NOT_FOUND で
    // 起動できない。rustc-link-arg-tests は lib のユニットテストに適用されない(cargo仕様)ため
    // rustc-link-arg を使う。アプリ本体bin側は tauri-build の埋め込みマニフェスト(.res)が
    // そのまま有効（/MANIFEST:EMBED と重複しないことをCI buildで検証済み）。
    if std::env::var("CARGO_CFG_WINDOWS").is_ok() {
        println!("cargo::rustc-link-arg=/MANIFEST:EMBED");
        println!(
            "cargo::rustc-link-arg=/MANIFESTDEPENDENCY:type='win32' \
             name='Microsoft.Windows.Common-Controls' version='6.0.0.0' \
             processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'"
        );
    }
    tauri_build::build()
}
