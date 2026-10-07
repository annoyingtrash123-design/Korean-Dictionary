# Reader add-on — spec

An optional, offline **library pack** (`texts`) of public-domain and openly licensed Korean texts,
read in a Pleco-style Reader with tap-to-look-up, plus dictionary packs for older and
Chinese-script texts. Interview decisions are recorded here and are binding.

## Collections (shelves)

| Shelf | Content | Source / licence |
|---|---|---|
| Classical prose | 열녀춘향수절가, 홍길동전, 심청전, 흥부전, 토끼전, 박씨전, 구운몽 (selected chapters), 사씨남정기, 한중록 (excerpts), 금오신화 (hanmun) | Korean Wikisource; public domain |
| Verse | 정읍사, 청산별곡, 가시리; sijo (정몽주 단심가, 이방원 하여가, 황진이, 이순신, 윤선도 오우가·어부사시사); gasa (정철 관동별곡, 사미인곡) | Wikisource; PD |
| Hanmun | 최치원 (추야우중, 격황소서), 이규보, 박지원 (양반전, 허생전, 열하일기 excerpts), 정약용 (목민심서 excerpts), 삼국유사 단군 조, 삼국사기 excerpts | Wikisource (zh/ko); PD |
| Sino-Korean relations | 열하일기 (above), 홍대용 연기 / 의산문답 excerpts, 최부 표해록 excerpts, 박제가 북학의 excerpts, 삼국사기 on the Silla–Tang war, 실록 entries on 임진왜란 Ming aid and 병자호란 | Wikisource; PD originals |
| Historical documents | 훈민정음 (해례 서문 hanmun + 언해본), 조선왕조실록 excerpts (hanmun originals), 기미독립선언서, 대한민국 임시헌장 (1919) | Wikisource; PD |
| Constitution & key documents | 대한민국 헌법 (preamble + key articles), Universal Declaration of Human Rights (Korean) | Statutes are unprotected (Copyright Act art. 7); UDHR translation per OHCHR terms |
| Modern poetry | 김소월, 한용운, 윤동주, 이육사, 이상화, 정지용, 김영랑 | Wikisource; authors died before 1963 → PD in Korea |
| Modern fiction | 현진건, 김유정, 이효석, 나도향, 김동인, 최서해, 채만식, 이상, 이광수 (무정 excerpts, with a note on his collaboration) | Wikisource; PD |
| Essays & children's | 방정환 stories, essays (이효석 etc.) | Wikisource; PD |
| News | Recent 정책브리핑 (korea.kr) articles marked 공공누리 제1유형, refreshed each build | KOGL Type 1 (attribution) |
| Graded readers | ~40 original texts on history & culture, TOPIK 1–6, with vocabulary lists + comprehension questions | Written for this app; labelled **AI-written** |

Excluded on copyright grounds: anything by authors who died in or after 1963 (e.g. 백석, 염상섭, 윤석중),
modern textbooks, post-1963 translations.

## Per-text presentation
- **Card** (bilingual): title (ko/en), author + life dates, date of composition, genre, period,
  difficulty (TOPIK band or "advanced/classical"), length, edition used + source URL + revision,
  licence. **Full notes** (expandable, bilingual): context, significance, themes, vocabulary to
  watch for. Notes are AI-written and labelled; only well-established facts.
- **Parallel English** per paragraph (toggle). Historical public-domain translations preferred
  (Gale, *The Cloud Dream of the Nine*, 1922; Gale, *Korean Folk Tales*, 1913; Allen,
  *Korean Tales*, 1889); otherwise AI translation labelled "AI-generated translation — may
  contain errors".
- **Spelling toggle**: original (default) ↔ modernised. Wikisource modern editions where they
  exist, else AI-modernised (labelled). Lookups use the modern form.
- **Hanmun texts**: original hanja, optional Korean reading (음) line, Korean/English translation.
- **Tap a word** → pop-up card (headword, hanja, short gloss, source) → "Open entry"
  (on iPad: right pane). Hangul: eojeol → deconjugation + 옛말; hanja: longest match in
  CC-CEDICT / Wiktionary Chinese, then per-character 훈음.
