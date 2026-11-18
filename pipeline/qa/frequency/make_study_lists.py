#!/usr/bin/env python3
"""Build pipeline/data/study_sokdam.tsv and study_seongeo.tsv (the Idioms & proverbs study lists).

    python3 -I make_study_lists.py RESULTS_DIR PACK_DIR OUT_DIR [--review-dir DIR]

RESULTS_DIR   sokdam_counts.tsv + seongeo_counts.tsv written by count_corpus.py / report.py
PACK_DIR      core.sqlite, stdict.sqlite, opendict.sqlite (read-only; used for ext_id + headword)
OUT_DIR       where study_sokdam.tsv / study_seongeo.tsv go (pipeline/data)
review-dir    holds the AI-drafted decisions (sokdam_review.tsv, seongeo_review.tsv; default: this folder)

The 성어 ranking is the corpus score of the candidates the review file marks `keep`.  The 속담 ranking
applies the review rules in `adjusted_scores`: title-suspect entries are scored on the non-Wikipedia
corpora only, partial-match-only entries lose their partial hits, and near-duplicate variants are dropped.
"""
import csv
import os
import sqlite3
import sys
import unicodedata

HERE = os.path.dirname(os.path.abspath(__file__))
# eojeol counts of the corpora (summary.md), to turn a raw count back into a per-million rate
TOKENS = {"kowiki": 174_704_046, "kowikisource": 48_344_284, "opensubtitles": 5_205_855}
CORPORA = ["kowiki", "kowikisource", "opensubtitles"]
PACKS = {"krdict": "core", "stdict": "stdict", "opendict": "opendict"}
SOKDAM_TOP, SEONGEO_TOP, SEONGEO_COMMON = 100, 100, 300

# --- decisions about proverbs whose hits look like titles (see sokdam_review.tsv for the reasons) ---
TITLE_SUSPECT = {
    "주먹이 운다": "film title (2005); 148 of 151 hits are kowiki, 1 subtitle line",
    "지성이면 감천": "drama/film titles; 71 of 84 hits are kowiki, none in subtitles",
    "무자식 상팔자": "drama title (JTBC, 2012); 58 of 60 hits are kowiki",
    "금이야 옥이야": "drama title; 34 of 43 hits are kowiki, none in subtitles",
    "피는 물보다 진하다": "drama/film titles; 22 of 24 hits are kowiki",
    "울며 겨자 먹기": "possible work titles; 31 of 42 hits are kowiki, none in subtitles",
}


# variants the automatic check (same counts, same stem) cannot see because their counts differ
DUPLICATE_OF = {"팔도를 무른 메주 밟듯": "무른 메주 밟듯"}


# wiki-heavy entries that were looked at and kept (the corpus text itself is not available offline: judged from counts and knowledge)
KEEP_NOTES = {
    "눈에는 눈을 이에는 이를": "kowiki-heavy (53 of 58) but the maxim itself (Code of Hammurabi, Bible), not a work title: kept",
    "우물 안 개구리": "24 of 30 hits are kowiki; the idiom is used in articles, no work title known: kept",
    "겉과 속이 다르다": "generic descriptive phrase used in articles, not a title: kept",
    "주사위는 던져졌다": "Caesar quote used in articles, 4 subtitle hits: kept",
}


def display(text):
    """The proverb as shown in the list: no hanja glosses ("백문이 불여일견不如一見" -> "백문이 불여일견")."""
    s = "".join(c for c in text if not ("一" <= c <= "鿿" or "豈" <= c <= "﫿"))
    return s.replace("()", "").strip()


def nfc(s):
    return unicodedata.normalize("NFC", s)


def read_tsv(path):
    with open(path, encoding="utf-8") as f:
        lines = [ln for ln in f if not ln.startswith("#")]
    return list(csv.DictReader(lines, delimiter="\t"))


