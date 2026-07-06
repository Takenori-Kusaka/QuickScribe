#!/usr/bin/env python3
# HuggingFace の日本語音声コーパスを CI 実行時に取得する（ADR-0024 / #578）。
# **音声はリポジトリに同梱しない**（ライセンス遵守＋リポジトリ肥大回避）。streaming で N 発話だけ
# 取得し、音声を WAV で書き出す＋参照文を manifest(TSV) に書く（1行=「ファイル名<TAB>参照文」）。
#   例) Common Voice ja(CC0ミラー):
#       --dataset fsicoli/common_voice_17_0 --config ja --split test --text-column sentence
#   例) FLEURS ja(CC-BY-4.0):
#       --dataset google/fleurs --config ja_jp --split test --text-column raw_transcription
# 版数は --dataset で固定し、Actions キャッシュと併用する（再現性確保）。
import argparse
import os


def main() -> int:
    ap = argparse.ArgumentParser(description="Fetch a HF Japanese speech corpus for CER eval.")
    ap.add_argument("--out-dir", required=True)
    ap.add_argument("--num", type=int, default=100, help="取得発話数")
    ap.add_argument("--split", default="test", help="held-out の test を既定にする")
    ap.add_argument("--dataset", default="fsicoli/common_voice_17_0")
    ap.add_argument("--config", default="ja")
    ap.add_argument("--text-column", default="sentence", help="参照文の列名(CV=sentence / FLEURS=raw_transcription)")
    ap.add_argument("--prefix", default="s", help="出力WAVファイル名の接頭辞")
    args = ap.parse_args()

    # datasets / soundfile は CI でのみ導入（requirements.txt 参照）。未導入なら明示的に失敗させる。
    try:
        import soundfile as sf
        from datasets import load_dataset
    except ImportError:
        return "datasets/soundfile 未導入。pip で導入（CI専用依存）。"

    ds = load_dataset(
        args.dataset,
        args.config,
        split=args.split,
        streaming=True,
        trust_remote_code=True,
    )
    # audio は既定で numpy 配列へデコードされる。WAVで書き出して whisper に渡す
    # （QuickScribe は WAV をそのままデコードできる）。

    os.makedirs(args.out_dir, exist_ok=True)
    manifest_path = os.path.join(args.out_dir, "manifest.tsv")
    n = 0
    with open(manifest_path, "w", encoding="utf-8") as mf:
        for item in ds:
            sentence = (item.get(args.text_column) or "").strip()
            audio = item.get("audio") or {}
            array = audio.get("array")
            sr = audio.get("sampling_rate")
            if not sentence or array is None or not sr:
                continue  # 参照文か音声が欠けるものは飛ばす。
            fn = f"{args.prefix}_{n:05d}.wav"
            sf.write(os.path.join(args.out_dir, fn), array, int(sr))
            ref = sentence.replace("\t", " ").replace("\n", " ")
            mf.write(f"{fn}\t{ref}\n")
            n += 1
            if n >= args.num:
                break

    print(f"fetched {n} samples -> {args.out_dir} (manifest: {manifest_path})")
    if n == 0:
        return "no samples fetched（dataset/config/split/text-column を確認）。"
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
