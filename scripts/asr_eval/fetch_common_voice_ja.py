#!/usr/bin/env python3
# Common Voice ja (CC0) を CI 実行時に取得する（ADR-0024 / #578）。
# **音声はリポジトリに同梱しない**（ライセンス遵守＋リポジトリ肥大回避）。CC0 再ホストの
# HF ミラー(fsicoli)から streaming で N 発話だけ取得し、音声(mp3バイト)＋manifest(TSV)を書き出す。
#   出力: <out-dir>/cv_00000.mp3 ... と <out-dir>/manifest.tsv（1行=「ファイル名<TAB>参照文」）。
# 版数は --dataset で固定し、Actions キャッシュと併用する（非公式ミラーゆえの再現性確保）。
import argparse
import os
import sys


def main() -> int:
    ap = argparse.ArgumentParser(description="Fetch Common Voice ja (CC0) samples for CER eval.")
    ap.add_argument("--out-dir", required=True)
    ap.add_argument("--num", type=int, default=100, help="取得発話数")
    # このミラーの split は train/validation/test/other/invalidated（"validated" は無い）。
    # 評価は held-out の test を既定にする。
    ap.add_argument("--split", default="test")
    ap.add_argument("--dataset", default="fsicoli/common_voice_17_0", help="CC0再ホストのHFミラー")
    ap.add_argument("--config", default="ja")
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
    # audio は既定で numpy 配列へデコードされる（mp3→array）。WAVで書き出して whisper に渡す
    # （QuickScribe は WAV をそのままデコードできる）。

    os.makedirs(args.out_dir, exist_ok=True)
    manifest_path = os.path.join(args.out_dir, "manifest.tsv")
    n = 0
    with open(manifest_path, "w", encoding="utf-8") as mf:
        for item in ds:
            sentence = (item.get("sentence") or "").strip()
            audio = item.get("audio") or {}
            array = audio.get("array")
            sr = audio.get("sampling_rate")
            if not sentence or array is None or not sr:
                continue  # 参照文か音声が欠けるものは飛ばす。
            fn = f"cv_{n:05d}.wav"
            sf.write(os.path.join(args.out_dir, fn), array, int(sr))
            ref = sentence.replace("\t", " ").replace("\n", " ")
            mf.write(f"{fn}\t{ref}\n")
            n += 1
            if n >= args.num:
                break

    print(f"fetched {n} samples -> {args.out_dir} (manifest: {manifest_path})")
    if n == 0:
        return "no samples fetched（ミラー/split/config を確認）。"
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
