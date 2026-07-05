#!/usr/bin/env python3
# 日本語 ASR 評価用テキスト正規化（ADR-0024 / #578）。
# 参照文・仮説文に**同一**適用する。CER の前に空白を完全除去するのが日本語の必須点
# （日本語は語間空白が無いのに正規化で空白が生まれ、文字単位比較で1文字と数えてしまうため）。
#
# パイプライン（ADR-0024）:
#   括弧内注記(ルビ/タグ)除去 → (任意)neologdn → NFKC → ラテン小文字化 → 約物/記号除去 → 空白完全除去
# 数字表記・かな種別の吸収は「モデルの実出力差を隠す」ため**既定 OFF**（コア価値=実差を消さない）。
import re
import unicodedata

try:
    import neologdn  # 任意依存（C拡張）。長音短縮/ダッシュ統一/繰り返し短縮/半角カナ正規化。

    _HAS_NEOLOGDN = True
except Exception:  # 未導入環境（ローカルWindows等）ではスキップ。CI では requirements で導入する。
    _HAS_NEOLOGDN = False

# YAML フロントマター（先頭の --- ... ---）。
_FRONTMATTER = re.compile(r"^\s*---.*?---\s*", re.DOTALL)
# 行頭タイムスタンプ [HH:MM:SS]。
_TIMESTAMP = re.compile(r"\[\d{2}:\d{2}:\d{2}\]")
# 括弧内注記（ルビ/タグ）: [..] (..) （..） 【..】 《..》 〔..〕 <..>。ネストは想定しない。
_BRACKETS = re.compile(r"[\[\(（【《〔<][^\]\)）】》〕>]*[\]\)）】》〕>]")


def neologdn_available() -> bool:
    """neologdn が使えるか（テスト/CIでの分岐用）。"""
    return _HAS_NEOLOGDN


def _strip_symbols(s: str) -> str:
    """Unicode カテゴリで約物(P*)・記号(S*)を除去する。々ー等の Lm は保持される。"""
    return "".join(ch for ch in s if unicodedata.category(ch)[0] not in ("P", "S"))


def _strip_spaces(s: str) -> str:
    """半角/全角を含むあらゆる空白を完全除去する（日本語CERの必須処理）。"""
    return re.sub(r"\s+", "", s).replace("　", "")


def normalize(s: str, use_neologdn: bool = True) -> str:
    """参照/仮説テキストを CER 比較用に正規化する。両者へ同一適用すること。

    use_neologdn=False で neologdn 段をスキップ（未導入環境やテストでの決定性確保）。
    """
    s = _FRONTMATTER.sub("", s, count=1)
    s = _TIMESTAMP.sub("", s)
    s = _BRACKETS.sub("", s)
    if use_neologdn and _HAS_NEOLOGDN:
        s = neologdn.normalize(s)
    s = unicodedata.normalize("NFKC", s)
    s = s.lower()  # NFKC はラテンを小文字化しないため明示的に。
    s = _strip_symbols(s)
    s = _strip_spaces(s)
    return s
