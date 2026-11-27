# Later — owner-approved work not started yet

Kept in the repo so it survives sessions. Move an item out when it starts.

- **Translation screening C04–C42:** paused by the owner; when resumed: guided Sonnet + one outside model per batch,
  Opus judges (pipeline/qa/screening/GUIDE.md).
- **Search benchmark:** once the packs carry the word_en overlay, drop the `known` mark of `futures → 선물`
  (core/tests/golden_queries.tsv); the benchmark prints it under "now passing".
- **Rarer English gaps (not planned):** ~235,000 more 표준국어대사전 meanings have no English; almost all archaic,
  historical or highly technical (qa/english_gap_shortlist.tsv covers the modern ones, shipped via data/word_en.tsv).
