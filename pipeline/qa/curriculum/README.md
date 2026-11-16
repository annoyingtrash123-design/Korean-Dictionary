# Curriculum fetch (one-off)

Downloads the 국립국어원 "국제 통용 한국어 표준 교육과정" pages and attachments so we can look at them.
korean.go.kr / kcenter.korean.go.kr are unreachable from the dev sandbox, so this only runs in GitHub Actions.

Run: Actions -> "Fetch curriculum" -> Run workflow, then download the `curriculum` artifact (kept 7 days).
It contains `pages/` (saved HTML), `files/` (attachments) and `manifest.json` (every URL tried, status,
content type, size, saved name, source page, raw onclick strings, and licence / 공공누리 text found near the links).

Local test of the parsing: `python3 -I pipeline/qa/curriculum/test_fetch_curriculum.py`

Results are for inspection only. The licence terms (공공누리 type etc.) must be confirmed before any use or redistribution.
