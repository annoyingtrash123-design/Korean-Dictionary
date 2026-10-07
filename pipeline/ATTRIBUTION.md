# Data sources, licences and attribution

The dictionary data bundled with this app is derived from the open datasets
listed below. Each retains its original licence; the combined databases
(`core.sqlite`, `stdict.sqlite`) are distributed under the most restrictive
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

## Wiktionary (via kaikki.org)
* Content used: Korean entries with English glosses, hanja forms, etymology,
  related terms and translated example sentences.
* Source: English Wiktionary contributors; machine-readable extraction by
  Tatu Ylonen's wiktextract (https://kaikki.org/dictionary/Korean/).
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
