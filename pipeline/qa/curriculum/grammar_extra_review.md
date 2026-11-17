# grammar_extra.tsv: AI-written fields awaiting independent review

Source file: `pipeline/data/grammar_extra.tsv` (core pack entries, source `hanppuri`). Reviewed independently on
2026-11-17 (`content-review` skill; record below; the table shows the text as approved). Fields that are NOT listed were copied verbatim from stdict
(checked against `stdict.sqlite`, ext_id 66723).

Status column: `unreviewed` until an independent verifier signs it off; `approved` once signed off.

| Pattern | Field | Text | Status |
|---|---|---|---|
| -노라면 | en_summary | An old-fashioned, literary-sounding connective ending meaning "if you keep doing ..., (in time) ..." (= -다가 보면). Attaches to a verb stem, to 있다/없다/계시다, or after -으시-. | approved |
| -노라면 | example 1 en (ko is stdict) | If you keep living diligently, surely there will be good days too. | approved |
| -노라면 | example 2 en (ko is stdict) | If you keep going straight ahead, a house with a red-tiled roof will come into view on your right. | approved |
| -기에 따라 | ko_def | 앞에 오는 말이 나타내는 행동을 어떻게 하는가에 따라 뒤에 오는 결과나 판단이 달라짐을 나타내는 표현. | approved |
| -기에 따라 | en_summary | An expression meaning "depending on how (one) does something": what follows (a result or a judgement) changes with the way the action before -기 is done. Often with 달라지다/다르다, and in 보기에 따라서는 / 생각하기에 따라서는 "depending on how you look at it". | approved |
| -기에 따라 | forms | 기에 따라서 | approved |
| -기에 따라 | example 1 ko | 이 옷은 입기에 따라 느낌이 많이 달라진다. | approved |
| -기에 따라 | example 1 en | The look of this outfit changes a lot depending on how you wear it. | approved |
| -기에 따라 | example 2 ko | 공부는 마음먹기에 따라 얼마든지 즐거울 수 있다. | approved |
| -기에 따라 | example 2 en | Studying can be as enjoyable as you like, depending on your frame of mind. | approved |
| -기에 따라 | example 3 ko | 같은 재료도 요리하기에 따라 전혀 다른 맛이 난다. | approved |
| -기에 따라 | example 3 en | Even the same ingredients taste completely different depending on how you cook them. | approved |
| -기에 앞서 | ko_def | 앞에 오는 말이 나타내는 행동을 하기 전에 뒤에 오는 말이 나타내는 행동을 먼저 함을 나타내는 표현. | approved |
| -기에 앞서 | en_summary | An expression meaning "before doing ...": what follows is done first, ahead of the action named before -기에. Formal: typical of announcements, speeches and writing. The 서 may be added (-기에 앞서서). | approved |
| -기에 앞서 | forms | 기에 앞서서 | approved |
| -기에 앞서 | example 1 ko | 회의를 시작하기에 앞서 자료를 나누어 드리겠습니다. | approved |
| -기에 앞서 | example 1 en | Before we begin the meeting, I will hand out the materials. | approved |
| -기에 앞서 | example 2 ko | 여행을 떠나기에 앞서 날씨부터 확인해 보세요. | approved |
| -기에 앞서 | example 2 en | Before you set off on your trip, check the weather first. | approved |
| -기에 앞서 | example 3 ko | 계약서에 서명하기에 앞서 내용을 꼼꼼히 읽어 보시기 바랍니다. | approved |
| -기에 앞서 | example 3 en | Before signing the contract, please read its contents carefully. | approved |

Not AI content but hand-checked matcher decisions (see `src/curriculum.rs`), for the reviewer's eye:
`-그려` -> krdict particle 그려 (id 6712); `-을래야` -> `-으려야`; `에 따라` -> 따르다 (homonym 1, 8th sense);
`에 의하여` -> 의하다; `이 아니다` -> 아니다; `-어 치우다` -> auxiliary verb 치우다.

## Review — 2026-11-17 — content-review (independent)

