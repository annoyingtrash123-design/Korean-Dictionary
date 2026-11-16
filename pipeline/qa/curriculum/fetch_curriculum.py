#!/usr/bin/env python3
"""Download the 국립국어원 "국제 통용 한국어 표준 교육과정" materials for inspection (stdlib only).

Run from the repo root (GitHub Actions only: korean.go.kr is not reachable from the dev sandbox):
    python3 pipeline/qa/curriculum/fetch_curriculum.py [outdir]     # default out/curriculum
Writes pages/*.html, files/*, manifest.json (also copied to out/manifest.json when outdir is out/curriculum).
"""
import hashlib
import html
import json
import mimetypes
import os
import re
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from html.parser import HTMLParser

UA = "Mozilla/5.0 (compatible; hanppuri-fetch)"
TIMEOUT = 60
RETRIES = 2
MAX_FILES = 40
MAX_TOTAL = 200 * 1024 * 1024
MAX_PAGE = 20 * 1024 * 1024

SEEDS = [
    "https://kcenter.korean.go.kr/kcenter/attach/download.do?attachSeq=63090",
    "https://www.korean.go.kr/front/reportData/reportDataView.do?mn_id=45&report_seq=932&pageIndex=1",
    "https://www.korean.go.kr/front/reportData/reportDataView.do?report_seq=633&mn_id=45",
    "https://korean.go.kr/front/etcData/etcDataView.do?etc_seq=71&mn_id=46",
    "https://www.korean.go.kr/front/reportData/reportDataList.do?mn_id=45&searchCondition=title"
    "&searchKeyword=%ED%91%9C%EC%A4%80+%EA%B5%90%EC%9C%A1%EA%B3%BC%EC%A0%95",
]

ATTACH_RE = re.compile(r"download|fileDown|attach|file_path|c_file_name", re.I)
PAGE_RE = re.compile(r"reportDataView|etcDataView", re.I)
PAGE_TEXT_RE = re.compile(r"표준\s*교육과정|교육과정|어휘|문법|등급")
LICENCE_RE = re.compile(r"공공누리|KOGL|저작권|이용\s*(?:조건|허락|범위|약관|안내)|이용")
KOGL_IMG_RE = re.compile(r"kogl|공공누리|opentype|license|licence", re.I)


