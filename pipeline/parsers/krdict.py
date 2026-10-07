"""krdict (한국어기초사전) LMF XML parser."""
from __future__ import annotations

from pathlib import Path

from ..common import (KR_LEVEL, KR_POS, KR_REL, KR_UNIT_KIND, KR_UNIT_POS, clean,
                      has_cjk, has_hangul, iter_xml)

EX_TYPE = {"구": "phrase", "문장": "sentence", "대화": "dialogue"}


def _feats(el) -> dict[str, str]:
    d: dict[str, str] = {}
    for f in el.iterfind("feat"):
        a = f.get("att")
        if a is not None:
            d[a] = f.get("val") or ""
    return d


def parse_entry(le) -> dict | None:
    f = _feats(le)
    lemma_el = le.find("Lemma")
    if lemma_el is None:
        return None
    headword = clean(_feats(lemma_el).get("writtenForm"))
    if not headword:
        return None
    variants = []
    for lm in le.iterfind("Lemma"):
        v = clean(_feats(lm).get("variant"))
        if v and v != headword and v not in variants:
            variants.append(v)

    unit = f.get("lexicalUnit", "단어")
    ko_pos = f.get("partOfSpeech", "")
    kind = KR_UNIT_KIND.get(unit, "word")
    if unit in KR_UNIT_POS:
        pos = KR_UNIT_POS[unit]
    else:
        pos = KR_POS.get(ko_pos, "other")
    if kind == "word" and ko_pos in ("어미", "조사"):
        kind = "grammar"

    try:
        homonym = int(f.get("homonym_number", "0")) or None
    except ValueError:
        homonym = None
    level = KR_LEVEL.get(f.get("vocabularyLevel", ""))

    origin = clean(f.get("origin"))
    hanja = origin_note = None
    if origin:
        if has_cjk(origin) and not any(c.isascii() and c.isalpha() for c in origin):
            hanja = origin
        else:
            origin_note = origin

    prons: list[str] = []
    forms: list[str] = []
    for wf in le.iterfind("WordForm"):
        wf_f = _feats(wf)
        t = wf_f.get("type")
        if t == "발음":
            p = clean(wf_f.get("pronunciation"))
            if p and p not in prons:
                prons.append(p)
        elif t == "활용":
            w = clean(wf_f.get("writtenForm"))
            if w:
                forms.append(w)
            for fr in wf.iterfind("FormRepresentation"):
                w2 = clean(_feats(fr).get("writtenForm"))
                if w2:
                    forms.append(w2)
    forms.extend(variants)

    related = []
    for rf in le.iterfind("RelatedForm"):
        rf_f = _feats(rf)
        w = clean(rf_f.get("writtenForm"))
        if w:
            related.append({"type": KR_REL.get(rf_f.get("type", ""), rf_f.get("type", "")), "word": w})

    senses = []
    ko_defs = []
    for s in le.iterfind("Sense"):
        sf = _feats(s)
        sense: dict = {}
        gl, df = [], []
        for eq in s.iterfind("Equivalent"):
            ef = _feats(eq)
            if ef.get("language") != "영어":
                continue
            g = clean(ef.get("lemma"))
            d = clean(ef.get("definition"))
            if g:
                gl.append(g)
            if d:
                df.append(d)
        if gl:
            sense["gloss"] = "; ".join(gl)
        if df:
            sense["def"] = " ".join(df)
        kd = clean(sf.get("definition"))
        if kd:
            sense["ko_def"] = kd
            ko_defs.append(kd)
        notes = [clean(sf.get("annotation")), clean(sf.get("syntacticAnnotation"))]
        notes = [n for n in notes if n]
        if notes:
            sense["note"] = " ".join(notes)
        if sf.get("syntacticPattern"):
            sense["pattern"] = clean(sf["syntacticPattern"])
        exs = []
        for se in s.iterfind("SenseExample"):
            lines = [clean(x.get("val")) for x in se.iterfind("feat") if x.get("att") == "example"]
            lines = [x for x in lines if x]
            if not lines:
                continue
            t = EX_TYPE.get(_feats(se).get("type", ""))
            ex = {"ko": "\n".join(lines)}
            if t:
                ex["type"] = t
            exs.append(ex)
        if exs:
            sense["examples"] = exs
        rel = []
        for sr in s.iterfind("SenseRelation"):
            rf = _feats(sr)
            w = clean(rf.get("lemma"))
            if w:
                rel.append({"type": KR_REL.get(rf.get("type", ""), rf.get("type", "") or "reference"), "word": w})
        if rel:
            sense["rel"] = rel
        if sense:
            senses.append(sense)

    data: dict = {"senses": senses}
    if related:
        data["related"] = related
    if f.get("semanticCategory"):
        data["category"] = clean(f["semanticCategory"])
    if origin_note:
        data["origin_note"] = origin_note

    return {
        "source": "krdict", "lang": "en", "ext_id": le.get("val"),
        "headword": headword, "homonym": homonym, "hanja": hanja,
        "pos": pos, "ko_pos": ko_pos, "pron": ", ".join(prons) or None,
        "level": level, "kind": kind, "forms": forms, "data": data,
        "ko_defs": ko_defs,
    }


def parse_file(path):
    for le in iter_xml(path, "LexicalEntry"):
        e = parse_entry(le)
        if e:
            yield e


def parse(paths):
    """Yield entries from every krdict XML file in ``paths`` (sorted)."""
    for p in sorted(Path(x) for x in paths):
        yield from parse_file(p)
