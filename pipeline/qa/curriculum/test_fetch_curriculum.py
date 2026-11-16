"""Run: python3 -I pipeline/qa/curriculum/test_fetch_curriculum.py"""
import importlib.util
import os
import sys
import urllib.parse

here = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("fc", os.path.join(here, "fetch_curriculum.py"))
fc = importlib.util.module_from_spec(spec)
spec.loader.exec_module(fc)

BASE = "https://www.korean.go.kr/front/reportData/reportDataView.do?mn_id=45&report_seq=932"
FIXTURE = """<html><body><script>var x = "download.do";</script>
<div class="view"><h3>국제 통용 한국어 표준 교육과정 최종 개발</h3>
<img src="/images/kogl/opentype_1.png" alt="공공누리 제1유형">
<p>이 저작물은 공공누리 제1유형 조건에 따라 이용할 수 있습니다. 출처표시.</p>
<ul>
<li><a href="/common/download.do?file_path=2020/a&amp;c_file_name=a.hwp">a.hwp</a></li>
<li><a href="https://kcenter.korean.go.kr/kcenter/attach/download.do?attachSeq=63090">seq</a></li>
<li><a href="javascript:fileDown('/data/2020/b.pdf','b.pdf');" onclick="fileDown('/data/2020/b.pdf','b.pdf')">b.pdf</a></li>
<li><a href="#" onclick="fn_download('12345'); return false;">num</a></li>
<li><a href="#" onclick="window.open('/common/download.do?file_path=x');">w</a></li>
<li><a href="/front/reportData/reportDataView.do?report_seq=700&mn_id=45">한국어 어휘 등급 목록</a></li>
<li><a href="/front/etcData/etcDataView.do?etc_seq=9">공지사항</a></li>
</ul></div></body></html>"""

ok = True
def check(name, got, want):
    global ok
    if got != want:
        ok = False
        print("FAIL", name, "\n  got ", repr(got), "\n  want", repr(want))

r = fc.extract(FIXTURE, BASE)
urls = [a["url"] for a in r["attachments"]]
check("relative href", urls[0], "https://www.korean.go.kr/common/download.do?file_path=2020/a&c_file_name=a.hwp")
check("absolute href", urls[1], "https://kcenter.korean.go.kr/kcenter/attach/download.do?attachSeq=63090")
check("js call", urls[2], "https://www.korean.go.kr/data/2020/b.pdf")
check("window.open", urls[3], "https://www.korean.go.kr/common/download.do?file_path=x")
check("attachment count", len(urls), 4)
check("raw onclick kept", any("fn_download('12345')" in o["onclick"] for o in r["raw_onclicks"]), True)
check("page links", [l["url"] for l in r["page_links"]],
      ["https://www.korean.go.kr/front/reportData/reportDataView.do?report_seq=700&mn_id=45"])
check("kogl image", r["kogl_images"][0]["alt"], "공공누리 제1유형")
check("licence text", any("공공누리 제1유형 조건" in s for s in r["licence"]), True)
check("script ignored", any("var x" in s for s in r["licence"]), False)

D = fc.filename_from_disposition
name = "표준 교육과정.hwp"
check("rfc5987", D("attachment; filename*=UTF-8''" + urllib.parse.quote(name)), name)
check("rfc5987 + fallback", D("attachment; filename=\"x.hwp\"; filename*=UTF-8''" + urllib.parse.quote(name)), name)
check("percent plain", D('attachment; filename="%s"' % urllib.parse.quote(name)), name)
check("percent euc-kr", D('attachment; filename="%s"' % urllib.parse.quote(name.encode("cp949"))), name)
check("utf8 mojibake", D('attachment; filename="%s"' % name.encode("utf-8").decode("latin-1")), name)
check("euc-kr mojibake", D('attachment; filename="%s"' % name.encode("cp949").decode("latin-1")), name)
check("ascii unquoted", D("attachment; filename=a.pdf"), "a.pdf")
check("path stripped", D('attachment; filename="../../etc/p.pdf"'), "p.pdf")
check("none", D(""), None)
check("fallback name", fc.pick_filename("http://x/y", {"Content-Type": "application/pdf"}, b"%PDF", 3).endswith(".pdf"), True)

print("OK" if ok else "FAILED")
sys.exit(0 if ok else 1)
