#!/usr/bin/env python3
"""Count 속담 / 성어 candidates in one streamed corpus.

    python3 -I count_corpus.py --name kowiki --format wikixml --source URL [--source URL ...] \
        --sokdam candidates_sokdam.tsv --seongeo candidates_seongeo.tsv --out DIR [--workers N]

The corpus is streamed (HTTP -> (b|g)zip decompressor -> line reader) and cut into ~4 MB chunks
that a process pool searches with an Aho-Corasick automaton (pyahocorasick; a slow pure-Python
fallback keeps the tests runnable without it).  Nothing is written to disk but the result tables:

  DIR/<name>.sokdam.tsv   id, full, partial   ('partial' = first-half quotes NOT followed by the rest)
  DIR/<name>.seongeo.tsv  id, count
  DIR/<name>.stats.json   tokens (Hangul eojeol), syllables, segments, bytes read, seconds
"""
import argparse
import bz2
import csv
import gzip
import html
import json
import multiprocessing as mp
import os
import re
import sys
import threading
import time
import urllib.request

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kdfreq  # noqa: E402

try:
    import ahocorasick  # type: ignore
except ImportError:  # pragma: no cover - exercised only where the wheel is missing
    ahocorasick = None

CHUNK_CHARS = 4_000_000
USER_AGENT = "korean-dictionary-frequency-study/1.0 (GitHub Actions; one-off corpus counts)"


# ---------------------------------------------------------------- matcher abstraction

class SlowMatcher:
    """Pure-python stand-in with the same iter() contract (end index, values); tests only."""

    def __init__(self):
        self.words = {}

    def add(self, key, value):
        self.words.setdefault(key, []).append(value)

    def finish(self):
        pass

    def iter(self, text):
        for key, vals in self.words.items():
            i = text.find(key)
            while i >= 0:
                yield i + len(key) - 1, key, vals
                i = text.find(key, i + 1)


class FastMatcher:
    def __init__(self):
        self.a = ahocorasick.Automaton()
        self.n = 0  # pyahocorasick refuses to build or search an automaton with no keys

    def add(self, key, value):
        cur = self.a.get(key, None)
        if cur is None:
            self.a.add_word(key, (key, [value]))
            self.n += 1
        else:
            cur[1].append(value)

    def finish(self):
        if self.n:
            self.a.make_automaton()

    def iter(self, text):
        if not self.n:
            return
        for end, (key, vals) in self.a.iter(text):
            yield end, key, vals


def make_matcher():
    return FastMatcher() if ahocorasick is not None else SlowMatcher()


# ---------------------------------------------------------------- candidates

def read_tsv(path):
    with open(path, encoding="utf-8", newline="") as f:
        return list(csv.DictReader(f, delimiter="\t"))


STATE = {}


def init_worker(sokdam_path, seongeo_path):
    """Build both automata once per process."""
    sok = read_tsv(sokdam_path) if sokdam_path else []
    seo = read_tsv(seongeo_path) if seongeo_path else []
    m1 = make_matcher()
    for i, r in enumerate(sok):
        if int(r["short"]):
            continue
        for hw in r["headwords"].split(" || "):
            p = kdfreq.proverb_patterns(hw)
            for s in p["full"]:
                m1.add(s, (i, 0))
            for s in p["partial"]:
                m1.add(s, (i, 1))
    m1.finish()
    m2 = make_matcher()
    for i, r in enumerate(seo):
        m2.add(r["text"], i)
    m2.finish()
    STATE.update(m1=m1, m2=m2, nsok=len(sok), nseo=len(seo))


# ---------------------------------------------------------------- wikitext cleaning

_TMPL = re.compile(r"\{\{[^{}]*\}\}")
_TABLE = re.compile(r"\{\|.*?\|\}", re.S)
_REF = re.compile(r"<ref[^>]*?/>|<ref[^>]*>.*?</ref>", re.S | re.I)
_COMMENT = re.compile(r"<!--.*?-->", re.S)
_TAG = re.compile(r"</?[a-zA-Z][^>]*>")
_FILE = re.compile(r"\[\[(?:File|Image|파일|그림|Category|분류):[^\[\]]*(?:\[\[[^\[\]]*\]\][^\[\]]*)*\]\]", re.I)
_LINK2 = re.compile(r"\[\[[^\[\]|]*\|([^\[\]]*)\]\]")
_LINK1 = re.compile(r"\[\[([^\[\]]*)\]\]")
_EXT = re.compile(r"\[(?:https?:)?//\S+\s*([^\]]*)\]")
_QUOTES = re.compile(r"'{2,}")
_HEAD = re.compile(r"^=+\s*(.*?)\s*=+\s*$", re.M)
_LISTMARK = re.compile(r"^[*#:;]+", re.M)


