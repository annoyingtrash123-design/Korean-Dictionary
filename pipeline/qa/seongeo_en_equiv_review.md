# Review: English equivalents of 성어 (`pipeline/data/seongeo_en_equiv.tsv`)

AI-proposed (draft). Nothing ships until a row's `review` column says `approved...`; shipped text is labelled AI in the app.

Roster: 350 성어 (seongeo300 + seongeo_learner, deduplicated). Proposed equivalents: 146. The other 204 are left empty on purpose: no established English idiom or proverb means the same.

## What to check for each row

1. The English is a real, established idiom or proverb (not a descriptive paraphrase), spelled and worded as native speakers say it.
2. It means the same as the 성어 (not just something related). Where it is close but not identical, the `note` column says how; reject if the gap is too wide.
3. Register: neutral or common. British-only items are flagged in the note (much of a muchness, do a moonlight flit).
4. Empty rows: confirm there is really no equivalent; if there is, add it.
5. Set `review` to `approved` (or `approved: no equivalent` for an empty row), or `rejected: reason`.

## Weakest proposals (look at these first)

- 새옹지마 -> a blessing in disguise: fits only half (luck and misfortune trade places); 전화위복 is the true match. Consider rejecting for 새옹지마.
- 인면수심 -> a wolf in sheep's clothing: the nuance differs (hiding vs. looking human).
- 천신만고 -> blood, sweat and tears; 주경야독 -> burn the midnight oil; 동병상련 -> in the same boat; 도원결의 -> blood brothers; 암중모색 -> a shot in the dark; 사필귀정 -> truth will out; 상전벽해 -> a sea change; 아전인수 -> have an axe to grind: loose matches.
- 풍전등화 and 백척간두 share 'hanging by a thread'; 막상막하 and 백중지세 share 'neck and neck'.

## Proposed rows

