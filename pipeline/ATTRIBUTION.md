# Data sources, licences and attribution

The dictionary data bundled with this app is derived from the open datasets
listed below. Each retains its original licence; the combined databases
(`core.sqlite`, `stdict.sqlite`, `opendict.sqlite`, `cedict.sqlite`, `zhwikt.sqlite`) are distributed under the most restrictive
share-alike terms that apply (CC BY-SA 4.0 compatible). The app source code is
licensed separately.

## krdict — 한국어기초사전 (Korean Learners' Dictionary)
* Publisher: National Institute of Korean Language (국립국어원, NIKL)
* Content used: headwords, hanja origin, pronunciation, level, English equivalents
  and definitions, Korean definitions, example sentences, related words, grammar
  and expression entries.
* Licence: Creative Commons Attribution-ShareAlike 2.0 Korea (CC BY-SA 2.0 KR) —
  https://creativecommons.org/licenses/by-sa/2.0/kr/
* Obtained from the mirror https://github.com/spellcheck-ko/korean-dict-nikl
  (original: https://krdict.korean.go.kr).

## stdict — 표준국어대사전 (Standard Korean Language Dictionary)
* Publisher: National Institute of Korean Language (국립국어원, NIKL)
* Content used: headwords, hanja, pronunciation, conjugation, Korean definitions
  and example sentences. Examples that carry a `<source>` citation (quotations from
  copyrighted works) are **excluded**.
* Licence: Creative Commons Attribution-ShareAlike 2.0 Korea (CC BY-SA 2.0 KR).
* Obtained from https://github.com/spellcheck-ko/korean-dict-nikl
  (original: https://stdict.korean.go.kr).

## opendict — 우리말샘 (Urimalsaem, open Korean dictionary)
* Publisher: National Institute of Korean Language (국립국어원, NIKL), user-contributed.
* Content used: headwords, hanja, pronunciation, Korean definitions, dialect / region /
  archaic / North Korean labels, categories, English translations and examples.
  Examples that carry a `<source>` citation (quotations from copyrighted works) are **excluded**.
  Entries that duplicate a stdict entry (same normalised headword and POS) are omitted.
* Licence: Creative Commons Attribution-ShareAlike 2.0 Korea (CC BY-SA 2.0 KR).
* Obtained from https://github.com/spellcheck-ko/korean-dict-nikl (original: https://opendict.korean.go.kr).

## Wiktionary (via kaikki.org)
* Content used: Korean entries with English glosses, hanja forms, etymology,
  related terms and translated example sentences; the 훈음 (Korean gloss word and reading)
  of single hanja characters.
* Source: English Wiktionary contributors; machine-readable extraction by
  Tatu Ylonen's wiktextract (https://kaikki.org/dictionary/Korean/).
* Licence: Creative Commons Attribution-ShareAlike 4.0 (CC BY-SA 4.0) —
  https://creativecommons.org/licenses/by-sa/4.0/ and the GNU Free Documentation License.

## Wiktionary English → Korean translations (via kaikki.org)
* Content used: the Korean entries of the translation tables of English Wiktionary entries
  (English term, part of speech, sense label, Korean word, romanisation), shown for English
  searches as "Wiktionary translations" and "English phrases" (`en_ko` table of `core.sqlite`).
  Hanja in brackets, non-Hangul items and "translations to be checked" are left out.
* Source: English Wiktionary contributors; machine-readable extraction by
  Tatu Ylonen's wiktextract (https://kaikki.org/dictionary/English/).
* Licence: Creative Commons Attribution-ShareAlike 4.0 (CC BY-SA 4.0) —
  https://creativecommons.org/licenses/by-sa/4.0/ and the GNU Free Documentation License.

## kengdic
* Author: Charles Muller / Garfield Nate (https://github.com/garfieldnate/kengdic),
  derived from the EZ-Corean / Korean-English dictionary data.
* Content used: Korean–English word pairs and hanja.
* Licence: Mozilla Public License 2.0 / LGPL (see the upstream repository).

## Tatoeba
* Content used: Korean sentences with English translations.
* Source: https://tatoeba.org — contributors are credited by the Tatoeba project.
* Licence: Creative Commons Attribution 2.0 France (CC BY 2.0 FR) —
  https://creativecommons.org/licenses/by/2.0/fr/

## Unihan Database
* Content used: hangul readings (kHangul), English definitions (kDefinition),
  stroke counts and radicals for Han characters.
* Source: Unicode, Inc. — https://www.unicode.org/charts/unihan.html
* Licence: Unicode License v3 — https://www.unicode.org/license.txt

## FrequencyWords
* Content used: word frequency ranks (OpenSubtitles-derived) used only to order
  search results.
* Source: Hermit Dave — https://github.com/hermitdave/FrequencyWords
* Licence: CC BY-SA 4.0 (content), MIT (code).

## CC-CEDICT (optional `cedict` pack)
* Content used: Chinese (traditional / simplified) headwords, pinyin and English glosses.
  The Sino-Korean reading shown with each entry is computed from the hanja readings of this app.
* Source: https://www.mdbg.net/chinese/dictionary?page=cc-cedict — CC-CEDICT, community maintained.
* Licence: Creative Commons Attribution-ShareAlike 4.0 (CC BY-SA 4.0) —
  https://creativecommons.org/licenses/by-sa/4.0/

## Wiktionary Chinese (optional `zhwikt` pack, via kaikki.org)
* Content used: Chinese entries (traditional / simplified forms), glosses and usage tags
  (e.g. Classical / literary), short etymology, Mandarin / Middle Chinese / Sino-Korean
  (and Cantonese, Sino-Vietnamese, Sino-Japanese) readings. Translations, inflection tables
  and examples are not included.
* Source: English Wiktionary contributors; extraction by wiktextract (https://kaikki.org/dictionary/Chinese/).
* Licence: CC BY-SA 4.0 and the GNU Free Documentation License.

## Reader library (optional `texts` pack)
* Content used: public-domain and openly licensed Korean texts shown in the Reader, with
  AI-written introductions, notes, translations and modernised spelling (always labelled as such
  in the app), and ~40 original graded readers written for this app (labelled AI-written).
  Every text carries its own provenance (page, revision, URL, licence, fetch date) in the pack.
* Korean and Chinese Wikisource (https://ko.wikisource.org, https://zh.wikisource.org):
  texts of authors who died before 1963 and older anonymous works are in the public domain;
  Wikisource editions, transcriptions and page text are available under Creative Commons
  Attribution-ShareAlike 4.0 (CC BY-SA 4.0) — https://creativecommons.org/licenses/by-sa/4.0/ —
  contributors are credited in each page's revision history, linked from the text's provenance.
* Laws and court rulings (e.g. the Constitution of the Republic of Korea): not protected by
  copyright (Copyright Act of Korea, art. 7).
* Universal Declaration of Human Rights, Korean text: © United Nations; reproduced under the
  OHCHR terms of use (https://www.ohchr.org/en/about-this-site/terms-of-use), unaltered.
* News articles: Policy Briefing (정책브리핑, https://www.korea.kr), Republic of Korea government,
  marked 공공누리 제1유형 (Korea Open Government Licence, Type 1: attribution; commercial use and
  modification allowed). 출처: 정책브리핑 (korea.kr).
* Historical English translations (Gale, Allen, etc., 1889–1922; Project Gutenberg / Internet
  Archive): public domain.

## Fonts
* `app/public/fonts/yethangul-jamo.woff2`: subset of 나눔명조 옛한글 (NanumMyeongjo YetHangul),
  © 2014 NHN Corporation (NAVER), designed by FONTRIX Inc., SIL Open Font License 1.1. Used only to
  render Old Hangul (옛한글) jamo sequences.
