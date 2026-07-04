// 文字起こしパイプラインの統合テスト（決定論的検証）。
// CIで小型whisperモデルと既知音声(espeak生成)を用意し、QS_MODEL_PATH/QS_AUDIO_PATH を
// 渡したときのみ実行する。デコード→whisper→テキスト(非空)を検証する。
// アセット未設定時はスキップ（通常のローカルcargo testを壊さない）。

use std::path::Path;

#[test]
fn transcribes_known_audio_when_assets_present() {
    let (model, audio) = match (
        std::env::var("QS_MODEL_PATH"),
        std::env::var("QS_AUDIO_PATH"),
    ) {
        (Ok(m), Ok(a)) => (m, a),
        _ => {
            eprintln!("skip: QS_MODEL_PATH / QS_AUDIO_PATH 未設定のためスキップ");
            return;
        }
    };

    let samples = quickscribe_lib::stt::decode_to_16k_mono(Path::new(&audio))
        .expect("音声のデコードに失敗");
    assert!(!samples.is_empty(), "デコード結果が空");

    let text = quickscribe_lib::stt::transcribe(Path::new(&model), &samples, None)
        .expect("文字起こしに失敗");
    eprintln!("transcript = {text:?}");
    // 精度ベンチ(#403 CER)向けに、認識テキストをファイルへ出力する(env 指定時のみ)。
    // stderr パースより堅牢に、後段の CER 計算へ渡す。
    if let Ok(out) = std::env::var("QS_TRANSCRIPT_OUT") {
        std::fs::write(&out, &text).expect("transcript の書き出しに失敗");
    }
    assert!(!text.trim().is_empty(), "文字起こし結果が空であってはならない");
}
