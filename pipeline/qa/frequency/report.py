#!/usr/bin/env python3
"""Merge the per-corpus counts into results/{sokdam,seongeo}_counts.tsv and summary.md.

    python3 -I report.py CAND_DIR COUNTS_DIR RESULTS_DIR

Score = mean over corpora of (full + partial) occurrences per million eojeol.  The rank-frequency
curve is computed on the score order; the min/median columns show the raw total occurrences."""
import csv
import glob
import json
import os
import statistics
import sys

BUCKETS = [(1, 50), (51, 100), (101, 200), (201, 300), (301, 500), (501, 1000), (1001, None)]
AT = [100, 300, 500, 1000]


def read_tsv(p):
    with open(p, encoding="utf-8", newline="") as f:
        return list(csv.DictReader(f, delimiter="\t"))


def load_corpora(counts_dir):
    out = {}
    for sp in sorted(glob.glob(os.path.join(counts_dir, "*.stats.json"))):
        st = json.load(open(sp, encoding="utf-8"))
        name = st["name"]
        out[name] = {"stats": st,
                     "sok": {r["id"]: r for r in read_tsv(os.path.join(counts_dir, f"{name}.sokdam.tsv"))},
                     "seo": {r["id"]: r for r in read_tsv(os.path.join(counts_dir, f"{name}.seongeo.tsv"))}}
    return out


def pm(n, tokens):
    return n * 1e6 / tokens if tokens else 0.0


def build(cands, corpora, kind):
    rows = []
    for c in cands:
        r = {k: c[k] for k in ("id", "text", "sources", "krdict")}
        r["short"] = c.get("short", "0")
        r["tier"] = c.get("tier", "")
        tot_full = tot_part = 0
        pms = []
        for name, d in corpora.items():
            rec = (d["sok"] if kind == "sok" else d["seo"]).get(c["id"])
            tk = d["stats"]["tokens"]
            if kind == "sok":
                f, p = (int(rec["full"]), int(rec["partial"])) if rec else (0, 0)
                r[f"{name}_full"], r[f"{name}_partial"] = f, p
                r[f"{name}_pm"] = f"{pm(f + p, tk):.3f}"
                tot_full += f
                tot_part += p
                pms.append(pm(f + p, tk))
            else:
                n = int(rec["count"]) if rec else 0
                r[f"{name}_count"] = n
                r[f"{name}_pm"] = f"{pm(n, tk):.3f}"
                tot_full += n
                pms.append(pm(n, tk))
        r["total_full"], r["total_partial"] = tot_full, tot_part
        r["total"] = tot_full + tot_part
        r["score"] = sum(pms) / len(pms) if pms else 0.0
        rows.append(r)
    rankable = [r for r in rows if r["short"] != "1"]
    rankable.sort(key=lambda r: (-r["score"], -r["total"], r["id"]))
    for n, r in enumerate(rankable, 1):
        r["rank"] = n
    for r in rows:
        r.setdefault("rank", "")
    rows.sort(key=lambda r: (r["rank"] == "", r["rank"] if r["rank"] != "" else 0))
    for r in rows:
        r["score"] = f"{r['score']:.3f}"
    return rows, rankable


def write(path, rows, cols):
    with open(path, "w", encoding="utf-8") as f:
        f.write("\t".join(cols) + "\n")
        for r in rows:
            f.write("\t".join(str(r.get(c, "")) for c in cols) + "\n")


def curve(rankable):
    lines = ["| ranks | n | min total | median total | max total |", "|---|---|---|---|---|"]
    for lo, hi in BUCKETS:
        sl = rankable[lo - 1: hi] if hi else rankable[lo - 1:]
        if not sl:
            lines.append(f"| {lo}-{hi or ''} | 0 | | | |")
            continue
        t = [r["total"] for r in sl]
        lines.append(f"| {lo}{'-' + str(hi) if hi else '+'} | {len(sl)} | {min(t)} | {statistics.median(t):g} | {max(t)} |")
    at = ["| rank | total occurrences | score (per million eojeol) |", "|---|---|---|"]
    for k in AT:
        if len(rankable) >= k:
            r = rankable[k - 1]
            at.append(f"| {k} | {r['total']} | {float(r['score']):.3f} |")
    return "\n".join(lines) + "\n\n" + "\n".join(at)


