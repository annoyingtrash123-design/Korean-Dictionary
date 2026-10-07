"""Shared helpers: text normalisation, POS maps, logging."""
from __future__ import annotations

import html
import logging
import re

log = logging.getLogger("pipeline")

# --- CJK ---------------------------------------------------------------------

_CJK_RANGES = (
    (0x3400, 0x4DBF),
    (0x4E00, 0x9FFF),
    (0xF900, 0xFAFF),
    (0x20000, 0x2FA1F),
)


def is_cjk(ch: str) -> bool:
    o = ord(ch)
    return any(lo <= o <= hi for lo, hi in _CJK_RANGES)


def has_cjk(s: str | None) -> bool:
    return bool(s) and any(is_cjk(c) for c in s)


def cjk_chars(s: str | None) -> list[str]:
    """Distinct CJK ideographs of ``s`` in order of first appearance."""
    out: list[str] = []
    for c in s or "":
        if is_cjk(c) and c not in out:
            out.append(c)
    return out


def is_hangul(ch: str) -> bool:
    o = ord(ch)
    return 0xAC00 <= o <= 0xD7A3 or 0x1100 <= o <= 0x11FF or 0x3130 <= o <= 0x318F


def has_hangul(s: str | None) -> bool:
    return bool(s) and any(is_hangul(c) for c in s)


# --- text --------------------------------------------------------------------

_WS = re.compile(r"\s+")


def clean(s: str | None) -> str:
    """Unescape stray entities (krdict double-escapes), collapse whitespace."""
    if not s:
        return ""
    if "&" in s:
        s = html.unescape(s)
    return _WS.sub(" ", s).strip()


_NORM_STRIP = str.maketrans("", "", "-^ ··・ㆍ")


def hw_norm(headword: str) -> str:
    """Lookup key: headword with '-', '^', ' ', '·' removed."""
    return headword.translate(_NORM_STRIP)


def truncate(s: str, n: int = 120) -> str:
    if len(s) <= n:
        return s
    cut = s[: n - 1].rstrip()
    return cut + "…"


def make_gloss(glosses: list[str], limit: int = 120) -> str:
    """Join glosses with '; ' while they fit within ``limit`` characters."""
    out = ""
    for g in glosses:
        g = g.strip()
        if not g:
            continue
        cand = g if not out else out + "; " + g
        if len(cand) > limit:
            if not out:
                return truncate(g, limit)
            break
        out = cand
    return out


# --- POS / kind --------------------------------------------------------------

KR_POS = {
    "명사": "noun",
    "동사": "verb",
    "형용사": "adjective",
    "부사": "adverb",
    "조사": "particle",
    "어미": "ending",
    "접사": "affix",
    "의존 명사": "bound noun",
    "보조 동사": "auxiliary verb",
    "보조 형용사": "auxiliary adjective",
    "대명사": "pronoun",
    "수사": "numeral",
    "관형사": "determiner",
    "감탄사": "interjection",
    "구": "phrase",
    "품사 없음": "other",
    "품사없음": "other",
}

KR_UNIT_POS = {  # lexicalUnit overrides POS for non-word units
    "구": "phrase",
    "관용구": "idiom",
    "속담": "proverb",
    "문법‧표현": "expression",
    "문법·표현": "expression",
}

KR_UNIT_KIND = {
    "단어": "word",
    "구": "phrase",
    "관용구": "idiom",
    "속담": "proverb",
    "문법‧표현": "grammar",
    "문법·표현": "grammar",
}

KR_LEVEL = {"초급": 1, "중급": 2, "고급": 3}

KR_REL = {
    "반대말": "antonym",
    "유의어": "synonym",
    "비슷한말": "synonym",
    "동의어": "synonym",
    "높임말": "honorific",
    "낮춤말": "humble",
    "참고어": "see also",
    "참고 어휘": "see also",
    "큰말": "larger form",
    "작은말": "smaller form",
    "센말": "stronger form",
    "여린말": "softer form",
    "준말": "abbreviation",
    "본말": "full form",
    "파생어": "derived",
    "부표제어": "sub-entry",
    "☞(가 보라)": "reference",
}

WIKT_POS = {
    "noun": "noun",
    "name": "noun",
    "verb": "verb",
    "adj": "adjective",
    "adv": "adverb",
    "pron": "pronoun",
    "num": "numeral",
    "det": "determiner",
    "intj": "interjection",
    "particle": "particle",
    "postp": "particle",
    "prep": "particle",
    "suffix": "affix",
    "prefix": "affix",
    "infix": "affix",
    "affix": "affix",
    "circumfix": "affix",
    "proverb": "proverb",
    "phrase": "phrase",
    "prep_phrase": "phrase",
    "idiom": "idiom",
    "contraction": "other",
    "conj": "other",
    "classifier": "bound noun",
    "root": "other",
    "character": "other",
    "symbol": "other",
    "syllable": "other",
    "romanization": "other",
}


def kind_for_pos(pos: str | None) -> str:
    if pos in ("phrase", "idiom", "proverb"):
        return pos
    return "word"


# --- streaming XML -----------------------------------------------------------

_ILLEGAL = bytes(list(range(0, 9)) + [0x0B, 0x0C] + list(range(0x0E, 0x20)))


class _CleanReader:
    """File wrapper that drops control bytes illegal in XML 1.0 (they occur in
    a handful of NIKL entries and would otherwise abort the whole file)."""

    def __init__(self, f):
        self._f = f

    def read(self, n=-1):
        return self._f.read(n).translate(None, _ILLEGAL)


def iter_xml(path, tag: str):
    """Stream ``tag`` elements from a large XML file, freeing memory as we go."""
    from lxml import etree

    with open(path, "rb") as fh:
        yield from _iter_xml(etree, _CleanReader(fh), tag)


def _iter_xml(etree, fh, tag):
    ctx = etree.iterparse(
        fh, events=("end",), tag=tag,
        load_dtd=False, resolve_entities=False, no_network=True,
        huge_tree=True, recover=False,
    )
    for _, elem in ctx:
        yield elem
        elem.clear()
        parent = elem.getparent()
        if parent is not None:
            while elem.getprevious() is not None:
                del parent[0]
