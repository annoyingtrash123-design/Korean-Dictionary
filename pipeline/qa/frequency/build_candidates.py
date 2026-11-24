#!/usr/bin/env python3
"""Build the 속담 / 성어 candidate lists from the dictionary packs (read-only).

    python3 -I build_candidates.py PACK_DIR OUT_DIR

PACK_DIR holds core.sqlite (krdict), stdict.sqlite and opendict.sqlite.  Writes
candidates_sokdam.tsv and candidates_seongeo.tsv into OUT_DIR.
"""
import json
import os
import re
import sqlite3
import sys
import unicodedata

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kdfreq  # noqa: E402

MIN_OTHER_STEM = 6
SOURCES = [("krdict", "core"), ("stdict", "stdict"), ("opendict", "opendict")]


def connect(pack_dir, name):
    p = os.path.join(pack_dir, name + ".sqlite")
    return sqlite3.connect(f"file:{p}?mode=ro", uri=True)


# ------------------------------------------------------------------ 속담

def load_proverbs(pack_dir):
    recs = []
    for src, pack in SOURCES:
        con = connect(pack_dir, pack)
        rows = con.execute(
            "select id, headword from entries where kind='proverb' "
            + ("and source='krdict' " if src == "krdict" else "")
            + "order by quality desc, rank asc, id asc").fetchall()
        for eid, hw in rows:
            if kdfreq.has_unrenderable(hw):
                continue
            vs = kdfreq.proverb_variants(hw)
            if not vs:
                continue
            recs.append({"src": src, "id": eid, "hw": hw, "keys": [kdfreq.strip_key(v) for v in vs]})
    return recs


def group_proverbs(recs):
    """Union records whose primary key equals another record's primary or variant key."""
    parent = list(range(len(recs)))

    def find(x):
        while parent[x] != x:
            parent[x] = parent[parent[x]]
            x = parent[x]
        return x

    def union(a, b):
        ra, rb = find(a), find(b)
        if ra != rb:
            parent[max(ra, rb)] = min(ra, rb)

    primary = {}
    for i, r in enumerate(recs):
        k = r["keys"][0]
        if k in primary:
            union(i, primary[k])
        else:
            primary[k] = i
    for i, r in enumerate(recs):
        for k in r["keys"][1:]:
            j = primary.get(k)
            if j is not None:
                union(i, j)
    groups = {}
    for i in range(len(recs)):
        groups.setdefault(find(i), []).append(recs[i])
    return list(groups.values())


def sokdam_rows(pack_dir):
    groups = group_proverbs(load_proverbs(pack_dir))
    out = []
    for g in groups:
        best = {}
        for r in g:  # recs are ordered best-first per source
            best.setdefault(r["src"], r)
        order = [s for s, _ in SOURCES if s in best]
        canon = best[order[0]]  # krdict headword if present, else stdict, else opendict
        keys, headwords = [], []
        for s in order:
            headwords.append(best[s]["hw"])
            for k in best[s]["keys"]:
                if k not in keys:
                    keys.append(k)
        pats = [kdfreq.proverb_patterns(h) for h in headwords]
        # krdict proverbs are deliberate learner picks, so a 4-syllable stem is enough; the other
        # dictionaries contain many fragments / place-name entries filed as proverbs ("강원도 삼척"),
        # so their stems must be at least 6 syllables to be rankable
        stem_len = max([len(unicodedata.normalize("NFC", f)) for p in pats for f in p["full"]] or [0])
        short = stem_len < (kdfreq.MIN_STEM_SYLLABLES if "krdict" in best else MIN_OTHER_STEM)
        out.append({
            "text": kdfreq.proverb_variants(canon["hw"])[0],
            "headwords": " || ".join(headwords),
            "keys": "|".join(keys),
            "krdict_id": best.get("krdict", {}).get("id", ""),
            "stdict_id": best.get("stdict", {}).get("id", ""),
            "opendict_id": best.get("opendict", {}).get("id", ""),
            "sources": ",".join(order),
            "krdict": int("krdict" in best),
            "short": int(short),
        })
    out.sort(key=lambda r: (-r["krdict"], -len(r["sources"].split(",")), r["keys"]))
    for n, r in enumerate(out, 1):
        r["id"] = f"sk{n:05d}"
    return out


