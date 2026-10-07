"""stdict (표준국어대사전) XML parser."""
from __future__ import annotations

import re
from pathlib import Path

from ..common import KR_POS, KR_REL, clean, has_cjk, iter_xml

_HOMONYM = re.compile(r"(\d{1,3})$")
_UNIT_KIND = {"단어": "word", "구": "phrase", "속담": "proverb", "관용구": "idiom"}


def split_homonym(word: str) -> tuple[str, int | None]:
    m = _HOMONYM.search(word)
    if m and len(word) > len(m.group(1)):
        n = int(m.group(1))
        return word[: m.start()], (n or None)
    return word, None


def display_word(raw: str) -> str:
    """Remove '-' (morpheme) and '^' (space) markers; keep leading/trailing '-'
    that denote affixes/endings.  '^' becomes a plain space."""
    raw = raw.strip()
    lead = raw.startswith("-")
    trail = len(raw) > 1 and raw.endswith("-")
    core = raw.strip("-").replace("-", "").replace("^", " ")
    core = " ".join(core.split())
    return ("-" if lead else "") + core + ("-" if trail else "")


def _t(el, path: str) -> str:
    x = el.find(path)
    return clean(x.text) if x is not None and x.text else ""


def parse_item(item) -> dict | None:
    wi = item.find("word_info")
    if wi is None:
        return None
    raw, homonym = split_homonym(_t(wi, "word"))
    headword = display_word(raw)
    if not headword:
        return None
    unit = _t(wi, "word_unit") or "단어"

    # origin
    parts = []
    has_hanja_part = False
    other_origin = []
    for ol in wi.iterfind("original_language_info"):
        txt = _t(ol, "original_language")
        lt = _t(ol, "language_type")
        if not txt:
            continue
        parts.append((txt, lt))
        if lt == "한자":
            has_hanja_part = True
        else:
            other_origin.append((txt, lt))
    hanja = origin_note = None
    if has_hanja_part:
        cand = "".join(t for t, lt in parts if lt in ("한자", "고유어"))
        if has_cjk(cand):
            hanja = cand
    elif parts:
        origin_note = "; ".join(f"{lt}: {t}" if lt and lt not in ("안 밝힘", "/(병기)") else t
                                for t, lt in parts)

    pron = ", ".join(dict.fromkeys(
        clean(p.text) for p in wi.iterfind("pronunciation_info/pronunciation") if p.text)) or None

    forms: list[str] = []
    for tag in ("conju_info/conjugation_info/conjugation", "conju_info/abbreviation_info/abbreviation"):
        for c in wi.iterfind(tag):
            if c.text and clean(c.text):
                forms.append(clean(c.text))

    related = []

    def rel_word(w):
        w, _ = split_homonym(w)
        return display_word(w)

    for li in wi.iterfind("lexical_info"):
        w = rel_word(_t(li, "word"))
        if w:
            ty = _t(li, "type")
            related.append({"type": KR_REL.get(ty, ty), "word": w})
    for ri in wi.iterfind("relation_info"):
        w = rel_word(_t(ri, "word"))
        if w:
            ty = _t(ri, "type")
            related.append({"type": KR_REL.get(ty, ty), "word": w})

    senses = []
    cats: list[str] = []
    ko_pos_first = None
    multi_pos = len(wi.findall("pos_info")) > 1
    for pi in wi.iterfind("pos_info"):
        ko_pos = _t(pi, "pos")
        if ko_pos_first is None:
            ko_pos_first = ko_pos
        sense_pos = KR_POS.get(ko_pos, "other")
        for cp in pi.iterfind("comm_pattern_info"):
            pattern = _t(cp, "pattern_info/pattern")
            gram = "; ".join(clean(g.text) for g in cp.iterfind("grammar_info/grammar") if g.text)
            for si in cp.iterfind("sense_info"):
                sense: dict = {}
                if multi_pos:
                    sense["pos"] = sense_pos
                kd = _t(si, "definition")
                if kd:
                    sense["ko_def"] = kd
                if gram:
                    sense["note"] = gram
                if pattern:
                    sense["pattern"] = pattern
                ty = _t(si, "type")
                tags = []
                if ty and ty != "일반어":
                    tags.append(ty)
                for c in si.iterfind("cat_info/cat"):
                    ct = clean(c.text)
                    if ct and ct != "없음":
                        if ct not in cats:
                            cats.append(ct)
                        tags.append(ct)
                if tags:
                    sense["tags"] = tags
                exs = []
                for ei in si.iterfind("example_info"):
                    if ei.find("source") is not None:
                        continue  # cited from copyrighted works -> excluded
                    ex = _t(ei, "example")
                    if ex:
                        exs.append({"ko": ex})
                if exs:
                    sense["examples"] = exs
                rel = []
                for li in si.iterfind("lexical_info"):
                    w = rel_word(_t(li, "word"))
                    if w:
                        ty = _t(li, "type")
                        rel.append({"type": KR_REL.get(ty, ty), "word": w})
                if rel:
                    sense["rel"] = rel
                if sense:
                    senses.append(sense)

    ko_pos = ko_pos_first or ""
    if unit in _UNIT_KIND and unit != "단어":
        pos, kind = _UNIT_KIND[unit], _UNIT_KIND[unit]
    else:
        pos = KR_POS.get(ko_pos, "other")
        kind = "phrase" if pos == "phrase" else "word"
    if kind == "word" and ko_pos in ("어미", "조사"):
        kind = "grammar"

    data: dict = {"senses": senses}
    if related:
        data["related"] = related
    if cats:
        data["category"] = " / ".join(cats)
    if origin_note:
        data["origin_note"] = origin_note
    o = _t(wi, "origin")
    if o:
        data["etym"] = o
    al = _t(wi, "allomorph")
    if al:
        data["allomorph"] = al

    return {
        "source": "stdict", "lang": "ko", "ext_id": _t(item, "target_code"),
        "headword": headword, "homonym": homonym, "hanja": hanja,
        "pos": pos, "ko_pos": ko_pos, "pron": pron, "level": None, "kind": kind,
        "forms": forms, "data": data,
    }


def parse_file(path):
    for it in iter_xml(path, "item"):
        e = parse_item(it)
        if e:
            yield e


def parse(paths):
    for p in sorted(Path(x) for x in paths):
        yield from parse_file(p)
