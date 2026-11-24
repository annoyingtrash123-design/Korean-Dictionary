# Hanja fill sample: independent review

Reviewer: independent pass, 2026-11-24, over `pipeline/qa/hanja_fill_sample.tsv` (150 rows). Question per row: is the
filled hanja right for this word **in the meaning the entry gives** (not a homograph, not a native word given hanja)?
Checked against `pipeline/out/stdict.sqlite` and `core.sqlite` (read-only) where in doubt.

## Totals

| rule | rows | wrong | rate | doubtful |
|---|---|---|---|---|
| kengdic | 114 | 3 | 2.6% | 1 |
| derived | 30 | 3 | 10.0% | 0 |
| homograph | 4 | 0 | 0.0% | 0 |
| unique | 2 | 0 | 0.0% | 0 |
| **all** | 150 | 6 | 4.0% | 1 |

Counting the doubtful 조보 as wrong: kengdic 4/114 = 3.5%, overall 7/150 = 4.7%.

## Systematic issues

1. **kengdic rule: stdict does not list every transparent Sino-Korean compound.** The rule assumes that when stdict has exactly
   one entry with hanja (and no native homograph), the kengdic word is that entry. It fails when the kengdic word is a
   productive compound of a *homographic* stem + a suffix/element that stdict happens not to list, while stdict does list a rarer
   word of the same sound: 회의자 (懷疑者 sceptic vs stdict 會意字), 사례사 (事例史 case history vs stdict 謝禮辭), 쌍각 (雙殼
   bivalve vs stdict 雙脚), 재분 (再分 vs stdict 才分). Precise condition: 3/114 in the sample, all with a first element
   (회의, 사례, 쌍/재) that has several hanja homographs in stdict. A possible guard: when the headword splits as
   (stdict word with 2+ hanja homographs) + one more syllable, or (one-syllable Sino-Korean prefix with several readings) + rest,
   and the full word is a rare stdict entry, require something more (e.g. an English gloss word shared with krdict/wikt glosses of
   the chosen stdict sense's components) or skip. Recall cost should be small; these compounds are rare.
2. **derived rule trusts the core stem's hanja, and stdict's native entries do not overrule it.** Two of three derived errors
   come from stems whose hanja is already wrong in core from krdict (타박 打撲, 구경 究竟; not fills), one from a wrong kengdic
   fill (재분 才分). Precise failure points in `decide_derived` / `apply` (pipeline/src/hanja_fill.rs):
   - `Stem::unclear` is computed from core entries only. stdict's native 구경¹ (흥미나 관심을 가지고 봄 — the krdict definition
     verbatim) and native 타박¹ are never consulted, so the stem looks unambiguous.
   - For 타박하다 `own` holds a confirming 打撲하다 **and** a native 타박하다¹ of the same POS. The code then checks
     `defs_score(t.defs, stem.defs)`, i.e. the target against the (mis-hanja'd) krdict stem, which of course agrees. It should
     instead decide between the confirming and the native stdict entries by the target's own definitions (as the homograph
     rule does): the target matches native 타박하다¹ almost verbatim and 打撲하다 (때리고 치다) not at all.
   - With `own` empty (구경시키다, 재분하다) the only check is target vs stem (Korean defs or shared English words), which cannot
     catch a stem whose hanja is itself wrong (재분하다/재분 share 'subdi-').
   Suggested guard: before using a stem, compare the stem's core definitions with stdict's entries of the stem spelling; if a
   native stdict entry scores best (or the entry carrying the stem's hanja scores < 0.3), treat the stem as unclear. A quick
   scan of core with that test (krdict entries with hanja whose best stdict match is native, score >= 0.6, while the stdict entry
   with that hanja scores < 0.3) flags 16 krdict stems, e.g. 타박 打撲, 구경 究竟, 매질 媒質, 종지 宗旨, 방정 方正, 위통 胃痛,
   제철 製鐵, 어음 語音, 비지 鼻紙, 불길 不吉, 멱 蓂 (some are true homograph merges, but most look like pre-existing wrong
   hanja in core worth a separate look). Derived fills on kengdic-filled stems inherit any kengdic error (재분하다).
3. Not errors, but worth knowing: kengdic glosses are noisy (조보 'canteen', 노쇠자 'impotent', 교사죄 'barratry'); the kengdic rule cannot see this and attaches the only stdict hanja regardless. krdict unique/homograph rows (6)
   were all correct.

## Per row

| # | headword | hanja | source | rule | entry gloss/definition | verdict |
|---|---|---|---|---|---|---|
| 1 | 가사 | 歌辭 | krdict | homograph | 고려 말기에서 조선 초기에 나타난, 시가와 산문의 중간 형태의 문학. | ok |
| 2 | 회수 | 回數 | krdict | homograph | → 횟수 | ok (회수 → 횟수 回數 pointer entry) |
| 3 | 동선 | 動線 | krdict | homograph | 어떤 일을 할 때 사람이 움직이는 거리나 방향을 나타내는 선. | ok |
| 4 | 만 | 萬 | krdict | homograph | 천의 열 배가 되는 수. | ok |
| 5 | 장해물 | 障害物 | krdict | unique | 하려고 하는 일을 하지 못하게 막아서 방해하는 일이나 물건. | ok |
| 6 | 범죄율 | 犯罪率 | krdict | unique | 한 나라나 지역에서 일정 기간 동안 범죄가 일어난 비율. | ok |
| 7 | 즉시불 | 卽時拂 | kengdic | kengdic | prompt cash, immediate cash | ok |
| 8 | 연소관 | 燃燒管 | kengdic | kengdic | combustion tube | ok |
| 9 | 척주 | 脊柱 | kengdic | kengdic | the spinal column | ok |
| 10 | 담홍색 | 淡紅色 | kengdic | kengdic | carnation color, damask rose, rose-color | ok |
| 11 | 모병 | 募兵 | kengdic | kengdic | conscription | ok (gloss 'conscription' is loose; same word) |
| 12 | 증수회 | 贈收賄 | kengdic | kengdic | bribery | ok |
| 13 | 대부호 | 大富豪 | kengdic | kengdic | nabob; megamillionaire | ok |
| 14 | 노쇠자 | 老衰者 | kengdic | kengdic | impotent | ok (gloss 'impotent' is noise; same word) |
| 15 | 혈관종 | 血管腫 | kengdic | kengdic | angioma | ok |
| 16 | 음악학 | 音樂學 | kengdic | kengdic | musicology | ok |
| 17 | 보병대 | 步兵隊 | kengdic | kengdic | cohort | ok |
| 18 | 포도즙 | 葡萄汁 | kengdic | kengdic | stum | ok |
| 19 | 농혈증 | 膿血症 | kengdic | kengdic | pyaemia | ok |
| 20 | 강자성 | 強磁性 | kengdic | kengdic | ferromagnetism | ok (強 is stdict's form of 强) |
| 21 | 관념학 | 觀念學 | kengdic | kengdic | ideology | ok (stdict's definition is really 관념론's, but the hanja is right) |
| 22 | 노경 | 老境 | kengdic | kengdic | senescence | ok |
| 23 | 방수포 | 防水布 | kengdic | kengdic | oilskin, tarpaulin | ok |
| 24 | 연해지 | 沿海地 | kengdic | kengdic | littoral | ok |
| 25 | 유아론 | 唯我論 | kengdic | kengdic | solipsism | ok |
| 26 | 고속도강 | 高速度鋼 | kengdic | kengdic | highspeed steel | ok |
| 27 | 박모 | 薄暮 | kengdic | kengdic | gloam | ok |
| 28 | 난소염 | 卵巢炎 | kengdic | kengdic | oophoritis; ovaritis | ok |
| 29 | 편성표 | 編成表 | kengdic | kengdic | table of organization | ok |
| 30 | 선전자 | 宣傳者 | kengdic | kengdic | disseminator, propagandist | ok |
| 31 | 준평원 | 準平原 | kengdic | kengdic | peneplain | ok |
| 32 | 새각류 | 鰓脚類 | kengdic | kengdic | daphnia | ok |
| 33 | 잔유 | 殘油 | kengdic | kengdic | residual oil | ok |
| 34 | 흉골 | 胸骨 | kengdic | kengdic | sternum | ok |
| 35 | 육운 | 陸運 | kengdic | kengdic | landcarriage | ok |
| 36 | 천자문 | 千字文 | kengdic | kengdic | the thousand character chinese text | ok |
| 37 | 분망 | 奔忙 | kengdic | kengdic | rush | ok |
| 38 | 궤도차 | 軌道車 | kengdic | kengdic | a car, railcar | ok |
| 39 | 자연신 | 自然神 | kengdic | kengdic | nature deity | ok |
| 40 | 성유법 | 聲諭法 | kengdic | kengdic | onomatopoeia | ok (stdict spells 聲諭法; the usual character is 喩 as in 比喩 — a stdict-data quirk, faithfully copied) |
| 41 | 치마분 | 齒磨粉 | kengdic | kengdic | dentifrice | ok |
| 42 | 조보 | 朝報 | kengdic | kengdic | canteen | doubtful: kengdic gloss 'canteen' matches no 조보 in any dictionary (gloss noise); 朝報 is the only real 조보, but it now sits next to 'canteen'. Not counted as a fill error; the entry itself is bad |
| 43 | 원사체 | 原絲體 | kengdic | kengdic | protonema | ok |
| 44 | 교전자 | 交戰者 | kengdic | kengdic | belligerent | ok |
| 45 | 불제 | 祓除 | kengdic | kengdic | lustral | ok |
| 46 | 아마인유 | 亞麻仁油 | kengdic | kengdic | linseed oil | ok |
| 47 | 규방 | 閨房 | kengdic | kengdic | gynaeceum | ok |
| 48 | 약맥 | 弱脈 | kengdic | kengdic | acrotism | ok |
| 49 | 공상가 | 空想家 | kengdic | kengdic | daydreamer, dreamer, fancier, idealist, ideologist, visio... | ok |
| 50 | 핵융합로 | 核融合爐 | kengdic | kengdic | fusion reactor | ok |
| 51 | 전엽체 | 前葉體 | kengdic | kengdic | prothallium | ok |
| 52 | 총대리점 | 總代理店 | kengdic | kengdic | sole; universal agent | ok |
| 53 | 대리권 | 代理權 | kengdic | kengdic | agency; attorneyship | ok |
| 54 | 접촉기 | 接觸器 | kengdic | kengdic | contact maker; contactor | ok |
| 55 | 단접 | 鍛接 | kengdic | kengdic | hammer welding | ok |
| 56 | 연접봉 | 連接棒 | kengdic | kengdic | connecting rod | ok |
| 57 | 광도계 | 光度計 | kengdic | kengdic | photometer | ok |
| 58 | 청산염 | 靑酸鹽 | kengdic | kengdic | cyanide, prussiate | ok |
| 59 | 결의론 | 決疑論 | kengdic | kengdic | casuistry | ok |
| 60 | 빙퇴석 | 氷堆石 | kengdic | kengdic | moraine | ok |
| 61 | 동물계 | 動物界 | kengdic | kengdic | animal kingdom | ok |
| 62 | 필사생 | 筆寫生 | kengdic | kengdic | transcriber | ok |
| 63 | 방사속 | 放射束 | kengdic | kengdic | radiant flux | ok |
| 64 | 호색가 | 好色家 | kengdic | kengdic | lech, sensualist, whoremaster, whoremonger | ok |
| 65 | 취득물 | 取得物 | kengdic | kengdic | acquisition | ok |
| 66 | 교환수 | 交換手 | kengdic | kengdic | operator | ok |
| 67 | 지하경 | 地下莖 | kengdic | kengdic | rhizome | ok |
| 68 | 배타심 | 排他心 | kengdic | kengdic | cliquism | ok |
| 69 | 수양액 | 水樣液 | kengdic | kengdic | eyewater | ok |
| 70 | 반대투표 | 反對投票 | kengdic | kengdic | non placet | ok |
| 71 | 쇠미 | 衰微 | kengdic | kengdic | decadence, declination, fall, wane | ok |
| 72 | 타율성 | 他律性 | kengdic | kengdic | heteronomy | ok |
| 73 | 기증본 | 寄贈本 | kengdic | kengdic | a giftbook | ok |
| 74 | 관함식 | 觀艦式 | kengdic | kengdic | review | ok |
| 75 | 포자낭군 | 胞子囊群 | kengdic | kengdic | sorus | ok |
| 76 | 쌍각 | 雙脚 | kengdic | kengdic | bivlave | WRONG: gloss 'bivalve' is 쌍각 雙殼 (two shells, cf. 쌍각류 雙殼類); stdict has only 雙脚 'two legs', so the single stdict entry is a different word |
| 77 | 친수성 | 親水性 | kengdic | kengdic | hydrophilicity | ok |
| 78 | 평저선 | 平底船 | kengdic | kengdic | bateau | ok |
| 79 | 여성운 | 女性韻 | kengdic | kengdic | feminine rhyme | ok |
| 80 | 구빈세 | 救貧稅 | kengdic | kengdic | poor rate | ok |
| 81 | 부분할 | 部分割 | kengdic | kengdic | meroblast | ok |
| 82 | 적동광 | 赤銅鑛 | kengdic | kengdic | cuprite; copper red | ok |
| 83 | 실생 | 實生 | kengdic | kengdic | seedling | ok |
| 84 | 어휘론 | 語彙論 | kengdic | kengdic | lexis | ok |
| 85 | 십자형 | 十字形 | kengdic | kengdic | crisscross, in a cross shape | ok |
| 86 | 보양지 | 保養地 | kengdic | kengdic | health resort | ok |
| 87 | 국제어 | 國際語 | kengdic | kengdic | interlingua; world language | ok |
| 88 | 부절제 | 不節制 | kengdic | kengdic | excess | ok |
| 89 | 동숙인 | 同宿人 | kengdic | kengdic | roommate | ok |
| 90 | 교사죄 | 敎唆罪 | kengdic | kengdic | barratry | ok (gloss 'barratry' is loose; only one 교사죄) |
| 91 | 목욕통 | 沐浴桶 | kengdic | kengdic | tub | ok |
| 92 | 회의자 | 會意字 | kengdic | kengdic | unbeliever | WRONG: gloss 'unbeliever' is 회의자 懷疑者 (sceptic, 懷疑+者); stdict has only 會意字 (a 회의-type character) |
| 93 | 최면제 | 催眠劑 | kengdic | kengdic | hypnotic | ok |
| 94 | 흑화 | 黑化 | kengdic | kengdic | melanism | ok |
| 95 | 무용실 | 舞踊室 | kengdic | kengdic | dancing room | ok |
| 96 | 전환법 | 轉換法 | kengdic | kengdic | transposition | ok (transposition ~ 轉換; only one 전환법) |
| 97 | 저항자 | 抵抗者 | kengdic | kengdic | sb who is being resistant, a resister | ok |
| 98 | 수포진 | 水疱疹 | kengdic | kengdic | tetter | ok |
| 99 | 동시주의 | 同時主義 | kengdic | kengdic | synchronism | ok |
| 100 | 금어초 | 金魚草 | kengdic | kengdic | antirrhinum; snapdragon | ok |
| 101 | 흡각 | 吸角 | kengdic | kengdic | cupping glass; cup | ok (吸角 = cupping glass) |
| 102 | 다신론 | 多神論 | kengdic | kengdic | polytheism | ok |
| 103 | 군표 | 軍票 | kengdic | kengdic | a military payment certificate | ok |
| 104 | 간이복 | 簡易服 | kengdic | kengdic | sportswear | ok |
| 105 | 자오의 | 子午儀 | kengdic | kengdic | transit instrument | ok |
| 106 | 핵이성 | 核異性 | kengdic | kengdic | isomerism | ok |
| 107 | 탄력계 | 彈力計 | kengdic | kengdic | elastometer | ok |
| 108 | 물품세 | 物品稅 | kengdic | kengdic | commodity tax, excise tax, sales tax | ok |
| 109 | 이극광 | 異極鑛 | kengdic | kengdic | calamine, hemimorphite, smithsonite, minerals | ok |
| 110 | 세탁업자 | 洗濯業者 | kengdic | kengdic | laundryman; washerman | ok |
| 111 | 규암 | 硅巖 | kengdic | kengdic | quartzite | ok |
| 112 | 소극주의 | 消極主義 | kengdic | kengdic | passivism | ok |
| 113 | 가황 | 加黃 | kengdic | kengdic | cure | ok |
| 114 | 고전어 | 古典語 | kengdic | kengdic | classic | ok |
| 115 | 해도실 | 海圖室 | kengdic | kengdic | chart house (room) | ok |
| 116 | 대서인 | 代書人 | kengdic | kengdic | scrivener | ok |
| 117 | 박하유 | 薄荷油 | kengdic | kengdic | pennyroyal | ok |
| 118 | 맥파 | 脈波 | kengdic | kengdic | a pulse | ok |
| 119 | 양조법 | 釀造法 | kengdic | kengdic | zymotechnics | ok |
| 120 | 사례사 | 謝禮辭 | kengdic | kengdic | case history | WRONG: gloss 'case history' is 사례사 事例史 (事例+史); stdict has only 謝禮辭 (words of thanks) |
| 121 | 타박하다 | 打撲하다 | krdict | derived | 다른 사람의 실수나 결함을 나무라거나 핀잔하다. | WRONG: the krdict sense (나무라거나 핀잔하다) is the native 타박하다¹ of stdict, not 打撲하다 (때리고 치다). The stem 타박 already carries a wrong krdict hanja 打撲 and the derived rule copied it; stdict's native 타박하다¹ has the same definition nearly verbatim |
| 122 | 구경시키다 | 究竟시키다 | krdict | derived | 흥미나 관심을 가지고 볼 만한 것을 보여 주다. | WRONG: 구경 'sightseeing' is native (stdict 구경¹, no hanja); 究竟 means 'the very end / ultimate enlightenment'. The stem 구경 carries a wrong krdict hanja 究竟 and the derived rule propagated it |
| 123 | 월등히 | 越等히 | krdict | derived | 수준이 다른 것이나 보통보다 훨씬 뛰어나게. | ok |
| 124 | 무진히 | 無盡히 | krdict | derived | 끝이 없을 만큼 매우. | ok |
| 125 | 혼작하다 | 混作하다 | kengdic | derived | doublecrop | ok |
| 126 | 면역시키다 | 免疫시키다 | kengdic | derived | immunize | ok |
| 127 | 증식시키다 | 增殖시키다 | kengdic | derived | proliferate | ok |
| 128 | 골화하다 | 骨化하다 | kengdic | derived | ossify | ok |
| 129 | 낙뢰하다 | 落雷하다 | kengdic | derived | thunderstrike | ok |
| 130 | 혼류하다 | 混流하다 | kengdic | derived | interflow | ok |
| 131 | 재무장시키다 | 再武裝시키다 | kengdic | derived | rearm | ok |
| 132 | 유통시키다 | 流通시키다 | kengdic | derived | to circulate | ok |
| 133 | 자활하다 | 自活하다 | kengdic | derived | maintain | ok |
| 134 | 차폐하다 | 遮蔽하다 | kengdic | derived | mask | ok |
| 135 | 대리시키다 | 代理시키다 | kengdic | derived | substitute | ok |
| 136 | 반향시키다 | 反響시키다 | kengdic | derived | rebound, reverberate, revoice | ok |
| 137 | 결빙시키다 | 結氷시키다 | kengdic | derived | freeze | ok |
| 138 | 오해시키다 | 誤解시키다 | kengdic | derived | to create a misunderstanding | ok |
| 139 | 재분하다 | 才分하다 | kengdic | derived | subdivide | WRONG: gloss 'subdivide' is 再分하다; 才分 is a noun 'measure of talent' with no 하다 verb. Cascade: kengdic 재분 ('subdivision') was first filled 才分 by the kengdic rule (also wrong), then 재분하다 derived from it |
| 140 | 퇴폐시키다 | 頹廢시키다 | kengdic | derived | debauch, demoralize | ok |
| 141 | 몰락시키다 | 沒落시키다 | kengdic | derived | ruin | ok |
| 142 | 패소시키다 | 敗訴시키다 | kengdic | derived | case | ok (gloss 'case' is noise) |
| 143 | 과열시키다 | 過熱시키다 | kengdic | derived | to overheat | ok |
| 144 | 흑화하다 | 黑化하다 | kengdic | derived | melanize | ok |
| 145 | 유증하다 | 遺贈하다 | kengdic | derived | bequeath, devise, will | ok |
| 146 | 전입되다 | 轉入되다 | kengdic | derived | transfer | ok |
| 147 | 부영양화되다 | 富營養化되다 | kengdic | derived | eutrophicate | ok |
| 148 | 오보하다 | 誤報하다 | kengdic | derived | misinform | ok |
| 149 | 호색하다 | 好色하다 | kengdic | derived | lech | ok |
| 150 | 중립시키다 | 中立시키다 | kengdic | derived | neutralize | ok |

## Re-review (tightened rules) — 2026-11-24

Reviewer: independent pass over the fresh sample `pipeline/qa/hanja_fill_sample.tsv` (seed 2, 150 rows), produced by the
tightened rules (kengdic fills need 우리말샘/CC-CEDICT English corroboration; derived fills are vetoed by a stdict native
entry of the same POS and by `stem_contradicted`). Same question per row: is the hanja right for this word in the meaning
the entry gives? Doubtful rows were checked against `pipeline/out/{stdict,opendict,core}.sqlite` (read-only).

### Totals

| rule | rows | wrong | rate | doubtful |
|---|---|---|---|---|
| homograph | 4 | 0 | 0.0% | 0 |
| unique | 2 | 0 | 0.0% | 0 |
| kengdic | 84 | 1 | 1.2% | 0 |
| derived | 60 | 0 | 0.0% | 0 |
| **all** | 150 | **1** | **0.7%** | 0 |

Previous sample: 6 wrong + 1 doubtful / 150 (4.0%). All three failure classes named above are gone from this sample
(no wrong derived row; the stem-contradiction guard and the native-veto would have stopped 타박하다/구경시키다/재분하다).

### Wrong / noted rows

| # | headword | filled | rule | gloss | verdict |
|---|---|---|---|---|---|
| 53 | 추적자 | 追跡子 | kengdic (opendict) | a pursuer, a tracker | **WRONG**: the gloss is the person 追跡者 (追跡+者). stdict only lists 追跡子 'tracer' (a substance used to follow a reaction, 화학). The corroboration passed on the shared English root *track-* (tracker vs tracer/tracking), which does not tell the person from the substance. Same class as 회의자 (懷疑者 vs stdict 會意字) in the previous review: a productive X+者 agent noun whose stdict homograph ends in 子/字. |
| 74 | 야경증 | 夜警症 | kengdic (opendict) | night terror | ok (note): copied faithfully from stdict, which writes 夜警症. The etymologically expected form is 夜驚症 ('startle'); not a fill error, a stdict quirk. Not counted. |
| 68 | 공액 | 共軶 | kengdic (opendict) | conjugate | ok: stdict's own form (軶 = variant of 軛). |
| 83 | 견인기 | 牽引機 | kengdic (opendict) | a retractor | ok: stdict 牽引機 '어떤 물체를 끌어당기는 기구'; a surgical retractor is that kind of instrument. |

All other rows (4 homograph, 2 unique, 82 kengdic, 60 derived) are correct for their meaning. The kengdic glosses remain
noisy (조건문 'condition', 숙영하다 'lie', 퇴장시키다 'bench', 패소-type one-word glosses), but in every such row the
hanja is still the right word.

### Residual pattern: 者 vs 子/字

Not frequent in the sample (1/84), but it is the second time it shows up, and a scan of core shows where it can recur:
kengdic `…자` headwords with an agent-noun gloss whose only stdict entry ends in 子/字, e.g. 명령자 螟蛉子 'dictator',
반박자 半拍子 'gainsayer', 부정자 副正字 'naysayer', 수양자 收養子 'cultivator', 신입자 新粒子 'entrant', 운반자 運搬子
'conveyancer', 정리자 整理字 'regulator', 회의자 會意字 'unbeliever', 추적자 追跡子 'tracker' (about 20 such headwords in all
kengdic, most of which corroboration already rejects because the English does not overlap). A cheap guard closes it:
in `decide_kengdic`, skip when the headword ends in 자, the stdict hanja ends in 子 or 字, and the gloss is an agent noun
(-er/-or/-ist/-ant/-ent, 'one who'); add 추적자 to `pipeline/tests/hanja_fill.rs` as the regression case.

### 불만하다

stdict has 불만하다 不滿하다 (adjective, '마음에 흡족하지 아니하다', synonym 불만족하다), and the derived rule fills it
from stem 불만 不滿 with stdict confirmation. The hanja **should be filled**: 不滿하다 is right for the word whatever its
frequency. What is wrong is the kengdic gloss 'not a word. see 불평하다 (to complain)': stdict lists the word (it is rare;
불만스럽다 / 불만이다 are the usual forms), and 불평하다 'complain' is a different verb. That gloss is a gloss-fix item,
not a hanja problem.

### Recommendation

**Ship.** 1 wrong in 150 (0.7%) meets the bar (≤ 1/150). The one error belongs to a narrow, identifiable class (者 vs
子/字 agent nouns, about 20 candidate headwords in the whole set) rather than a broad failure of a rule; add the guard above
(small, before or right after shipping) so the class cannot recur, in line with "every recurring mistake becomes a check".