# ------------------------------------------------------------------ 성어

STRONG = [
    re.compile(r"뜻으로[^.]*(이르|일컫|일컬|비유|말함|말\.|말$|나타냄|뜻함)"),
    re.compile(r"비유적으로|비유하여|비유한"),
    re.compile(r"에 나오는 말|에서 유래|에서 나온|에서 비롯|고사|출전"),
]
EN_IDIOM = re.compile(r"idiom|literally|figurative|metaphor", re.I)
# verbal-noun endings of a clause-style gloss ("…돌아감.", "…없음."); plain nouns that happen to end
# in ㅁ (수염, 마음) must not match, so '음' needs a consonant-final syllable before it (먹음, 있음)
_CONS = "".join(chr(0xAC00 + i) for i in range(11172) if i % 28 not in (0, 16))
WEAK_END = re.compile(f"(?:[함됨임감옴봄줌둠짐김]|[{_CONS}]음)\\s*(?:\\.|$)")
NOT_A_THING = re.compile(r"또는|따위|의 하나|한 가지|\bㆍ")
ALLOWED_TAGS = {"북한어"}
# unmarked (tier 2) entries ending in a productive noun suffix are technical terms, not 성어
SUFFIXY = set("화성적론학력률법식형계용품제상체층자원소")


def classify_seongeo(data, source, hw=""):
    """-> (tier, marker, nk) or None.  tier 1 = definition marks it as an idiom, tier 2 = unmarked
    clause-style definition (a verbal-noun gloss like '…돌아감.'), None = ordinary word / domain term."""
    j = json.loads(data)
    senses = j.get("senses") or []
    tags = {t for s in senses for t in s.get("tags", [])}
    if tags - ALLOWED_TAGS:
        return None
    nk = int("북한어" in tags)
    defs = [s.get("ko_def", "") for s in senses if s.get("ko_def")]
    en = " ".join(s.get("def", "") for s in senses)
    text = " ".join(defs)
    for n, rx in enumerate(STRONG):
        m = rx.search(text)
        if m:
            return 1, ["뜻으로", "비유", "출전·유래"][n], nk
    if source == "krdict" and EN_IDIOM.search(en):
        return 1, "krdict:idiom/figurative", nk
    if hw and hw[-1] in SUFFIXY:
        return None
    if defs and WEAK_END.search(defs[0]) and not NOT_A_THING.search(defs[0]) and "idiom" not in en:
        return 2, "clause-def", nk
    return None