def strip_wikitext(t):
    t = html.unescape(t)
    t = _COMMENT.sub("", t)
    t = _REF.sub("", t)
    t = _TABLE.sub("", t)
    for _ in range(4):
        t2 = _TMPL.sub("", t)
        if t2 == t:
            break
        t = t2
    t = _FILE.sub("", t)
    for _ in range(2):
        t = _LINK2.sub(r"\1", t)
        t = _LINK1.sub(r"\1", t)
    t = _EXT.sub(r"\1", t)
    t = _TAG.sub("", t)
    t = _QUOTES.sub("", t)
    t = _HEAD.sub(r"\1", t)
    t = _LISTMARK.sub("", t)
    return t


# ---------------------------------------------------------------- the worker

def count_chunk(args):
    """args = (kind, text).  kind 'wiki': a list of raw wikitext pages joined by '\x00';
    kind 'text': plain text lines.  Returns (sok, seo, tokens, syllables, segments)."""
    kind, raw = args
    if kind == "wiki":
        pages = raw.split("\x00")
        text = "\n".join(strip_wikitext(p) for p in pages)
        segments = len(pages)
    else:
        text = raw
        segments = raw.count("\n") + 1
    spaced = kdfreq.spaced_view(text)
    compact = kdfreq.compact_view(text)
    jam = kdfreq.jamo(compact)
    tokens = sum(1 for w in spaced.split() if w != "|")
    syll = len(compact) - compact.count("|")

    full = {}
    part = {}
    for end, key, vals in STATE["m1"].iter(jam):
        start = end - len(key) + 1
        for (i, kind01) in vals:
            (part if kind01 else full).setdefault(i, set()).add(start)
    sok = {}
    for i, fs in full.items():
        sok.setdefault(i, [0, 0])[0] += len(fs)
    for i, ps in part.items():
        only = len(ps - full.get(i, set()))
        if only:
            sok.setdefault(i, [0, 0])[1] += only

    seo = {}
    for end, key, vals in STATE["m2"].iter(spaced):
        start = end - len(key) + 1
        if start > 0 and spaced[start - 1] not in " |":
            continue
        j = end + 1
        while j < len(spaced) and spaced[j] not in " |":
            j += 1
        if kdfreq.seongeo_tail_ok(spaced[end + 1: j]):
            for i in vals:
                seo[i] = seo.get(i, 0) + 1
    return sok, seo, tokens, syll, segments


# ---------------------------------------------------------------- streaming readers

def open_stream(sources):
    last = None
    for url in sources:
        try:
            if os.path.exists(url):
                return open(url, "rb"), url
            req = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
            return urllib.request.urlopen(req, timeout=120), url
        except Exception as e:  # try the next mirror
            last = e
            print(f"source failed: {url}: {e}", file=sys.stderr)
    raise RuntimeError(f"no source reachable: {last}")


class Prefixed:
    """File-like that replays `head` and then reads on from `rest` (so the magic bytes can be sniffed)."""

    def __init__(self, head, rest):
        self.head, self.rest = head, rest

    def read(self, n=-1):
        if self.head:
            if n is None or n < 0 or n >= len(self.head):
                out, self.head = self.head, b""
                return out + (self.rest.read(-1 if n is None or n < 0 else n - len(out)) or b"")
            out, self.head = self.head[:n], self.head[n:]
            return out
        return self.rest.read(n)

    def readable(self):
        return True

    def close(self):
        self.rest.close()


def decompress(fileobj):
    """Pick the decompressor from the magic bytes (the OPUS mirror URLs do not end in .gz)."""
    head = fileobj.read(3)
    f = Prefixed(head, fileobj)
    if head.startswith(b"BZh"):
        return bz2.BZ2File(f)
    if head.startswith(b"\x1f\x8b"):
        return gzip.GzipFile(fileobj=f)
    return f


def raw_lines(f):
    pending = b""
    while True:
        block = f.read(1 << 20)
        if not block:
            break
        pending += block
        *done, pending = pending.split(b"\n")
        for ln in done:
            yield ln + b"\n"
    if pending:
        yield pending


def wiki_pages(lines):
    """Yield the raw wikitext of every main-namespace, non-redirect page of a pages-articles dump."""
    in_page = in_text = False
    ns_ok = True
    buf = []
    for raw in lines:
        line = raw.decode("utf-8", "replace")
        if not in_page:
            if "<page>" in line:
                in_page, ns_ok = True, True
            continue
        if not in_text:
            if "<ns>" in line:
                ns_ok = "<ns>0</ns>" in line
            k = line.find("<text")
            if k >= 0:
                in_text = True
                gt = line.find(">", k)
                rest = line[gt + 1:]
                if "</text>" in rest:
                    in_text = False
                    buf.append(rest.split("</text>")[0])
                else:
                    buf.append(rest)
            elif "</page>" in line:
                in_page = False
        else:
            if "</text>" in line:
                buf.append(line.split("</text>")[0])
                in_text = False
            else:
                buf.append(line)
        if not in_text and buf:
            text = "".join(buf)
            buf = []
            head = text[:20].lstrip().lower()
            if ns_ok and not head.startswith(("#redirect", "#넘겨주기")):
                yield text
        if "</page>" in line:
            in_page = False


