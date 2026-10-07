"""Unihan.zip (or a directory with the txt files) -> {char: info}."""
from __future__ import annotations

import io
import zipfile
from pathlib import Path

FILES = ("Unihan_Readings.txt", "Unihan_IRGSources.txt")
KEEP = {"kHangul", "kDefinition", "kKorean", "kTotalStrokes", "kRSUnicode"}


def _open_texts(path):
    p = Path(path)
    if p.is_dir():
        for name in FILES:
            f = p / name
            if f.exists():
                yield name, f.open(encoding="utf-8")
        return
    with zipfile.ZipFile(p) as zf:
        for name in zf.namelist():
            base = name.rsplit("/", 1)[-1]
            if base in FILES:
                yield base, io.TextIOWrapper(zf.open(name), encoding="utf-8")


def parse(path) -> dict[str, dict]:
    raw: dict[str, dict[str, str]] = {}
    for _, fh in _open_texts(path):
        with fh:
            for line in fh:
                if not line.startswith("U+"):
                    continue
                parts = line.rstrip("\n").split("\t", 2)
                if len(parts) < 3 or parts[1] not in KEEP:
                    continue
                try:
                    ch = chr(int(parts[0][2:], 16))
                except ValueError:
                    continue
                raw.setdefault(ch, {})[parts[1]] = parts[2]
    out: dict[str, dict] = {}
    for ch, f in raw.items():
        readings: list[str] = []
        for tok in f.get("kHangul", "").split():
            r = tok.split(":", 1)[0]
            if r and r not in readings:
                readings.append(r)
        if not readings:  # fallback: Yale romanisation from kKorean
            readings = [t.lower() for t in f.get("kKorean", "").split()]
        strokes = None
        ts = f.get("kTotalStrokes", "").split()
        if ts and ts[0].isdigit():
            strokes = int(ts[0])
        radical = radical_char = None
        rs = f.get("kRSUnicode", "").split()
        if rs:
            num = rs[0].split(".")[0].rstrip("'")
            if num.isdigit() and 1 <= int(num) <= 214:
                radical = num
                radical_char = chr(0x2F00 + int(num) - 1)
        if not (readings or f.get("kDefinition") or strokes):
            continue
        out[ch] = {
            "readings": readings,
            "meaning_en": f.get("kDefinition") or None,
            "strokes": strokes,
            "radical": radical,
            "radical_char": radical_char,
            "has_hangul": "kHangul" in f,
        }
    return out
