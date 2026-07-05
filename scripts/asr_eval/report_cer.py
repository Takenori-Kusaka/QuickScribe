#!/usr/bin/env python3
# manifest(音声<TAB>参照文) と 各音声の hypothesis テキストから、正規化CER＋ブートストラップ95%CIを
# 算出してレポートする（ADR-0024 / #578）。参照/仮説は同一正規化を通す。信頼区間が重なる差は「差なし」。
import argparse
import os
import sys

from cer import bootstrap_ci, utterance_stats
from normalize_ja import neologdn_available, normalize


def main() -> int:
    ap = argparse.ArgumentParser(description="Compute normalized CER + bootstrap 95% CI.")
    ap.add_argument("--manifest", required=True, help="行=「音声ファイル名<TAB>参照文」")
    ap.add_argument("--hyp-dir", required=True, help="<hyp-dir>/<音声ファイル名>.txt に仮説")
    ap.add_argument("--label", default="Common Voice ja", help="レポート上のコーパス名")
    ap.add_argument("--no-neologdn", action="store_true", help="正規化で neologdn を使わない")
    ap.add_argument("--iters", type=int, default=1000)
    ap.add_argument("--report", default=None, help="Markdown を追記する先（任意）")
    args = ap.parse_args()

    use_neologdn = not args.no_neologdn
    pairs = []
    missing = 0
    with open(args.manifest, encoding="utf-8") as f:
        for line in f:
            line = line.rstrip("\n")
            if not line or "\t" not in line:
                continue
            fn, ref = line.split("\t", 1)
            hyp_path = os.path.join(args.hyp_dir, fn + ".txt")
            if os.path.exists(hyp_path):
                with open(hyp_path, encoding="utf-8") as hf:
                    hyp = hf.read()
            else:
                hyp = ""
                missing += 1
            pairs.append(
                (normalize(ref, use_neologdn=use_neologdn), normalize(hyp, use_neologdn=use_neologdn))
            )

    if not pairs:
        return "manifest から評価ペアを構成できなかった。"

    stats = utterance_stats(pairs)
    point, lo, hi = bootstrap_ci(stats, iters=args.iters, seed=1234)
    n = len(pairs)
    neo = "on" if (use_neologdn and neologdn_available()) else "off"

    print(
        f"[{args.label}] N={n} missing_hyp={missing} neologdn={neo} "
        f"mean_cer={point:.4f} ci95=[{lo:.4f},{hi:.4f}]"
    )
    row = f"| {args.label} | {n} | {point * 100:.1f}% | [{lo * 100:.1f}, {hi * 100:.1f}] |"
    if args.report:
        header = "\n### 日本語CER（公開コーパス・ブートストラップ95%CI / ADR-0024）\n\n"
        header += "| コーパス | N | 平均CER | 95%CI |\n|---|---|---|---|\n"
        with open(args.report, "a", encoding="utf-8") as r:
            r.write(header + row + "\n")
    else:
        print(row)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