def chunks(kind, lines, dedupe):
    seen = set()
    buf, size = [], 0
    for item in lines:
        if kind == "wiki":
            s = item
        else:
            s = item.decode("utf-8", "replace").rstrip("\n")
            if dedupe and len(s) >= 12:
                h = hash(s)
                if h in seen:
                    continue
                seen.add(h)
        buf.append(s)
        size += len(s)
        if size >= CHUNK_CHARS:
            yield (kind, ("\x00" if kind == "wiki" else "\n").join(buf))
            buf, size = [], 0
    if buf:
        yield (kind, ("\x00" if kind == "wiki" else "\n").join(buf))


def run(a):
    t0 = time.time()
    sok_rows = read_tsv(a.sokdam) if a.sokdam else []
    seo_rows = read_tsv(a.seongeo) if a.seongeo else []
    stream, used = open_stream(a.source)
    counted = {"bytes": 0}

    class Meter:
        def read(self, n=-1):
            b = stream.read(n)
            counted["bytes"] += len(b)
            return b

        def readable(self):
            return True

        def close(self):
            stream.close()

    dec = decompress(Meter())
    if isinstance(dec, Prefixed):
        dec = raw_lines(dec)
    kind = "wiki" if a.format == "wikixml" else "text"
    lines = wiki_pages(dec) if kind == "wiki" else dec
    sok_tot = [[0, 0] for _ in sok_rows]
    seo_tot = [0] * len(seo_rows)
    stats = {"tokens": 0, "syllables": 0, "segments": 0}
    sem = threading.BoundedSemaphore(max(2, a.workers * 2))
    lock = threading.Lock()
    done = {"chunks": 0}

    def merge(res):
        sok, seo, tk, sy, sg = res
        with lock:
            for i, (f, p) in sok.items():
                sok_tot[i][0] += f
                sok_tot[i][1] += p
            for i, c in seo.items():
                seo_tot[i] += c
            stats["tokens"] += tk
            stats["syllables"] += sy
            stats["segments"] += sg
            done["chunks"] += 1
            if done["chunks"] % 20 == 0:
                print(f"[{a.name}] chunks={done['chunks']} read={counted['bytes'] / 1e6:.0f}MB "
                      f"tokens={stats['tokens'] / 1e6:.1f}M t={time.time() - t0:.0f}s", flush=True)
        sem.release()

    def fail(e):
        print("worker error:", repr(e), file=sys.stderr)
        sem.release()

    with mp.Pool(a.workers, initializer=init_worker, initargs=(a.sokdam, a.seongeo)) as pool:
        for ch in chunks(kind, lines, a.dedupe):
            sem.acquire()
            pool.apply_async(count_chunk, (ch,), callback=merge, error_callback=fail)
            if a.limit_mb and counted["bytes"] > a.limit_mb * 1e6:
                break
        pool.close()
        pool.join()

    os.makedirs(a.out, exist_ok=True)
    with open(os.path.join(a.out, f"{a.name}.sokdam.tsv"), "w", encoding="utf-8") as f:
        f.write("id\tfull\tpartial\n")
        for r, (fu, pa) in zip(sok_rows, sok_tot):
            f.write(f"{r['id']}\t{fu}\t{pa}\n")
    with open(os.path.join(a.out, f"{a.name}.seongeo.tsv"), "w", encoding="utf-8") as f:
        f.write("id\tcount\n")
        for r, c in zip(seo_rows, seo_tot):
            f.write(f"{r['id']}\t{c}\n")
    stats.update(name=a.name, source=used, bytes_read=counted["bytes"], seconds=round(time.time() - t0),
                 dedupe=a.dedupe, workers=a.workers)
    with open(os.path.join(a.out, f"{a.name}.stats.json"), "w", encoding="utf-8") as f:
        json.dump(stats, f, ensure_ascii=False, indent=1)
    print(json.dumps(stats, ensure_ascii=False))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--name", required=True)
    ap.add_argument("--format", choices=["wikixml", "text"], required=True)
    ap.add_argument("--source", action="append", required=True, help="URL or local path; tried in order")
    ap.add_argument("--sokdam")
    ap.add_argument("--seongeo")
    ap.add_argument("--out", required=True)
    ap.add_argument("--workers", type=int, default=os.cpu_count() or 2)
    ap.add_argument("--dedupe", action="store_true", help="drop repeated long lines (subtitle duplicates)")
    ap.add_argument("--limit-mb", type=float, default=0, help="stop after this many compressed MB (smoke tests)")
    run(ap.parse_args())


if __name__ == "__main__":
    main()
