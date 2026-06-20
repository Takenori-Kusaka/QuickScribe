#!/usr/bin/env python3
"""ブランドアイコンの生アート → 配布用クリーン透過アイコンへの決定論的処理。

QuickScribe のロゴ原画 (assets/brand/icon-raw.png, Gemini 生成の不透明白背景＋発光) を、
背景・グロウを完全に除去した「角丸スクエア本体のみ・外側完全透過・余白最小」の
配布用ソース (assets/brand/icon-source.png) に変換する。出力は `npm run icons`
(tauri icon) が各プラットフォーム用アイコンを生成する元になる。

設計意図:
- 手作業の一発処理 (場当たり対応) をやめ、再現可能な1スクリプトに集約する。
  原画を差し替えて再実行すれば同じ品質基準で再生成される。
- パラメータは先頭の定数に集約。ロゴを洗練/差し替える際はここだけ調整する。
- 末尾で品質ゲート (透過・ハロー無し・サイズ) を assert し、劣化した出力をコミットさせない。

実行 (デザイン時のみ。CI では実行しない。出力 PNG はコミット済みのものを使う):
    pip install pillow numpy scipy
    python scripts/process-brand-icon.py
    npm run icons   # 続けて各サイズのアイコンを生成

なぜ CI で動かさないか: 透過処理はデザイン時の確定作業で、成果物 icon-source.png を
コミットする。CI は Python 依存を増やさず、コミット済み PNG から tauri icon を回す。
"""
from __future__ import annotations

import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageFilter
from scipy import ndimage

# --- パラメータ (ロゴ調整時はここを変える) ---------------------------------
ROOT = Path(__file__).resolve().parent.parent
RAW = ROOT / "assets" / "brand" / "icon-raw.png"
OUT = ROOT / "assets" / "brand" / "icon-source.png"

# 角丸スクエア本体と「背景＋グロウ」を分ける彩度しきい値 (min(R,G,B) < この値=本体)。
# 原画は本体が min<120、グロウが min>200 で明確に分離するため 170 で安全に切れる。
BODY_MIN_THRESHOLD = 170
# 縁のアンチエイリアス用フェザー (px)。大きいほど柔らかいがクロマ環境で滲みやすい。
FEATHER_RADIUS = 0.8
# 切り出し後に付ける対称マージン (本体サイズ比)。余白を詰めつつ端の見切れを防ぐ。
MARGIN_RATIO = 0.04
# 出力一辺 (px)。tauri icon は正方形 1024 を推奨。
OUTPUT_SIZE = 1024

# --- 品質ゲート -------------------------------------------------------------
# 角(コーナー)はこのアルファ以下=透過していること。
MAX_CORNER_ALPHA = 8
# 不透明 (alpha>16) 画素の許容割合。角丸スクエアは概ね 0.6〜0.88。
# これを超える=背景/グロウが残存 (ハロー)、下回る=本体が欠けた、とみなし失敗させる。
MIN_OPAQUE_FRACTION = 0.55
MAX_OPAQUE_FRACTION = 0.90


def process(raw: Image.Image) -> Image.Image:
    arr = np.asarray(raw.convert("RGBA")).astype(np.int16)
    rgb = arr[..., :3]
    mn = rgb.min(axis=2)

    # 1) 本体マスク = 彩度の高い角丸スクエア (白い内部ロゴと淡いグロウを除外)
    body = mn < BODY_MIN_THRESHOLD
    # 2) 内部の白いマイク/波形ロゴ (本体に囲まれた穴) を埋めてソリッドな本体にする
    filled = ndimage.binary_fill_holes(body)
    # 3) 最大連結成分だけ残し、孤立したグロウのかけらを捨てる
    labels, n = ndimage.label(filled)
    if n > 1:
        sizes = ndimage.sum(np.ones_like(labels), labels, range(1, n + 1))
        filled = labels == (int(np.argmax(sizes)) + 1)

    alpha = (filled * 255).astype(np.uint8)
    img = Image.fromarray(np.dstack([arr[..., :3].astype(np.uint8), alpha]), "RGBA")
    # 4) 縁を少しだけフェザーして硬さを和らげる
    img.putalpha(img.split()[3].filter(ImageFilter.GaussianBlur(FEATHER_RADIUS)))

    # 5) 本体にタイトクロップ → 対称マージン → 正方キャンバス → 規定サイズへ
    bbox = img.split()[3].point(lambda v: 255 if v > 16 else 0).getbbox()
    if bbox is None:
        raise SystemExit("error: 本体マスクが空。BODY_MIN_THRESHOLD を見直すこと。")
    crop = img.crop(bbox)
    w, h = crop.size
    side = max(w, h)
    margin = int(side * MARGIN_RATIO)
    canvas_side = side + 2 * margin
    canvas = Image.new("RGBA", (canvas_side, canvas_side), (0, 0, 0, 0))
    canvas.paste(crop, ((canvas_side - w) // 2, (canvas_side - h) // 2), crop)
    return canvas.resize((OUTPUT_SIZE, OUTPUT_SIZE), Image.LANCZOS)


def assert_quality(out: Image.Image) -> None:
    a = np.asarray(out)[..., 3]
    corners = [a[0, 0], a[0, -1], a[-1, 0], a[-1, -1]]
    if max(int(c) for c in corners) > MAX_CORNER_ALPHA:
        raise SystemExit(f"quality gate失敗: 角が透過していない (alpha={corners}). 背景除去を確認。")
    frac = float((a > 16).mean())
    if not (MIN_OPAQUE_FRACTION <= frac <= MAX_OPAQUE_FRACTION):
        raise SystemExit(
            f"quality gate失敗: 不透明割合 {frac:.3f} が許容 [{MIN_OPAQUE_FRACTION},{MAX_OPAQUE_FRACTION}] 外。"
            " 高すぎ=グロウ残存(ハロー)、低すぎ=本体欠け。"
        )
    if out.size != (OUTPUT_SIZE, OUTPUT_SIZE) or out.mode != "RGBA":
        raise SystemExit(f"quality gate失敗: 出力が {out.size} {out.mode} (期待 {OUTPUT_SIZE}px RGBA)。")


def main() -> int:
    if not RAW.exists():
        raise SystemExit(f"error: 生アートが見つからない: {RAW}")
    raw = Image.open(RAW)
    out = process(raw)
    assert_quality(out)
    out.save(OUT)
    frac = float((np.asarray(out)[..., 3] > 16).mean())
    print(f"ok: {RAW.name} -> {OUT.relative_to(ROOT)}  ({OUTPUT_SIZE}px RGBA, opaque={frac:.3f})")
    print("次に `npm run icons` で各プラットフォーム用アイコンを生成すること。")
    return 0


if __name__ == "__main__":
    sys.exit(main())
