# Korean Dictionary — Scope & Architecture

An offline, Pleco-inspired Korean–English dictionary for iPhone and Android,
delivered as an installable Progressive Web App (PWA) hosted on GitHub Pages.

## Requirements (from interview)

| Area | Decision |
|---|---|
| Platforms | iPhone + Android, one codebase |
| Delivery | Offline PWA (Add to Home Screen), GitHub Pages, public repo |
| Data size | As large as needed (~100–200 MB one-time download) |
| Dictionaries | English: krdict (NIKL learner's dict), Wiktionary (kaikki), kengdic. Korean–Korean: 표준국어대사전 (stdict) as a secondary, toggleable dictionary |
| Per entry | Headword, hanja, pronunciation, POS, level, English definitions, example sentences, related words / synonyms / antonyms, usage notes |
| Search | Korean (prefix + exact), conjugation-aware (갔어요 → 가다), English → Korean, hanja (學 → 학교, 학생…) |
| Extra features | Bookmarks in folders, search history, hanja character breakdown, grammar reference, usage notes |
| Home screen | Word of the day + recent history |
| Theming | Light / dark / sepia presets + custom accent, background, text, hangul and hanja colours, font size |
| Not included | OCR, flashcards, TTS, romanization input, text reader (can be added later) |

## Architecture

```
GitHub Actions (full internet)                     Phone (offline)
┌────────────────────────────────┐                 ┌───────────────────────────────┐
│ pipeline/ (Rust CLI)            │                 │ PWA (Vite + Preact + TS)      │
│  fetch → parse → merge → SQLite │──► Pages ──────►│  service worker: app shell    │
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
  VFS works on iOS Safari 16.4+ without COOP/COEP headers).
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
CREATE TABLE sentences (id INTEGER PRIMARY KEY, ko TEXT NOT NULL, en TEXT, source TEXT);  -- Tatoeba + wikt
CREATE VIRTUAL TABLE sentences_fts USING fts5(ko, content='sentences', content_rowid='id', tokenize='trigram');
CREATE TABLE grammar (
  id INTEGER PRIMARY KEY, entry_id INTEGER, pattern TEXT NOT NULL,  -- '-아서/어서'
  category TEXT NOT NULL,   -- 'Particles' | 'Connective endings' | 'Final endings' | 'Pre-final endings'
                            -- | 'Nominal/adnominal endings' | 'Expressions' | 'Affixes'
  level INTEGER, summary_en TEXT, sort INTEGER
);
```

### Contract additions: older-text packs
* `entries.hist INTEGER NOT NULL DEFAULT 0` (all packs): 1 = 옛말 / old word (stdict, opendict). Partial index `entries_hist(hist, hw_norm) WHERE hist = 1` in those packs.
* `hanja_chars.hun TEXT` (Korean gloss word, '배울') and `eumhun TEXT` ('배울 학'; several joined '; ') in core.
* Optional packs `cedict` and `zhwikt` (`required: false` in the manifest), same `entries`/`forms`/`hanja_words` tables, `lang 'en'`, `source` = pack id, no FTS/gloss_terms:
  `headword` = `hw_norm` = `hanja` = traditional; `forms.form` = simplified; `pron` = Sino-Korean reading in hangul; `data.simplified`, `data.pinyin` (tone marks), `data.pinyin_num`, `data.cl` (cedict); `data.pron` {mandarin, middle_chinese, cantonese, sino_korean, sino_vietnamese, sino_japanese}, `data.classical`, `data.etym` (zhwikt). Chinese packs rank after all Korean ranks.
* Engine: Han-script searches also query installed `cedict`/`zhwikt` (exact, simplified via `forms`, longest known prefix, words starting with the query), rows after the Korean ones with `via: 'hanja'`. `lookupInText(text, offset, {packs, limit?}) -> {match, start, end, rows, hanja?, deconj?}` (offsets are UTF-16 code units) serves the Reader; extra `via` values: `hist`, `spelling`, `prefix`.

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
* **Bookmarks**: folders, add/remove/move, export/import JSON backup.
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
  the service worker, and the download manager that streams chunks into the engine).
  The UI talks to the engine through `app/src/db/engine.ts` (the `Engine` interface).

## Repository layout

```
Cargo.toml           Rust workspace (pipeline, core)
pipeline/            Rust data build CLI: `kdict-pipeline fetch|build`
core/                Rust engine → WASM (search, deconjugate, hangul, db)
app/                 Vite + Preact + TypeScript PWA UI
  src/core-wasm/     wasm-pack output (generated, gitignored)
  src/db/            worker, engine.ts adapter, types.ts, download manager
.github/workflows/   build-data + deploy to Pages
docs/                this scope document
```