Status: **approved** (with the small edits listed below, applied in `grammar_extra.tsv`; the table above
shows the approved text). `cargo test -p kdict-pipeline --test grammar_extra` passes.

Provenance check: the -노라면 ko_def and both Korean examples match `stdict.sqlite` ext_id 66723
byte-for-byte (definition text and `examples[].ko`); the en_summary's attachment note matches stdict's
note (‘있다’, ‘없다’, ‘계시다’의 어간, 동사 어간 또는 어미 ‘-으시-’ 뒤에 붙어).

Per entry:
- **-노라면** — stdict marks it 예스러운 (old-fashioned), so "somewhat literary" undersold it; now
  "old-fashioned, literary-sounding" plus the gloss -다가 보면 that the ko_def itself uses. Example 1 lost
  the speaker's supposition of -겠지; restored with "surely". Example 2: 붉은 기와집 is a house roofed
  with red tiles, and 나타나다 here is "come into view".
- **-기에 따라** — pattern is right (depending on how one does V; result/judgement varies). The ko_def
  said 행위나 상태, but the pattern takes action verbs (입기에, 마음먹기에, 보기에, 생각하기에), not states;
  rewritten in krdict wording (cf. krdict -기 나름이다). en_summary reworded and given the very common
  보기에 따라서는 / 생각하기에 따라서는. Korean examples are natural and correct; example 1 en
  "how this outfit feels" read as physical comfort, but 느낌 here is the look/impression; example 2 en
  "set your mind" is unidiomatic. Example 3 fine. forms 기에 따라서 fine.
- **-기에 앞서** — pattern is right ("before doing", formal). ko_def made explicit about which action
  comes first (krdict style). en_summary now states the formal register. All three Korean examples are
  natural, correctly formed and level-appropriate (meeting announcement, travel advice, contract notice);
  translations faithful. forms 기에 앞서서 fine.

Edits (old → new):
1. -노라면 en_summary: `A somewhat literary connective ending: "if you keep doing ..., (in time) ...".`
   → `An old-fashioned, literary-sounding connective ending meaning "if you keep doing ..., (in time) ..." (= -다가 보면).`
2. -노라면 ex1 en: `If you keep living diligently, there will be good days too.`
   → `If you keep living diligently, surely there will be good days too.`
3. -노라면 ex2 en: `... a red-tiled house will appear on your right.`
   → `... a house with a red-tiled roof will come into view on your right.`
4. -기에 따라 ko_def: `앞에 오는 행위나 상태가 어떠한가에 따라 뒤에 오는 결과나 판단이 달라짐을 나타내는 표현.`
   → `앞에 오는 말이 나타내는 행동을 어떻게 하는가에 따라 뒤에 오는 결과나 판단이 달라짐을 나타내는 표현.`
5. -기에 따라 en_summary: `... the result after it changes with the way the verb before -기 is done. Often with 달라지다/다르다.`
   → `... what follows (a result or a judgement) changes with the way the action before -기 is done. Often with 달라지다/다르다, and in 보기에 따라서는 / 생각하기에 따라서는 "depending on how you look at it".`
6. -기에 따라 ex1 en: `How this outfit feels changes a lot depending on how you wear it.`
   → `The look of this outfit changes a lot depending on how you wear it.`
7. -기에 따라 ex2 en: `... depending on how you set your mind.` → `... depending on your frame of mind.`
8. -기에 앞서 ko_def: `어떤 일을 하기 전에 먼저 다른 일을 함을 나타내는 표현.`
   → `앞에 오는 말이 나타내는 행동을 하기 전에 뒤에 오는 말이 나타내는 행동을 먼저 함을 나타내는 표현.`
9. -기에 앞서 en_summary: `The 서 may be added (-기에 앞서서).`
   → `Formal: typical of announcements, speeches and writing. The 서 may be added (-기에 앞서서).`

Matcher decisions listed above (그려, -을래야 → -으려야, 에 따라 → 따르다, 에 의하여 → 의하다,
이 아니다 → 아니다, -어 치우다 → 치우다): all sensible mappings; not re-verified against krdict ids.
