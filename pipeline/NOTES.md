# Pipeline notes (Rust: `kdict-pipeline`)

```
cargo run --release -p kdict-pipeline -- fetch --work pipeline/.work
cargo run --release -p kdict-pipeline -- build --work pipeline/.work --out pipeline/out [--limit-files N]
cargo test -p kdict-pipeline
```

Layout: `src/{krdict,stdict,kaikki,kengdic,tatoeba,unihan,freq}.rs` parse one source each into `common::Entry`;
`build.rs` writes the SQLite packs (schema in `schema.rs`, contract in docs/SCOPE.md); `pack.rs` gzips + chunks;
`fetch.rs` downloads. `xml.rs` is a tiny streaming DOM (quick-xml) that yields one subtree per `<LexicalEntry>` / `<item>`.
Fixtures for every format are in `tests/fixtures/`. `ATTRIBUTION.md` is embedded in the binary and copied to `out/site-data/`.

Status: krdict and stdict were verified on real data (krdict: 11 files -> 56,555 entries; stdict: 5 files -> 25,000 items).
kaikki / Tatoeba / Unihan parsers are tested only against hand-written fixtures (those hosts are blocked in the dev
sandbox); the first CI run is their first contact with real data. Each optional source that is missing or fails only logs a WARNING.

## Fetch
- NIKL: blobless sparse clone `git clone --depth 1 --filter=blob:none --sparse https://github.com/spellcheck-ko/korean-dict-nikl work/nikl`, then `sparse-checkout set krdict stdict opendict` (`opendict/` is included for the optional opendict pack). krdict/001..011.xml (~35 MB each), stdict/005000.xml.. (~88 files, ~8 MB each).
- Files land in `work/`: `kengdic.tsv`, `ko_50k.txt`, `kaikki-ko.jsonl`, `Unihan.zip`, `tatoeba/{kor_sentences.tsv.bz2,eng_sentences.tsv.bz2,links.tar.bz2}`. Existing non-empty files are skipped (delete to refresh). Download uses ureq (rustls); on failure it retries once and then falls back to `curl` if installed.
- To reuse an existing NIKL checkout: symlink it to `work/nikl` (it needs `krdict/` and `stdict/` directly inside).

## krdict (LMF XML, DOCTYPE references a remote DTD, which is ignored)
- Everything is `<feat att="X" val="Y"/>`. LexicalEntry(att=id) > feat*, Lemma (a 2nd Lemma may hold feat variant), WordForm*, RelatedForm*, Sense*; Sense > feat, SenseExample*, SenseRelation*, Equivalent*; WordForm may contain FormRepresentation (type 준말) -> extra form.
- Entry feats: homonym_number (0 = none), lexicalUnit (단어, 관용구, 구, 문법‧표현 (U+2027), 속담), partOfSpeech, vocabularyLevel (초급/중급/고급/없음), semanticCategory, origin (hanja, or English for loanwords -> `origin_note`).
- WordForm type 발음 (pronunciation) / 활용 (writtenForm). Only a few 활용 forms per entry, so conjugation lookup needs the app's deconjugator.
- RelatedForm types: ☞(가 보라) (reference), 파생어 (derived). SenseRelation types: 유의어 반대말 참고어 높임말 낮춤말 큰말 작은말 센말 여린말 준말 본말. Equivalent: take language 영어 only.
- Quirks: raw control bytes (0x01-0x1f) in 3 files make the XML invalid -> stripped at byte level (`xml::CleanReader`). Annotations are double-escaped (`&amp;apos;`) -> `clean()` unescapes once more. Grammar entries (and many auxiliary verbs, bound nouns, affixes) have a *romanisation* of the headword as English lemma ("-aseo", "gajida"): grammar kind always, others via `looks_romanized()` (plain RR transliteration within edit distance) -> stored as `sense.roman`, not `sense.gloss`; the English definition is used for `gloss`. "(no equivalent expression)" is dropped.
- Mapping: kind 단어 word (어미/조사 -> grammar), 구 phrase, 관용구 idiom, 속담 proverb, 문법‧표현 grammar (pos 'expression').

## stdict (RSS-like `<channel><item>`, CDATA, ~436k items)
- item > target_code, word_info > word, word_unit, original_language_info*, pronunciation_info, conju_info*, lexical_info*, relation_info, origin, allomorph, pos_info* > pos, comm_pattern_info* > pattern_info/pattern, grammar_info/grammar, sense_info* > type, definition, cat_info/cat ('없음' = none), example_info* > example (+ optional `source` -> **excluded**).
- word: trailing 1-3 digit homonym number ("가03"), '-' morpheme marker, '^' = space ("가^는^길" -> "가 는 길"; hw_norm "가는길"). Leading/trailing '-' (affix/ending) are kept in `headword`.
- hanja = concatenation of the 한자 + 고유어 parts (ㄱㄴㄷ-순 -> ㄱㄴㄷ順) only if a 한자 part exists; other origins -> `origin_note`.