| 성어 | hanja | English equivalent | note |
|---|---|---|---|
| 심사숙고 | 深思熟考 | think long and hard |  |
| 배은망덕 | 背恩忘德 | bite the hand that feeds one |  |
| 남녀노소 | 男女老少 | young and old alike |  |
| 설상가상 | 雪上加霜 | when it rains, it pours | a second misfortune piles on; no one is causing it on purpose |
| 자포자기 | 自暴自棄 | throw in the towel |  |
| 승승장구 | 乘勝長驅 | go from strength to strength |  |
| 우유부단 | 優柔不斷 | sit on the fence |  |
| 전전긍긍 | 戰戰兢兢 | on tenterhooks |  |
| 십중팔구 | 十中八九 | nine times out of ten |  |
| 무궁무진 | 無窮無盡 | the sky's the limit |  |
| 공명정대 | 公明正大 | fair and square |  |
| 약육강식 | 弱肉強食 | law of the jungle |  |
| 개과천선 | 改過遷善 | turn over a new leaf | 개과천선 is repenting and reforming one's character, a stronger change than a fresh start |
| 유명무실 | 有名無實 | in name only |  |
| 속수무책 | 束手無策 | one's hands are tied |  |
| 기진맥진 | 氣盡脈盡 | dead on one's feet |  |
| 임기응변 | 臨機應變 | think on one's feet |  |
| 자업자득 | 自業自得 | you've made your bed, now lie in it |  |
| 막상막하 | 莫上莫下 | neck and neck |  |
| 야반도주 | 夜半逃走 | do a moonlight flit | British |
| 동상이몽 | 同床異夢 | not on the same page | the English is milder; 동상이몽 stresses that each side privately wants something different |
| 산전수전 | 山戰水戰 | been through the mill | 'seen it all' is also used |
| 사생결단 | 死生決斷 | do or die |  |
| 허심탄회 | 虛心坦懷 | have a heart-to-heart |  |
| 우왕좌왕 | 右往左往 | run around like a headless chicken |  |
| 의기투합 | 意氣投合 | hit it off |  |
| 권선징악 | 勸善懲惡 | poetic justice |  |
| 동고동락 | 同苦同樂 | through thick and thin |  |
| 솔선수범 | 率先垂範 | lead by example |  |
| 일사천리 | 一瀉千里 | smooth sailing | also 'without a hitch'; the Korean image is a river rushing a thousand 리 |
| 대동소이 | 大同小異 | much of a muchness | British; means there is little to choose between them |
| 전광석화 | 電光石火 | quick as a flash |  |
| 백년해로 | 百年偕老 | till death do us part | the English is a vow; 백년해로 is the wish that a couple grow old together |
| 전화위복 | 轉禍爲福 | a blessing in disguise |  |
| 절체절명 | 絕體絕命 | in dire straits |  |
| 태연자약 | 泰然自若 | cool as a cucumber |  |
| 의기소침 | 意氣銷沈 | down in the dumps |  |
| 혼비백산 | 魂飛魄散 | scared out of one's wits |  |
| 구사일생 | 九死一生 | by the skin of one's teeth | the English describes the narrowness, the Korean the survival itself |
| 자화자찬 | 自畫自讚 | toot one's own horn |  |
| 금지옥엽 | 金枝玉葉 | the apple of one's eye | 금지옥엽 is said of a cherished child |
| 이구동성 | 異口同聲 | with one voice |  |
| 일맥상통 | 一脈相通 | along the same lines |  |
| 일취월장 | 日就月將 | by leaps and bounds |  |
| 진두지휘 | 陣頭指揮 | lead from the front |  |
| 오합지졸 | 烏合之卒 | motley crew | 오합지졸 is an undisciplined rabble of soldiers or followers |
| 요지부동 | 搖之不動 | won't budge an inch |  |
| 우후죽순 | 雨後竹筍 | spring up like mushrooms |  |
| 청천벽력 | 靑天霹靂 | a bolt from the blue |  |
| 파죽지세 | 破竹之勢 | sweep all before one |  |
| 천신만고 | 千辛萬苦 | blood, sweat and tears | the English is about effort; 천신만고 is about enduring great hardship |
| 아비규환 | 阿鼻叫喚 | all hell broke loose | 아비규환 describes a scene of screaming chaos |
| 불철주야 | 不撤晝夜 | around the clock |  |
| 새옹지마 | 塞翁之馬 | a blessing in disguise | fits only one half: 새옹지마 says luck and misfortune keep trading places and cannot be judged in advance; 전화위복 is the closer match |
| 수수방관 | 袖手傍觀 | sit on one's hands | also 'stand idly by' |
| 칠전팔기 | 七顚八起 | if at first you don't succeed, try, try again |  |
| 만사형통 | 萬事亨通 | everything's coming up roses |  |
| 확고부동 | 確固不動 | firm as a rock |  |
| 언감생심 | 焉敢生心 | wouldn't dream of it |  |
| 천하제일 | 天下第一 | second to none |  |
| 안하무인 | 眼下無人 | high and mighty |  |
| 동분서주 | 東奔西走 | run oneself ragged |  |
| 일심동체 | 一心同體 | two hearts that beat as one |  |
| 심기일전 | 心機一轉 | make a fresh start |  |
| 불구대천 | 不俱戴天 | sworn enemy |  |
| 십시일반 | 十匙一飯 | many hands make light work | 십시일반 is ten people each giving a spoonful of rice so that one can eat; the English is about sharing labour |
| 십년감수 | 十年減壽 | it took years off my life |  |
| 대성통곡 | 大聲痛哭 | cry one's eyes out |  |
| 이실직고 | 以實直告 | come clean |  |
| 거두절미 | 去頭截尾 | cut to the chase |  |
| 일진일퇴 | 一進一退 | a see-saw battle |  |
| 기고만장 | 氣高萬丈 | get too big for one's boots |  |
| 문전박대 | 門前薄待 | show someone the door |  |
| 일석이조 | 一石二鳥 | kill two birds with one stone |  |
| 청렴결백 | 淸廉潔白 | squeaky clean |  |
| 견원지간 | 犬猿之間 | like cats and dogs | also 'at daggers drawn' |
| 무릉도원 | 武陵桃源 | Shangri-La |  |
| 팔방미인 | 八方美人 | jack of all trades | 팔방미인 is praise for someone good at everything; 'jack of all trades' often adds 'master of none' |
| 백척간두 | 百尺竿頭 | hanging by a thread |  |
| 호의호식 | 好衣好食 | live in the lap of luxury |  |
| 부화뇌동 | 附和雷同 | jump on the bandwagon |  |
| 풍전등화 | 風前燈火 | hanging by a thread | the Korean image is a lamp flame in the wind |
| 천재일우 | 千載一遇 | chance of a lifetime |  |
| 방약무인 | 傍若無人 | act as if one owns the place |  |
| 지행합일 | 知行合一 | practice what you preach | 지행합일 is the idea that knowing and doing are one; the English is about consistency of word and deed |
| 살신성인 | 殺身成仁 | make the ultimate sacrifice |  |
| 망지소조 | 罔知所措 | not know which way to turn |  |
| 광명정대 | 光明正大 | above board |  |
| 대경실색 | 大驚失色 | go white as a sheet |  |
| 이심전심 | 以心傳心 | on the same wavelength |  |
| 대서특필 | 大書特筆 | make headlines |  |
| 백중지세 | 伯仲之勢 | neck and neck |  |
| 마이동풍 | 馬耳東風 | fall on deaf ears |  |
| 사면초가 | 四面楚歌 | with one's back against the wall |  |
| 급전직하 | 急轉直下 | take a nosedive |  |
| 만고불변 | 萬古不變 | stand the test of time |  |
| 어불성설 | 語不成說 | it doesn't hold water |  |
| 수원수구 | 誰怨誰咎 | only oneself to blame |  |
| 다다익선 | 多多益善 | the more the merrier |  |
| 천정부지 | 天井不知 | go through the roof | also 'sky-high', said of prices |
| 대기만성 | 大器晩成 | a late bloomer |  |
| 일벌백계 | 一罰百戒 | make an example of someone |  |
| 사필귀정 | 事必歸正 | truth will out | 사필귀정 says that what is right wins in the end; the English is about hidden truth coming out |
| 호사다마 | 好事多魔 | every rose has its thorn |  |
| 상전벽해 | 桑田碧海 | a sea change | 상전벽해 is a huge change in the world over time; the English is about a profound transformation |
| 일구이언 | 一口二言 | speak with a forked tongue |  |
| 각자도생 | 各自圖生 | every man for himself |  |
| 천군만마 | 千軍萬馬 | a tower of strength |  |
| 청출어람 | 靑出於藍 | the student has surpassed the master |  |
| 천편일률 | 千篇一律 | cookie-cutter | all the same, no variety |
| 화룡점정 | 畫龍點睛 | the finishing touch | 화룡점정 is the one final stroke that brings the whole thing to life (painting the dragon's eyes) |
| 일파만파 | 一波萬波 | ripple effect |  |
| 무사태평 | 無事太平 | without a care in the world |  |
| 금상첨화 | 錦上添花 | icing on the cake | also 'the cherry on top' |
| 도원결의 | 桃園結義 | blood brothers | 도원결의 is the oath of brotherhood sworn by three heroes in the Three Kingdoms; the English is the nearest idea |
| 부전자전 | 父傳子傳 | like father, like son |  |
| 복지부동 | 伏地不動 | keep one's head down | 복지부동 is mostly said of officials who avoid acting so as to avoid risk |
| 동병상련 | 同病相憐 | in the same boat | 동병상련 is fellow-feeling between people who suffer the same; the English only names the shared situation |
| 경이원지 | 敬而遠之 | keep at arm's length |  |
| 분골쇄신 | 粉骨碎身 | work one's fingers to the bone |  |
| 고육지책 | 苦肉之策 | desperate times call for desperate measures | the Korean names the measure itself, a painful last resort |
| 역지사지 | 易地思之 | put oneself in someone else's shoes |  |
| 인면수심 | 人面獸心 | a wolf in sheep's clothing | different nuance: 인면수심 is a cruel person who looks human, not one who hides his nature |
| 고진감래 | 苦盡甘來 | after a storm comes a calm | also 'every cloud has a silver lining' |
| 송구영신 | 送舊迎新 | ring out the old, ring in the new |  |
| 청산유수 | 靑山流水 | talk a blue streak | also 'have the gift of the gab' |
| 주경야독 | 晝耕夜讀 | burn the midnight oil | the English is working late; 주경야독 is working by day and studying by night |
| 용두사미 | 龍頭蛇尾 | start with a bang, end with a whimper |  |
| 사상누각 | 沙上樓閣 | a house built on sand |  |
| 초지일관 | 初志一貫 | stick to one's guns |  |
| 암중모색 | 暗中摸索 | a shot in the dark | 암중모색 is groping for a solution without clues, closer to 'feeling one's way in the dark' |
| 십인십색 | 十人十色 | different strokes for different folks |  |
| 결자해지 | 結者解之 | clean up one's own mess |  |
| 자승자박 | 自繩自縛 | hoist with one's own petard | literary; plain English is 'be caught in one's own trap' |
| 불문가지 | 不問可知 | it goes without saying |  |
| 아전인수 | 我田引水 | have an axe to grind | 아전인수 is bending the facts to one's own benefit |
| 후안무치 | 厚顔無恥 | as bold as brass |  |
| 좌불안석 | 坐不安席 | on pins and needles |  |
| 타산지석 | 他山之石 | learn from others' mistakes |  |
| 침소봉대 | 針小棒大 | make a mountain out of a molehill |  |
| 유비무환 | 有備無患 | better safe than sorry | also 'forewarned is forearmed' |
| 과유불급 | 過猶不及 | too much of a good thing |  |
| 만병통치 | 萬病通治 | a cure-all |  |
| 견물생심 | 見物生心 | opportunity makes the thief |  |
| 금전만능 | 金錢萬能 | money makes the world go round |  |
| 회자정리 | 會者定離 | all good things must come to an end |  |
