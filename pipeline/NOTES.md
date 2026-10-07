# Source format notes (learned while building the Python prototype)

Status: Python prototype partially written (common.py, parsers/{krdict,stdict,kengdic,freq,tatoeba,unihan}.py).
NOT written: parsers/wikt.py, fetch.py, build.py, tests, ATTRIBUTION.md, requirements is present.
Verified against real data: krdict (all 11 files, 56,555 entries, ~26 s), stdict (5 files, 25,000 items, 3.6 s).
Unverified (written from memory of formats, no fixtures/tests): kengdic grouping, freq, tatoeba, unihan, kaikki.

## Fetch
- NIKL: `git clone --depth 1 --filter=blob:none --sparse https://github.com/spellcheck-ko/korean-dict-nikl` then `git sparse-checkout set krdict stdict`. krdict/001..011.xml (~35 MB each), stdict/005000.xml.. (~88 files, ~8 MB each). Do not use opendict/.
- kengdic: raw.githubusercontent.com/garfieldnate/kengdic/master/kengdic.tsv (11 MB, works). FrequencyWords: raw.githubusercontent.com/hermitdave/FrequencyWords/master/content/2018/ko/ko_50k.txt ("word count", 50,000 lines, works).
- kaikki / tatoeba / unicode.org were blocked in the sandbox; fine in CI.

## krdict (LMF XML, DOCTYPE references a remote DTD: use no_network, load_dtd=False)
- Everything is `<feat att="X" val="Y"/>`. Elements: LexicalEntry(att=id) > feat*, Lemma (a 2nd Lemma may hold feat variant), WordForm*, RelatedForm*, Sense*; Sense > feat, SenseExample*, SenseRelation*, Equivalent*; WordForm may contain FormRepresentation (type 준말, writtenForm) -> extra conjugated form.
- Entry feats: homonym_number (0 = none), lexicalUnit (단어 51555, 관용구 2227, 구 1120, 문법‧표현 996 (U+2027 char), 속담 657), partOfSpeech (명사, 동사, 품사 없음, 형용사, 부사, 관형사, 접사, 어미, 의존 명사, 감탄사, 조사, 대명사, 수사, 보조 동사, 보조 형용사), vocabularyLevel (초급/중급/고급/없음; mostly 고급), semanticCategory, origin (hanja, or English for loanwords e.g. "knot", mixed like "勞心焦思하다").
- WordForm type 발음 (pronunciation, sound) / 활용 (writtenForm, pronunciation). 활용 forms are few (e.g. 가다 -> 가는, 가, 가니, 갑니다), so conjugation lookup needs the app deconjugator.
- RelatedForm types: ☞(가 보라) (-> reference), 파생어 (derived); feats id, writtenForm.
- Sense feats: definition, annotation, syntacticPattern, syntacticAnnotation. SenseExample type 구/문장/대화; 대화 has 2 `example` feats. SenseRelation types: 유의어 반대말 참고어 높임말 낮춤말 큰말 작은말 센말 여린말 준말 본말 (feats lemma, id, homonymNumber). Equivalent feats language/lemma/definition; take language 영어 only (many other languages).
- Quirks: a few control bytes (0x01-0x1f) in 3 files make the XML invalid -> strip them at byte level before parsing. Annotations are double-escaped (`&amp;apos;`) -> html.unescape after parse. Grammar entries' English lemma is a romanisation (e.g. "-aseo"); use the English definition for gloss. Feat "type" also appears with values 사진/동영상 (Multimedia) - ignore.
- Mapping: kind 단어 word (어미/조사 -> grammar), 구 phrase, 관용구 idiom, 속담 proverb, 문법‧표현 grammar (pos 'expression'); POS map in common.py (KR_POS). Rel map KR_REL in common.py.

## stdict (RSS-like `<channel><item>`; CDATA everywhere; use iterparse on `item`, ~436k items)
- item > target_code, word_info > word, word_unit (단어/구/속담/관용구), word_type, original_language_info* (original_language + language_type: 한자/고유어/영어/안 밝힘/...), pronunciation_info/pronunciation, conju_info* (conjugation_info/conjugation, abbreviation_info/abbreviation), lexical_info* (word, unit 의미/어휘, type 동의어/참고 어휘/비슷한말/반대말/준말/본말, link), relation_info (type 부표제어), origin, allomorph, pos_info* > pos, comm_pattern_info* > pattern_info/pattern, grammar_info/grammar, sense_info* > type, definition (definition_original has <sense_no>/<word_no> tags - use `definition`), cat_info/cat ('없음' = none), example_info* > example (+ optional `source` -> EXCLUDE), lexical_info, multimedia_info.
- word: trailing 2-digit homonym number ("가03"), '-' morpheme/affix marker, '^' space marker ("가감-하다01"). Display: strip digits, drop inner '-', '^'->space, keep leading/trailing '-'.
- hanja = concat of original_language parts (한자 + 고유어 parts, in order) only if a 한자 part exists (ㄱㄴㄷ-순 -> ㄱㄴㄷ順); other languages -> origin_note.
- pos values: 명사 품사 없음 동사 구 형용사 부사 어미 관형사 접사 조사 의존 명사 감탄사 보조 동사. Entries may have several pos_info.

## kengdic
Tab-separated, header `id surface hanja gloss level created source`, 133,764 rows, QUOTE_NONE. 16k rows have empty gloss (hanja only); 38k have hanja; hanja may be comma separated variants ("交着하다,膠着하다"). level A/B/C/D (2.7k C, 2k B, 0.9k A) else empty. Dirty: double spaces, junk glosses ("VST + 먹다 , adds no meaning"), many rows are English->Korean artefacts with spaces in surface. Group by (surface, first hanja), dedupe glosses case-insensitively. Use as hanja fallback for other sources only when exactly one distinct hanja for the surface.

## FrequencyWords
"word count" lines, rank = line number. Rank for verbs/adjectives: min over own forms and stem+common-ending (see parsers/freq.py ENDINGS, 하->했/해 contraction).

## kaikki (wiktextract JSONL, from memory)
Per line: word, pos, senses[].glosses/raw_glosses/tags/examples[{text, english|translation, roman}]/form_of/alt_of, forms[{form,tags}] (hanja tag), etymology_text, etymology_number, sounds[{ipa|hangeul}], synonyms/antonyms/derived/related [{word}], head_templates. Skip senses with form-of/alt-of (record redirects). Wikt POS map in common.py (WIKT_POS).

## Tatoeba (from memory)
kor_sentences.tsv.bz2 / eng_sentences.tsv.bz2: `id \t lang \t text`. links.tar.bz2 contains links.csv `sentence_id \t translation_id` (all languages, both directions). Keep ids in the Korean set, first English match.

## Unihan (from memory)
Unihan.zip: Unihan_Readings.txt (`U+5B78\tkHangul\t학:0E`, kDefinition, kKorean Yale), Unihan_IRGSources.txt (kTotalStrokes, kRSUnicode "39.13", possibly with `'`). Radical number -> Kangxi char chr(0x2F00+n-1).

## Contract/design decisions made
- Entries are per source (no cross-source merging); rank = freq_rank*8 + tier (krdict L1..3 = 0..2, unleveled 3, wikt 4, kengdic 5, stdict 6); missing freq gets a per-source/level default.
- grammar table: 조사 Particles; 어미 by Korean def (선어말 Pre-final, 연결 Connective, 종결 Final, 전성/관형사형/명사형 Nominal/adnominal, else Final); 문법‧표현 Expressions; 접사 Affixes.
- sqlite: fts5 with trigram and porter available (SQLite 3.45). entries_fts is contentless: insert with rowid.
