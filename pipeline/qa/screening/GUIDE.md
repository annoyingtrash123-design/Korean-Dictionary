# Screening guide: catching meaning errors in AI-translated Korean examples

For the screener (Sonnet) and the judge (Opus). Each row gives a Korean example sentence, its
English translation (AI-written), the headword, the dictionary's English gloss and the Korean
definition (뜻). Your job is to find the rows where the English says something the Korean does not.

Why this guide exists: outside models (Grok, Gemini, ChatGPT and two unnamed models) found 253
errors the judge confirmed. On the 128 of those rows that a Sonnet screen had also read, Sonnet
flagged 45 and missed 83 (55 translation errors + 28 gloss errors). Sonnet was good at titles and
invented gender. It mostly missed **wrong word sense, idioms, who-did-what, quantities and
verb endings**: errors you only see if you work out the meaning from the Korean before you read the
English. Evidence: `results/guide_evidence.tsv`; counts at the end of this guide.

## 1. Procedure for every row (do not skip steps on "easy" rows)

1. **Read the Korean first. Do not look at the English yet.** Split it into clauses. For each one, write
   down in your head: who (이/가/은/는), does what (the verb), to whom or what (을/를), with whom (와/과/하고),
   where and when (에/에서/으로), and what each modifier belongs to (모두/다/만, ~의, a relative clause
   ending in -ㄴ/-는/-ㄹ). Settle any word that could be read two ways: 나와 = "with me" (나+와),
   not 나오다; spaced 이 층 = "second floor", not "this floor".
2. **Pin down the headword's sense from 뜻**, not from the English gloss. Say it in 3–6 English words.
   Note how strong it is and the exact physical action (a loud laugh or a small one, butting or charging,
   doing it yourself or directing it).
3. **Mark every phrase that is a set unit**: a body part or weather noun with a verb (날이 풀리다,
   귀가 먹먹하다, 애를 먹다, 허리가 곱다), a fixed phrase (남부럽지 않다, 보란 듯이, 꼴값하다),
   or a four-character idiom. Translate the unit as a whole, never word by word.
4. **Read every verb ending** and look it up in the endings table (section 3).
5. **Note every number, time, amount, and singular/plural** (하오 일곱 시, 반나절, 반 이하, 어른이 = one elder).
6. **Note every person**: title, rank, kinship term (table in section 4), and whether the Korean
   gives their sex.