- Long works (구운몽, 무정, 열하일기, 목민심서, 실록) appear as **selected chapters**.

## Library UI
Shelves + **timeline** (Gojoseon → today; each text placed by composition date) with filters
for shelf, period, level, length, script (hangul/hanmun/mixed). Themes: Ancient & Goryeo,
Joseon, Colonial era & independence, Modern Korea, Sino-Korean relations.

## Dictionary packs for older texts
- **옛말 filter**: 우리말샘 entries tagged 옛말 (no new download); Reader lookups fall back to them.
- **Hanja 훈음** (small): per-character 훈음 (學: 배울 학) from Wiktionary + Unihan.
- **CC-CEDICT** Chinese–English (CC BY-SA 4.0), ~10 MB.
- **Wiktionary Chinese** (optional, large): classical senses, Middle Chinese and Sino-Korean readings.

## Data flow
1. `kdict-pipeline fetch-texts` (GitHub Actions, `fetch-texts.yml`, manual) resolves the catalogue
   (`pipeline/texts/catalog.toml`) against Wikisource / other sources and commits raw texts +
   provenance (page, revision id, URL, licence, fetch date) to `pipeline/texts/raw/`.
2. Enrichment (intros, translations, modernised spelling, graded readers) is authored into
   `pipeline/texts/enriched/<id>.json`, reviewed, committed.
3. `kdict-pipeline build` produces the `texts` pack (SQLite) and the dictionary packs.

## Enriched text format (`pipeline/texts/enriched/<id>.json`)

```jsonc
{
  "id": "hong-gildong",                       // = catalog id; graded readers use "graded-<slug>"
  "meta": { /* graded readers only: same fields as catalog.toml entries, source "original" */ },
  "card": {
    "summary_ko": "…", "summary_en": "…",     // 1–2 sentence bilingual blurb
    "level": "TOPIK 3" | "TOPIK 5–6" | "advanced" | "classical",
    "edition_ko": "…", "edition_en": "…"      // human-readable edition/provenance line
  },
  "notes": { "ko": "markdown", "en": "markdown" },   // context, significance, themes, vocabulary to watch
  "paragraphs": [
    {
      "orig": "original text (verbatim from raw, verse lines separated by \n)",
      "modern": "modernised spelling or null if identical",
      "reading": "Korean reading line for hanmun, else null",
      "en": "English translation of this paragraph"
    }
  ],
  "vocab": [ { "word": "…", "gloss_en": "…", "level": 1 } ],       // graded readers (optional elsewhere)
  "questions": [ { "q_ko": "…", "q_en": "…", "answer_en": "…" } ], // graded readers
  "labels": {
    "notes": "ai",
    "translation": "ai" | "pd:<ref>",          // pd = historical public-domain translation used
    "modern": "ai" | "wikisource" | null,
    "text": "original" | "ai"                  // "ai" only for graded readers
  }
}
```
Every AI-produced field is labelled in the UI ("AI-written", "AI-generated translation — may contain errors").

## Pleco-derived interaction details (binding for the Reader UI)
- Tapping a word **highlights it in the text** (accent-tinted background) and opens a **bottom
  pop-up card** (phone) / updates the **right pane** (iPad wide layout) without moving the text.
- The pop-up has **◀ ▶** to move the selection to the previous/next word and **⇤ ⇥** (or
  long-press-drag) to shrink/extend the selection by a character — the lookup re-runs live, as in
  Pleco's reader. Swipe down or tap outside to dismiss.
- Pop-up content: headword (large), hanja, pronunciation, POS + level badge, 1–3 short glosses
  from the best dictionary, "Open full entry", bookmark ☆. For hanja selections: traditional
  form, Sino-Korean reading, 훈음 per character, CC-CEDICT gloss.
- Reading settings: font size, line spacing, serif/sans for Korean, show/hide English, original ↔
  modern spelling, hanmun reading line on/off; reading position remembered per text.
