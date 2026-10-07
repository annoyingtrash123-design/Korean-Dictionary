"""kengdic TSV parser (id, surface, hanja, gloss, level, created, source)."""
from __future__ import annotations

import csv
import re

from ..common import clean, has_cjk, has_hangul

_JUNK_EDGE = re.compile(r"^[\s,;:.\-–—/\\|\"'`~*+=]+|[\s,;:\-–—/\\|\"'`~*+=]+$")
_CTRL = re.compile(r"[\x00-\x1f\x7f]")
_LEVEL_ORDER = {"A": 0, "B": 1, "C": 2, "D": 3}


def clean_gloss(g: str) -> str:
    g = _CTRL.sub(" ", g or "")
    g = clean(g)
    g = _JUNK_EDGE.sub("", g)
    # fix "a ,b" spacing artefacts
    g = re.sub(r"\s+,", ",", g)
    return g


def _guess_pos(surface: str, gloss: str) -> str:
    low = gloss.lower()
    if surface.endswith("다") and " " not in surface:
        if low.startswith("to "):
            return "verb"
        if low.startswith("be "):
            return "adjective"
    return "other"


def parse(path) -> tuple[list[dict], dict[str, set[str]]]:
    """Return (entries, hanja_by_surface).

    Rows are grouped by (surface, hanja).  Rows without gloss only feed
    ``hanja_by_surface`` (surface -> set of distinct hanja strings)."""
    groups: dict[tuple[str, str], dict] = {}
    hanja_by_surface: dict[str, set[str]] = {}
    with open(path, encoding="utf-8", newline="") as fh:
        rd = csv.reader(fh, delimiter="\t", quoting=csv.QUOTE_NONE)
        header = next(rd, None)
        for row in rd:
            if len(row) < 4:
                continue
            surface = clean(_CTRL.sub(" ", row[1]))
            if not surface or not has_hangul(surface):
                continue
            variants = [v for v in (clean(x) for x in row[2].split(",")) if v and has_cjk(v)]
            hanja = variants[0] if variants else ""
            if variants:
                hanja_by_surface.setdefault(surface, set()).add(variants[0])
            gloss = clean_gloss(row[3])
            if not gloss or len(gloss) > 250:
                continue
            level = row[4].strip().upper() if len(row) > 4 else ""
            g = groups.get((surface, hanja))
            if g is None:
                g = groups[(surface, hanja)] = {
                    "source": "kengdic", "lang": "en", "ext_id": row[0],
                    "headword": surface, "homonym": None, "hanja": hanja or None,
                    "level": None, "forms": [], "variants_h": variants[1:],
                    "glosses": {}, "klevel": None,
                }
            g["glosses"].setdefault(gloss.lower(), gloss)
            if level in _LEVEL_ORDER and (g["klevel"] is None or _LEVEL_ORDER[level] < _LEVEL_ORDER[g["klevel"]]):
                g["klevel"] = level
    out = []
    for (surface, hanja), g in groups.items():
        glosses = list(g.pop("glosses").values())
        pos = _guess_pos(surface, glosses[0])
        kind = "phrase" if " " in surface else "word"
        data: dict = {"senses": [{"gloss": x} for x in glosses]}
        if g["variants_h"]:
            data["hanja_alt"] = g["variants_h"]
        if g["klevel"]:
            data["kengdic_level"] = g["klevel"]
        g.update(pos="phrase" if kind == "phrase" else pos, kind=kind, pron=None, data=data)
        del g["variants_h"], g["klevel"]
        out.append(g)
    return out, hanja_by_surface
