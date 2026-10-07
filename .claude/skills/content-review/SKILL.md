---
name: content-review
description: Independent verification of AI-written Reader content (translations, notes, modernised spelling, graded readers) before it ships.
---
Run as a *separate* agent from the one that wrote the content. For each `pipeline/texts/enriched/<id>.json`:
1. Schema: matches docs/READER.md "Enriched text format"; every paragraph of `pipeline/texts/raw/<id>.json` is present, in order, `orig` verbatim.
2. Translation: each `en` faithful to its `orig` (no omissions, additions, or softening); names/dates/terms consistent across the text.
3. Modernised spelling: same meaning, only orthography changed.
4. Notes/card: every factual claim (dates, people, events, editions) is well established; anything doubtful removed or hedged. Provenance line matches the raw file's source/revision.
5. Graded readers: level-appropriate vocabulary/grammar, correct Korean, factual accuracy.
6. Record the outcome in the file: `"review": {"status": "approved" | "changes", "by": "content-review", "date": "...", "notes": "..."}`. Fix small issues directly (and say so); send larger ones back. Only `approved` files are packed.