def connect(pack_dir, src):
    p = os.path.join(pack_dir, PACKS[src] + ".sqlite")
    return sqlite3.connect(f"file:{p}?mode=ro", uri=True)


class Entries:
    """ext_id + headword of the dictionary entries the candidate files point at (by pack id)."""

    def __init__(self, pack_dir):
        self.dbs = {s: connect(pack_dir, s) for s in PACKS}

    def get(self, src, eid):
        r = self.dbs[src].execute("select ext_id, headword from entries where id=?", (int(eid),)).fetchone()
        if r is None:
            raise SystemExit(f"{src} entry {eid} not found: candidates do not match the packs")
        return r


def best_ref(ents, cand):
    for src in ("krdict", "stdict", "opendict"):
        if cand.get(src + "_id"):
            ext, hw = ents.get(src, cand[src + "_id"])
            return src, ext, hw
    raise SystemExit("candidate without an entry: " + cand["id"])


def adjusted_scores(r):
    """-> (decision, reason, new_score) for one counted proverb row."""
    full = {c: int(r[f"{c}_full"]) for c in CORPORA}
    part = {c: int(r[f"{c}_partial"]) for c in CORPORA}
    total = sum(full.values()) + sum(part.values())

    def score(counts):
        # same definition as the report: mean over the three corpora of occurrences per million eojeol
        return sum(counts[c] / TOKENS[c] * 1e6 for c in CORPORA) / len(CORPORA)

    if total and sum(part.values()) / total >= 0.5:
        return ("partial", f"{sum(part.values())} of {total} hits are partial matches of a generic first half", score(full))
    if r["text"] in TITLE_SUSPECT:
        non_wiki = {c: (full[c] + part[c] if c != "kowiki" else 0) for c in CORPORA}
        return ("title", TITLE_SUSPECT[r["text"]], score(non_wiki))
    return ("keep", "", float(r["score"]))


def near_duplicate(a, b):
    """Variants of one proverb (… 하다 / 한다, 터지랴 / 터진다): same counts and the same stem."""
    if not all(a[k] == b[k] for k in a if k.endswith(("_full", "_partial"))):
        return False
    ka, kb = a["text"].replace(" ", ""), b["text"].replace(" ", "")
    n = min(len(ka), len(kb))
    if n < 6:
        return False
    return ka in kb or kb in ka or ka[: n - 3] == kb[: n - 3]