# ---------------------------------------------------------------- HTML extraction
class PageParser(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.anchors = []  # dicts: href, onclick, text
        self.imgs = []  # dicts: src, alt
        self.text = []
        self._skip = 0
        self._a = None

    def handle_starttag(self, tag, attrs):
        a = {k: (v or "") for k, v in attrs}
        if tag in ("script", "style"):
            self._skip += 1
        elif tag == "a":
            self._a = {"href": a.get("href", ""), "onclick": a.get("onclick", ""), "text": ""}
            self.anchors.append(self._a)
        elif tag == "img":
            self.imgs.append({"src": a.get("src", ""), "alt": a.get("alt", "")})
            if a.get("alt"):
                self.text.append(" " + a["alt"] + " ")
        elif tag in ("input", "button") and a.get("onclick"):
            self.anchors.append({"href": "", "onclick": a["onclick"], "text": a.get("value", "")})

    def handle_endtag(self, tag):
        if tag in ("script", "style") and self._skip:
            self._skip -= 1
        elif tag == "a":
            self._a = None

    def handle_data(self, data):
        if self._skip:
            return
        self.text.append(data)
        if self._a is not None:
            self._a["text"] += data


JS_CALL_RE = re.compile(r"([A-Za-z_$][\w$.]*)\s*\(([^()]*)\)")
ARG_RE = re.compile(r"""'([^']*)'|"([^"]*)"|([^,\s]+)""")


def decode_js_download(js, base):
    """Best-effort: turn a javascript download call into a URL. Returns (url|None, how)."""
    m = re.search(r"""(?:location(?:\.href)?\s*=|window\.open\s*\()\s*['"]([^'"]+)['"]""", js)
    if m:
        return urllib.parse.urljoin(base, m.group(1)), "location"
    for fn, argstr in JS_CALL_RE.findall(js):
        if not re.search(r"down|attach|file", fn, re.I):
            continue
        args = [next(g for g in t if g != "" or True) for t in ARG_RE.findall(argstr)]
        args = [a.strip() for a in args if a.strip() != ""]
        for a in args:
            if re.search(r"\.do\b|^/|^https?://", a):
                return urllib.parse.urljoin(base, a), "url-arg:" + fn
        if len(args) >= 2 and re.search(r"[/\\]", args[0]) is not None:
            q = urllib.parse.urlencode({"file_path": args[0], "c_file_name": args[1]})
            return urllib.parse.urljoin(base, "/common/download.do?" + q), "guess-path-name:" + fn
        if len(args) == 1 and re.fullmatch(r"\d+", args[0]):
            if "kcenter" in base:
                return urllib.parse.urljoin(base, "/kcenter/attach/download.do?attachSeq=" + args[0]), "guess-seq:" + fn
    return None, ""


def extract(html_text, base):
    """Return dict(attachments, page_links, licence, kogl_images, raw_onclicks)."""
    p = PageParser()
    p.feed(html_text)
    attachments, page_links, raw = [], [], []
    seen_att, seen_pg = set(), set()
    for a in p.anchors:
        href, onclick, text = a["href"].strip(), a["onclick"].strip(), " ".join(a["text"].split())
        blob = href + " " + onclick
        is_js = href.lower().startswith("javascript:") or bool(onclick)
        if onclick or href.lower().startswith("javascript:"):
            if ATTACH_RE.search(blob):
                raw.append({"href": href, "onclick": onclick, "text": text})
        url, how = None, ""
        if href and not href.lower().startswith(("javascript:", "#", "mailto:")) and ATTACH_RE.search(href):
            url, how = urllib.parse.urljoin(base, html.unescape(href)), "href"
        elif is_js and ATTACH_RE.search(blob):
            url, how = decode_js_download(onclick + " " + href, base)
        if url and url not in seen_att and not PAGE_RE.search(url):
            seen_att.add(url)
            attachments.append({"url": url, "text": text, "how": how})
        if href and PAGE_RE.search(href) and PAGE_TEXT_RE.search(text) and not href.lower().startswith("javascript:"):
            u = urllib.parse.urljoin(base, html.unescape(href))
            if u not in seen_pg:
                seen_pg.add(u)
                page_links.append({"url": u, "text": text})
    text = re.sub(r"\s+", " ", "".join(p.text))
    snippets = []
    for m in LICENCE_RE.finditer(text):
        s, e = max(0, m.start() - 300), min(len(text), m.end() + 300)
        if snippets and s <= snippets[-1][1]:
            snippets[-1][1] = e
        else:
            snippets.append([s, e])
    licence = [text[s:e] for s, e in snippets][:30]
    kogl = []
    for i in p.imgs:
        if KOGL_IMG_RE.search(i["src"]) or KOGL_IMG_RE.search(i["alt"]):
            kogl.append({"src": urllib.parse.urljoin(base, i["src"]), "alt": i["alt"]})
    return {"attachments": attachments, "page_links": page_links, "licence": licence,
            "kogl_images": kogl, "raw_onclicks": raw}


# ---------------------------------------------------------------- filenames
def _fix_mojibake(s):
    """Header values arrive latin-1 decoded; recover UTF-8 / EUC-KR (CP949) bytes if that is what they were."""
    try:
        raw = s.encode("latin-1")
    except UnicodeEncodeError:
        return s
    if all(b < 0x80 for b in raw):
        return s
    for enc in ("utf-8", "cp949"):
        try:
            return raw.decode(enc)
        except UnicodeDecodeError:
            pass
    return s


def _pct_decode(s):
    raw = urllib.parse.unquote_to_bytes(s)
    for enc in ("utf-8", "cp949"):
        try:
            return raw.decode(enc)
        except UnicodeDecodeError:
            pass
    return raw.decode("latin-1")


def _sanitize(name):
    name = name.replace("\\", "/").split("/")[-1]
    name = re.sub(r"[\x00-\x1f<>:\"|?*]", "_", name).strip(" .")
    return name[:150]


def filename_from_disposition(value):
    """Decode a Content-Disposition header value to a file name, or None."""
    if not value:
        return None
    m = re.search(r"filename\*\s*=\s*([\w.-]*)'[^']*'([^;]+)", value, re.I)
    if m:
        charset = (m.group(1) or "utf-8").lower()
        raw = urllib.parse.unquote_to_bytes(m.group(2).strip().strip('"'))
        for enc in (charset, "utf-8", "cp949"):
            try:
                return _sanitize(raw.decode(enc)) or None
            except (UnicodeDecodeError, LookupError):
                pass
    m = re.search(r'filename\s*=\s*(?:"((?:[^"\\]|\\.)*)"|([^;]+))', value, re.I)
    if not m:
        return None
    name = (m.group(1) if m.group(1) is not None else m.group(2)).strip()
    if re.search(r"%[0-9A-Fa-f]{2}", name):
        name = _pct_decode(name)
    else:
        name = _fix_mojibake(name)
    return _sanitize(name) or None


def pick_filename(url, headers, body, index):
    name = filename_from_disposition(headers.get("Content-Disposition", ""))
    if name:
        return name
    ctype = headers.get("Content-Type", "").split(";")[0].strip().lower()
    ext = {"application/x-hwp": ".hwp", "application/haansofthwp": ".hwp"}.get(ctype) or mimetypes.guess_extension(ctype) or ".bin"
    if ext == ".jpe":
        ext = ".jpg"
    return "file_%02d_%s%s" % (index, hashlib.sha1(url.encode() + body[:4096]).hexdigest()[:10], ext)


# ---------------------------------------------------------------- network
def fetch(url, limit):
    """GET with retries. Returns dict(status, final_url, headers, body, error)."""
    res = {"status": None, "final_url": None, "headers": {}, "body": b"", "error": None}
    for attempt in range(RETRIES + 1):
        try:
            req = urllib.request.Request(url, headers={"User-Agent": UA, "Accept": "*/*", "Accept-Language": "ko,en;q=0.8"})
            with urllib.request.urlopen(req, timeout=TIMEOUT) as r:
                res["status"], res["final_url"] = r.status, r.geturl()
                res["headers"] = dict(r.headers.items())
                res["body"] = r.read(limit + 1)[:limit]
                res["error"] = None
                return res
        except urllib.error.HTTPError as e:
            res.update(status=e.code, final_url=e.geturl(), headers=dict(e.headers.items()), error=str(e))
            try:
                res["body"] = e.read(limit)
            except Exception:
                pass
            if e.code < 500:
                return res
        except Exception as e:  # never abort on one failure
            res["error"] = "%s: %s" % (type(e).__name__, e)
        if attempt < RETRIES:
            time.sleep(2 * (attempt + 1))
    return res


def decode_html(body, headers):
    m = re.search(r"charset=([\w-]+)", headers.get("Content-Type", ""), re.I) or re.search(rb"charset=[\"']?([\w-]+)", body[:4000], re.I)
    cands = []
    if m:
        g = m.group(1)
        cands.append(g.decode() if isinstance(g, bytes) else g)
    cands += ["utf-8", "cp949"]
    for c in cands:
        try:
            return body.decode(c)
        except (UnicodeDecodeError, LookupError):
            pass
    return body.decode("utf-8", "replace")


def is_html(headers, body):
    ct = headers.get("Content-Type", "").lower()
    if "html" in ct:
        return True
    return "attachment" not in headers.get("Content-Disposition", "").lower() and body[:512].lstrip().lower().startswith((b"<!doctype html", b"<html"))


# ---------------------------------------------------------------- main
def main(outdir="out/curriculum"):
    pages_dir, files_dir = os.path.join(outdir, "pages"), os.path.join(outdir, "files")
    os.makedirs(pages_dir, exist_ok=True)
    os.makedirs(files_dir, exist_ok=True)
    manifest = {"fetched": [], "pages": [], "attachments": [], "limits": {"files": MAX_FILES, "bytes": MAX_TOTAL}}
    state = {"total": 0, "nfiles": 0, "page_n": 0}
    attach_queue, done_urls = [], set()

    def record(kind, url, res, saved, parent, extra=None):
        e = {"kind": kind, "url": url, "status": res["status"], "final_url": res["final_url"],
             "content_type": res["headers"].get("Content-Type"), "content_disposition": res["headers"].get("Content-Disposition"),
             "size": len(res["body"]), "saved_as": saved, "from_page": parent, "error": res["error"]}
        if extra:
            e.update(extra)
        manifest["fetched"].append(e)
        return e

    def do_attachment(a, parent):
        if a["url"] in done_urls:
            return
        done_urls.add(a["url"])
        if state["nfiles"] >= MAX_FILES or state["total"] >= MAX_TOTAL:
            record("attachment", a["url"], {"status": None, "final_url": None, "headers": {}, "body": b"", "error": None}, None, parent,
                   {"skipped": "cap reached", "link_text": a.get("text"), "how": a.get("how")})
            return
        res = fetch(a["url"], MAX_TOTAL - state["total"])
        saved = None
        if res["body"] and res["status"] and res["status"] < 400:
            state["nfiles"] += 1
            state["total"] += len(res["body"])
            name = "%02d_%s" % (state["nfiles"], pick_filename(a["url"], res["headers"], res["body"], state["nfiles"]))
            with open(os.path.join(files_dir, name), "wb") as f:
                f.write(res["body"])
            saved = "files/" + name
        record("attachment", a["url"], res, saved, parent, {"link_text": a.get("text"), "how": a.get("how")})

    def do_page(url, parent, depth):
        if url in done_urls:
            return None
        done_urls.add(url)
        res = fetch(url, MAX_PAGE)
        if res["body"] and res["status"] and res["status"] < 400 and not is_html(res["headers"], res["body"]):
            # a seed that is really a file
            do_attachment_res(url, res, parent)
            return None
        saved, info = None, None
        if res["body"]:
            state["page_n"] += 1
            saved = "pages/%02d.html" % state["page_n"]
            text = decode_html(res["body"], res["headers"])
            with open(os.path.join(outdir, saved), "w", encoding="utf-8") as f:
                f.write(text)
            info = extract(text, res["final_url"] or url)
        e = record("page", url, res, saved, parent, {"depth": depth})
        if info:
            manifest["pages"].append({"url": url, "saved_as": saved, "depth": depth, "from_page": parent, **info})
            for a in info["attachments"]:
                attach_queue.append((a, url))
            if depth == 0:
                for l in info["page_links"]:
                    follow.append((l["url"], url))
        return e

    def do_attachment_res(url, res, parent):
        done_urls.add(url)
        saved = None
        if res["body"] and state["nfiles"] < MAX_FILES and state["total"] < MAX_TOTAL:
            state["nfiles"] += 1
            state["total"] += len(res["body"])
            name = "%02d_%s" % (state["nfiles"], pick_filename(url, res["headers"], res["body"], state["nfiles"]))
            with open(os.path.join(files_dir, name), "wb") as f:
                f.write(res["body"])
            saved = "files/" + name
        record("attachment", url, res, saved, parent, {"link_text": None, "how": "seed"})

    follow = []
    for u in SEEDS:
        try:
            do_page(u, None, 0)
        except Exception as e:
            manifest["fetched"].append({"kind": "page", "url": u, "error": "unhandled %s: %s" % (type(e).__name__, e)})
    for u, parent in follow[:25]:
        try:
            do_page(u, parent, 1)
        except Exception as e:
            manifest["fetched"].append({"kind": "page", "url": u, "error": "unhandled %s: %s" % (type(e).__name__, e)})
    for a, parent in attach_queue:
        try:
            do_attachment(a, parent)
        except Exception as e:
            manifest["fetched"].append({"kind": "attachment", "url": a["url"], "error": "unhandled %s: %s" % (type(e).__name__, e)})
    manifest["attachments"] = [e for e in manifest["fetched"] if e["kind"] == "attachment" and e.get("saved_as")]
    text = json.dumps(manifest, ensure_ascii=False, indent=2)
    with open(os.path.join(outdir, "manifest.json"), "w", encoding="utf-8") as f:
        f.write(text)
    if os.path.normpath(outdir) == os.path.normpath("out/curriculum"):
        with open("out/manifest.json", "w", encoding="utf-8") as f:
            f.write(text)
    print("pages: %d, files: %d, bytes: %d" % (len(manifest["pages"]), state["nfiles"], state["total"]))
    return 0


if __name__ == "__main__":
    sys.exit(main(*(sys.argv[1:2] or [])))
