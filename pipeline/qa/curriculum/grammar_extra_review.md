# grammar_extra.tsv: AI-written fields awaiting independent review

Source file: `pipeline/data/grammar_extra.tsv` (core pack entries, source `hanppuri`). Nothing here is
reviewed yet (`content-review` skill). Fields that are NOT listed were copied verbatim from stdict
(checked against `stdict.sqlite`, ext_id 66723).

Status column: `unreviewed` until an independent verifier signs it off.

| Pattern | Field | Text | Status |
|---|---|---|---|
| -노라면 | en_summary | A somewhat literary connective ending: "if you keep doing ..., (in time) ...". Attaches to a verb stem, to 있다/없다/계시다, or after -으시-. | unreviewed |
| -노라면 | example 1 en (ko is stdict) | If you keep living diligently, there will be good days too. | unreviewed |
| -노라면 | example 2 en (ko is stdict) | If you keep going straight ahead, a red-tiled house will appear on your right. | unreviewed |
| -기에 따라 | ko_def | 앞에 오는 행위나 상태가 어떠한가에 따라 뒤에 오는 결과나 판단이 달라짐을 나타내는 표현. | unreviewed |
| -기에 따라 | en_summary | An expression meaning "depending on how (one) does something": the result after it changes with the way the verb before -기 is done. Often with 달라지다/다르다. | unreviewed |
| -기에 따라 | forms | 기에 따라서 | unreviewed |
| -기에 따라 | example 1 ko | 이 옷은 입기에 따라 느낌이 많이 달라진다. | unreviewed |
| -기에 따라 | example 1 en | How this outfit feels changes a lot depending on how you wear it. | unreviewed |
| -기에 따라 | example 2 ko | 공부는 마음먹기에 따라 얼마든지 즐거울 수 있다. | unreviewed |
| -기에 따라 | example 2 en | Studying can be as enjoyable as you like, depending on how you set your mind. | unreviewed |
| -기에 따라 | example 3 ko | 같은 재료도 요리하기에 따라 전혀 다른 맛이 난다. | unreviewed |
| -기에 따라 | example 3 en | Even the same ingredients taste completely different depending on how you cook them. | unreviewed |
| -기에 앞서 | ko_def | 어떤 일을 하기 전에 먼저 다른 일을 함을 나타내는 표현. | unreviewed |
| -기에 앞서 | en_summary | An expression meaning "before doing ...": what follows is done first, ahead of the action named before -기에. The 서 may be added (-기에 앞서서). | unreviewed |
| -기에 앞서 | forms | 기에 앞서서 | unreviewed |
| -기에 앞서 | example 1 ko | 회의를 시작하기에 앞서 자료를 나누어 드리겠습니다. | unreviewed |
| -기에 앞서 | example 1 en | Before we begin the meeting, I will hand out the materials. | unreviewed |
| -기에 앞서 | example 2 ko | 여행을 떠나기에 앞서 날씨부터 확인해 보세요. | unreviewed |
| -기에 앞서 | example 2 en | Before you set off on your trip, check the weather first. | unreviewed |
| -기에 앞서 | example 3 ko | 계약서에 서명하기에 앞서 내용을 꼼꼼히 읽어 보시기 바랍니다. | unreviewed |
| -기에 앞서 | example 3 en | Before signing the contract, please read its contents carefully. | unreviewed |

Not AI content but hand-checked matcher decisions (see `src/curriculum.rs`), for the reviewer's eye:
`-그려` -> krdict particle 그려 (id 6712); `-을래야` -> `-으려야`; `에 따라` -> 따르다 (homonym 1, 8th sense);
`에 의하여` -> 의하다; `이 아니다` -> 아니다; `-어 치우다` -> auxiliary verb 치우다.
