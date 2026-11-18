# Later — owner-approved work not started yet

Kept in the repo so it survives sessions. Move an item out when it starts.

- **성어: Chinese and English equivalents.** For each 성어 (study lists `seongeo_learner` / `seongeo300`), find the
  Chinese chengyu it comes from or matches (e.g. 과유불급 ↔ 过犹不及; CC-CEDICT / kaikki zh are fetchable in CI, see
  the cedict/zhwikt packs) and an established English equivalent where one exists (e.g. 타산지석 ↔ "learn from
  others' mistakes"). Mark AI-proposed ones as AI and review them independently before shipping.
- **Grammar lessons 3급 / 2급 / 1급:** independent content review, then ship (drafts on `save/lessons-3-2-1-drafts`).
- **-으니4 (6급 grammar lesson):** confirm which sense the curriculum's "비난" means (check 표준국어대사전 -으니04), then
  review and ship.
- **Home load time:** precompute word-of-the-day eligibility in the pipeline (the home query spends ~2 s on it).
- **krdict gloss fix:** 단연 "obviously" → "by far; decidedly" (gloss_fixes.tsv, with review).
- **쓰기 expressions and confusable-word groups** (owner chose them as built-in lists; AI-written, needs review).
- **Translation screening C04–C42:** paused by the owner; when resumed: guided Sonnet + one outside model per batch,
  Opus judges (pipeline/qa/screening/GUIDE.md).
- **Search benchmark:** the 14 weak spots in core/tests/golden_queries.tsv.
