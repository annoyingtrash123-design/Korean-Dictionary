# Korean Dictionary — Scope & Architecture

An offline, Pleco-inspired Korean–English dictionary, delivered as an Android app
(Capacitor WebView around the Preact UI). GitHub Pages is only the app's data host: it
serves the dictionary packs the app downloads. There is no website and no PWA.

## Requirements (from interview)

| Area | Decision |
|---|---|
| Platforms | Android (Capacitor app, signed APK on the `android-latest` release) |
| Delivery | Android APK from GitHub Actions; data packs served from GitHub Pages `/data/`, public repo |
| Data size | As large as needed (~100–200 MB one-time download) |
| Dictionaries | English: krdict (NIKL learner's dict), Wiktionary (kaikki), kengdic. Korean–Korean: 표준국어대사전 (stdict) as a secondary, toggleable dictionary |
| Per entry | Headword, hanja, pronunciation, POS, level, English definitions, example sentences, related words / synonyms / antonyms, usage notes |
| Search | Korean (prefix + exact), conjugation-aware (갔어요 → 가다), English → Korean, hanja (學 → 학교, 학생…) |
| Extra features | Word lists (entries and grammar patterns), search history, hanja character breakdown, grammar reference, usage notes |
| Home screen | Word of the day + recent history |
| Theming | Light / dark / sepia presets + custom accent, background, text, hangul and hanja colours, font size |
| Not included | OCR, flashcards, romanization input |

## Architecture

```
GitHub Actions (full internet)                     Android phone (offline)
┌────────────────────────────────┐                 ┌───────────────────────────────┐
│ pipeline/ (Rust CLI)            │                 │ App (Capacitor + Preact + TS) │
│  fetch → parse → merge → SQLite │──► Pages ──────►│  UI bundled in the APK        │
│  core.sqlite  (EN dicts, hanja, │   site/data/    │  worker: sqlite-wasm + OPFS   │
│   grammar, sentences)           │   *.gz chunks   │  IndexedDB: bookmarks/history │
│  stdict.sqlite (KO-KO)          │   manifest.json │  localStorage: settings/theme │
└────────────────────────────────┘                 └───────────────────────────────┘
```

* Data is built in CI because the source sites are large and change over time.
  Each source is optional: if a download fails the build continues without it.
* Each data **pack** is a standalone SQLite DB, gzip-compressed and split into
  ≤ 20 MB chunks. On first run the app streams the chunks, decompresses them
  with `DecompressionStream`, and writes the DB into OPFS (the `opfs-sahpool`
  VFS needs no COOP/COEP headers, so it works in the Android WebView).
* Packs: `core` (required) and `stdict` (optional, can be enabled or disabled in settings).

## Data sources

| Source | Content | Licence | Where fetched |
|---|---|---|---|
| krdict 한국어기초사전 | ~53k entries, English equivalents, hanja, examples, related words, grammar entries | CC BY-SA 2.0 KR | github.com/spellcheck-ko/korean-dict-nikl `krdict/*.xml` |
| stdict 표준국어대사전 | ~436k entries, Korean defs, hanja, conjugations | CC BY-SA 2.0 KR (examples with a `<source>` citation are **excluded** — not open) | same repo `stdict/*.xml` |
| Wiktionary via kaikki.org | Korean entries with English glosses, hanja, translated examples | CC BY-SA 4.0 | kaikki.org JSONL |
| kengdic | ~130k Korean–English pairs + hanja | MPL 2.0 / LGPL | raw.githubusercontent.com/garfieldnate/kengdic |
| Tatoeba | Korean–English sentence pairs | CC BY 2.0 FR | downloads.tatoeba.org |
| CC-CEDICT (optional `cedict` pack) | Chinese–English, traditional/simplified, pinyin | CC BY-SA 4.0 | mdbg.net |
| Wiktionary English translation tables (optional `en_ko` table in core) | English term → Korean translations by sense, English phrases | CC BY-SA 4.0 | kaikki.org English JSONL (multi-GB; `fetch` keeps only lines mentioning Korean → `kaikki-en-ko.jsonl`; `KDICT_SKIP_ENWIKT` skips it) |
| Wiktionary Chinese (optional `zhwikt` pack) | classical senses, Middle Chinese / Sino-Korean readings | CC BY-SA 4.0 | kaikki.org Chinese JSONL |
| Unihan | Hanja readings (kHangul), English meaning, strokes, radical | Unicode licence | unicode.org |
| FrequencyWords (OpenSubtitles) | Word frequency for ranking | CC BY-SA 4.0 | raw.githubusercontent.com/hermitdave/FrequencyWords |

## Database contract (both packs use the same `entries` schema)

```sql
CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT);          -- pack, version, built_at, counts JSON, sources JSON

CREATE TABLE entries (
  id        INTEGER PRIMARY KEY,
  headword  TEXT NOT NULL,     -- as displayed, e.g. '먹다', '-아서'
  hw_norm   TEXT NOT NULL,     -- lookup key: headword with '-', '^', ' ', '·' removed
  homonym   INTEGER,           -- homonym number (krdict/stdict), else NULL
  hanja     TEXT,              -- e.g. '學校' (mixed allowed, e.g. 'ㄱㄴㄷ順'); NULL if native
  pos       TEXT,              -- normalised English: noun, verb, adjective, adverb, pronoun, numeral,
                               -- determiner, particle, ending, affix, interjection, auxiliary verb,
                               -- auxiliary adjective, bound noun, phrase, idiom, proverb, expression, other
  pron      TEXT,              -- pronunciation in hangul, e.g. '머ː따'
  source    TEXT NOT NULL,     -- 'krdict' | 'wikt' | 'kengdic' | 'stdict'
  lang      TEXT NOT NULL,     -- definition language: 'en' | 'ko'
  level     INTEGER,           -- krdict 1=초급 2=중급 3=고급, else NULL
  rank      INTEGER NOT NULL,  -- sort key, lower = more important (see below)
  kind      TEXT NOT NULL,     -- 'word' | 'phrase' | 'idiom' | 'proverb' | 'grammar'
  gloss     TEXT,              -- one-line English summary for result lists (≤ 120 chars)
  data      TEXT NOT NULL,     -- JSON, see EntryData below
  ext_id    TEXT,              -- id in the source dictionary (addition)
  quality   INTEGER NOT NULL   -- result-quality hint, 0 best: source tier (krdict 0, wikt 1, stdict 2, kengdic 3, opendict 4) + 1 for phrase/proverb/idiom (addition)
);
CREATE INDEX entries_hw_rank ON entries(hw_norm, rank);  -- covering: exact / prefix id lists in rank order without touching the table (replaces entries_hw)
CREATE INDEX entries_hanja   ON entries(hanja) WHERE hanja IS NOT NULL;   -- whole-word hanja lookup (addition)
CREATE INDEX entries_rank    ON entries(rank);

-- Conjugated / variant forms → entry (from krdict WordForm 활용, stdict conjugation, wikt forms)
CREATE TABLE forms (form TEXT NOT NULL, entry_id INTEGER NOT NULL);
CREATE INDEX forms_form ON forms(form);

-- Each hanja character of an entry → entry  (for hanja search & "words with this character")
CREATE TABLE hanja_words (ch TEXT NOT NULL, entry_id INTEGER NOT NULL, rank INTEGER NOT NULL DEFAULT 0);  -- rank = entries.rank (addition)
CREATE INDEX hanja_words_ch ON hanja_words(ch, rank, entry_id);   -- covering: 'words with this character' by rank

-- English full-text search (core pack only; stdict has none)
-- rowid = entries.id. head = short English glosses (first 6); en = English definitions, only for entries WITHOUT glosses.
-- Query: MATCH 'school' ORDER BY bm25(entries_fts, 10.0, 1.0), quality, rank
CREATE VIRTUAL TABLE entries_fts USING fts5(head, en, content='', tokenize='porter unicode61');

-- core pack only (additions):
-- One row per (normalised English gloss item, entry): term = gloss item lower-cased, trimmed, leading 'to ' stripped,
-- whitespace collapsed (items > 40 chars skipped; ALL gloss items of all senses). tier 0 = the entry's first gloss item, 1 = any other.
-- score = (tier + quality) * 10_000_000 + min(rank, 9_999_999), so ORDER BY score within a term is the final result order.
-- Engine: exact = `term = ?`; prefix (tier 2) = `term > ? AND term < ?||char(0x10FFFF)`.
CREATE TABLE gloss_terms (
  term TEXT NOT NULL, tier INTEGER NOT NULL, score INTEGER NOT NULL, entry_id INTEGER NOT NULL,
  PRIMARY KEY (term, score, entry_id)
) WITHOUT ROWID;
-- Word-of-the-day candidates (krdict level 1-2 word entries with a gloss), n = 0..N-1 in entries.id order.
CREATE TABLE wotd (n INTEGER PRIMARY KEY, entry_id INTEGER NOT NULL);

CREATE TABLE hanja_chars (
  ch TEXT PRIMARY KEY, readings TEXT,   -- '학' (comma-separated if several)
  meaning_en TEXT, strokes INTEGER, radical TEXT, word_count INTEGER
);
CREATE TABLE sentences (id INTEGER PRIMARY KEY, ko TEXT NOT NULL, en TEXT, source TEXT);  -- Tatoeba + wikt (translated), krdict full-sentence examples (en NULL)
CREATE VIRTUAL TABLE sentences_fts USING fts5(ko, content='sentences', content_rowid='id', tokenize='trigram');
CREATE TABLE grammar (
  id INTEGER PRIMARY KEY, entry_id INTEGER, pattern TEXT NOT NULL,  -- '-아서/어서'
  category TEXT NOT NULL,   -- 'Particles' | 'Connective endings' | 'Final endings' | 'Pre-final endings'
                            -- | 'Nominal/adnominal endings' | 'Expressions' | 'Affixes'
  level INTEGER, summary_en TEXT, sort INTEGER
);
CREATE TABLE curriculum (   -- NIKL standard curriculum 1-6급 (공공누리 제1유형), one row per list row
  kind TEXT NOT NULL,       -- 'word' | 'grammar'
  level INTEGER NOT NULL,   -- 1..6
  entry_id INTEGER,         -- krdict entry (words) / the grammar row's entry (grammar); NULL = no confident match
  grammar_id INTEGER,       -- grammar.id (kind 'grammar' only)
  form TEXT NOT NULL,       -- as written in the list ('가격02', '-을 수 있다')
  rank INTEGER NOT NULL,    -- 전체 번호: the list's own order
  home_ok INTEGER NOT NULL DEFAULT 0  -- precomputed Home-card eligibility (pipeline/src/home_ok.rs): word = krdict level + English gloss
                            -- + a full example sentence; grammar = non-blank summary_en. Packs without it: the engine checks at runtime
);
CREATE INDEX curriculum_kind_level ON curriculum(kind, level);
CREATE TABLE topic (        -- Lists tab "Topics": krdict's semantic category of each word entry
  entry_id INTEGER NOT NULL, category TEXT NOT NULL   -- '인간 > 감각' (top level > sub-category)
);
CREATE INDEX topic_category ON topic(category, entry_id);
CREATE TABLE grammar_lesson (  -- AI-written grammar mini-lessons; only lessons whose review.status is 'approved' are packed
  level INTEGER NOT NULL,      -- curriculum level 1..6
  form TEXT NOT NULL,          -- the curriculum row's own `form` ('-거들랑1': the digit tells homonym rows apart)
  entry_id INTEGER, grammar_id INTEGER,   -- the row's links: the lesson shows on that entry's page
  body TEXT NOT NULL           -- JSON: label, meaning_en, usage, examples[{ko,en,ctx}], compare[{pattern,difference,pair[2],entry?}],
                               --       pitfalls[{note,wrong,right,en}], topik_tip?  (no review block)
);
CREATE INDEX grammar_lesson_entry ON grammar_lesson(entry_id);
CREATE TABLE study_item (   -- Lists tab "Idioms & proverbs": 속담 / 성어 ranked by corpus frequency (pipeline/data/study_*.tsv)
  list TEXT NOT NULL,       -- 'sokdam100' | 'sokdam_all' | 'seongeo100' | 'seongeo300' | 'seongeo_learner'
  rank INTEGER NOT NULL,    -- 1..N within the list
  kind TEXT NOT NULL,       -- 'sokdam' | 'seongeo'
  title TEXT NOT NULL,      -- the text as shown (no [variant] brackets, no hanja gloss)
  entry_id INTEGER,         -- core entry id when pack = 'core'; NULL otherwise (other packs' ids are not stable)
  pack TEXT NOT NULL,       -- 'core' (krdict) | 'stdict' | 'opendict': where the entry lives
  ext_id TEXT NOT NULL, headword TEXT NOT NULL, hw_norm TEXT NOT NULL,   -- find it again in that pack: hw_norm + ext_id
  gloss TEXT NOT NULL,      -- krdict English gloss, else the Korean definition (80 chars)
  home_ok INTEGER NOT NULL DEFAULT 0,  -- precomputed Home idiom-card eligibility (core krdict entry with gloss, a full example sentence, no ∼ slot)
  PRIMARY KEY (list, rank)
) WITHOUT ROWID;
```
`study_item` is filled after all three packs are built (most 성어 exist only in stdict). The engine opens the
entry when the user has that pack installed and otherwise searches for the title; a core pack without the
table simply has no "Idioms & proverbs" group.

Grammar lessons are written in `pipeline/data/grammar_lessons/<level>.json` (one file per 급, one object per
curriculum grammar row, keyed by the official form + level; `pipeline/src/grammar_lessons.rs` documents the
schema and `pipeline/tests/grammar_lessons.rs` validates it). They start as `review: {status: "draft"}`; a
separate `content-review` pass sets `approved` (with reviewer and date), and only then does the pipeline pack
them (`grammar lessons: A approved of N drafted` in the build log). The entry page shows the lesson under the
dictionary text, labelled AI-written; the grammar study lists mark rows that have one with a "Lesson" badge.

"Writing & usage" study lists (Lists > Study lists) are two small AI-written tables of the core pack, built by
`pipeline/src/usage_lists.rs` from `pipeline/data/writing_expressions.tsv` (쓰기 expressions for TOPIK writing 51-54,
grouped by function) and `pipeline/data/confusables.tsv` (confusable words). Only rows whose `review` column starts with
`approved` are packed; the rows start as `draft` and a `content-review` pass (sheet: `pipeline/qa/writing_confusables_review.md`)
approves them. The app labels everything AI-written and reviewed; a core pack without the tables has no "Writing & usage" group.
```sql
CREATE TABLE writing_expr (rank INTEGER PRIMARY KEY, id TEXT NOT NULL, grp TEXT NOT NULL, expr TEXT NOT NULL, meaning TEXT NOT NULL,
  register TEXT NOT NULL,          -- written / formal, and the spoken forms to avoid in an essay
  ex TEXT NOT NULL,                -- JSON [{ko, en}, ...]: model sentences in essay style
  entry_id INTEGER,                -- the grammar or word entry the row links to (resolved at build time), else NULL
  ai INTEGER NOT NULL DEFAULT 1);
CREATE TABLE confusable (rank INTEGER PRIMARY KEY, id TEXT NOT NULL, kind TEXT NOT NULL,   -- 'spelling' | 'meaning' | 'grammar'
  words TEXT NOT NULL,             -- JSON [{w, wrong, ko, en, entry}]: per word one contrasting sentence; wrong = a spelling people write but that is invalid
  explanation TEXT NOT NULL, mistake TEXT NOT NULL, ai INTEGER NOT NULL DEFAULT 1);
```

### Contract additions: older-text packs
* `entries.hist INTEGER NOT NULL DEFAULT 0` (all packs): 1 = 옛말 / old word (stdict, opendict). Partial index `entries_hist(hist, hw_norm) WHERE hist = 1` in those packs.
* Core `entries.hanja` filled from 표준국어대사전 (`pipeline/src/hanja_fill.rs`, rules in its module docs): krdict / kengdic entries that
  had no hanja get the stdict 원어 hanja when headword, part of speech and meaning agree (never invented: native words stay empty, an
  existing hanja is never overwritten). Such entries carry `data.hanja_src = "stdict"` and `data.hanja_rule` (`unique` | `homograph` |
  `kengdic` | `derived`); the entry page shows a small 표준 marker, `hanja_words` and `hanja_chars.word_count` include them, and the
  build writes `hanja_fill_report.tsv` / `hanja_fill_sample.tsv` next to the packs (QA sample: `pipeline/qa/hanja_fill_sample.tsv`).
* `hanja_chars.hun TEXT` (Korean gloss word, '배울') and `eumhun TEXT` ('배울 학'; several joined '; ') in core.
* Optional packs `cedict` and `zhwikt` (`required: false` in the manifest) keep only Korean-relevant
  entries: a headword (traditional or simplified form) is kept when it equals a run of hanja in some
  Korean entry's `hanja` (core/stdict/opendict), or is a single character with a Korean reading,
  compared after folding Unihan kZVariant/kSemanticVariant/kCompatibilityVariant classes (敎 = 教);
  `counts.dropped_not_korean` records the rest. They have the same `entries`/`forms`/`hanja_words` tables, `lang 'en'`, `source` = pack id, no FTS/gloss_terms:
  `headword` = `hw_norm` = `hanja` = traditional; `forms.form` = simplified; `pron` = Sino-Korean reading in hangul; `data.simplified`, `data.pinyin` (tone marks), `data.pinyin_num`, `data.cl` (cedict); `data.pron` {mandarin, middle_chinese, cantonese, sino_korean, sino_vietnamese, sino_japanese}, `data.classical`, `data.etym` (zhwikt). Chinese packs rank after all Korean ranks.
* Engine: Han-script searches also query installed `cedict`/`zhwikt` (exact, simplified via `forms`, longest known prefix, words starting with the query), rows after the Korean ones with `via: 'hanja'`. `lookupInText(text, offset, {packs, limit?}) -> {match, start, end, rows, hanja?, deconj?}` (offsets are UTF-16 code units) serves the Reader; extra `via` values: `hist`, `spelling`, `prefix`.

### Contract additions: English → Korean translations (core, optional)
Present only when the English Wiktionary extract was fetched (the build skips both tables otherwise;
the engine checks `Caps.en_ko` / `Caps.en_ko_words` and returns no translations for packs without them).

```sql
-- Korean items of English Wiktionary translation tables. term_norm = lower-cased, whitespace collapsed;
-- term = display form when it differs (capitals: 'Korea'), else NULL; pos = readable English POS
-- ('noun', 'verb', 'adjective', 'proper noun', 'phrase', …); sense = the table's sense label (or the
-- sense's first gloss), ≤ 90 chars; ko = Hangul only (bracketed hanja removed, items with Latin/Han
-- dropped); rank = order within the term (source order). Deduplicated per (term, pos, sense, ko);
-- caps: 12 (pos, sense) groups, 6 words per group, 48 rows per term. "Translations to be checked" skipped.
CREATE TABLE en_ko (term_norm TEXT NOT NULL, term TEXT, pos TEXT NOT NULL, sense TEXT,
                    ko TEXT NOT NULL, roman TEXT, rank INTEGER NOT NULL);
CREATE INDEX en_ko_term ON en_ko(term_norm, rank);
-- "phrases containing": every word after the first of a multi-word term_norm (punctuation trimmed,
-- stop words such as a/the/of/to/one's skipped). Words in first position use the term_norm range.
CREATE TABLE en_ko_words (word TEXT NOT NULL, term_norm TEXT NOT NULL, PRIMARY KEY (word, term_norm)) WITHOUT ROWID;
```
`meta.counts` gains `en_ko`, `en_ko_terms` and `en_ko_gz_bytes` (gzip size of the two tables' text,
budget 6 MB, a warning above it); the quality gate wants `en_ko ≥ 20 000` when the table was built.
Engine: an English search (Latin script only) adds `translations` (the query term's rows, leading
"to " dropped, grouped by (pos, sense), pos groups in order of first appearance:
`{term, pos, sense?, words: {ko, roman?}[]}[]`) and `phrases` (≤ 30 terms that start with
"<query> " or contain the query as a later word run, those starting with it first, then shortest:
`{term, pos, sense?, words}[]`, ≤ 6 words each). Both are omitted when empty; `rows` is unchanged.

### EntryData JSON

```jsonc
{
  "senses": [{
    "pos": "noun",                  // optional, when an entry mixes POS (wikt)
    "gloss": "edge; verge",         // short English equivalent(s)
    "def": "The perimeter or outer limits of a place or a thing.",  // English definition
    "ko_def": "어떤 장소나 물건의 둘레나 끝부분.",                         // Korean definition (krdict, stdict)
    "note": "Used after some nouns.", // annotation / usage note (Korean or English)
    "pattern": "1이 2를 먹다",          // syntactic pattern (krdict)
    "tags": ["honorific"],
    "examples": [{"ko": "…", "en": "…" /* optional */, "type": "phrase|sentence|dialogue"}],
    "rel": [{"type": "synonym|antonym|honorific|humble|see also|reference", "word": "불가"}]
  }],
  "related": [{"type": "derived|variant|abbreviation|…", "word": "가하다"}],
  "category": "자연 > 지형",          // semantic category (krdict)
  "etym": "…",                       // etymology text (wikt)
  "origin_note": "…"
}
```

### Ranking (`rank`)
`rank = source_base + level_adj + freq_adj` where frequency (FrequencyWords rank of the
headword stem) dominates; krdict 초급 < 중급 < 고급 < krdict-unleveled < wikt < kengdic < stdict.
Exact numeric formula lives in `pipeline/build.py`; only ordering matters to the app.

## App behaviour

* **Search box** detects script:
  * Hangul → exact `hw_norm` match, then `forms` match, then deconjugation
    candidates (`src/lib/deconjugate.ts`), then prefix matches (`hw_norm LIKE 'q%'`), ordered by `rank`.
  * Latin → `entries_fts MATCH` (prefix-aware), ranked by bm25 then `rank`.
  * Han characters → `hanja_chars` card(s) + entries via `hanja_words`.
* **Result list** (Pleco-style): one row per headword+hanja group: headword · hanja · POS ·
  gloss, with a level badge.
* **Entry view**: header (headword, hanja tappable per character, pronunciation, level badge),
  then a section per dictionary (krdict → wikt → kengdic → stdict), numbered senses,
  examples, related words (tappable), "More examples" from `sentences_fts`, a "Hanja"
  section that breaks the word down per character with other words sharing each character,
  and a link to the grammar reference for grammar entries. Bookmark button.
* **Grammar reference**: browse by category and level, then open the full entry.
* **Lists** (replaced Bookmarks): user lists of entries and grammar patterns (keyed on `source|headword|homonym|pos` / pattern text, never on row ids); tap the star to save to the last-used list, long-press / right-click / Shift+F10 to pick lists; old bookmarks migrate into "Saved words"; export/import JSON backup (old bookmark backups still import). Built-in read-only lists plug in as further `ListSource`s (`app/src/lib/list-sources.ts`).
* **History**: recent lookups on the home screen; clearable.
* **Word of the day**: deterministic pick from krdict level 1–2 entries by date.
* **Settings**: theme presets + colour pickers, font size, enable/disable stdict, data
  pack status and re-download, storage usage, backup/restore, licences and attributions.

## Languages

* **Rust**: the data pipeline (`pipeline/`, a native CLI run in CI) and the core engine
  (`core/`, compiled to WebAssembly with wasm-pack). The core engine owns SQLite
  (rusqlite + sqlite-wasm-rs, OPFS sahpool VFS), pack import, deconjugation and all
  search and query logic.
* **TypeScript**: a thin UI layer only (Preact components, routing, theming, bookmarks,
  and the download manager that streams chunks into the engine).
  The UI talks to the engine through `app/src/db/engine.ts` (the `Engine` interface).

## Repository layout

```
Cargo.toml           Rust workspace (pipeline, core)
pipeline/            Rust data build CLI: `kdict-pipeline fetch|build`
core/                Rust engine → WASM (search, deconjugate, hangul, db)
app/                 Vite + Preact + TypeScript UI, packaged as the Android app (Capacitor)
  src/core-wasm/     wasm-pack output (generated, gitignored)
  src/db/            worker, engine.ts adapter, types.ts, download manager
.github/workflows/   build-data + publish data to Pages; android.yml builds the APK
docs/                this scope document
```
