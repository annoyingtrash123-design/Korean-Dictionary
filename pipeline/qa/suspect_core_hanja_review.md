# Suspect core hanja — review (2026-11-24)

Input: `pipeline/qa/suspect_core_hanja.tsv` (14 krdict entries whose Korean definition best matches a
*native* 표준국어대사전 entry while the stdict entry with the entry's hanja matches poorly).
Output: `pipeline/data/hanja_fixes.tsv` (13 removals) and `pipeline/data/hanja_keep.tsv` (1 keep, skipped by the scan).

## Root cause

None of the 14 hanja is krdict's own 원어: the raw krdict XML (`.work/nikl/krdict/*.xml`) has **no `origin`
feat** for any of these entry ids. All 14 were set by the build's **kengdic hanja fallback** (`build.rs`,
"hanja fallback for krdict / wikt entries lacking it": a krdict noun without hanja takes kengdic's hanja when the
surface has exactly one hanja spelling in kengdic). kengdic attaches the hanja homograph's spelling to the
native word's rows, e.g. `구경 究竟 "an opening, an aperture"`, `매질 媒質 "beating, caning, flogging"`,
`제철 製鐵 "season"`, `어음 語音 "a note given to say you'll pay later"`, `흉 匈 "malignancy"`, and has bare
`곽 矡`, `멱 蓂`, `비지 鼻紙`, `타박 打撲` rows. The fallback leaves no marker in `data`, so the suspect scan
treated these as krdict's own hanja.

Evidence used per entry: krdict ko_def (core pack), 표준국어대사전 (`stdict.sqlite`), and 우리말샘 word_type
(`.work/nikl/opendict`, 고유어 / 한자어).

## Decisions

| headword | ext_id | hanja | decision |
|---|---|---|---|
| 타박 | 79627 | 打撲 | remove |
| 구경 | 64484 | 究竟 | remove |
| 매질 | 55736 | 媒質 | remove |
| 종지 | 76084 | 宗旨 | remove |
| 방정 | 58132 | 方正 | remove |
| 위통 | 92365 | 胃痛 | remove |
| 제철 | 93735 | 製鐵 | remove |
| 어음 | 67051 | 語音 | remove |
| 비지 | 60764 | 鼻紙 | remove |
| 불길 | 60545 | 不吉 | remove |
| 멱 | 55554 | 蓂 | remove |
| 곽 | 21807 | 矡 | remove |
| 흉 | 87835 | 匈 | remove |
| 동생 | 70748 | 同生 | keep |

No entry needs a *different* hanja: in every removal the sense is native in both 표준국어대사전 (no 원어) and
우리말샘 (word_type 고유어).

### 타박 (79627) 打撲 — remove
- krdict: "다른 사람의 실수나 결함을 나무라거나 핀잔함." (finding fault with; criticism)
- stdict native 타박: "허물이나 결함을 나무라거나 핀잔함."; 우리말샘 same, 고유어.
- stdict 타박(打撲): "사람이나 동물을 때리고 침." — a different word (cf. 타박상).

### 구경 (64484) 究竟 — remove
- krdict: "흥미나 관심을 가지고 봄." / "흥미나 관심을 일으키게 하는 것." / "눈으로 보거나 직접 경험함."
- stdict native 구경: "흥미나 관심을 가지고 봄." (identical); 우리말샘 고유어.
- stdict 구경(究竟): "마지막에 이르는 것." / "가장 지극한 깨달음." — unrelated Buddhist/literary term.

### 매질 (55736) 媒質 — remove
- krdict: "매로 때리는 일." (whipping; spanking)
- stdict native 매질: "매로 때리는 일." (identical; 매 + -질); 우리말샘 고유어.
- stdict 매질(媒質): "어떤 파동 또는 물리적 작용을 한곳에서 다른 곳으로 옮겨 주는 매개물." — physics "medium".

### 종지 (76084) 宗旨 — remove
- krdict: "간장이나 고추장 등을 담아 상에 놓는 작은 그릇." / "…종지에 담아 그 분량을 세는 단위."
- stdict native 종지: "간장·고추장 따위를 담아서 상에 놓는, 종발보다 작은 그릇."; 우리말샘 고유어.
- stdict 종지(宗旨): "종문(宗門)의 교의(敎義)의 취지." / "주장이 되는 요지나 근본이 되는 중요한 뜻."

### 방정 (58132) 方正 — remove
- krdict: "점잖고 조심스럽게 하지 못하고 가볍고 생각없이 하는 말이나 행동." (rash act; cf. 방정을 떨다)
- stdict native 방정: "찬찬하지 못하고 몹시 가볍고 점잖지 못하게 하는 말이나 행동."; 우리말샘 고유어.
- stdict 방정(方正): "‘방정하다’의 어근." — 품행이 바르다, the opposite meaning.

### 위통 (92365) 胃痛 — remove
- krdict: "→ 웃통" (a cross-reference to 웃통, upper body / top garment).
- stdict native 위통: "→ 웃통." / "물건의 위가 되는 부분."; 우리말샘 고유어 ("몸에서 허리 위의 부분. ⇒규범 표기는 ‘웃통’").
- stdict 위통(胃痛): "…위의 통증." — stomachache, a different word.

