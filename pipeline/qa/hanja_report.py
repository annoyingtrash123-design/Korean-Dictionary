#!/usr/bin/env python3
"""summary.md for the hanja certainty review (run by .github/workflows/hanja-report.yml).

usage: python3 -I hanja_report.py OUT_DIR DEST_DIR
Reads OUT_DIR/hanja_fill_report.tsv and OUT_DIR/hanja_kengdic_fallback.tsv, writes DEST_DIR/summary.md:
counts per certainty / rule / evidence, and 40 random LOW + 40 random HIGH rows of each report (seeded, so
a re-run of the same data gives the same sample).
"""
import csv
import random
import sys
from collections import Counter
from pathlib import Path

SEED = 20261124
N = 40


def read(p):
    with open(p, encoding="utf-8", newline="") as f:
        return list(csv.DictReader(f, delimiter="\t", quoting=csv.QUOTE_NONE))


def esc(s):
    return (s or "").replace("|", "\\|").replace("\n", " ")


def table(rows, cols):
    out = ["| " + " | ".join(c for c, _ in cols) + " |", "|" + "---|" * len(cols)]
    for r in rows:
        out.append("| " + " | ".join(esc(r.get(k, "")) for _, k in cols) + " |")
    return "\n".join(out)


def counts(rows, keys):
    c = Counter(tuple(r.get(k, "") for k in keys) for r in rows)
    out = ["| " + " | ".join(keys) + " | n |", "|" + "---|" * (len(keys) + 1)]
    for k, n in sorted(c.items(), key=lambda kv: (-kv[1], kv[0])):
        out.append("| " + " | ".join(k) + f" | {n} |")
    return "\n".join(out)


def sample(rows, cert):
    pool = [r for r in rows if r.get("certainty") == cert]
    random.Random(SEED).shuffle(pool)
    return pool[:N]


def main(out_dir, dest):
    out_dir, dest = Path(out_dir), Path(dest)
    fill = read(out_dir / "hanja_fill_report.tsv")
    fb_path = out_dir / "hanja_kengdic_fallback.tsv"
    fb = read(fb_path) if fb_path.exists() else []
    md = ["# Hanja certainty report", "", "Owner rule: if in doubt, no hanja (only keep what a Korean dictionary confirms).", ""]

    md += [f"## Stdict-based fill (`hanja_fill_report.tsv`): {len(fill)} fills", "", counts(fill, ["certainty"]), ""]
    md += ["### by certainty / rule / evidence", "", counts(fill, ["certainty", "rule", "source", "evidence"]), ""]
    kept = Counter(r.get("kept") for r in fill)
    md += [f"kept: {dict(kept)}", ""]

    fb_rows = [r for r in fb if r.get("certainty")]
    md += [f"## kengdic fallback (`hanja_kengdic_fallback.tsv`): {len(fb)} candidates", "", counts(fb, ["verdict", "certainty", "evidence"]), ""]

    fill_cols = [("headword", "headword"), ("hanja", "hanja"), ("source", "source"), ("rule", "rule"), ("score", "score"),
                 ("core gloss / definition", "core_def"), ("stdict definition", "stdict_def"), ("evidence", "evidence")]
    fb_cols = [("headword", "headword"), ("hanja", "hanja"), ("source", "source"), ("gloss", "gloss"), ("evidence", "evidence"), ("verdict", "verdict")]
    for cert in ("low", "high"):
        md += [f"## {N} random {cert.upper()} fills (stdict-based)", "", table(sample(fill, cert), fill_cols), ""]
        md += [f"## {N} random {cert.upper()} fills (kengdic fallback)", "", table(sample(fb_rows, cert), fb_cols), ""]
    dest.mkdir(parents=True, exist_ok=True)
    (dest / "summary.md").write_text("\n".join(md) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