def sokdam(results, cands, ents, review_path):
    rows = read_tsv(os.path.join(results, "sokdam_counts.tsv"))
    by_id = {c["id"]: c for c in cands}
    for r in rows:
        r["decision"], r["reason"], r["adj"] = adjusted_scores(r)
        r["cand"] = by_id[r["id"]]
    order = lambda r: (-r["adj"], -int(r["krdict"]), -int(r["total_full"]), nfc(r["text"]))
    ranked = sorted((r for r in rows if r["short"] == "0"), key=order)
    # near-duplicates: keep the first (best-ordered) variant of a proverb
    kept, dropped = [], {}
    for r in rows:
        if r["decision"] == "partial" and int(r["total_full"]) == 0 and r["krdict"] == "0":
            dropped[r["id"]] = ("drop", "only a generic partial phrase matched; the proverb itself never occurs")
    for r in ranked:
        if r["text"] in DUPLICATE_OF:
            dropped[r["id"]] = ("dup", f"variant of '{DUPLICATE_OF[r['text']]}' (a longer form of the same proverb)")
            continue
        if r["adj"] > 0:
            dup = next((k for k in kept[:400] if near_duplicate(k, r)), None)
            if dup is not None:
                dropped[r["id"]] = ("dup", f"variant of '{dup['text']}' (same counts, same stem)")
                continue
        kept.append(r)
    top = []
    for r in kept:
        if len(top) == SOKDAM_TOP:
            break
        if r["id"] in dropped or r["adj"] <= 0:
            continue
        top.append(r)
    # every krdict proverb (also the one under the syllable limit, 옥에 티, which is too generic to rank
    # and so counts as zero): zero-count proverbs after the counted ones, in 가나다 order
    learner = [r for r in rows if r["krdict"] == "1"]
    learner.sort(key=lambda r: (r["adj"] <= 0, -r["adj"] if r["adj"] > 0 else 0, nfc(r["text"])))
    # review file: every decision that changed something, plus the first 150 ranks as the original scorer saw them
    with open(review_path, "w", encoding="utf-8", newline="") as f:
        f.write("# 속담 review (AI draft, not yet checked by a human): decisions that change the ranking.\n")
        f.write("# decision: title = scored on the non-Wikipedia corpora only; partial = scored on full matches only;\n")
        f.write("#   drop = removed from the top-100 (junk); dup = variant of a better-ranked entry.  keep = unchanged (top 150 only).\n")
        f.write("# old_rank/old_score = the corpus ranking before these rules (counts in sokdam_counts.tsv).\n")
        w = csv.writer(f, delimiter="\t", lineterminator="\n")
        w.writerow(["id", "text", "decision", "reason", "by", "kowiki", "kowikisource", "opensubtitles", "old_rank", "old_score", "new_score"])
        for r in rows:
            d, why = (r["decision"], r["reason"])
            if r["id"] in dropped:
                d, why = dropped[r["id"]]
            rk = int(r["rank"] or 10**9)
            if d == "keep" and rk > 150:
                continue
            if d == "partial" and rk > 400:
                continue
            hits = [str(int(r[f"{c}_full"]) + int(r[f"{c}_partial"])) for c in CORPORA]
            w.writerow([r["id"], r["text"], d, why or KEEP_NOTES.get(r["text"], "no sign of a work title; hits spread over the corpora"), "ai-draft", *hits, r["rank"], r["score"], f"{r['adj']:.3f}"])
    return top, learner, rows


def sokdam_out(top, learner, ents):
    out = []
    for name, lst in (("sokdam100", top), ("sokdam_all", learner)):
        for i, r in enumerate(lst, 1):
            src, ext, hw = best_ref(ents, r["cand"])
            hits = [str(int(r[f"{c}_full"]) + int(r[f"{c}_partial"])) for c in CORPORA]
            note = r["decision"] if r["decision"] != "keep" else ""
            out.append([name, i, display(r["text"]), src, ext, hw, f"{r['adj']:.3f}", *hits, note])
    return out


def seongeo(results, cands, ents, review_path):
    rows = read_tsv(os.path.join(results, "seongeo_counts.tsv"))
    cand = {c["id"]: c for c in cands}
    review = {r["id"]: r for r in read_tsv(review_path)}
    keep = [r for r in rows if review.get(r["id"], {}).get("status") == "keep"]
    # the review covers the top ~500 by score and the lists end at rank 300 of the keepers
    keep.sort(key=lambda r: int(r["rank"]))
    out = []
    for name, n in (("seongeo100", SEONGEO_TOP), ("seongeo300", SEONGEO_COMMON)):
        if len(keep) < n:
            raise SystemExit(f"only {len(keep)} reviewed 성어 keepers, need {n}")
        for i, r in enumerate(keep[:n], 1):
            src, ext, hw = best_ref(ents, cand[r["id"]])
            out.append([name, i, r["text"], src, ext, hw, r["score"], r["kowiki_count"], r["kowikisource_count"], r["opensubtitles_count"], ""])
    # every krdict 성어 the review keeps (also those reviewed beyond the first cut): counted ones by score,
    # then the ones that never occur in the corpora, in 가나다 order
    learner = [r for r in keep if cand[r["id"]]["krdict"] == "1"]
    learner.sort(key=lambda r: (float(r["score"]) <= 0, -float(r["score"]), int(r["rank"]) if float(r["score"]) > 0 else 0, nfc(r["text"])))
    for i, r in enumerate(learner, 1):
        src, ext, hw = best_ref(ents, cand[r["id"]])
        out.append(["seongeo_learner", i, r["text"], src, ext, hw, r["score"], r["kowiki_count"], r["kowikisource_count"], r["opensubtitles_count"], ""])
    return out