7. **Now read the English.** Compare it with your own reading, clause by clause:
   - Is each of your clauses there, with the same subject, object and modifiers?
   - Does the headword (and every other content word) have the right sense and strength?
   - Is anything added (a "Lord", "their troops", "office", "a bag") or dropped (a title, a noun)?
   - Would a learner who reads only the English get a wrong picture? Then it is at least MINOR
     ("wing joint" for a person's shoulder, "lonelily", "golden hand", "as a goof").
8. **Check the gloss** against 뜻, item by item (section 2, GLOSS).
9. **Before you write a line, check the false flags list (section 5).** Your reason must name a word
   that really appears in *this* row.

Treat the English as an unreliable witness. Fluent English is the danger sign: most missed errors read
naturally and make sense. They are wrong only against the Korean.

## 2. Error types, with examples (Korean | wrong English | why | fix)

**A. Wrong sense or wrong strength (46 confirmed, 18 missed by Sonnet)**: the commonest miss.
- 산돼지가 머리로 사냥꾼을 받아서 사냥꾼이 그 자리에 쓰러졌다. | A wild boar *charged at* the hunter with its head |
  받다 here means hitting with the head, not running at | A wild boar *butted* the hunter with its head…
- 할아버지께서는 나만 보면 껄껄대며 웃으신다. | he *chuckles* heartily | 껄껄 = loud, hearty laughter; a chuckle is
  the opposite strength | he *roars with laughter*.
- 김 박사의 집도로 시체의 부검이 실시되었다. | carried out *under Dr. Kim's direction* | 집도 = holding the scalpel
  yourself | The autopsy was *performed by* Dr. Kim.
- Also: 호도 = gloss over (not "distortion"); 저의 = ulterior motive; 소아 = young child (not infant);
  유모 = wet nurse per 뜻; 박차다 = kick open (not "slam"); 생을 마감했다 = died (English "ended his life" implies
  suicide); 꼭두새벽 = crack of dawn (not "middle of the night"); 낙천 = not nominated (not "disqualified").

**B. Idiom or set phrase read word by word (13, 5 missed)**
- 날이 풀리자 서둘러 어선들이 바다로 고기잡이에 나섰다. | When the weather *cleared* | 날이 풀리다 = the cold eases |
  When the weather *turned mild*…
- …귀까지 먹먹했다. | my ears felt *numb* | 귀가 먹먹하다 = ears feel blocked or
  muffled | My heart was pounding so hard that even my ears felt muffled.
- 남부럽지 않은 (of a rich man) | *whom no one could envy* | 남부럽지 않다 = has no reason to envy others; the
  English reverses it | so rich that he had no one to envy.
- …지수는 꼴값하느라 남자 친구가 자주 바뀐다. | because she *acts above herself* | 꼴값하다 = behaves
  just as her looks would suggest | Jisu acts just as her looks would suggest, so her boyfriends keep changing.

**C. Who did what / what modifies what (10, 7 missed)**
- 나를 보자마자 오빠가 머리에 인 짐부터 내려 줬다. | lifted the bundle off *his* head | 내려 주다 = did it *for me*,
  so the bundle was on my head | …took the bundle off *my* head first.
- 아이가 열이 내리자 병세를 돌렸다. | When the child's fever came down, *the illness* turned around | the child is
  the one who 병세를 돌렸다 | …the child turned the corner.
- 부부 동반 모임에 나타난 친구 부부는 닭살 애정을 과시했다. | *The friend* who appeared at the couples' gathering
  showed off… | 친구 부부 = the friend and the friend's spouse | My friend and their spouse… flaunted their affection.

**D. Tense, aspect, modality (9, 3 missed)**
- …총상을 입고 죽게 되었다. | and *nearly died* | -게 되었다 = it came about: he died | …and died.
- 데이트가 있는지 지수는 고이고이 단장을 하고 집을 나섰다. | *With a date ahead*, Jisu left… | -는지 = the speaker's
  guess | Jisu, *who seemed to have* a date, got all dressed up and left the house.
- 소나기가 내린 날이면 유실물 보관소에는 … 우산이 가득했다. | *On the day of* a sudden shower | 날이면 = whenever |
  *On days when* a sudden shower fell…

**E. Titles, ranks, kinship (27, 1 missed)**: Sonnet's strongest category. Keep using the table.
- 박 부장은 이 사업을 총괄하여 지휘하고 있다. | *Section Chief* Park | 부장 = department head | Department Head Park…
- 경장이었던 삼촌이 이번 승진 시험에 합격하여 경사가 되었다. | passed the exam, *which was a happy event* |
  경사 after 경장 is the police rank, not 경사 "celebration" | …and became a police sergeant.
- 상사는 병사들을 불러 지시 사항을 전달했다. | The *sergeant* | military 상사 = master sergeant | The master sergeant…

**F. Numbers, times, amounts, singular/plural (6, 4 missed)**
- 친구는 하오 일곱 시에 미국으로 가는 비행기를 탔다. | at seven *in the afternoon* | 7 p.m. is evening in English |
  at seven in the evening.
- 가장 윗자리에는 어른이 앉고 아이들은 아랫자리에 앉았다. | *The adults* sat in the top seat | one top seat, one elder |
  *The elder* sat in the top seat…
- Also: 반나절 = a few hours (not "half a day"); 반 이하 = half or less; spaced 이 층 = second floor;
  일대일로 나누다 = split fifty-fifty (not "one-on-one").

**G. Content added or dropped (9, 3 missed)**
- 저희의 기도를 들어주시옵소서, 아멘. | Hear our prayer, *Lord*, amen | "Lord" is not in the Korean | Hear our prayer, amen.
- 장군은 폐허가 된 마을을 보고 적을 향해 전의를 불태웠다. | fired up *their troops'* fighting spirit | troops added |
  burned with the will to fight the enemy.
- 당신 나 모르게 꿍쳐 놓은 비자금 몽땅 털어놔요. | tell me *everything, including* the slush fund | widens the
  object | Spill it: all of the slush fund you stashed away behind my back.

**H. Invented gender, possessive; 그/그녀 (60, 8 missed)** (MINOR, except 그/그녀 rendered wrongly: MAJOR)
- 장인은 손으로 빚은 자기를 가마에 넣고 구웠다. | The *craftsman* placed the porcelain *he* had shaped |
  sex not given | The artisan put the handmade porcelain into the kiln and fired it.
- …노인이 나뭇등걸에 걸터앉아 시간을 보내고 있었다. | A white-haired *old man* | 노인 is unsexed | elderly person.
- 평생 많은 역경을 혼자서 헤쳐 온 그의 눈에는 어떤 살기가 어려 있었다. | In the eyes of *a man* who… | 그 = he |
  Having weathered many hardships alone, *he* had a certain menace in his eyes.

**I. Garbled word-for-word English (27, 6 missed)** (MINOR, but MAJOR if it misleads)
- 왼쪽 팔목부터 죽지를 넘어 어깨까지 통증이 이어졌다. | over the *wing joint* | for a person 죽지 = shoulder joint |
  past the shoulder joint.
- 유민이는 손재주가 좋아 친구들에게 금손으로 불린다. | call her *a golden hand* | | say she has magic hands.

**J. Register (2)**: keep insults insulting and baby-talk babyish.
- 평소에 욕을 잘했던 아버지는 나를 항상 네놈이라고 불렀다. | called me *that fellow* | 네놈 = rude second-person
  "you" | called me "you bastard".

**K. Gloss wrong for 뜻 (44, 28 missed: the old Sonnet prompt had no gloss task)**
- 상수: "portable water" → "potable water" (spelling). Other typos found: desparate, lightening, breath out.
- 금고: "safety; strongbox" → "safe; strongbox". 세균: "virus" → "germ; bacteria".
- 자식 (abusive 뜻): "chap" is friendly → "bastard; punk". 흑인: "Negroid" is offensive → "Black person".
- 남파되다 (passive): "dispatch a spy" → "be sent south". 난치: "being incurable" → "being hard to cure".
- 남침하다: "invade South Korea" is narrower than 뜻 → "invade from the north".
Check every item between semicolons: spelling, sense, part of speech, active vs passive, too narrow
or too wide, register, offensive or archaic words.

## 3. Endings table (read every ending)

| Korean | Means | Common wrong English |
|---|---|---|
| -았/었- | past | present ("lives", "doesn't matter") |
| -았었- | had done / used to (no longer so) | simple past without "had" or "used to" |
| -게 되었다 | came to, ended up: it happened | "nearly", "was about to" |
| -ㄹ 뻔했다 | nearly (it did NOT happen) | "did" |
| -(으)ㄴ지/-는지, -나 보다, -는 모양이다 | apparently, perhaps (a guess) | stated as fact |
| -(으)면, -는 날이면, -ㄹ 때마다 | whenever (habit) | "on the day" (one event) |
| -ㄹ 만큼, -ㄹ 정도로 | enough to (degree) | "did" (event) |
| -는 게 꿈이다, -려고, -고 싶다 | aim or wish, not done yet | "learned", "did" |
| -다가 | while doing, then interrupted | completed action |
| -듯(이), -처럼 | as if (keep which side is the comparison) | simile reversed or circular |
| -아/어 주다 | did it for someone (that person is the beneficiary) | beneficiary lost |
| -지 않다/못하다/안 | negation: check what it covers | negation moved or dropped |

## 4. Fixed renderings (titles, ranks, kinship, people)

| Korean | Render as | Never |
|---|---|---|
| 사장, 사장님 | (company) president; "President Kim" | boss, owner, chairman, Mr. Kim |
| 부장 | department head | director, section chief, manager |
| 과장 | section chief | manager |
| 차장 | deputy department head | |
| 대리 | assistant manager | |
| 회장 | chairman (of a company or group) | president |
| 의원, 국회의원 | lawmaker; Representative Kim | Assemblyman, congressman |
| 화백 | painter (keep it as a title: "the painter Kim") | dropped |
| 박사 | Dr. | |
| 상사 (military, see 뜻) | master sergeant | sergeant |
| 상사 (workplace) | superior, boss | |
| 경장 / 경사 | police corporal / police sergeant | 경사 = "happy event" in a rank context |
| 사부님 | follow 뜻: teacher's husband, or one's master | "master teacher" when 뜻 says husband |
| 아주머니 (and its male pair *ajeossi*) | a (middle-aged) woman, ma'am (a man, mister) | aunt (uncle), unless kin |
| 삼촌 | uncle (father's unmarried brother) | |
| 사돈 | in-law (parent of one's child's spouse) | "his in-law" with invented sex |
| 고조할머니 | great-great-grandmother (from the speaker's view) | shifted a generation by "her" |
| 오빠 / 형 | older brother of a female / of a male speaker | |
| 언니 / 누나 | older sister of a female / of a male speaker | |
| 동생 | younger sibling | brother, sister (unless 남동생/여동생) |
| 형제 | siblings (brothers only if context says) | brothers by default |
| 부부, 우리 부부 | (married) couple; my spouse and I | my husband and I |
| 친구 부부 | my friend and their spouse | the friend |
| surname + polite *ssi* (e.g. Kim *ssi*) | Kim | Mr. Kim |
| surname + 모 + *ssi* | a certain Kim | Mr. Kim |
| 노인, 젊은이, 아이, 친구, 선생님, 주인공 | elderly person, young person, child, friend, teacher, main character | old man, young man, his/her, heroine |
| 그 (pronoun) / 그녀 | he / she | them, a man |
| 그 + noun (그 장수, 그 형사) | that general, that detective | he |
| 네놈, 자식 (abuse) | you bastard, you punk | that fellow, chap |
| 뚱보 / 멍멍이 | fatso / doggy | chubby guy / puppy |
| 필름이 끊기다 | black out (from drinking) | the film broke |

House conventions (never flag): he/his for 승규 and 민준, she/her for 지수 and 유민; singular "they" for a
person whose sex is not given; romanised names as Seung-gyu, Jisu, Minjun, Yumin.

## 5. False flags (the judge rejected these; do not repeat them)

- **House-convention names**: 34 of Sonnet's 38 rejected flags were "he/she for 승규/민준/지수/유민". Never flag these.
- **Flags that belong to another row**: 7 ChatGPT flags described a different sentence. Your reason must quote a
  word from this row's Korean.
- **Gloss judged against the wrong sense**: if 뜻 says "bad smell", do not flag the gloss for missing the idiom
  "something suspicious". Flag only items that are actually in the gloss.
- **Sex that the context does give**: a child who calls someone 누나 is male; 김 *ssi* with 아내 is a husband, so "Mr." is
  fine; 그에게 referring back to a person is "him" by rule.
- **그 as a determiner**: 그 장수 = "that general"; singular they is right there.
- **Tense nuance only**: narrative present for 웃는다, or past vs present where the meaning does not change.
- **Romanisation you have not checked**: 좋은 → joeun is correct Revised Romanization (the h is silent).
- **Spaced Sino-Korean numbers**: 이 학년 = second year, not "this grade".
- **Taste and register preferences**: "endlessly welcome", "deteriorated to an extreme": plain but correct.
- **Synonyms within the word's range**: "tamed" for 순화되다, "gasp" for 탄성, "dictated" for an oral statement.
- **Attachment that is truly ambiguous in Korean**, when both readings give the same facts.
- **A faithful translation of an odd Korean sentence** is not an error.

## 6. Evidence summary

Sources: trial batches T1 and T2 (`trials_decided.json`, `trials_chat_flags.tsv`, `sonnet_t2.txt`, Grok T2), B02–B03
(two outside models), C01 (Gemini 1–500, Sonnet 501–1000), C02 (ChatGPT), C03 (Grok), judged in
`*_decided.json` and `C01_C02_C03_rejudge_decided.json`. Rows of the held-out test set are excluded.
"Sonnet screened" = the same sentence was in a Sonnet run (T2, C01b, or the partial B01/B04/B05 runs, read up to
their last flagged id).

| Category | Confirmed | Caught by (outside models; Sonnet's own run) | Sonnet screened | Sonnet missed |
|---|---|---|---|---|
| A wrong sense / strength | 46 | Grok 21, ChatGPT 10, Gemini 8, B03 4, B02 1; Sonnet 3 | 25 | 18 |
| B idiom read literally | 13 | Grok 6, B03 6; Sonnet 1 | 6 | 5 |
| C who did what / modifiers | 10 | Grok 5, Gemini 3, ChatGPT 1; Sonnet 2 | 9 | 7 |
| D tense / modality | 9 | Grok 6, Gemini 2; Sonnet 3 | 8 | 3 |
| E titles / ranks / kinship | 27 | B02 7, ChatGPT 5, Gemini 4, Grok 4, B03 2; Sonnet 5 | 10 | 1 |
| F numbers / time / plural | 6 | Grok 3, B02 1, Gemini 1; Sonnet 1 | 5 | 4 |
| G added / dropped | 9 | Grok 3, B03 2, Gemini 2, ChatGPT 1; Sonnet 1 | 5 | 3 |
| H gender / possessive / 그 | 60 | B02 16, Grok 14, B03 10, Gemini 7, ChatGPT 7; Sonnet 7 | 23 | 8 |
| I garbled English, romanisation | 27 | B03 20, Grok 5, Gemini 1; Sonnet 1 | 7 | 6 |
| J register | 2 | Gemini 1, ChatGPT 1 | 1 | 0 |
| K gloss | 44 | Grok 34, Gemini 5, ChatGPT 4; Sonnet 1 | 29 | 28 |
| **Total** | **253** | Grok 101, B03 44, Gemini 34, ChatGPT 29, B02 25; Sonnet 25 | **128** | **83** |

Precision (confirmed / flags on judged rows): Gemini 34/35, B02 25/25, B03 44/57, Grok 101/151, ChatGPT 29/54,
Sonnet 25/63. Grok's and ChatGPT's rejected flags were mostly gloss flags against a wrong 뜻 in their input
or flags for another row. Sonnet's were almost all house-convention names.

What the outside models did that Sonnet did not: (1) they rebuilt the meaning from the Korean particles before
trusting the English (C); (2) they compared each content word's exact action and strength with 뜻 (A);
(3) they treated body-part, weather and set phrases as units (B); (4) they read endings like -는지, -면, -게 되다
(D); (5) they converted times and amounts and checked singular vs plural (F); (6) they checked every gloss item
for spelling, voice and register (K).