def top(rankable, corpora, kind, n=50):
    names = list(corpora)
    head = "| # | text | src | " + " | ".join(f"{c}" for c in names) + " | total |"
    sep = "|---|---|---|" + "---|" * len(names) + "---|"
    out = [head, sep]
    for r in rankable[:n]:
        cells = []
        for c in names:
            cells.append(f"{r[c + '_full']}+{r[c + '_partial']}" if kind == "sok" else str(r[c + "_count"]))
        out.append(f"| {r['rank']} | {r['text']} | {r['sources']} | " + " | ".join(cells) + f" | {r['total']} |")
    return "\n".join(out)


def main():
    cand_dir, counts_dir, out_dir = sys.argv[1:4]
    os.makedirs(out_dir, exist_ok=True)
    corpora = load_corpora(counts_dir)
    sok = read_tsv(os.path.join(cand_dir, "candidates_sokdam.tsv"))
    seo = read_tsv(os.path.join(cand_dir, "candidates_seongeo.tsv"))
    srows, srank = build(sok, corpora, "sok")
    grows, grank = build(seo, corpora, "seo")
    base = ["rank", "id", "text", "sources", "krdict", "short"]
    scols = base + [f"{c}_{k}" for c in corpora for k in ("full", "partial", "pm")] + \
        ["total_full", "total_partial", "total", "score"]
    gcols = ["rank", "id", "text", "sources", "krdict", "tier"] + \
        [f"{c}_{k}" for c in corpora for k in ("count", "pm")] + ["total", "score"]
    write(os.path.join(out_dir, "sokdam_counts.tsv"), srows, scols)
    write(os.path.join(out_dir, "seongeo_counts.tsv"), grows, gcols)

    md = ["# 속담 / 성어 corpus frequency", ""]
    md.append("| corpus | source | eojeol (tokens) | Hangul syllables | segments | read | seconds |")
    md.append("|---|---|---|---|---|---|---|")
    for name, d in corpora.items():
        s = d["stats"]
        md.append(f"| {name} | {s['source']} | {s['tokens']:,} | {s['syllables']:,} | {s['segments']:,} | "
                  f"{s['bytes_read'] / 1e6:.0f} MB | {s['seconds']} |")
    md += ["", "Score = mean over corpora of (full + partial) occurrences per million eojeol. "
           "`full` = the fixed stem up to the last predicate with any ending; `partial` = the quoted first "
           "half (e.g. 가는 말이 고와야) NOT followed by the rest. Proverbs with a fixed stem under "
           f"4 syllables (krdict) or 6 syllables (stdict/opendict only) are too generic and are not ranked ({sum(r['short'] == '1' for r in srows)} such).", ""]
    for title, rank, kind, n_all in (("속담 (proverbs)", srank, "sok", len(srows)),
                                     ("성어 (four-character idioms)", grank, "seo", len(grows))):
        nz = sum(1 for r in rank if r["total"] > 0)
        kr = [r for r in rank if r["krdict"] == "1"]
        md += [f"## {title}", "", f"{n_all} candidates, {nz} with at least one hit, ranked {len(rank)}.", "",
               "### Rank-frequency curve, all dictionaries (total raw occurrences across corpora)", "", curve(rank), "",
               "### Top 50, all dictionaries", "", top(rank, corpora, kind), "",
               f"### Rank-frequency curve, krdict entries only ({len(kr)} ranked; ranks within this subset)", "",
               curve(kr), "",
               "### Top 50, krdict entries only (# = rank among all)", "", top(kr, corpora, kind), ""]
    with open(os.path.join(out_dir, "summary.md"), "w", encoding="utf-8") as f:
        f.write("\n".join(md))
    print("\n".join(md[:30]))


if __name__ == "__main__":
    main()
