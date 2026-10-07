"""Tatoeba kor-eng sentence pairs."""
from __future__ import annotations

import bz2
import tarfile

from ..common import clean, has_hangul


def read_sentences(path, wanted: set[str] | None = None) -> dict[str, str]:
    out: dict[str, str] = {}
    with bz2.open(path, "rt", encoding="utf-8", newline="") as fh:
        for line in fh:
            parts = line.rstrip("\r\n").split("\t")
            if len(parts) < 3:
                continue
            if wanted is not None and parts[0] not in wanted:
                continue
            out[parts[0]] = parts[2]
    return out


def iter_links(path):
    """Stream (sentence_id, translation_id) from links.tar.bz2 (links.csv)."""
    with tarfile.open(path, "r|bz2") as tf:
        for member in tf:
            if not member.isfile() or not member.name.endswith(".csv"):
                continue
            fh = tf.extractfile(member)
            for raw in fh:
                parts = raw.decode("utf-8").rstrip("\r\n").split("\t")
                if len(parts) >= 2:
                    yield parts[0], parts[1]
            return
    # also accept an un-tarred bz2 csv
    return


def iter_links_any(path):
    p = str(path)
    if p.endswith(".tar.bz2") or p.endswith(".tar"):
        yield from iter_links(path)
    else:
        with bz2.open(path, "rt", encoding="utf-8") as fh:
            for line in fh:
                parts = line.rstrip("\r\n").split("\t")
                if len(parts) >= 2:
                    yield parts[0], parts[1]


def parse(kor_path, eng_path, links_path):
    """Yield {'ko','en','tatoeba_id'} with the first English translation per Korean sentence."""
    kor = read_sentences(kor_path)
    cand: dict[str, list[str]] = {}
    for a, b in iter_links_any(links_path):
        if a in kor:
            cand.setdefault(a, []).append(b)
    need = {b for v in cand.values() for b in v}
    eng = read_sentences(eng_path, need)
    for kid in sorted(kor, key=lambda x: int(x) if x.isdigit() else 0):
        ko = clean(kor[kid])
        if not ko or not has_hangul(ko):
            continue
        en = None
        for b in cand.get(kid, ()):
            if b in eng:
                en = clean(eng[b])
                break
        if en:
            yield {"ko": ko, "en": en, "tatoeba_id": kid}