def seongeo_rows(pack_dir):
    """Candidates = headwords whose entry in ANY pack looks like a 성어 (classify_seongeo).  The classifier only
    decides candidacy; which packs have the word is decided by headword + hanja afterwards.  (krdict's own
    definitions rarely carry a 성어 marker -- 설상가상: "곤란하거나 불행한 일이 잇따라 일어남." -- so the
    classifier misses them, and a word only stdict's definition flagged used to be recorded as stdict-only.)"""
    by_word = {}
    every = {}  # (src, headword, hanja) -> first (best-ordered) entry id, classified or not
    for src, pack in SOURCES:
        con = connect(pack_dir, pack)
        q = ("select id, headword, hanja, data from entries where kind='word' and hanja is not null "
             "and length(headword)=4 and length(hanja)=4 "
             + ("and source='krdict' " if src == "krdict" else "")
             + "order by quality desc, rank asc, id asc")
        for eid, hw, hanja, data in con.execute(q):
            if not re.fullmatch(r"[가-힣]{4}", hw) or not re.fullmatch(r"[一-鿿]{4}", hanja):
                continue
            every.setdefault((src, hw, hanja), eid)
            c = classify_seongeo(data, src, hw)
            if c is None:
                continue
            d = by_word.setdefault(hw, {"hanja": [], "src": {}, "tier": 9, "marker": "", "nk": 1})
            if hanja not in d["hanja"]:
                d["hanja"].append(hanja)
            d["src"].setdefault(src, eid)
            if c[0] < d["tier"]:
                d["tier"], d["marker"] = c[0], c[1]
            d["nk"] = min(d["nk"], c[2])
    for hw, d in by_word.items():
        for src, _ in SOURCES:
            if src not in d["src"]:
                for hanja in d["hanja"]:
                    if (src, hw, hanja) in every:
                        d["src"][src] = every[(src, hw, hanja)]
                        break
    out = []
    for hw, d in by_word.items():
        out.append({
            "text": hw, "hanja": "|".join(d["hanja"]),
            "krdict_id": d["src"].get("krdict", ""), "stdict_id": d["src"].get("stdict", ""),
            "opendict_id": d["src"].get("opendict", ""),
            "sources": ",".join(s for s, _ in SOURCES if s in d["src"]),
            "krdict": int("krdict" in d["src"]), "tier": d["tier"], "marker": d["marker"],
            "north_korean_only": int(d["nk"] and set(d["src"]) == {"opendict"}),
        })
    out.sort(key=lambda r: (r["tier"], -r["krdict"], -len(r["sources"].split(",")), r["text"]))
    for n, r in enumerate(out, 1):
        r["id"] = f"sg{n:05d}"
    return out


def keep_ids(rows, prev_path, prefix):
    """Ids are what seongeo_review.tsv / the corpus counts refer to, so they must not move when a fix changes the sort
    order: rows whose text is in the previous file keep their id (and position); new texts get fresh ids at the end."""
    if not os.path.exists(prev_path):
        return rows
    with open(prev_path, encoding="utf-8") as f:
        prev = {}
        for ln in list(f)[1:]:
            c = ln.rstrip("\n").split("\t")
            prev[c[1]] = c[0]
    nxt = max((int(i[len(prefix):]) for i in prev.values()), default=0)
    for r in rows:
        if r["text"] in prev:
            r["id"] = prev[r["text"]]
        else:
            nxt += 1
            r["id"] = f"{prefix}{nxt:05d}"
    rows.sort(key=lambda r: r["id"])
    return rows


def write_tsv(path, rows, cols):
    with open(path, "w", encoding="utf-8") as f:
        f.write("\t".join(cols) + "\n")
        for r in rows:
            f.write("\t".join(str(r[c]).replace("\t", " ").replace("\n", " ") for c in cols) + "\n")


SOKDAM_COLS = ["id", "text", "keys", "headwords", "krdict_id", "stdict_id", "opendict_id", "sources", "krdict", "short"]
SEONGEO_COLS = ["id", "text", "hanja", "krdict_id", "stdict_id", "opendict_id", "sources", "krdict", "tier",
                "marker", "north_korean_only"]


def main():
    pack_dir, out_dir = sys.argv[1], sys.argv[2]
    os.makedirs(out_dir, exist_ok=True)
    s = sokdam_rows(pack_dir)
    write_tsv(os.path.join(out_dir, "candidates_sokdam.tsv"), s, SOKDAM_COLS)
    g = keep_ids(seongeo_rows(pack_dir), os.path.join(out_dir, "candidates_seongeo.tsv"), "sg")
    write_tsv(os.path.join(out_dir, "candidates_seongeo.tsv"), g, SEONGEO_COLS)
    print(f"sokdam: {len(s)} unique ({sum(r['krdict'] for r in s)} in krdict, "
          f"{sum(r['short'] for r in s)} too short to rank)")
    print(f"seongeo: {len(g)} unique; tier1={sum(r['tier'] == 1 for r in g)} tier2={sum(r['tier'] == 2 for r in g)} "
          f"krdict={sum(r['krdict'] for r in g)}")


if __name__ == "__main__":
    main()