### 제철 (93735) 製鐵 — remove
- krdict: "알맞은 시기나 때." (right time; 제철 과일)
- stdict native 제철: "알맞은 시절." (제 + 철 'season'); 우리말샘 고유어.
- stdict 제철(製鐵): "철광석을 제련하여 철을 뽑아내는 일." — iron making.

### 어음 (67051) 語音 — remove
- krdict: "일정한 금액을 일정한 날짜와 장소에서 지불할 것을 약속하거나 다른 사람에게 그 지급을 맡기는 유가 증권." (note; bill)
- stdict native 어음: "일정한 금액을 일정한 날짜와 장소에서 치를 것을 약속하거나 제삼자에게 그 지급을 위탁하는 유가 증권."; 우리말샘 고유어.
- stdict 어음(語音): "말의 소리." — phonetics term.

### 비지 (60764) 鼻紙 — remove
- krdict: "두부를 만들고 남은 콩 찌꺼기." / "콩을 불려 갈아서 채소와 고기 등을 넣고 끓인 음식."
- stdict native 비지: "두부를 만들고 남은 찌꺼기." / "콩을 불려 갈아서 끓인 음식."; 우리말샘 고유어.
- stdict 비지(鼻紙): "코를 풀거나 닦는 데에 사용하는 종이."

### 불길 (60545) 不吉 — remove
- The core hanja is spelled with the compatibility ideograph U+F967 (不), as in kengdic; the fix row matches that exact spelling.
- krdict: "세차게 타오르는 불의 줄기." / "(비유적으로) 세차게 일어나는 감정이나 정열." / "(비유적으로) 세찬 기세로 퍼져 나가는 사회적 현상." (flame; blaze)
- stdict native 불길: "세차게 타오르는 불꽃." / "세차게 일어나는 감정이나 정열을 비유적으로 이르는 말." (불 + 길); 우리말샘 고유어.
- stdict 불길(不吉): "‘불길하다’의 어근." — ominous, a different word.

### 멱 (55554) 蓂 — remove
- krdict: "목의 앞쪽." (throat; 멱살)
- stdict native 멱: "목의 앞쪽." (identical); 우리말샘 고유어.
- No stdict 멱 with 蓂 (蓂 is read 명, 명협 'calendar plant'); the only hanja 멱 is 冪 (mathematical power).

### 곽 (21807) 矡 — remove
- krdict: "→ 갑 2" (cross-reference to 갑 'small box').
- stdict native 곽: "→ 갑03."; 우리말샘 곽 고유어 "물건을 담는 작은 상자. ⇒규범 표기는 ‘갑’이다." The standard 갑 is 匣,
  but 匣 reads 갑, not 곽, and no NIKL dictionary gives 곽 a 원어, so no replacement hanja.
- No stdict 곽 with 矡 (a rare character for a large hoe); the hanja 곽 entries are 郭 (surname) and 槨 (outer coffin).

### 흉 (87835) 匈 — remove
- krdict: "상처가 아물고 남은 자국." / "남에게 비웃음을 살 만한 말이나 행동." (scar; fault)
- stdict native 흉: "상처가 아물고 남은 자국." / "남에게 비웃음을 살 만한 거리."; 우리말샘 고유어.
- No stdict 흉 with 匈; the hanja 흉 entry is 凶 ("‘흉하다’의 어근"), a different word (흉하다 'ugly/ominous').

### 동생 (70748) 同生 — keep
- krdict: "형제나 친척 형제들 중에서 나이가 적은 사람. 또는 그 사람을 부르는 말." / "나이가 더 많은 사람이 나이가 어린 사람을 친숙하게 이르거나 부르는 말." (younger sibling)
- stdict / 우리말샘 file the kinship 동생 as 고유어 ("같은 부모에게서 태어난 사이거나… 손윗사람이 손아랫사람을 이르거나 부르는 말"),
  and 동생(同生) separately as "함께 삶." / "함께 남."
- Kept: the kinship word is the Sino-Korean 同生 'born of the same (parents)', a well-known etymology (older 同生 =
  siblings, narrowed to *younger* sibling); NIKL lists it as native only because the meaning drifted from the
  characters. The hanja is genuinely the word's origin and useful to a learner, so it stays, and the entry is on
  `pipeline/data/hanja_keep.tsv` so the scan does not re-flag it.

## Follow-up: the kengdic fallback is guarded (2026-11-24)

`hanja_fill::fallback_backed` now gates the fallback: when stdict has the word, its entry with kengdic's hanja must
be the best match of the krdict definitions (and score >= 0.3 when a native homograph exists); the keep list
overrides it. Kept fills are marked `"hanja_src": "kengdic"`; every verdict is written to
`out/hanja_kengdic_fallback.tsv`. Local build: 15 of 42 fills kept (대결산, 대란, 대업, 대칭, 동생 [keep list], 동전,
무책임, 범죄성, 수상작, 자료실, 첨삭, 팔각형, 풍취, 필독, 홍수), 27 dropped: the 13 above plus 감투 敢鬪, 고명 高名,
광대 廣大, 농어 農魚, 대목 大木, 막대 莫大, 신명 神明, 자라 紫羅, 활 佸, 황소 黃소 (native words), 외상값 外上값, 주춧돌 柱춧돌
(stdict gives the compound no 원어), 대망 大望 and 절정기 絶頂期 (the stdict fill then gives 待望 / 絕頂期).
The rows in `hanja_fixes.tsv` therefore now apply to 0 entries; they stay as a guard should the fallback change.