## opendict (우리말샘, optional third pack, lang 'ko', default OFF in the app)
- `opendict/*.xml` (25 files, ~77 MB, 50k items each). Unlike stdict, **each `<item>` is one sense**: `wordInfo` (word, word_unit 어휘/구/속담/관용구, word_type, original_language_info, pronunciation_info, conju_info) + `senseInfo` (sense_no, pos, type, definition, cat_info/cat, example_info, relation_info, region_info/region, translation_info, pattern_info, grammar_info, abbreviation_info; also history_info, norm_info, multimedia_info, proverb_info, sl_info_link which are ignored). Items sharing `group_code` are senses of one word; they are merged within a file ordered by `group_order` (groups split across files stay separate entries). Words carry no homonym digits.
- Labels -> `sense.tags`: sense `type` (방언, 북한어, 옛말, 순화...; 일반어 omitted), region (경상...), category. relation_info types add hypernym (상위어), hyponym (하위어), dialect (방언), archaic form (옛말). English `translation_info` -> `sense.en`. Composite POS such as "관·명" use the first component.
- Copyright: examples with `<source>` are excluded; `history_example_info` and long `history_info` descriptions are not stored.
- Dedupe: opendict entries whose `(hw_norm, pos)` already exists in stdict are skipped (counted in `meta.counts.skipped_stdict_duplicates`). Only done when the stdict pack is built in the same run.
- Measured (2 files, stdict limited to 5 files so dedupe is understated): ~1.6 s/file, ~430 MB peak RSS; 74k entries = 43.6 MB sqlite / 10.4 MB gz.

## kengdic
Tab-separated, header `id surface hanja gloss level created source`, ~133k rows, no quoting. 16k rows have an empty gloss (hanja only, still feed the hanja-by-surface map); hanja may be comma separated variants (first = key, rest -> `hanja_alt`). Rows grouped by (surface, first hanja); glosses deduped case-insensitively; junk glosses ("VST + ... adds no meaning", >250 chars) dropped; level A-D kept as `kengdic_level`. Surfaces with spaces -> phrase. Real build: 108,586 entries from 34,510 hanja-bearing surfaces.
Hanja fallback: krdict/wikt noun entries without hanja, no origin note, and unique within their source get the hanja from kengdic only when kengdic has exactly one distinct hanja for that surface (41 hits on krdict).

## FrequencyWords
"word count" lines; rank = line number. Verb/adjective stems get the best rank of stem+common-ending (`freq::ENDINGS`) incl. 하 -> 했/해 contractions.
`rank = freq_rank*8 + tier` (tier: krdict L1/L2/L3 = 0/1/2, krdict unleveled 3, wikt 4, kengdic 5, stdict 6). Entries with no frequency use a per-tier default (15k/25k/40k/55k/60k/70k/80k, +20k for phrases/idioms/proverbs).

## kaikki (wiktextract JSONL)
One line per (word, pos, etymology). Merged per (word, etymology_number) -> one entry (`homonym` = etymology number); `sense.pos` only when the merged entry mixes POS. Senses with `form_of`/`alt_of` (or form-of/alt-of tags) are skipped and become `forms` rows pointing at the target (resolved against wikt then krdict entries by hw_norm). Nested glosses: last element is the gloss, parents -> `sense.parent`. Hanja = first `forms[]` item tagged "hanja"; entries without hangul in the headword are skipped. Examples `{text, english|translation}` -> sense examples + `sentences` (source 'wikt', deduped by Korean text).

## Tatoeba
`kor_sentences.tsv.bz2` / `eng_sentences.tsv.bz2`: `id \t lang \t text`. `links.tar.bz2` holds `links.csv` (`sentence_id \t translation_id`, all languages), streamed (never loaded fully); only links from a Korean id are kept and the first English match is used. All bz2 readers are multi-stream safe.

## Unihan
`Unihan.zip` -> `Unihan_Readings.txt` (kHangul `학:0E`, kDefinition, kKorean Yale fallback) + `Unihan_IRGSources.txt` (kTotalStrokes, kRSUnicode `39.13` / `1'.2`). `hanja_chars.radical` = Kangxi radical character (U+2F00 + n-1); added column `radical_num`. Rows exist for every hanja used by an entry plus every character with a kHangul reading.

## Output / contract notes
- Additions to the contract: `entries.ext_id` (source id), `hanja_chars.radical_num`, `meta.schema`, index `grammar_cat`, `grammar_entry`; counts JSON keys are flat numbers (entries, krdict, wikt, kengdic, forms, hanja_words, hanja_chars, sentences, grammar...).
- `forms.form` is stored `hw_norm`-ed (no spaces/'-'/'^'); the headword itself is never stored as a form. Entries are per source (no cross-source merge).
- `entries_fts` is contentless (`content=''`), rowid = entries.id; indexed text = English glosses (definition only when there is no gloss). `sentences_fts` is trigram over `sentences` (external content, rebuilt at the end): MATCH needs a quoted phrase for multi-word queries (`'"책을 읽"'`), and terms shorter than 3 characters never match.
- grammar categories: 조사 Particles; 어미 by Korean definition (선어말 Pre-final, 연결 Connective, 종결 Final, 전성/관형사형/명사형 Nominal/adnominal, else Final); 문법‧표현 Expressions; 접사 Affixes (krdict only).
- DB finishing: `ANALYZE; PRAGMA journal_mode=DELETE; VACUUM`, page_size 4096. rusqlite 0.32 bundles SQLite 3.46.0 (FTS5 with `trigram` and `porter unicode61`).
- Manifest `chunks` entries are `{"file","bytes"}`; chunk size is 20,000,000 bytes; one gzip stream (level 9) per pack.