HEADER = """# {title}
# Built by pipeline/qa/frequency/make_study_lists.py from corpus counts of the dictionary candidates
# (pipeline/qa/frequency/candidates_*.tsv; count_corpus.py).  Corpora: Korean Wikipedia (kowiki dump),
# OpenSubtitles 2018 Korean (OPUS), Korean Wikisource (kowikisource dump).
# score = mean over the three corpora of (occurrences per million eojeol).  {method}
# Columns: list, rank, text (display), source (pack of the entry: krdict = core, stdict, opendict),
#   ext_id + headword (the entry in that pack: the pack's own id and exact headword),
#   score, kowiki / kowikisource / opensubtitles = raw hit counts, note.
# {lists}
"""
COLS = ["list", "rank", "text", "source", "ext_id", "headword", "score", "kowiki", "kowikisource", "opensubtitles", "note"]


def write(path, header, rows):
    with open(path, "w", encoding="utf-8", newline="") as f:
        f.write(header)
        w = csv.writer(f, delimiter="\t", lineterminator="\n")
        w.writerow(COLS)
        w.writerows(rows)


def main(argv):
    args = [a for a in argv if not a.startswith("--")]
    review_dir = HERE
    if "--review-dir" in argv:
        review_dir = argv[argv.index("--review-dir") + 1]
        args.remove(review_dir)
    results, pack_dir, out_dir = args
    ents = Entries(pack_dir)
    sok_c = read_tsv(os.path.join(HERE, "candidates_sokdam.tsv"))
    seo_c = read_tsv(os.path.join(HERE, "candidates_seongeo.tsv"))
    top, learner, _ = sokdam(results, sok_c, ents, os.path.join(review_dir, "sokdam_review.tsv"))
    write(os.path.join(out_dir, "study_sokdam.tsv"), HEADER.format(
        title="속담 study lists: sokdam100 (Common proverbs, top 100) and sokdam_all (All learner proverbs)",
        method=("Proverbs are matched by their fixed stem (any ending) or, for long ones, by the quoted first half "
                "(partial). Review rules (sokdam_review.tsv): entries whose hits are mostly Wikipedia and look like film/drama/book "
                "titles are scored on the other two corpora only; entries that only matched a generic partial phrase are scored on "
                "full matches only (dropped from the top 100 when there is none); variants of one proverb count once."),
        lists="sokdam100 = top 100 of all dictionaries (krdict, stdict, opendict); sokdam_all = krdict's proverbs, counted ones first, then zero-count ones in 가나다 order."),
        sokdam_out(top, learner, ents))
    seo = seongeo(results, seo_c, ents, os.path.join(review_dir, "seongeo_review.tsv"))
    write(os.path.join(out_dir, "study_seongeo.tsv"), HEADER.format(
        title="성어 study lists: seongeo100 (Essential 성어, top 100), seongeo300 (Common 성어, top 300), seongeo_learner (All learner 성어)",
        method=("The candidates are four-syllable dictionary entries with a 성어-style definition. seongeo_review.tsv records the AI-drafted "
                "keep/drop decision for each of the top ~500 by score: ordinary compound nouns (국가대표, 돌연변이 ...) are dropped; "
                "the lists are the first 100 / 300 keepers by score."),
        lists="seongeo100 is the first 100 rows of seongeo300; seongeo_learner = every krdict 성어 the review keeps, by score, then zero-count ones in 가나다 order."),
        seo)
    print("top100", len(top), "learner", len(learner), "seongeo_learner", sum(1 for r in seo if r[0] == "seongeo_learner"))


if __name__ == "__main__":
    main(sys.argv[1:])
