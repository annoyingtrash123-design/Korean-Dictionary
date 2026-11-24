# Review list: AI English for 속담 and 성어 (idioms)

Everything below is AI-written and **unreviewed** (all rows are `draft`). Nothing ships until an independent
reviewer (`content-review` skill) sets the row's `review` column to `approved...`. This file lists every AI field.

How to review: change `review` from `draft` to `approved:<who>/<date>` in the TSV (a row you reject or correct stays `draft`
with the fix applied). Check: meaning of the gloss and definition, the literal reading `(lit. ...)`, that each example is
natural Korean that really uses the idiom and that the English is faithful (nothing added or dropped, no invented gender).

## Part B: example translations (staging file)

`pipeline/data/example_en_idioms.tsv`: 1051 AI English translations (sentences and dialogues of krdict 속담/성어 in the study
lists `sokdam_all` and `seongeo_learner`), columns `ext_id, sense, example, ko, en, review`. Dialogue turns are joined by a
literal `\n`. All `draft`; the review column is the only thing to change (rows then merge into the core pack at build time with
the AI label). Every row is an AI field (`en`). Three krdict "examples" that are only a full stop were left alone.

## Part A: new entries (`pipeline/data/idiom_extra.tsv`)

264 items (246 성어, 18 속담).
AI fields per item: `en_gloss`, `en_def` (always); Korean example sentences marked **ai** (454 sentences); every English
translation of an example (532). Korean copied verbatim from stdict/우리말샘 (checked against the packs by `tests/idiom_extra.rs`):
the Korean definition and the 78 examples marked with the dictionary name.

Things to look at first: 독불장군 (literal reading), 환골탈태, 유만부동 (two senses), 운우지정, 노류장화, 창가소부, 삼종지도, 일부종사
(old gender norms, described as such), 출가외인, and the proverbs 새우 싸움에 고래 등 터지랴 / 꿈자리가 사납더니 / 무른 메주 밟듯 (North Korean).


### 1. 불가사의 (不可思議) [seongeo, stdict:152790, draft]

- en_gloss (AI): mystery; wonder
- en_def (AI): Something so strange that human thought cannot grasp or explain it; a mystery or a wonder (lit. cannot be thought or discussed).
- ko_def (stdict, verbatim): 사람의 생각으로는 미루어 헤아릴 수 없이 이상하고 야릇함.
- ex (AI Korean): 그 피라미드를 어떻게 지었는지는 지금도 불가사의로 남아 있다.
  - en (AI): How that pyramid was built still remains a mystery today.
- ex (AI Korean): 그가 그 사고에서 무사히 살아남은 것은 정말 불가사의한 일이다.
  - en (AI): It is truly baffling that he survived that accident unharmed.

### 2. 자급자족 (自給自足) [seongeo, stdict:269982, draft]

- en_gloss (AI): self-sufficiency
- en_def (AI): Producing for oneself what one needs, without depending on others (lit. supplying oneself, being sufficient to oneself).
- ko_def (stdict, verbatim): 필요한 물자를 스스로 생산하여 충당함.
- ex (AI Korean): 그 마을 사람들은 먹을거리를 자급자족하며 살아간다.
  - en (AI): The people of that village live self-sufficiently, producing their own food.
- ex (AI Korean): 요즘에는 전기까지 자급자족하는 집이 늘고 있다.
  - en (AI): These days, more and more households even generate their own electricity.

### 3. 고군분투 (孤軍奮鬪) [seongeo, stdict:21964, draft]

- en_gloss (AI): fighting a lone struggle
- en_def (AI): Struggling hard on one's own, without help, against heavy odds or a difficult task (lit. a lone army fighting fiercely).
- ko_def (stdict, verbatim): 따로 떨어져 도움을 받지 못하게 된 군사가 많은 수의 적군과 용감하게 잘 싸움.
- ex (AI Korean): 그는 팀원 없이 혼자 고군분투하며 프로젝트를 끝냈다.
  - en (AI): With no team members, he struggled on alone and finished the project.
- ex (AI Korean): 어려운 환경에서도 고군분투하는 아이들의 모습이 감동적이었다.
  - en (AI): The sight of the children battling on against the odds, even in such hard conditions, was moving.

### 4. 남녀노소 (男女老少) [seongeo, stdict:60333, draft]

- en_gloss (AI): men and women, young and old; everybody
- en_def (AI): People of every kind, regardless of sex or age (lit. man, woman, old, young).
- ko_def (stdict, verbatim): 남자와 여자, 늙은이와 젊은이란 뜻으로, 모든 사람을 이르는 말.
- ex (stdict, verbatim): 그는 남녀노소를 막론하고 좋아하는 가수이다.
  - en (AI): He is a singer loved by everyone, whatever their age or sex.
- ex (AI Korean): 이 공원은 남녀노소가 함께 즐길 수 있는 곳이다.
  - en (AI): This park is a place that people of all ages and both sexes can enjoy together.

### 5. 설상가상 (雪上加霜) [seongeo, stdict:448384, draft]

- en_gloss (AI): to make matters worse; misfortune upon misfortune
- en_def (AI): One misfortune or difficulty following hard on another (lit. adding frost on top of snow).
- ko_def (stdict, verbatim): 눈 위에 서리가 덮인다는 뜻으로, 난처한 일이나 불행한 일이 잇따라 일어남을 이르는 말.
- ex (stdict, verbatim): 시간도 없는데 설상가상으로 길까지 막혔다.
  - en (AI): I had no time, and to make matters worse, the road was blocked too.
- ex (stdict, verbatim): 눈보라가 몰아쳐 산을 오르기가 어려웠는데 설상가상으로 주위마저 어두워지기 시작하였다.
  - en (AI): A snowstorm made climbing the mountain difficult, and to make matters worse, it was starting to get dark all around.

### 6. 다재다능 (多才多能) [seongeo, stdict:74284, draft]

- en_gloss (AI): multi-talented; versatile
- en_def (AI): Having many talents and abilities (lit. many talents, many abilities).
- ko_def (stdict, verbatim): 재주와 능력이 여러 가지로 많음.
- ex (AI Korean): 그녀는 노래도 잘하고 그림도 잘 그리는 다재다능한 사람이다.
  - en (AI): She is a versatile person who sings well and draws well too.
- ex (AI Korean): 다재다능한 배우라서 어떤 역할을 맡겨도 잘 소화한다.
  - en (AI): He is such a versatile actor that he handles any role he is given.

### 7. 명실상부 (名實相符) [seongeo, stdict:116862, draft]

- en_gloss (AI): living up to one's name
- en_def (AI): The name and the reality match each other perfectly; being in fact what one is called (lit. name and substance fit together).
- ko_def (stdict, verbatim): 이름과 실상이 서로 꼭 맞음.
- ex (stdict, verbatim): 김 감독은 세 편의 영화를 연이어 성공시킨 데 이어 국제 영화제까지 휩쓸며 명실상부 거장의 자리에 올랐다.
  - en (AI): Director Kim had one hit film after another and then swept an international film festival, rising to the position of a truly great master in name and in fact.
- ex (AI Korean): 그 선수는 올해 모든 대회를 휩쓸며 명실상부 세계 최고가 되었다.
  - en (AI): That player swept every tournament this year and became the best in the world in both name and fact.

### 8. 십중팔구 (十中八九) [seongeo, stdict:452942, draft]

- en_gloss (AI): nine times out of ten; most likely
- en_def (AI): Almost certainly, or in the great majority of cases (lit. eight or nine out of ten).
- ko_def (stdict, verbatim): 열 가운데 여덟이나 아홉 정도로 거의 대부분이거나 거의 틀림없음.
- ex (stdict, verbatim): 그 말이 그가 한 말이라면 십중팔구 거짓말이다.
  - en (AI): If it's something he said, it's most likely a lie.
- ex (stdict, verbatim): 우리 학급의 학생들은 십중팔구가 아침을 거른다.
  - en (AI): Nine out of ten students in our class skip breakfast.
- ex (stdict, verbatim): 그 수다쟁이의 얘기는 십중팔구 뜬소문이다.
  - en (AI): That chatterbox's story is almost certainly a groundless rumor.

### 9. 일편단심 (一片丹心) [seongeo, stdict:473217, draft]

- en_gloss (AI): unwavering devotion
- en_def (AI): A sincere, unchanging heart that stays loyal to one person or cause (lit. one piece of red heart).
- ko_def (stdict, verbatim): 한 조각의 붉은 마음이라는 뜻으로, 진심에서 우러나오는 변치 아니하는 마음을 이르는 말.
- ex (stdict, verbatim): 신하는 임금을 일편단심으로 섬겨야 한다.
  - en (AI): A subject must serve the king with unwavering devotion.
- ex (AI Korean): 그는 평생 한 사람만을 일편단심으로 사랑했다.
  - en (AI): All his life he loved only one person, with unwavering devotion.

### 10. 전대미문 (前代未聞) [seongeo, stdict:285975, draft]

- en_gloss (AI): unheard-of; unprecedented
- en_def (AI): Never heard of before; without precedent (lit. not heard in any earlier age).
- ko_def (stdict, verbatim): 이제까지 들어 본 적이 없음.
- ex (AI Korean): 이번 폭우는 전대미문의 피해를 가져왔다.
  - en (AI): This heavy rain caused unprecedented damage.
- ex (AI Korean): 그 회사는 전대미문의 사기 사건으로 문을 닫았다.
  - en (AI): The company shut down because of an unheard-of fraud scandal.

### 11. 약육강식 (弱肉強食) [seongeo, stdict:456958, draft]

- en_gloss (AI): law of the jungle; the strong preying on the weak
- en_def (AI): The weak are devoured by the strong, who thrive at their expense (lit. the weak are meat, the strong eat).
- ko_def (stdict, verbatim): 약한 자가 강한 자에게 먹힌다는 뜻으로, 강한 자가 약한 자를 희생시켜서 번영하거나, 약한 자가 강한 자에게 끝내는 멸망됨을 이르는 말.
- ex (AI Korean): 자연계는 약육강식의 법칙이 지배한다.
  - en (AI): The law of the jungle rules the natural world.
- ex (AI Korean): 약육강식의 세상에서 살아남으려면 힘을 길러야 한다.
  - en (AI): To survive in a world where the strong prey on the weak, you have to build up your strength.

### 12. 천진난만 (天眞爛漫) [seongeo, stdict:320819, draft]

- en_gloss (AI): innocent and unaffected; childlike
- en_def (AI): Pure and natural, with nothing put on in word or deed (lit. heaven-given truth, brimming over).
- ko_def (stdict, verbatim): 말이나 행동에 아무런 꾸밈이 없이 그대로 나타날 만큼 순진하고 천진함.
- ex (AI Korean): 아이들은 천진난만한 얼굴로 눈 위를 뛰어다녔다.
  - en (AI): The children ran about on the snow with innocent, carefree faces.
- ex (AI Korean): 그의 천진난만한 웃음을 보면 누구나 마음이 풀린다.
  - en (AI): Anyone who sees his innocent laugh feels their heart soften.

### 13. 용감무쌍 (勇敢無雙) [seongeo, stdict:253584, draft]

- en_gloss (AI): dauntless; extremely brave
- en_def (AI): Brave, spirited and full of vigor, without equal (lit. courageous and bold, with no peer).
- ko_def (stdict, verbatim): 용기가 있으며 씩씩하고 기운차기 짝이 없음.
- ex (stdict, verbatim): 우리 부대는 이번 전쟁에서 용감무쌍의 전공을 세우고 돌아왔다.
  - en (AI): Our unit returned from this war having won distinction through dauntless bravery.
- ex (AI Korean): 용감무쌍한 소방관이 불길 속으로 뛰어들어 아이를 구했다.
  - en (AI): A dauntless firefighter rushed into the flames and saved the child.

### 14. 조강지처 (糟糠之妻) [seongeo, stdict:477986, draft]

- en_gloss (AI): the wife who shared one's hard times
- en_def (AI): A wife who went through poverty and hardship together with her husband (lit. wife of the dregs and chaff, i.e. of the days when they lived on them).
- ko_def (stdict, verbatim): 지게미와 쌀겨로 끼니를 이을 때의 아내라는 뜻으로, 몹시 가난하고 천할 때에 고생을 함께 겪어 온 아내를 이르는 말. ≪후한서≫의 <송홍전(宋弘傳)>에 나오는 말이다.
- ex (AI Korean): 그는 성공한 뒤에도 조강지처를 버리지 않았다.
  - en (AI): Even after he became successful, he did not abandon the wife who had stood by him in hard times.
- ex (AI Korean): 가난할 때 곁을 지켜 준 조강지처에게 늘 고마운 마음을 가지고 있다.
  - en (AI): He is always grateful to the wife who stayed by his side when he was poor.

### 15. 극악무도 (極惡無道) [seongeo, stdict:46935, draft]

- en_gloss (AI): utterly wicked; heinous
- en_def (AI): Extremely evil and completely contrary to human decency (lit. evil to the extreme, without the proper way).
- ko_def (stdict, verbatim): 더할 나위 없이 악하고 도리에 완전히 어긋나 있음.
- ex (AI Korean): 극악무도한 범인은 끝내 법의 심판을 받았다.
  - en (AI): The heinous criminal was finally brought to justice.
- ex (AI Korean): 그런 극악무도한 짓을 저지르고도 반성하지 않다니 믿을 수 없다.
  - en (AI): It is unbelievable that he committed such a vile act and shows no remorse.

### 16. 독불장군 (獨不將軍) [seongeo, stdict:85926, draft]

- en_gloss (AI): stubborn loner; someone who ignores others
- en_def (AI): A person who does everything their own way, alone (the saying is "No one is a general alone"; lit. alone, not a general).
- ko_def (stdict, verbatim): 무슨 일이든 자기 생각대로 혼자서 처리하는 사람.
- ex (stdict, verbatim): 그는 독불장군이라서 충고해 줘 봐야 소용없다.
  - en (AI): He is such a lone wolf that it's pointless to give him advice.
- ex (AI Korean): 독불장군처럼 굴면 아무도 너와 일하고 싶어 하지 않을 거야.
  - en (AI): If you act like a stubborn loner, nobody will want to work with you.

### 17. 불로불사 (不老不死) [seongeo, stdict:160613, draft]

- en_gloss (AI): eternal youth and immortality
- en_def (AI): Neither growing old nor dying (lit. not aging, not dying).
- ko_def (stdict, verbatim): 늙지도 아니하고 죽지도 아니함.
- ex (AI Korean): 진시황은 불로불사의 약을 찾으려고 사람들을 보냈다.
  - en (AI): The First Emperor of Qin sent people out to find a drug of immortality.
- ex (AI Korean): 불로불사를 꿈꾸는 사람들은 예나 지금이나 많다.
  - en (AI): There are many people, then as now, who dream of never aging or dying.

### 18. 흥망성쇠 (興亡盛衰) [seongeo, stdict:386511, draft]

- en_gloss (AI): rise and fall; ups and downs
- en_def (AI): The flourishing and perishing, prospering and declining, of a state, a family or any enterprise (lit. rise, fall, prosper, decline).
- ko_def (stdict, verbatim): 흥하고 망함과 성하고 쇠함.
- ex (stdict, verbatim): 모든 일에는 흥망성쇠가 있다.
  - en (AI): Everything has its rise and fall.
- ex (AI Korean): 이 책은 한 왕조의 흥망성쇠를 다루고 있다.
  - en (AI): This book deals with the rise and fall of a dynasty.

### 19. 다종다양 (多種多樣) [seongeo, stdict:74442, draft]

- en_gloss (AI): great diversity; wide variety
- en_def (AI): Many kinds, with many different forms (lit. many kinds, many shapes).
- ko_def (stdict, verbatim): 가짓수나 양식, 모양이 여러 가지로 많음.
- ex (AI Korean): 시장에는 다종다양한 과일이 진열되어 있었다.
  - en (AI): A wide variety of fruit was on display at the market.
- ex (AI Korean): 이 도시에는 다종다양한 문화가 어우러져 있다.
  - en (AI): A great many different cultures blend together in this city.

### 20. 의기투합 (意氣投合) [seongeo, stdict:464810, draft]

- en_gloss (AI): hitting it off; being of one mind
- en_def (AI): Two or more people finding that their hearts and aims agree (lit. spirits and wills fitting together).
- ko_def (stdict, verbatim): 마음이나 뜻이 서로 맞음.
- ex (AI Korean): 두 사람은 처음 만난 자리에서 바로 의기투합했다.
  - en (AI): The two of them hit it off the moment they met.
- ex (AI Korean): 우리는 의기투합하여 함께 가게를 열기로 했다.
  - en (AI): We saw eye to eye and decided to open a shop together.

### 21. 천차만별 (千差萬別) [seongeo, stdict:320904, draft]

- en_gloss (AI): as different as can be; endlessly varied
- en_def (AI): Things all differing from one another in countless ways (lit. a thousand differences, ten thousand distinctions).
- ko_def (stdict, verbatim): 여러 가지 사물이 모두 차이가 있고 구별이 있음.
- ex (stdict, verbatim): 요즘 상품은 색상부터 모양과 기능까지 천차만별이어서 어느 것을 사야 좋을지 모를 정도이다.
  - en (AI): Today's products vary so much, from color and shape to function, that I have no idea which one to buy.
- ex (AI Korean): 사람의 성격은 천차만별이라서 한마디로 말하기 어렵다.
  - en (AI): People's personalities differ so widely that it is hard to sum them up in a word.

### 22. 권선징악 (勸善懲惡) [seongeo, stdict:400466, draft]

- en_gloss (AI): rewarding good and punishing evil
- en_def (AI): Encouraging good deeds and punishing evil ones (lit. urge good, punish evil).
- ko_def (stdict, verbatim): 착한 일을 권장하고 악한 일을 징계함.
- ex (stdict, verbatim): 고대 소설의 주제는 권선징악이 대부분이다.
  - en (AI): Most classical novels have the rewarding of good and the punishing of evil as their theme.
- ex (stdict, verbatim): 흥부전은 권선징악을 주제로 한 대표적인 이야기이다.
  - en (AI): Heungbujeon is a representative story whose theme is rewarding good and punishing evil.

### 23. 피차일반 (彼此一般) [seongeo, stdict:361815, draft]

- en_gloss (AI): the same on both sides; six of one, half a dozen of the other
- en_def (AI): Two parties being just alike (lit. that side and this side, all one kind).
- ko_def (stdict, verbatim): 두 편이 서로 같음.
- ex (stdict, verbatim): 돈도 없고 집도 없어 떠돌이 생활하는 것은 자네나 나나 피차일반이 아닌가?
  - en (AI): With no money and no home, you and I are living a wandering life, so we are in the same boat, aren't we?
- ex (AI Korean): 이번 일은 누가 잘못했다고 따질 것 없이 피차일반이다.
  - en (AI): There is no point arguing over who was at fault this time; it is six of one, half a dozen of the other.

### 24. 대동소이 (大同小異) [seongeo, stdict:80087, draft]

- en_gloss (AI): much the same; differing only slightly
- en_def (AI): Almost identical, with no great difference (lit. big sameness, small difference).
- ko_def (stdict, verbatim): 큰 차이 없이 거의 같음.
- ex (AI Korean): 두 가게의 가격은 대동소이하다.
  - en (AI): The prices at the two shops are much the same.
- ex (AI Korean): 사람들의 생각은 크게 다르지 않고 대동소이했다.
  - en (AI): People's views were not very different; they were all much alike.

### 25. 절체절명 (絕體絕命) [seongeo, stdict:478089, draft]

- en_gloss (AI): at death's door; in desperate straits
- en_def (AI): A desperate situation from which there is no escape (lit. body at its end, life at its end).
- ko_def (stdict, verbatim): 몸도 목숨도 다 되었다는 뜻으로, 어찌할 수 없는 절박한 경우를 비유적으로 이르는 말.
- ex (AI Korean): 절체절명의 위기에서 그는 침착하게 길을 찾았다.
  - en (AI): In a moment of desperate crisis, he calmly found a way out.
- ex (AI Korean): 회사는 절체절명의 순간에 새로운 투자자를 만났다.
  - en (AI): The company met a new investor at the very last moment.

### 26. 오리무중 (五里霧中) [seongeo, stdict:461971, draft]

- en_gloss (AI): completely in the dark; shrouded in mystery
- en_def (AI): Unable to find any direction or clue about something (lit. in the middle of five li of thick fog).
- ko_def (stdict, verbatim): 오 리나 되는 짙은 안개 속에 있다는 뜻으로, 무슨 일에 대하여 방향이나 갈피를 잡을 수 없음을 이르는 말. ≪후한서≫의 <장해전(張楷傳)>에 나오는 말이다.
- ex (stdict, verbatim): 살인 사건이 여전히 오리무중 상태이다.
  - en (AI): The murder case is still shrouded in mystery.
- ex (stdict, verbatim): 범인의 행방이 오리무중이다.
  - en (AI): The whereabouts of the culprit are unknown.

### 27. 구사일생 (九死一生) [seongeo, stdict:38389, draft]

- en_gloss (AI): a narrow escape from death
- en_def (AI): Barely surviving after passing through many near-death moments (lit. nine deaths, one life).
- ko_def (stdict, verbatim): 아홉 번 죽을 뻔하다 한 번 살아난다는 뜻으로, 죽을 고비를 여러 차례 넘기고 겨우 살아남을 이르는 말.
- ex (AI Korean): 그는 총격 속에서 구사일생으로 살아 돌아왔다.
  - en (AI): He came back alive from the gunfire after a narrow escape from death.
- ex (stdict, verbatim): 그는 징용에 끌려갔다가 광복이 되어 구사일생으로 살아 돌아왔다.
  - en (AI): He was taken off for forced labor, and when Korea was liberated he returned alive after a narrow escape from death.

### 28. 만경창파 (萬頃蒼波) [seongeo, stdict:109340, draft]

- en_gloss (AI): the boundless blue sea
- en_def (AI): The vast sea stretching endlessly (lit. ten thousand "gyeong" of blue waves; a gyeong is a Chinese field measure).
- ko_def (stdict, verbatim): 만 이랑의 푸른 물결이라는 뜻으로, 한없이 넓고 넓은 바다를 이르는 말.
- ex (AI Korean): 배는 만경창파를 가르며 앞으로 나아갔다.
  - en (AI): The ship moved forward, cutting across the boundless sea.
- ex (AI Korean): 만경창파 위로 붉은 해가 떠올랐다.
  - en (AI): A red sun rose over the vast blue waters.

### 29. 전도유망 (前途有望) [seongeo, stdict:288001, draft]

- en_gloss (AI): promising; with a bright future
- en_def (AI): Having hope of doing well in the future (lit. the road ahead has promise).
- ko_def (stdict, verbatim): 앞으로 잘될 희망이 있음.
- ex (AI Korean): 그는 전도유망한 젊은 과학자로 평가받고 있다.
  - en (AI): He is regarded as a promising young scientist.
- ex (AI Korean): 전도유망한 신입 사원 덕분에 팀의 분위기가 밝아졌다.
  - en (AI): Thanks to a promising new employee, the mood on the team has brightened.

### 30. 시의적절 (時宜適切) [seongeo, stdict:516977, draft]

- en_gloss (AI): timely and appropriate
- en_def (AI): Fitting the circumstances and needs of the moment exactly (lit. suited to the times, apt and fitting).
- ko_def (stdict, verbatim): 그 당시의 사정이나 요구에 아주 알맞음.
- ex (AI Korean): 정부의 시의적절한 대응으로 피해를 줄일 수 있었다.
  - en (AI): The government's timely and appropriate response limited the damage.
- ex (AI Korean): 시의적절한 충고를 해 주셔서 감사합니다.
  - en (AI): Thank you for giving me such timely and fitting advice.

### 31. 일언반구 (一言半句) [seongeo, stdict:472145, draft]

- en_gloss (AI): a single word; the slightest remark
- en_def (AI): A very short remark (lit. one word, half a phrase).
- ko_def (stdict, verbatim): 한 마디 말과 반 구절이라는 뜻으로, 아주 짧은 말을 이르는 말.
- ex (AI Korean): 그는 떠나면서 일언반구도 남기지 않았다.
  - en (AI): He left without a single word.
- ex (AI Korean): 사전에 일언반구 상의도 없이 일을 진행하다니 너무하다.
  - en (AI): It is too much to go ahead without even a word of consultation beforehand.

### 32. 이구동성 (異口同聲) [seongeo, stdict:469668, draft]

- en_gloss (AI): with one voice; unanimously
- en_def (AI): Many people saying the very same thing (lit. different mouths, the same voice).
- ko_def (stdict, verbatim): 입은 다르나 목소리는 같다는 뜻으로, 여러 사람의 말이 한결같음을 이르는 말.
- ex (stdict, verbatim): 모든 사람이 그를 이구동성으로 칭찬한다.
  - en (AI): Everyone praises him with one voice.
- ex (AI Korean): 학생들은 이구동성으로 시험이 너무 어려웠다고 말했다.
  - en (AI): The students all said with one voice that the exam had been too hard.

### 33. 일취월장 (日就月將) [seongeo, stdict:476094, draft]

- en_gloss (AI): improving day by day; rapid progress
- en_def (AI): Growing and developing every day and every month (lit. advancing daily, growing monthly).
- ko_def (stdict, verbatim): 나날이 다달이 자라거나 발전함.
- ex (stdict, verbatim): 그가 한번 마음을 먹고 공부에 전념하니 일취월장이었다.
  - en (AI): Once he made up his mind and threw himself into his studies, he made rapid progress.
- ex (AI Korean): 매일 연습한 덕분에 그의 실력은 일취월장했다.
  - en (AI): Thanks to daily practice, his skills improved by leaps and bounds.
- ex (AI Korean): 그 학생은 일취월장하는 모습을 보여 선생님을 기쁘게 했다.
  - en (AI): The student showed steady, rapid improvement, which pleased the teacher.

### 34. 명불허전 (名不虛傳) [seongeo, stdict:116588, draft]

- en_gloss (AI): living up to one's reputation
- en_def (AI): A famous name that is not spread in vain; there is a good reason for its fame (lit. a name is not falsely handed down).
- ko_def (stdict, verbatim): 명성이나 명예가 헛되이 퍼진 것이 아니라는 뜻으로, 이름날 만한 까닭이 있음을 이르는 말.
- ex (AI Korean): 이 집 냉면은 정말 맛있어서 명불허전이라는 말이 절로 나온다.
  - en (AI): The cold noodles here are so good that "it lives up to its reputation" just comes to mind.
- ex (AI Korean): 세계적인 지휘자의 연주는 명불허전이었다.
  - en (AI): The performance by the world-famous conductor lived up to its reputation.

### 35. 오합지졸 (烏合之卒) [seongeo, stdict:463935, draft]

- en_gloss (AI): a disorderly rabble; a mob
- en_def (AI): A group gathered in haste, without discipline or order (lit. soldiers gathered like crows).
- ko_def (stdict, verbatim): 까마귀가 모인 것처럼 질서가 없이 모인 병졸이라는 뜻으로, 임시로 모여들어서 규율이 없고 무질서한 병졸 또는 군중을 이르는 말.
- ex (AI Korean): 훈련을 받지 못한 그의 군대는 오합지졸에 불과했다.
  - en (AI): His untrained army was nothing but a disorderly rabble.
- ex (AI Korean): 오합지졸로는 강한 상대를 이길 수 없다.
  - en (AI): A mere rabble cannot beat a strong opponent.

### 36. 일조일석 (一朝一夕) [seongeo, stdict:472024, draft]

- en_gloss (AI): overnight; in a single day
- en_def (AI): A very short time (lit. one morning, one evening).
- ko_def (stdict, verbatim): 하루의 아침과 하루의 저녁이란 뜻으로, 짧은 시일을 이르는 말.
- ex (stdict, verbatim): 이 문제를 일조일석에 해결할 수 있다고 기대하지 마라.
  - en (AI): Don't expect this problem to be solved overnight.
- ex (stdict, verbatim): 얼마나 공들인 일인데 일조일석에 무너뜨릴 수는 없다.
  - en (AI): After all the effort that went into it, I can't let it collapse overnight.
- ex (stdict, verbatim): 인간이 만물의 영장이 된 것은 일조일석의 돌발적 현상이 아니며, 어떤 한 개인의 힘만으로 가능했던 것도 아니다.
  - en (AI): Humans did not become the lords of creation overnight, nor was it possible through the strength of any one individual alone.

### 37. 파죽지세 (破竹之勢) [seongeo, stdict:499112, draft]

- en_gloss (AI): irresistible advance; carrying all before one
- en_def (AI): A momentum that sweeps aside the enemy without hindrance (lit. the force of splitting bamboo).
- ko_def (stdict, verbatim): 대를 쪼개는 기세라는 뜻으로, 적을 거침없이 물리치고 쳐들어가는 기세를 이르는 말. ≪진서(晉書)≫의 <두예전(杜預傳)>에서 나온 말이다.
- ex (stdict, verbatim): 아군은 파죽지세로 적군을 이 땅에서 몰아냈다.
  - en (AI): Our army drove the enemy out of this land with irresistible momentum.
- ex (AI Korean): 그 팀은 파죽지세로 결승까지 올라갔다.
  - en (AI): The team swept all the way to the final with irresistible momentum.

### 38. 다사다난 (多事多難) [seongeo, stdict:407536, draft]

- en_gloss (AI): eventful and troubled
- en_def (AI): Having many events and many troubles or setbacks (lit. many affairs, many hardships).
- ko_def (stdict, verbatim): 여러 가지 일도 많고 어려움이나 탈도 많음.
- ex (stdict, verbatim): 행사도 많고 사건도 많았던 올 한 해는 다사다난이라는 말이 어울린다.
  - en (AI): With so many events and incidents, "eventful and troubled" is a fitting phrase for this past year.
- ex (AI Korean): 다사다난했던 한 해를 보내며 새해에는 모두 건강하기를 바란다.
  - en (AI): As we see out a year full of ups and downs, I hope everyone stays healthy in the new year.

### 39. 새옹지마 (塞翁之馬) [seongeo, stdict:446587, draft]

- en_gloss (AI): life's unpredictable ups and downs
- en_def (AI): Good and bad fortune in life change so often that they cannot be predicted (lit. the old man at the frontier and his horse).
- ko_def (stdict, verbatim): 인생의 길흉화복은 변화가 많아서 예측하기가 어렵다는 말. 옛날에 새옹이 기르던 말이 오랑캐 땅으로 달아나서 노인이 낙심하였는데, 그 후에 달아났던 말이 준마를 한 필 끌고 와서 그 덕분에 훌륭한 말을 얻게 되었으나 아들이 그 준마를 타다가 떨어져서 다리가 부러졌으므로 노인이 다시 낙심하였는데, 그로 인하여 아들이 전쟁에 끌려 나가지 아니하고 죽음을 면할 수 있었다는 이야기에서 유래한다. 중국 ≪회남자≫의 ‘인간훈(人間訓)’에 나오는 말이다.
- ex (AI Korean): 새옹지마라더니 실직한 덕분에 오히려 더 좋은 일자리를 얻었다.
  - en (AI): They say you never know how fortune will turn, and losing my job actually got me a better one.
- ex (AI Korean): 인생은 새옹지마이니 지금의 실패에 너무 낙심하지 마라.
  - en (AI): Life's fortunes are unpredictable, so don't be too discouraged by your present failure.

### 40. 보국안민 (輔國安民) [seongeo, stdict:436046, draft]

- en_gloss (AI): serving the nation and bringing peace to the people
- en_def (AI): Helping the affairs of state and keeping the people at peace (lit. assist the country, settle the people).
- ko_def (stdict, verbatim): 나랏일을 돕고 백성을 편안하게 함.
- ex (AI Korean): 동학 농민군은 보국안민을 내걸고 일어섰다.
  - en (AI): The Donghak peasant army rose up under the banner of serving the nation and giving peace to the people.
- ex (AI Korean): 그 장군은 보국안민의 뜻을 품고 평생 나라를 지켰다.
  - en (AI): With the aim of serving the nation and keeping its people safe, the general defended the country all his life.

### 41. 이용후생 (利用厚生) [seongeo, stdict:261493, draft]

- en_gloss (AI): making practical use of things to improve people's lives
- en_def (AI): Using tools and resources conveniently and making food and clothing plentiful, so as to improve the people's livelihood (lit. use things well, make life abundant).
- ko_def (stdict, verbatim): 기구를 편리하게 쓰고 먹을 것과 입을 것을 넉넉하게 하여, 국민의 생활을 나아지게 함.
- ex (AI Korean): 실학자들은 이용후생을 위한 기술의 발전을 강조했다.
  - en (AI): The Silhak scholars stressed technological progress for the sake of improving people's livelihood.
- ex (AI Korean): 새로운 농기구는 이용후생의 정신으로 만들어졌다.
  - en (AI): The new farm tool was made in the spirit of putting things to practical use for people's well-being.

### 42. 천하제일 (天下第一) [seongeo, stdict:320843, draft]

- en_gloss (AI): second to none; the best in the world
- en_def (AI): Without any rival in the world; the very best (lit. first under heaven).
- ko_def (stdict, verbatim): 세상에 견줄 만한 것이 없이 최고임.
- ex (stdict, verbatim): 누가 뭐래도 그의 솜씨는 천하제일이다.
  - en (AI): Whatever anyone says, his skill is second to none.
- ex (AI Korean): 이 집의 김치찌개 맛은 천하제일이라고 소문이 났다.
  - en (AI): The kimchi stew at this restaurant is rumored to be the best there is.

### 43. 동분서주 (東奔西走) [seongeo, stdict:89859, draft]

- en_gloss (AI): rushing about busily
- en_def (AI): Running around in every direction in a great hurry (lit. dash east, run west).
- ko_def (stdict, verbatim): 동쪽으로 뛰고 서쪽으로 뛴다는 뜻으로, 사방으로 이리저리 몹시 바쁘게 돌아다님을 이르는 말.
- ex (AI Korean): 그는 아들의 병원비를 마련하려고 동분서주했다.
  - en (AI): He rushed about in every direction to raise money for his son's hospital bills.
- ex (AI Korean): 행사 준비로 직원들이 하루 종일 동분서주하고 있다.
  - en (AI): The staff have been running around busily all day preparing for the event.

### 44. 정리정돈 (整理整頓) [seongeo, stdict:535298, draft]

- en_gloss (AI): tidying up; putting things in order
- en_def (AI): Gathering up what is scattered or in disarray and arranging it neatly so that everything is orderly (lit. arrange and put in order).
- ko_def (stdict, verbatim): 흐트러지거나 혼란스러운 상태에 있는 것을 한데 모으거나 가지런히 바로잡아 질서 있는 상태가 되게 함.
- ex (stdict, verbatim): 정리정돈을 하면 좁은 장소를 넓게 쓸 수 있다.
  - en (AI): If you tidy up, you can make a small space feel bigger.
- ex (AI Korean): 방학 동안 아이들에게 정리정돈하는 습관을 길러 주고 싶다.
  - en (AI): I want to help my children build the habit of keeping things tidy during the vacation.

### 45. 출가외인 (出嫁外人) [seongeo, stdict:490302, draft]

- en_gloss (AI): a married daughter belongs to her husband's family
- en_def (AI): The old view that a married daughter is no longer a member of her birth family but is like an outsider (lit. a person outside, having gone out to marry).
- ko_def (stdict, verbatim): 시집간 딸은 친정 사람이 아니고 남이나 마찬가지라는 뜻으로 이르는 말.
- ex (AI Korean): 옛날에는 출가외인이라고 해서 딸에게 재산을 물려주지 않았다.
  - en (AI): In the past, daughters were not left any property, on the grounds that a married daughter belongs to another family.
- ex (AI Korean): 출가외인이라는 말은 요즘 시대에는 맞지 않는다.
  - en (AI): The saying that a married daughter is an outsider no longer suits the present age.

### 46. 일심동체 (一心同體) [seongeo, stdict:273554, draft]

- en_gloss (AI): one in heart and body; completely united
- en_def (AI): Being so firmly bound together that two or more people act as one (lit. one heart, one body).
- ko_def (stdict, verbatim): 한마음 한 몸이라는 뜻으로, 서로 굳게 결합함을 이르는 말.
- ex (stdict, verbatim): 노사가 일심동체가 되어 쓰러져 가던 회사를 일으켰다.
  - en (AI): Labor and management became united as one and revived the company that had been collapsing.
- ex (stdict, verbatim): 자고로 부부는 일심동체라 하였다.
  - en (AI): Since ancient times it has been said that a married couple are one in heart and body.

### 47. 일거일동 (一擧一動) [seongeo, stdict:467868, draft]

- en_gloss (AI): every move; each and every action
- en_def (AI): Each single movement or action (lit. one lifting, one moving).
- ko_def (stdict, verbatim): 하나하나의 동작이나 움직임.
- ex (stdict, verbatim): 나와 사이가 안 좋은 그는 나의 일거일동에 시비를 걸었다.
  - en (AI): He, who was on bad terms with me, found fault with my every move.
- ex (AI Korean): 감시 카메라가 직원들의 일거일동을 기록하고 있다.
  - en (AI): Surveillance cameras are recording the staff's every movement.

### 48. 불구대천 (不俱戴天) [seongeo, stdict:157108, draft]

- en_gloss (AI): sworn enemy; undying hatred
- en_def (AI): A grudge so deep that one cannot live in the same world as the other (lit. not bearing the same sky).
- ko_def (stdict, verbatim): 하늘을 함께 이지 못한다는 뜻으로, 이 세상에서 같이 살 수 없을 만큼 큰 원한을 가짐을 비유적으로 이르는 말.
- ex (AI Korean): 그는 아버지를 해친 사람을 불구대천의 원수로 여겼다.
  - en (AI): He regarded the man who had harmed his father as his sworn enemy.
- ex (AI Korean): 두 집안은 불구대천의 사이가 되어 서로 말도 섞지 않았다.
  - en (AI): The two families became mortal enemies and would not even speak to each other.

### 49. 다정다감 (多情多感) [seongeo, stdict:73426, draft]

- en_gloss (AI): warm-hearted and sensitive
- en_def (AI): Full of affection and rich in feeling (lit. much affection, much feeling).
- ko_def (stdict, verbatim): 정이 많고 감정이 풍부함.
- ex (AI Korean): 그는 다정다감한 성격이라 주변에 친구가 많다.
  - en (AI): He has a warm, sensitive nature, so he has many friends around him.
- ex (AI Korean): 다정다감한 선생님은 아이들의 고민을 잘 들어 주셨다.
  - en (AI): The warm-hearted teacher listened well to the children's worries.

### 50. 매점매석 (買占賣惜) [seongeo, stdict:518781, draft]

- en_gloss (AI): hoarding goods to profit from a price rise
- en_def (AI): Buying up goods all at once in expectation of higher prices and holding them back from sale (lit. buy up and corner, be reluctant to sell).
- ko_def (stdict, verbatim): 물건값이 오를 것을 예상하여 한꺼번에 샀다가 팔기를 꺼려 쌓아 둠.
- ex (stdict, verbatim): 매점매석은 불공정 거래의 대표적인 예이다.
  - en (AI): Hoarding goods to profit from a price rise is a typical example of unfair trade.
- ex (AI Korean): 태풍 소식에 일부 상인들이 매점매석에 나서 물가가 폭등했다.
  - en (AI): With news of the typhoon, some merchants started hoarding goods and prices shot up.

### 51. 잔인무도 (殘忍無道) [seongeo, stdict:278275, draft]

- en_gloss (AI): cruel and brutal
- en_def (AI): As cruel as can be, and without any humanity (lit. cruel and ruthless, lacking the proper way).
- ko_def (stdict, verbatim): 더할 수 없이 잔인함.
- ex (AI Korean): 그 독재자는 잔인무도한 방법으로 반대 세력을 제거했다.
  - en (AI): The dictator eliminated his opponents by cruel and brutal means.
- ex (AI Korean): 어린아이를 괴롭히다니 정말 잔인무도한 사람이다.
  - en (AI): To torment a little child, he is truly a vicious person.

### 52. 해괴망측 (駭怪罔測) [seongeo, stdict:363441, draft]

- en_gloss (AI): outrageous; shockingly bizarre
- en_def (AI): Strange and uncanny beyond words (lit. shocking, strange, beyond all conjecture).
- ko_def (stdict, verbatim): 말할 수 없이 괴상하고 야릇함.
- ex (AI Korean): 길거리에서 벌어진 해괴망측한 광경에 모두 눈을 의심했다.
  - en (AI): Everyone could hardly believe their eyes at the bizarre scene on the street.
- ex (AI Korean): 그런 해괴망측한 소문은 들어 본 적이 없다.
  - en (AI): I have never heard such an outlandish rumor.

### 53. 백귀야행 (百鬼夜行) [seongeo, stdict:137072, draft]

- en_gloss (AI): a procession of demons at night; a bizarre rabble
- en_def (AI): A crowd of strangely dressed people making a weird commotion, like all kinds of ghosts roaming at night (lit. a hundred demons walking at night).
- ko_def (stdict, verbatim): 온갖 잡귀가 밤에 나다닌다는 뜻으로, 괴상한 꼴을 하고 해괴한 짓을 하는 무리가 웅성거리며 돌아다님을 이르는 말.
- ex (AI Korean): 핼러윈 밤의 거리는 백귀야행을 방불케 했다.
  - en (AI): The streets on Halloween night looked like a parade of demons.
- ex (AI Korean): 괴상한 분장을 한 사람들이 백귀야행처럼 몰려다녔다.
  - en (AI): People in weird costumes roamed about in a crowd like a night procession of demons.

### 54. 대성통곡 (大聲痛哭) [seongeo, stdict:408385, draft]

- en_gloss (AI): wailing loudly; weeping bitterly
- en_def (AI): Weeping aloud in deep sorrow (lit. loud voice, bitter weeping).
- ko_def (stdict, verbatim): 큰 소리로 몹시 슬프게 곡을 함.
- ex (stdict, verbatim): 그는 부친의 부음을 듣자마자 대성통곡을 터뜨렸다.
  - en (AI): As soon as he heard the news of his father's death, he burst into loud wailing.
- ex (AI Korean): 그녀는 합격 소식을 듣고 기쁨에 겨워 대성통곡을 했다.
  - en (AI): On hearing she had passed, she wept loudly, overcome with joy.

### 55. 문전박대 (門前薄待) [seongeo, stdict:534037, draft]

- en_gloss (AI): turning someone away at the door; a cold reception
- en_def (AI): Treating someone so coldly that one drives them away at the door instead of letting them in (lit. meager treatment in front of the gate).
- ko_def (stdict, verbatim): 집으로 들이지 않고 문 앞에서 내쫓을 정도로 야박하게 대함.
- ex (AI Korean): 그는 돈을 빌리러 갔다가 친척에게 문전박대를 당했다.
  - en (AI): He went to borrow money and was turned away at the door by a relative.
- ex (AI Korean): 사정을 들어 보지도 않고 문전박대하다니 너무하는군요.
  - en (AI): Turning me away at the door without even hearing my situation is too much.

### 56. 임시변통 (臨時變通) [seongeo, stdict:271515, draft]

- en_gloss (AI): a makeshift fix; stopgap
- en_def (AI): Handling a sudden problem in a simple way just to get by for the moment (lit. adapting to the occasion).
- ko_def (stdict, verbatim): 갑자기 터진 일을 우선 간단하게 둘러맞추어 처리함.
- ex (AI Korean): 임시변통으로 의자 위에 상자를 올려 책상으로 썼다.
  - en (AI): As a makeshift arrangement, we put a box on a chair and used it as a desk.
- ex (AI Korean): 문제를 임시변통으로 넘기면 나중에 더 커질 수 있다.
  - en (AI): If you just patch a problem up for now, it may grow bigger later.

### 57. 환골탈태 (換骨奪胎) [seongeo, stdict:505070, draft]

- en_gloss (AI): a complete transformation
- en_def (AI): Changing so completely for the better that one becomes a different person; originally, reworking an older poem's form so that it surpasses the original (lit. exchange the bones, take over the womb).
- ko_def (stdict, verbatim): 뼈대를 바꾸어 끼고 태를 바꾸어 쓴다는 뜻으로, 고인의 시문의 형식을 바꾸어서 그 짜임새와 수법이 먼저 것보다 잘되게 함을 이르는 말. 중국 남송의 승려 혜홍(惠洪)의 <냉재야화(冷齋夜話)>에 나오는 말이다.
- ex (AI Korean): 그 가수는 데뷔 십 년 만에 환골탈태하여 완전히 다른 음악을 선보였다.
  - en (AI): Ten years after her debut, the singer underwent a complete transformation and presented music entirely unlike her earlier work.
- ex (AI Korean): 낡은 건물이 리모델링을 거쳐 환골탈태했다.
  - en (AI): The old building was completely transformed through a renovation.

### 58. 백척간두 (百尺竿頭) [seongeo, stdict:430805, draft]

- en_gloss (AI): on the brink of disaster; in extreme peril
- en_def (AI): A very difficult and dangerous situation (lit. on top of a hundred-foot pole).
- ko_def (stdict, verbatim): 백 자나 되는 높은 장대 위에 올라섰다는 뜻으로, 몹시 어렵고 위태로운 지경을 이르는 말.
- ex (AI Korean): 회사는 부도 위기에 몰려 백척간두에 서 있었다.
  - en (AI): Driven to the verge of bankruptcy, the company stood on the brink.
- ex (AI Korean): 나라가 백척간두의 위기에 놓였을 때 의병들이 일어났다.
  - en (AI): When the nation was in extreme peril, the volunteer armies rose up.

### 59. 일기당천 (一騎當千) [seongeo, stdict:473401, draft]

- en_gloss (AI): worth a thousand men; a fearsome fighter
- en_def (AI): Extremely outstanding in fighting ability (lit. one horseman a match for a thousand).
- ko_def (stdict, verbatim): 한 사람의 기병이 천 사람을 당한다는 뜻으로, 싸우는 능력이 아주 뛰어남을 이르는 말.
- ex (AI Korean): 그 장수는 일기당천의 용사로 이름을 떨쳤다.
  - en (AI): That general became famous as a warrior worth a thousand men.
- ex (AI Korean): 일기당천의 실력을 갖춘 선수가 팀에 합류했다.
  - en (AI): A player with the skills of a one-man army joined the team.

### 60. 표리일체 (表裏一體) [seongeo, stdict:359418, draft]

- en_gloss (AI): inseparable; two sides of one coin
- en_def (AI): Two things so closely connected that they are one (lit. outside and inside as one body).
- ko_def (stdict, verbatim): 안팎이 한 덩어리가 된다는 뜻으로, 두 가지 사물의 관계가 밀접하게 됨을 이르는 말.
- ex (AI Korean): 이론과 실천은 표리일체의 관계에 있다.
  - en (AI): Theory and practice are inseparably bound to each other.
- ex (AI Korean): 말과 행동이 표리일체인 사람이 진정한 지도자이다.
  - en (AI): A person whose words and actions are one is a true leader.

### 61. 천태만상 (千態萬象) [seongeo, stdict:490722, draft]

- en_gloss (AI): myriad forms; endless variety
- en_def (AI): Things in the world are not uniform, and each has its own appearance (lit. a thousand appearances, ten thousand forms).
- ko_def (stdict, verbatim): 천 가지 모습과 만 가지 형상이라는 뜻으로, 세상 사물이 한결같지 아니하고 각각 모습·모양이 다름을 이르는 말.
- ex (AI Korean): 시장에는 천태만상의 사람들이 오고 간다.
  - en (AI): All sorts of people come and go at the market.
- ex (AI Korean): 세상 사람들의 살아가는 모습은 천태만상이다.
  - en (AI): The ways people live in the world are endlessly varied.

### 62. 천재일우 (千載一遇) [seongeo, stdict:326645, draft]

- en_gloss (AI): chance of a lifetime
- en_def (AI): A rare, wonderful opportunity that comes hardly once in a very long time (lit. meeting it once in a thousand years).
- ko_def (stdict, verbatim): 천 년 동안 단 한 번 만난다는 뜻으로, 좀처럼 만나기 어려운 좋은 기회를 이르는 말.
- ex (AI Korean): 이번 기회는 천재일우이니 절대 놓치지 마라.
  - en (AI): This is a chance of a lifetime, so don't miss it for anything.
- ex (AI Korean): 그는 천재일우의 기회를 잡아 세계 무대에 올랐다.
  - en (AI): He seized a once-in-a-lifetime opportunity and made it onto the world stage.

### 63. 방약무인 (傍若無人) [seongeo, stdict:136470, draft]

- en_gloss (AI): acting as if no one else were present
- en_def (AI): Speaking and acting recklessly, without restraint, as if there were nobody around (lit. as if there were no one at one's side).
- ko_def (stdict, verbatim): 곁에 사람이 없는 것처럼 아무 거리낌이 없이 함부로 말하고 행동하는 태도가 있음.
- ex (stdict, verbatim): 남이 싫어하는 줄도 모르고 방약무인으로 떠들어 댄다.
  - en (AI): Not noticing that others dislike it, he chatters away as if no one else were around.
- ex (AI Korean): 방약무인한 태도로 어른들 앞에서 소리를 질렀다.
  - en (AI): With an arrogant disregard for others, he shouted in front of his elders.

### 64. 지행합일 (知行合一) [seongeo, stdict:313932, draft]

- en_gloss (AI): unity of knowledge and action
- en_def (AI): Knowledge and action agreeing with each other (lit. knowing and doing joined as one).
- ko_def (stdict, verbatim): 지식과 행동이 서로 맞음.
- ex (AI Korean): 지행합일은 아는 것을 실제로 행하라는 가르침이다.
  - en (AI): Unity of knowledge and action is the teaching that you should actually do what you know.
- ex (AI Korean): 그는 지행합일의 삶을 살기 위해 늘 노력했다.
  - en (AI): He always strove to live a life in which what he knew and what he did were one.

### 65. 산천초목 (山川草木) [seongeo, stdict:444739, draft]

- en_gloss (AI): mountains, rivers, plants and trees; nature
- en_def (AI): Nature as a whole (lit. mountains, streams, grasses and trees).
- ko_def (stdict, verbatim): 산과 내와 풀과 나무라는 뜻으로, ‘자연01’을 이르는 말.
- ex (AI Korean): 봄이 오자 산천초목이 푸른 옷으로 갈아입었다.
  - en (AI): When spring came, all of nature changed into green clothes.
- ex (AI Korean): 그 시인은 산천초목을 벗 삼아 평생을 보냈다.
  - en (AI): The poet spent his whole life with the mountains, streams and trees as his companions.

### 66. 자력갱생 (自力更生) [seongeo, stdict:474644, draft]

- en_gloss (AI): self-reliance; getting back on one's feet by one's own efforts
- en_def (AI): Escaping from a hard situation and starting a new life by one's own strength, without depending on others (lit. one's own power, a new life).
- ko_def (stdict, verbatim): 남에게 의지하지 아니하고 자신의 힘만으로 어려운 처지에서 벗어나 새로운 삶을 살아감.
- ex (stdict, verbatim): 사업에 실패한 그는 자력갱생의 정신으로 재기에 노력하고 있다.
  - en (AI): Having failed in business, he is striving to get back on his feet in a spirit of self-reliance.
- ex (AI Korean): 마을 사람들은 자력갱생으로 황폐해진 땅을 되살렸다.
  - en (AI): The villagers revived the ruined land by their own efforts.

### 67. 망지소조 (罔知所措) [seongeo, stdict:110923, draft]

- en_gloss (AI): at a loss; not knowing what to do
- en_def (AI): So flustered or pressed that one does not know what to do and dithers (lit. not knowing what to do about it).
- ko_def (stdict, verbatim): 너무 당황하거나 급하여 어찌할 줄을 모르고 갈팡질팡함.
- ex (AI Korean): 갑작스러운 사고에 그는 망지소조하여 서 있기만 했다.
  - en (AI): At the sudden accident, he just stood there, at a complete loss.
- ex (AI Korean): 아이가 사라지자 부모는 망지소조했다.
  - en (AI): When the child disappeared, the parents didn't know what to do.

### 68. 삼척동자 (三尺童子) [seongeo, stdict:168586, draft]

- en_gloss (AI): a mere child
- en_def (AI): A small child who knows nothing of the world (lit. a boy of three "cheok", i.e. under a meter tall).
- ko_def (stdict, verbatim): 키가 석 자 정도밖에 되지 않는 어린아이. 철없는 어린아이를 이른다.
- ex (AI Korean): 그 정도는 삼척동자도 아는 사실이다.
  - en (AI): Even a small child knows that much.
- ex (AI Korean): 삼척동자도 웃을 만큼 어이없는 변명이었다.
  - en (AI): It was such an absurd excuse that even a small child would laugh.

### 69. 국태민안 (國泰民安) [seongeo, stdict:39985, draft]

- en_gloss (AI): a peaceful nation and contented people
- en_def (AI): The country is at peace and the people are at ease (lit. the nation secure, the people safe).
- ko_def (stdict, verbatim): 나라가 태평하고 백성이 편안함.
- ex (AI Korean): 새해를 맞아 국태민안을 기원하는 제사를 올렸다.
  - en (AI): For the new year, they held a rite praying for peace in the country and well-being for the people.
- ex (AI Korean): 임금은 국태민안을 이루려고 밤낮으로 애썼다.
  - en (AI): The king worked day and night to bring about a peaceful nation and contented people.

### 70. 광명정대 (光明正大) [seongeo, stdict:31848, draft]

- en_gloss (AI): honest and aboveboard
- en_def (AI): Open and upright in one's words and conduct (lit. bright, clear, upright, grand).
- ko_def (stdict, verbatim): 말이나 행실이 떳떳하고 정당함.
- ex (AI Korean): 그는 어떤 일이든 광명정대하게 처리하기로 유명하다.
  - en (AI): He is known for handling every matter honestly and aboveboard.
- ex (AI Korean): 광명정대한 승부를 펼친 두 선수에게 박수가 쏟아졌다.
  - en (AI): Applause poured out for the two players who had played a fair and honest match.

### 71. 천상천하 (天上天下) [seongeo, stdict:322744, draft]

- en_gloss (AI): in heaven and on earth; the whole world
- en_def (AI): Everything under the sky (lit. above heaven, below heaven).
- ko_def (stdict, verbatim): 하늘 위와 하늘 아래라는 뜻으로, 온 세상을 이르는 말.
- ex (AI Korean): 천상천하에 나만큼 그를 아끼는 사람은 없을 것이다.
  - en (AI): There can be no one in the whole world who cares for him as much as I do.
- ex (AI Korean): 이 세상 천상천하에 둘도 없는 보물을 얻었다.
  - en (AI): I have obtained a treasure that has no equal anywhere under heaven.

### 72. 천라지망 (天羅地網) [seongeo, stdict:486485, draft]

- en_gloss (AI): an inescapable net; a tight dragnet
- en_def (AI): A net of surveillance that is impossible to slip through, or a disaster that cannot be avoided (lit. a net in heaven, a snare on earth).
- ko_def (stdict, verbatim): 하늘에 새 그물, 땅에 고기 그물이라는 뜻으로, 아무리 하여도 벗어나기 어려운 경계망이나 피할 수 없는 재액을 이르는 말.
- ex (AI Korean): 경찰은 천라지망을 펼쳐 범인을 추적했다.
  - en (AI): The police spread a tight dragnet and pursued the culprit.
- ex (AI Korean): 천라지망 같은 감시를 피해 달아나기는 불가능했다.
  - en (AI): It was impossible to escape the dragnet-like surveillance.

### 73. 남부여대 (男負女戴) [seongeo, stdict:60980, draft]

- en_gloss (AI): poor families wandering with their belongings on their backs
- en_def (AI): The poor wandering about in search of a place to live, the man carrying loads on his back and the woman on her head (lit. men carry on the back, women carry on the head).
- ko_def (stdict, verbatim): 남자는 지고 여자는 인다는 뜻으로, 가난한 사람들이 살 곳을 찾아 이리저리 떠돌아다님을 비유적으로 이르는 말.
- ex (AI Korean): 흉년이 들자 사람들은 남부여대하여 고향을 떠났다.
  - en (AI): When the harvest failed, people left their hometown with their belongings on their backs and heads.
- ex (AI Korean): 전쟁을 피해 남부여대하는 피난민 행렬이 이어졌다.
  - en (AI): A line of refugees, carrying what they had on their backs and heads, kept streaming out to flee the war.

### 74. 연락부절 (連絡不絕) [seongeo, stdict:229649, draft]

- en_gloss (AI): a ceaseless stream of contact
- en_def (AI): Visits or messages that are so frequent that word never stops coming (lit. contact without a break).
- ko_def (stdict, verbatim): 왕래가 잦아 소식이 끊이지 아니함.
- ex (AI Korean): 소문이 퍼진 뒤로 가게에는 문의 전화가 연락부절이었다.
  - en (AI): Since the rumor spread, inquiry calls to the shop have come in without a break.
- ex (AI Korean): 두 사람은 멀리 떨어져 살았지만 편지로 연락부절이었다.
  - en (AI): Although the two lived far apart, they kept in unbroken contact by letter.

### 75. 백의종군 (白衣從軍) [seongeo, stdict:142653, draft]

- en_gloss (AI): serving in the army without rank
- en_def (AI): Following the army to the battlefield without holding any official post (lit. in white clothes, joining the army).
- ko_def (stdict, verbatim): 벼슬 없이 군대를 따라 싸움터로 감.
- ex (AI Korean): 이순신 장군은 백의종군하며 나라를 지켰다.
  - en (AI): General Yi Sun-sin defended the country while serving in the army as a commoner without rank.
- ex (AI Korean): 억울한 누명을 쓰고도 그는 백의종군을 마다하지 않았다.
  - en (AI): Even though he had been falsely accused, he did not refuse to serve as a commoner soldier.

### 76. 유야무야 (有耶無耶) [seongeo, stdict:251602, draft]

- en_gloss (AI): fizzling out; left vague and unresolved
- en_def (AI): Being left in a muddle, as though it both exists and does not (lit. there it is? there it isn't?).
- ko_def (stdict, verbatim): 있는 듯 없는 듯 흐지부지함.
- ex (stdict, verbatim): 그 사건의 수사는 유야무야로 끝났다.
  - en (AI): The investigation of that case ended up fizzling out.
- ex (stdict, verbatim): 사건이 유야무야 처리되었다.
  - en (AI): The case was dealt with in a vague, inconclusive way.
- ex (stdict, verbatim): 문제를 유야무야 덮어 버렸다.
  - en (AI): They covered the problem up and left it unresolved.

### 77. 삼순구식 (三旬九食) [seongeo, stdict:177276, draft]

- en_gloss (AI): extreme poverty
- en_def (AI): Being so poor that one gets only nine meals in thirty days (lit. three ten-day periods, nine meals).
- ko_def (stdict, verbatim): 삼십 일 동안 아홉 끼니밖에 먹지 못한다는 뜻으로, 몹시 가난함을 이르는 말.
- ex (AI Korean): 그 집안은 삼순구식을 면하지 못할 만큼 가난했다.
  - en (AI): The family was so poor that they could hardly escape going hungry for days on end.
- ex (AI Korean): 삼순구식의 어려움 속에서도 그는 공부를 포기하지 않았다.
  - en (AI): Even in the hardship of living on scraps, he did not give up studying.

### 78. 대서특필 (大書特筆) [seongeo, stdict:81061, draft]

- en_gloss (AI): splashing across the headlines
- en_def (AI): Giving a news item big, prominent treatment in a newspaper or other publication (lit. write large, write especially).
- ko_def (stdict, verbatim): 특별히 두드러지게 보이도록 글자를 크게 쓴다는 뜻으로, 신문 따위의 출판물에서 어떤 기사에 큰 비중을 두어 다룸을 이르는 말.
- ex (stdict, verbatim): 기자는 그 사건을 신문에 대서특필로 보도하였다.
  - en (AI): The reporter covered the incident in the newspaper under big headlines.
- ex (AI Korean): 신문마다 그의 우승 소식을 대서특필했다.
  - en (AI): Every newspaper splashed the news of his victory across its pages.

### 79. 격물치지 (格物致知) [seongeo, stdict:18676, draft]

- en_gloss (AI): investigating things to attain knowledge
- en_def (AI): Studying the principles of actual things to make one's knowledge complete (lit. investigate things, reach knowledge).
- ko_def (stdict, verbatim): 실제 사물의 이치를 연구하여 지식을 완전하게 함. ≪대학≫에 나오는 말이다.
- ex (AI Korean): 격물치지는 사물의 이치를 탐구해 앎에 이르는 것이다.
  - en (AI): "Gyeongmul chiji" is to reach understanding by examining the principles of things.
- ex (AI Korean): 성리학자들은 격물치지를 학문의 출발점으로 삼았다.
  - en (AI): The Neo-Confucian scholars took the investigation of things as the starting point of learning.

### 80. 토사구팽 (兔死狗烹) [seongeo, stdict:349104, draft]

- en_gloss (AI): casting someone aside once they have served their purpose
- en_def (AI): Using someone while they are needed and heartlessly discarding them when they are not (lit. when the hare dies, the hunting dog is boiled).
- ko_def (stdict, verbatim): 토끼가 죽으면 토끼를 잡던 사냥개도 필요 없게 되어 주인에게 삶아 먹히게 된다는 뜻으로, 필요할 때는 쓰고 필요 없을 때는 야박하게 버리는 경우를 이르는 말.
- ex (AI Korean): 전쟁이 끝나자 공을 세운 장수들은 토사구팽을 당했다.
  - en (AI): When the war ended, the generals who had rendered great service were cast aside.
- ex (AI Korean): 필요할 때만 쓰고 버리는 토사구팽을 당하고 싶지 않다.
  - en (AI): I don't want to be used when I'm needed and then thrown away.

### 81. 백중지세 (伯仲之勢) [seongeo, stdict:138869, draft]

- en_gloss (AI): neck and neck; evenly matched
- en_def (AI): A situation in which it is hard to say which side is better (lit. the standing of elder and younger brother).
- ko_def (stdict, verbatim): 서로 우열을 가리기 힘든 형세. 위나라 문제(文帝)의 ≪전론(典論)≫에서 나온 말이다.
- ex (AI Korean): 두 팀의 실력은 백중지세라서 결과를 예측하기 어렵다.
  - en (AI): The two teams are evenly matched, so it's hard to predict the result.
- ex (AI Korean): 경기는 마지막까지 백중지세로 이어졌다.
  - en (AI): The game stayed neck and neck right to the end.

### 82. 중구난방 (衆口難防) [seongeo, stdict:485919, draft]

- en_gloss (AI): everyone talking at once
- en_def (AI): So many people babbling at random that they cannot be stopped (lit. the mouths of the many are hard to guard against).
- ko_def (stdict, verbatim): 뭇사람의 말을 막기가 어렵다는 뜻으로, 막기 어려울 정도로 여럿이 마구 지껄임을 이르는 말.
- ex (AI Korean): 회의에서 모두가 중구난방으로 떠들어 결론이 나지 않았다.
  - en (AI): At the meeting, everyone talked at once and no conclusion was reached.
- ex (AI Korean): 중구난방 쏟아지는 의견을 정리하기가 쉽지 않았다.
  - en (AI): It wasn't easy to sort out the opinions pouring in from all directions.

### 83. 마이동풍 (馬耳東風) [seongeo, stdict:422770, draft]

- en_gloss (AI): falling on deaf ears
- en_def (AI): Not listening to what others say and letting it pass by (lit. an east wind in a horse's ear).
- ko_def (stdict, verbatim): 동풍이 말의 귀를 스쳐 간다는 뜻으로, 남의 말을 귀담아듣지 아니하고 지나쳐 흘려버림을 이르는 말.
- ex (stdict, verbatim): 그에게는 나의 충고가 마이동풍이었다.
  - en (AI): My advice fell on deaf ears with him.
- ex (AI Korean): 아무리 말려도 마이동풍이니 더 이상 말하지 않겠다.
  - en (AI): No matter how much I try to stop him he doesn't listen, so I won't say any more.

### 84. 시비곡직 (是非曲直) [seongeo, stdict:206164, draft]

- en_gloss (AI): right and wrong; the rights and wrongs
- en_def (AI): What is right and wrong, crooked and straight (lit. right, wrong, crooked, straight).
- ko_def (stdict, verbatim): 옳고 그르고 굽고 곧음.
- ex (AI Korean): 재판관은 시비곡직을 가려 판결을 내렸다.
  - en (AI): The judge sorted out the rights and wrongs and handed down a verdict.
- ex (AI Korean): 이 일의 시비곡직은 시간이 지나면 밝혀질 것이다.
  - en (AI): The rights and wrongs of this matter will become clear with time.

### 85. 사면초가 (四面楚歌) [seongeo, stdict:440307, draft]

- en_gloss (AI): surrounded on all sides; completely isolated
- en_def (AI): Being in a lonely, awkward position with help from nowhere (lit. songs of Chu on all four sides).
- ko_def (stdict, verbatim): 아무에게도 도움을 받지 못하는, 외롭고 곤란한 지경에 빠진 형편을 이르는 말. 초나라 항우가 사면을 둘러싼 한나라 군사 쪽에서 들려오는 초나라의 노랫소리를 듣고 초나라 군사가 이미 항복한 줄 알고 놀랐다는 데서 유래한다. ≪사기≫의 <항우본기(項羽本紀)>에 나오는 말이다.
- ex (AI Korean): 시험은 어렵고 돈도 없고, 그야말로 사면초가였다.
  - en (AI): The exam was hard and I had no money, so I really was surrounded on all sides.
- ex (AI Korean): 사면초가의 상황에서도 그는 포기하지 않았다.
  - en (AI): Even when everything was against him, he did not give up.

### 86. 낙심천만 (落心千萬) [seongeo, stdict:59194, draft]

- en_gloss (AI): utterly disheartened
- en_def (AI): Being deeply upset because something one hoped for has not come about (lit. heart fallen, a thousand ten-thousand-fold).
- ko_def (stdict, verbatim): 바라던 일을 이루지 못하여 마음이 몹시 상함.
- ex (stdict, verbatim): 계획이 실패로 돌아가 낙심천만이다.
  - en (AI): The plan fell through, and I am utterly disheartened.
- ex (AI Korean): 시험에 또 떨어지자 그는 낙심천만하여 말을 잇지 못했다.
  - en (AI): When he failed the exam again, he was utterly disheartened and could not speak.

### 87. 무가내하 (無可奈何) [seongeo, stdict:426063, draft]

- en_gloss (AI): nothing can be done about it
- en_def (AI): There is no other way, and nothing that anyone can do (lit. nothing possible to do about it).
- ko_def (stdict, verbatim): 달리 어찌할 수 없음.
- ex (stdict, verbatim): 동생은 집에 가겠다며 무가내하로 울어 댄다.
  - en (AI): My younger sibling keeps crying that they want to go home, and there is nothing anyone can do about it.
- ex (AI Korean): 그 일은 이미 끝난 일이니 이제는 무가내하이다.
  - en (AI): That matter is already over and done with, so now there is nothing to be done.

### 88. 영생불멸 (永生不滅) [seongeo, stdict:236756, draft]

- en_gloss (AI): eternal life; immortality
- en_def (AI): Living forever and never dying (lit. live forever, never perish).
- ko_def (stdict, verbatim): 영원히 살고 죽지 아니함.
- ex (stdict, verbatim): 인간이면 누구나 영생불멸을 원한다.
  - en (AI): Every human being wants to live forever and never die.
- ex (AI Korean): 진시황은 영생불멸을 얻으려고 불로초를 찾았다.
  - en (AI): The First Emperor of Qin looked for the herb of eternal youth in order to gain immortality.

### 89. 가렴주구 (苛斂誅求) [seongeo, stdict:388368, draft]

- en_gloss (AI): harsh taxation and extortion
- en_def (AI): Collecting taxes ruthlessly and seizing people's wealth by force (lit. harsh levies, extorting demands).
- ko_def (stdict, verbatim): 세금을 가혹하게 거두어들이고, 무리하게 재물을 빼앗음.
- ex (AI Korean): 탐관오리의 가렴주구에 백성들의 삶은 날로 피폐해졌다.
  - en (AI): Under the harsh taxation and extortion of corrupt officials, the people's lives grew more wretched by the day.
- ex (AI Korean): 농민들은 지나친 가렴주구를 견디다 못해 봉기하였다.
  - en (AI): Unable to bear the excessive extortion, the peasants rose in revolt.

### 90. 급전직하 (急轉直下) [seongeo, stdict:401321, draft]

- en_gloss (AI): a sudden, steep turn; plunging
- en_def (AI): A situation developing so abruptly that it cannot be held in check (lit. turning sharply, going straight down).
- ko_def (stdict, verbatim): 사정이나 형세가 걷잡을 수 없을 만큼 급작스럽게 전개됨.
- ex (stdict, verbatim): 주가가 급전직하로 폭락했다.
  - en (AI): Stock prices plunged sharply.
- ex (AI Korean): 사건이 급전직하로 해결되면서 모두가 놀랐다.
  - en (AI): Everyone was surprised as the case was suddenly resolved in a rush.

### 91. 만고불변 (萬古不變) [seongeo, stdict:423349, draft]

- en_gloss (AI): everlasting; never changing
- en_def (AI): Not changing for a very long time (lit. ten thousand ages, unchanging).
- ko_def (stdict, verbatim): 아주 오랜 세월 동안 변하지 아니함.
- ex (stdict, verbatim): 새벽녘의 바다는 만고불변의 아름다움을 빛내며 출렁이고 있었다.
  - en (AI): The sea at dawn was rolling, shining with an everlasting beauty that never changes.
- ex (AI Korean): 부모의 사랑은 만고불변이라고 한다.
  - en (AI): They say a parent's love never changes, in any age.

### 92. 공존공영 (共存共榮) [seongeo, stdict:31535, draft]

- en_gloss (AI): coexistence and mutual prosperity
- en_def (AI): Living together and flourishing together (lit. exist together, prosper together).
- ko_def (stdict, verbatim): 함께 존재하고 함께 번영함.
- ex (AI Korean): 인간과 자연이 공존공영하는 길을 찾아야 한다.
  - en (AI): We must find a way for humans and nature to coexist and flourish together.
- ex (AI Korean): 두 나라는 공존공영의 원칙에 따라 협력하기로 했다.
  - en (AI): The two countries agreed to cooperate on the principle of coexistence and mutual prosperity.

### 93. 대담무쌍 (大膽無雙) [seongeo, stdict:78825, draft]

- en_gloss (AI): fearless; extremely bold
- en_def (AI): Boldness beyond comparison (lit. great gall, without peer).
- ko_def (stdict, verbatim): 대담하기가 어디에 비할 데가 없음.
- ex (AI Korean): 그는 대담무쌍하게 적진으로 뛰어들었다.
  - en (AI): He leapt boldly into the enemy camp, without fear.
- ex (AI Korean): 대담무쌍한 도전 끝에 그는 정상에 올랐다.
  - en (AI): After a daring challenge, he reached the summit.

### 94. 유만부동 (類萬不同) [seongeo, stdict:254751, draft]

- en_gloss (AI): alike yet each different; beyond all bounds
- en_def (AI): Many things resemble one another but are not the same; in use also, going beyond the proper limit (lit. ten thousand kinds, none the same).
- ko_def (stdict, verbatim): 비슷한 것이 많으나 서로 같지는 아니함.
- ex (stdict, verbatim): 형제간이라도 유만부동이라 그는 막내를 유달리 사랑한다.
  - en (AI): Even among siblings people differ, so he is especially fond of the youngest.
- ex (AI Korean): 농담도 유만부동이지 사람들 앞에서 그렇게 놀리면 어떡해?
  - en (AI): Even jokes have their limits; how could you tease someone like that in front of everybody?

### 95. 어불성설 (語不成說) [seongeo, stdict:461110, draft]

- en_gloss (AI): utter nonsense; absurd
- en_def (AI): Words that make no sense at all (lit. words that do not form an argument).
- ko_def (stdict, verbatim): 말이 조금도 사리에 맞지 아니함.
- ex (stdict, verbatim): 일하는 것은 가정을 위해서인데 가정이 파괴될 정도로 열심히 일한다면 이는 어불성설이 아닌가.
  - en (AI): Work is for the sake of the family, so if you work so hard that the family falls apart, isn't that absurd?
- ex (AI Korean): 공부를 하겠다며 책 한 권 펴지 않다니 어불성설이다.
  - en (AI): Saying you'll study and yet not opening a single book is nonsense.

### 96. 십상팔구 (十常八九) [seongeo, stdict:454946, draft]

- en_gloss (AI): nine times out of ten; almost certainly
- en_def (AI): In eight or nine cases out of ten, with almost no exception (lit. ten times, usually eight or nine).
- ko_def (stdict, verbatim): 열에 여덟이나 아홉 정도로 거의 예외가 없음.
- ex (stdict, verbatim): 전화를 해도 그가 자리에 없을 것이 십상팔구이다.
  - en (AI): Even if I call, he is almost certainly not going to be at his desk.
- ex (stdict, verbatim): 늦게 일어났으니 지각은 십상팔구이다.
  - en (AI): Having woken up late, I'll almost certainly be late.

### 97. 광제창생 (廣濟蒼生) [seongeo, stdict:33244, draft]

- en_gloss (AI): saving the common people
- en_def (AI): Rescuing the people widely from their hardships (lit. widely relieve the common people).
- ko_def (stdict, verbatim): 널리 백성을 구제함.
- ex (AI Korean): 동학은 광제창생의 뜻을 내세워 민중의 지지를 얻었다.
  - en (AI): Donghak put forward the aim of saving the common people and won popular support.
- ex (AI Korean): 그 한의사는 광제창생을 평생의 사명으로 삼았다.
  - en (AI): That doctor of traditional medicine took saving the common people as their life's mission.

### 98. 연전연패 (連戰連敗) [seongeo, stdict:238788, draft]

- en_gloss (AI): defeat after defeat
- en_def (AI): Losing every time one fights (lit. fight in a row, lose in a row).
- ko_def (stdict, verbatim): 싸울 때마다 계속하여 짐.
- ex (AI Korean): 그 팀은 시즌 내내 연전연패하며 최하위로 떨어졌다.
  - en (AI): The team lost game after game all season and fell to last place.
- ex (AI Korean): 연전연패에도 감독은 선수들을 믿었다.
  - en (AI): Despite the string of defeats, the coach had faith in his players.

### 99. 수원수구 (誰怨誰咎) [seongeo, stdict:452514, draft]

- en_gloss (AI): no one to resent or blame
- en_def (AI): With nobody to resent and nobody to blame (lit. whom to resent, whom to blame).
- ko_def (stdict, verbatim): 누구를 원망하고 누구를 탓하겠냐는 뜻으로, 남을 원망하거나 탓할 것이 없음을 이르는 말.
- ex (AI Korean): 스스로 선택한 길이니 수원수구할 일도 없다.
  - en (AI): It is a road I chose myself, so there is no one to resent or blame.
- ex (AI Korean): 모든 것이 내 잘못인데 수원수구하겠는가.
  - en (AI): Everything is my own fault, so whom could I resent or blame?

### 100. 구곡간장 (九曲肝腸) [seongeo, stdict:35679, draft]

- en_gloss (AI): the depths of one's heart
- en_def (AI): Deep inside one's heart, or a heart weighed down with accumulated grief (lit. liver and bowels in nine twists).
- ko_def (stdict, verbatim): 굽이굽이 서린 창자라는 뜻으로, 깊은 마음속 또는 시름이 쌓인 마음속을 비유적으로 이르는 말.
- ex (AI Korean): 자식을 먼저 보낸 어머니의 구곡간장이 녹아내렸다.
  - en (AI): The heart of the mother who had outlived her child melted with grief.
- ex (AI Korean): 그는 구곡간장에 쌓인 사연을 털어놓았다.
  - en (AI): He poured out the story that had piled up deep in his heart.

### 101. 불편부당 (不偏不黨) [seongeo, stdict:157460, draft]

- en_gloss (AI): impartial; non-partisan
- en_def (AI): Perfectly fair and leaning to neither side (lit. not biased, not partisan).
- ko_def (stdict, verbatim): 아주 공평하여 어느 쪽으로도 치우침이 없음.
- ex (AI Korean): 심판은 불편부당하게 경기를 진행해야 한다.
  - en (AI): A referee must run the game impartially.
- ex (AI Korean): 언론은 불편부당한 태도로 사실을 전달해야 한다.
  - en (AI): The press should report the facts in a non-partisan manner.

### 102. 불사이군 (不事二君) [seongeo, stdict:158560, draft]

- en_gloss (AI): serving only one lord; undivided loyalty
- en_def (AI): Not serving two rulers (lit. not serve two lords).
- ko_def (stdict, verbatim): 두 임금을 섬기지 아니함.
- ex (AI Korean): 그는 불사이군의 신념으로 끝까지 옛 왕조를 지켰다.
  - en (AI): With a belief in never serving two lords, he stayed loyal to the old dynasty to the end.
- ex (AI Korean): 충신은 불사이군이라고 하여 나라가 망해도 절개를 지켰다.
  - en (AI): A loyal subject, the saying goes, does not serve two lords, and he kept faith even when the country fell.

### 103. 필마단창 (匹馬單槍) [seongeo, stdict:363243, draft]

- en_gloss (AI): a lone rider with a spear
- en_def (AI): Setting off alone on one horse with only simple weapons; also, a person who does so (lit. one horse, one spear).
- ko_def (stdict, verbatim): 한 필의 말과 한 자루의 창이란 뜻으로, 혼자 간단한 무장을 하고 한 필의 말을 타고 감을 이르는 말. 또는 그렇게 하는 사람.
- ex (AI Korean): 그는 필마단창으로 적진에 뛰어들었다.
  - en (AI): He charged into the enemy's camp alone, on a single horse with just a spear.
- ex (AI Korean): 필마단창의 장수가 홀로 성문 앞에 나타났다.
  - en (AI): A general with a single horse and a spear appeared alone before the castle gate.

### 104. 적반하장 (賊反荷杖) [seongeo, stdict:473798, draft]

- en_gloss (AI): the guilty party blaming the innocent
- en_def (AI): A wrongdoer scolding someone who has done nothing wrong (lit. the thief turns round and takes up the stick).
- ko_def (stdict, verbatim): 도둑이 도리어 매를 든다는 뜻으로, 잘못한 사람이 아무 잘못도 없는 사람을 나무람을 이르는 말.
- ex (AI Korean): 잘못은 자기가 해 놓고 화를 내다니 적반하장도 유분수다.
  - en (AI): He makes the mistake himself and then gets angry; the guilty blaming the innocent has its limits.
- ex (AI Korean): 도둑이 오히려 집주인을 나무라는 적반하장의 상황이 벌어졌다.
  - en (AI): A situation arose in which the thief scolded the householder instead, the guilty turning on the innocent.

### 105. 대기만성 (大器晩成) [seongeo, stdict:79752, draft]

- en_gloss (AI): a late bloomer
- en_def (AI): Great people take a long time to be formed (lit. a great vessel is completed late).
- ko_def (stdict, verbatim): 큰 그릇을 만드는 데는 시간이 오래 걸린다는 뜻으로, 크게 될 사람은 늦게 이루어짐을 이르는 말.
- ex (AI Korean): 대기만성이라더니 그는 마흔이 넘어서야 크게 성공했다.
  - en (AI): They say great talents mature late, and he did not find great success until after forty.
- ex (AI Korean): 지금 성적이 낮다고 낙담하지 마라, 대기만성이라는 말도 있잖니.
  - en (AI): Don't be discouraged by low grades now; there is a saying that great talents mature late.

### 106. 일벌백계 (一罰百戒) [seongeo, stdict:473395, draft]

- en_gloss (AI): making an example of someone
- en_def (AI): Punishing one person severely as a warning to many others (lit. one punishment, a hundred warnings).
- ko_def (stdict, verbatim): 한 사람을 벌주어 백 사람을 경계한다는 뜻으로, 다른 사람들에게 경각심을 불러일으키기 위하여 본보기로 한 사람에게 엄한 처벌을 하는 일을 이르는 말.
- ex (stdict, verbatim): 회장은 예전엔 용인해 줄 만한 사안도 일벌백계 차원에서 엄중 문책하겠다고 밝혔다.
  - en (AI): The chairman stated that even matters that used to be tolerated would be severely dealt with, as an example to others.
- ex (AI Korean): 검찰은 일벌백계의 원칙에 따라 주범을 엄하게 처벌했다.
  - en (AI): Following the principle of making an example of one, the prosecution punished the ringleader severely.

### 107. 만분지일 (萬分之一) [seongeo, stdict:109500, draft]

- en_gloss (AI): a tiny fraction
- en_def (AI): A very small share, one part in ten thousand (lit. one of ten thousand parts).
- ko_def (stdict, verbatim): 만으로 나눈 것의 하나라는 뜻으로, 아주 적은 경우를 이르는 말.
- ex (stdict, verbatim): 부모의 마음을 만분지일이라도 헤아린다면, 네가 그런 나쁜 짓을 할 수 있겠는가.
  - en (AI): If you understood even a fraction of your parents' feelings, could you do such a bad thing?
- ex (AI Korean): 그의 노력은 성공의 만분지일에도 미치지 못했다.
  - en (AI): His efforts did not amount to even a tiny fraction of the success.

### 108. 사고무친 (四顧無親) [seongeo, stdict:167902, draft]

- en_gloss (AI): with no one to turn to
- en_def (AI): Having no relatives or anyone at all to depend on (lit. look all around, no kin).
- ko_def (stdict, verbatim): 의지할 만한 사람이 아무도 없음.
- ex (AI Korean): 전쟁으로 가족을 잃은 아이는 사고무친의 처지가 되었다.
  - en (AI): The child who lost their family in the war was left with no one to turn to.
- ex (AI Korean): 낯선 도시에서 사고무친으로 살아가기란 쉽지 않았다.
  - en (AI): It was not easy to live in an unfamiliar city with nobody to turn to.

### 109. 일구이언 (一口二言) [seongeo, stdict:270299, draft]

- en_gloss (AI): saying two different things; going back on one's word
- en_def (AI): Saying one thing and then another about the same matter (lit. one mouth, two words).
- ko_def (stdict, verbatim): 한 입으로 두말을 한다는 뜻으로, 한 가지 일에 대하여 말을 이랬다저랬다 함을 이르는 말.
- ex (stdict, verbatim): 일구이언은 사내대장부가 할 바가 아니다.
  - en (AI): Going back on your word is not what a real man does.
- ex (stdict, verbatim): 그는 일구이언을 밥 먹듯 하여 아무도 그를 믿지 않게 되었다.
  - en (AI): He went back on his word as casually as he ate his meals, so nobody trusted him any longer.

### 110. 각자도생 (各自圖生) [seongeo, stdict:4599, draft]

- en_gloss (AI): every man for himself
- en_def (AI): Each person seeking their own way to survive (lit. each plans their own survival).
- ko_def (stdict, verbatim): 제각기 살아 나갈 방법을 꾀함.
- ex (AI Korean): 위기가 닥치자 사람들은 각자도생을 외치며 흩어졌다.
  - en (AI): When the crisis hit, people scattered, each crying every man for himself.
- ex (AI Korean): 복지가 부족한 사회에서는 각자도생의 분위기가 퍼지기 쉽다.
  - en (AI): In a society with weak welfare, an every-man-for-himself mood can easily spread.

### 111. 녹음방초 (綠陰芳草) [seongeo, stdict:68166, draft]

- en_gloss (AI): lush greenery and fragrant grass
- en_def (AI): The scenery of summer, with thick green trees and sweet-smelling grass (lit. green shade, fragrant grass).
- ko_def (stdict, verbatim): 푸르게 우거진 나무와 향기로운 풀이라는 뜻으로, 여름철의 자연 경관을 이르는 말.
- ex (stdict, verbatim): 봄이 가고 여름이 돌아와 녹음방초의 계절을 맞게 되었다.
  - en (AI): Spring has gone and summer has returned, bringing the season of lush green trees and fragrant grass.
- ex (AI Korean): 녹음방초가 우거진 산길을 걸으니 마음이 시원해졌다.
  - en (AI): Walking a mountain path thick with green leaves and fragrant grass refreshed my mind.

### 112. 천군만마 (千軍萬馬) [seongeo, stdict:326905, draft]

- en_gloss (AI): a mighty host; like having an army behind you
- en_def (AI): A great number of soldiers and war horses, used for strong backing (lit. a thousand soldiers, ten thousand horses).
- ko_def (stdict, verbatim): 천 명의 군사와 만 마리의 군마라는 뜻으로, 아주 많은 수의 군사와 군마를 이르는 말.
- ex (stdict, verbatim): 자네가 나를 밀어준다니 천군만마를 얻은 것 같군.
  - en (AI): If you're backing me, I feel as though I've gained a whole army.
- ex (AI Korean): 친구가 곁에 있어 주니 천군만마를 얻은 기분이다.
  - en (AI): With my friend by my side, it feels as if I've got a mighty army behind me.

### 113. 청출어람 (靑出於藍) [seongeo, stdict:488401, draft]

- en_gloss (AI): the pupil surpassing the master
- en_def (AI): A student or junior who outdoes the teacher or senior (lit. blue comes out of the indigo and is bluer than the indigo).
- ko_def (stdict, verbatim): 쪽에서 뽑아낸 푸른 물감이 쪽보다 더 푸르다는 뜻으로, 제자나 후배가 스승이나 선배보다 나음을 비유적으로 이르는 말. ≪순자(荀子)≫의 <권학편(勸學篇)>에 나오는 말이다.
- ex (AI Korean): 제자의 실력이 스승을 넘어섰으니 청출어람이라 할 만하다.
  - en (AI): The pupil's skill has gone beyond his teacher's, so it deserves to be called the pupil surpassing the master.
- ex (AI Korean): 청출어람이라는 말처럼 후배가 선배의 기록을 깼다.
  - en (AI): Just as the saying goes, the junior has broken the senior's record and outdone the master.

### 114. 빙공영사 (憑公營私) [seongeo, stdict:167389, draft]

- en_gloss (AI): using public office for private gain
- en_def (AI): Seeking private profit under cover of public duty (lit. relying on the public, running a private business).
- ko_def (stdict, verbatim): 공적인 것을 빙자하여 사적인 이득을 꾀함.
- ex (AI Korean): 그 공무원은 빙공영사로 재산을 불렸다가 처벌받았다.
  - en (AI): The official built up his wealth by using his public post for private gain and was punished.
- ex (AI Korean): 공적인 일을 빙자한 빙공영사는 근절되어야 한다.
  - en (AI): Using public affairs as a pretext for private gain must be eradicated.

### 115. 의리부동 (義理不同) [seongeo, stdict:265685, draft]

- en_gloss (AI): contrary to righteousness
- en_def (AI): Not in keeping with what is right or with one's duty of loyalty (lit. duty, not the same).
- ko_def (stdict, verbatim): 의리에 맞지 아니함.
- ex (AI Korean): 친구를 배신하는 것은 의리부동한 짓이다.
  - en (AI): Betraying a friend is an act contrary to righteousness.
- ex (AI Korean): 은혜를 입고도 모르는 척하는 것은 의리부동이다.
  - en (AI): To pretend not to know someone after receiving their kindness is to go against what is right.

### 116. 포악무도 (暴惡無道) [seongeo, stdict:500382, draft]

- en_gloss (AI): savage and lawless
- en_def (AI): Fierce and brutal, without any law or decency (lit. violent and evil, without the way).
- ko_def (stdict, verbatim): 법도 도리도 없이 포악하다는 뜻으로, 사납고 악착하기가 이를 데 없음을 이르는 말.
- ex (AI Korean): 포악무도한 폭군은 백성들의 원성을 샀다.
  - en (AI): The savage, lawless tyrant earned the people's resentment.
- ex (AI Korean): 그 도적 떼는 포악무도하여 마을 사람들이 두려워했다.
  - en (AI): The band of robbers was so brutal and lawless that the villagers feared them.

### 117. 적수공권 (赤手空拳) [seongeo, stdict:475111, draft]

- en_gloss (AI): empty-handed
- en_def (AI): Having nothing at all, as if with only bare hands and empty fists (lit. red hands, empty fists).
- ko_def (stdict, verbatim): 맨손과 맨주먹이라는 뜻으로, 아무것도 가진 것이 없음을 이르는 말.
- ex (stdict, verbatim): 그는 적수공권으로 외국에 나가 큰돈을 모았다.
  - en (AI): He went abroad empty-handed and amassed a fortune.
- ex (AI Korean): 그는 적수공권으로 시작해 지금의 회사를 세웠다.
  - en (AI): He started with nothing and built the company he has today.

### 118. 장생불사 (長生不死) [seongeo, stdict:278706, draft]

- en_gloss (AI): eternal life; immortality
- en_def (AI): Living for a long time and never dying (lit. long life, no death).
- ko_def (stdict, verbatim): 오래도록 살고 죽지 아니함.
- ex (AI Korean): 도교에서는 장생불사의 신선이 되기를 꿈꾸었다.
  - en (AI): In Daoism, people dreamed of becoming immortal sages who live forever.
- ex (AI Korean): 장생불사의 약을 구하려고 사신이 바다를 건넜다.
  - en (AI): An envoy crossed the sea to obtain an elixir of immortality.

### 119. 부화수행 (附和隨行) [seongeo, stdict:153287, draft]

- en_gloss (AI): blindly following others
- en_def (AI): Going along with other people's opinions and acting as they do, without any mind of one's own (lit. chime in and follow along).
- ko_def (stdict, verbatim): 줏대 없이 다른 사람의 주장에만 따라서 그가 하는 짓을 따라 행동함.
- ex (AI Korean): 그는 소신 없이 부화수행하기만 한다.
  - en (AI): He has no convictions of his own and just goes along with whatever others do.
- ex (AI Korean): 부화수행하는 태도로는 좋은 결정을 내릴 수 없다.
  - en (AI): You cannot make good decisions with an attitude of simply following the crowd.

### 120. 화룡점정 (畫龍點睛) [seongeo, stdict:374772, draft]

- en_gloss (AI): the finishing touch
- en_def (AI): Completing the most important part of a task (lit. paint the dragon, dot its eyes).
- ko_def (stdict, verbatim): 무슨 일을 하는 데에 가장 중요한 부분을 완성함을 비유적으로 이르는 말. 용을 그리고 난 후에 마지막으로 눈동자를 그려 넣었더니 그 용이 실제 용이 되어 홀연히 구름을 타고 하늘로 날아 올라갔다는 고사에서 유래한다.
- ex (stdict, verbatim): 깔끔한 반찬과 갓 지은 밥도 만족스러웠지만 화룡점정은 마지막에 나온 직접 담근 식혜였다.
  - en (AI): The neat side dishes and freshly cooked rice were satisfying, but the finishing touch was the homemade sikhye that came out last.
- ex (stdict, verbatim): 이어달리기야말로 운동회의 화룡점정이다.
  - en (AI): The relay race is the crowning touch of the sports day.

### 121. 보편타당 (普遍妥當) [seongeo, stdict:144522, draft]

- en_gloss (AI): universally valid
- en_def (AI): Being sound and proper in a way that is not special to any one case, so that it holds everywhere (lit. universal and fitting).
- ko_def (stdict, verbatim): 특별하지 않고 사리에 맞아 타당함.
- ex (AI Korean): 인권은 보편타당한 가치로 인정받고 있다.
  - en (AI): Human rights are recognized as a universally valid value.
- ex (AI Korean): 그의 주장은 보편타당해서 반박하기 어렵다.
  - en (AI): His argument is universally sound, so it is hard to refute.

### 122. 천인공노 (天人共怒) [seongeo, stdict:327726, draft]

- en_gloss (AI): an outrage to heaven and man
- en_def (AI): Something so hateful or unforgivable that everyone is enraged (lit. heaven and humans are angry together).
- ko_def (stdict, verbatim): 하늘과 사람이 함께 노한다는 뜻으로, 누구나 분노할 만큼 증오스럽거나 도저히 용납할 수 없음을 이르는 말.
- ex (AI Korean): 천인공노할 범죄에 온 국민이 분노했다.
  - en (AI): The whole nation was angered by a crime that outraged heaven and humanity.
- ex (AI Korean): 어린아이를 학대하다니 천인공노할 일이다.
  - en (AI): Abusing a young child is an outrage that no one can forgive.

### 123. 불충불효 (不忠不孝) [seongeo, stdict:159556, draft]

- en_gloss (AI): disloyal and unfilial
- en_def (AI): Failing both in loyalty and in filial devotion (lit. not loyal, not filial).
- ko_def (stdict, verbatim): 충성스럽지 못하고 효성스럽지 못함.
- ex (stdict, verbatim): 제후국 임금으로 천자를 치는 것과 아직 아버지의 상중에 전쟁을 일으키는 것은 불충불효이다.
  - en (AI): For a king of a vassal state to attack the emperor, and to start a war while still in mourning for his father, is to be disloyal and unfilial.
- ex (AI Korean): 부모를 버리고 나라를 등지는 것을 불충불효라 하였다.
  - en (AI): Abandoning one's parents and turning one's back on the country was called being disloyal and unfilial.

### 124. 배산임수 (背山臨水) [seongeo, stdict:138275, draft]

- en_gloss (AI): with mountains behind and water in front
- en_def (AI): A site that has a mountain at its back and faces water (lit. back to the mountain, facing the water).
- ko_def (stdict, verbatim): 지세(地勢)가 뒤로는 산을 등지고 앞으로는 물에 면하여 있음.
- ex (stdict, verbatim): 한국 사람들은 전통적으로 배산임수의 남향집을 선호하였다.
  - en (AI): Koreans traditionally preferred south-facing houses with a mountain behind and water in front.
- ex (AI Korean): 이 마을은 배산임수의 명당에 자리 잡고 있다.
  - en (AI): This village sits on an auspicious site with a mountain behind it and water before it.

### 125. 영원무궁 (永遠無窮) [seongeo, stdict:237834, draft]

- en_gloss (AI): eternal; everlasting
- en_def (AI): Lasting forever, without end (lit. forever, without limit).
- ko_def (stdict, verbatim): 영원하여 끝이 없음.
- ex (AI Korean): 우리의 우정이 영원무궁하기를 바란다.
  - en (AI): I hope our friendship will last forever.
- ex (AI Korean): 신 앞에서 영원무궁한 사랑을 맹세했다.
  - en (AI): They swore an everlasting love before God.

### 126. 점입가경 (漸入佳境) [seongeo, stdict:479148, draft]

- en_gloss (AI): getting better and better as one goes on
- en_def (AI): Becoming more and more interesting the further in one goes; sometimes used ironically for things getting worse (lit. gradually entering a fine place).
- ko_def (stdict, verbatim): 들어갈수록 점점 재미가 있음.
- ex (stdict, verbatim): 설악산은 안으로 깊이 들어갈수록 그 멋이 점입가경이다.
  - en (AI): The deeper into Seoraksan you go, the more splendid it becomes.
- ex (AI Korean): 이야기는 뒤로 갈수록 점입가경이어서 책을 놓을 수 없었다.
  - en (AI): The story got better and better as it went on, so I could not put the book down.

### 127. 금상첨화 (錦上添花) [seongeo, stdict:401345, draft]

- en_gloss (AI): icing on the cake
- en_def (AI): A good thing added on top of another good thing (lit. adding flowers on brocade).
- ko_def (stdict, verbatim): 비단 위에 꽃을 더한다는 뜻으로, 좋은 일 위에 또 좋은 일이 더하여짐을 비유적으로 이르는 말. 왕안석의 글에서 유래한다.
- ex (AI Korean): 맛도 좋은데 값까지 싸다니 금상첨화이다.
  - en (AI): The food is tasty, and it's cheap on top of that: icing on the cake.
- ex (AI Korean): 능력에 인성까지 갖춘 직원이라면 금상첨화다.
  - en (AI): An employee who has good character as well as ability is icing on the cake.

### 128. 도원결의 (桃園結義) [seongeo, stdict:415851, draft]

- en_gloss (AI): sworn brotherhood
- en_def (AI): Becoming sworn brothers, from the oath that Liu Bei, Guan Yu and Zhang Fei made in the peach garden (lit. oath of righteousness in the peach garden).
- ko_def (stdict, verbatim): 의형제를 맺음을 이르는 말. ≪삼국지연의≫에 나오는 말로 유비, 관우, 장비가 도원에서 의형제를 맺은 데에서 유래한다.
- ex (AI Korean): 두 사람은 도원결의를 맺은 것처럼 서로를 형제처럼 아꼈다.
  - en (AI): The two treasured each other like brothers, as though they had sworn an oath of brotherhood.
- ex (AI Korean): 어릴 적 친구들과 도원결의를 하고 평생 우정을 지키기로 했다.
  - en (AI): We swore brotherhood in the peach garden, as it were, with our childhood friends and promised to keep our friendship for life.

### 129. 불원천리 (不遠千里) [seongeo, stdict:436300, draft]

- en_gloss (AI): traveling any distance
- en_def (AI): Not thinking even a thousand li too far to go (lit. not consider a thousand li far).
- ko_def (stdict, verbatim): 천 리 길도 멀다고 여기지 않음.
- ex (AI Korean): 그는 스승을 만나려고 불원천리하고 찾아왔다.
  - en (AI): He came all the way, thinking no distance too great, to meet his teacher.
- ex (AI Korean): 불원천리 달려온 손님들 덕분에 잔치가 빛났다.
  - en (AI): The feast was brightened by the guests who had come from far away.

### 130. 무실역행 (務實力行) [seongeo, stdict:423579, draft]

- en_gloss (AI): striving to put truth into practice
- en_def (AI): Working hard to carry out what is genuine and substantial (lit. devote oneself to what is real, practice with effort).
- ko_def (stdict, verbatim): 참되고 실속 있도록 힘써 실행함.
- ex (AI Korean): 무실역행은 도산 안창호가 강조한 실천 정신이다.
  - en (AI): "Musil yeokhaeng" is the spirit of practice that Dosan Ahn Changho stressed.
- ex (AI Korean): 그는 무실역행의 자세로 맡은 일을 해냈다.
  - en (AI): He carried out his duties with an attitude of earnest, practical effort.

### 131. 운우지정 (雲雨之情) [seongeo, stdict:465314, draft]

- en_gloss (AI): sexual love; lovemaking
- en_def (AI): A euphemism for sexual union between a man and a woman (lit. feelings of clouds and rain, from the dream of King Huai of Chu).
- ko_def (stdict, verbatim): 구름 또는 비와 나누는 정이라는 뜻으로, 남녀의 정교(情交)를 이르는 말. 중국 초나라의 회왕(懷王)이 꿈속에서 어떤 부인과 잠자리를 같이했는데, 그 부인이 떠나면서 자기는 아침에는 구름이 되고 저녁에는 비가 되어 양대(陽臺) 아래에 있겠다고 했다는 고사에서 유래한다.
- ex (AI Korean): 고전 문학에서는 운우지정을 구름과 비에 빗대어 표현했다.
  - en (AI): In classical literature, sexual love was expressed through the metaphor of clouds and rain.
- ex (AI Korean): 시인은 운우지정을 은근히 암시하는 시를 남겼다.
  - en (AI): The poet left a poem that subtly hints at sexual love.

### 132. 일구월심 (日久月深) [seongeo, stdict:270290, draft]

- en_gloss (AI): growing stronger with each passing day
- en_def (AI): Increasing as days and months go by (lit. days grow long, months grow deep).
- ko_def (stdict, verbatim): 날이 오래고 달이 깊어 간다는 뜻으로, 세월이 흐를수록 더함을 이르는 말.
- ex (AI Korean): 고향에 대한 그리움이 일구월심으로 깊어 갔다.
  - en (AI): The longing for home deepened with each passing day.
- ex (AI Korean): 그녀를 향한 마음은 일구월심 더해만 갔다.
  - en (AI): The feelings toward her only grew stronger as time went on.

### 133. 치지도외 (置之度外) [seongeo, stdict:489927, draft]

- en_gloss (AI): disregarding; leaving out of consideration
- en_def (AI): Not taking something to heart, as if setting it beyond the limits of concern (lit. place it outside the bounds).
- ko_def (stdict, verbatim): 마음에 두지 아니함.
- ex (AI Korean): 그는 주변의 비난을 치지도외하고 자기 길을 갔다.
  - en (AI): He disregarded the criticism around him and went his own way.
- ex (AI Korean): 사소한 일은 치지도외해도 좋다.
  - en (AI): Trivial matters can be safely left out of consideration.

### 134. 고색창연 (古色蒼然) [seongeo, stdict:24076, draft]

- en_gloss (AI): venerable and time-worn
- en_def (AI): Looking old and having the quiet charm of antiquity (lit. ancient color, deep green).
- ko_def (stdict, verbatim): 오래되어 예스러운 풍치나 모습이 그윽함.
- ex (AI Korean): 고색창연한 사찰이 산자락에 자리 잡고 있었다.
  - en (AI): A venerable old temple stood at the foot of the mountain.
- ex (AI Korean): 이 도시에는 고색창연한 건물이 많이 남아 있다.
  - en (AI): Many buildings of antique charm remain in this city.

### 135. 백화제방 (百花齊放) [seongeo, stdict:139997, draft]

- en_gloss (AI): letting a hundred flowers bloom
- en_def (AI): Many flowers blooming at once; also, all kinds of learning, art and ideas flourishing together (lit. a hundred flowers bloom together).
- ko_def (stdict, verbatim): 많은 꽃이 일제히 핌.
- ex (AI Korean): 봄이 되니 들판에 백화제방의 풍경이 펼쳐졌다.
  - en (AI): With spring, a scene of a hundred flowers blooming at once spread across the fields.
- ex (AI Korean): 이 시기에는 학문과 예술이 백화제방하며 크게 발전했다.
  - en (AI): In this period learning and the arts flourished side by side and developed greatly.

### 136. 억하심정 (抑何心情) [seongeo, stdict:455811, draft]

- en_gloss (AI): what could be behind it; a deep-seated grudge
- en_def (AI): A phrase asking what on earth someone is feeling, or the bitter feelings deeply lodged in the heart (lit. just what sort of feeling?).
- ko_def (stdict, verbatim): 도대체 무슨 심정이냐는 뜻으로, 무슨 생각으로 그러는지 알 수 없거나 마음속 깊이 맺힌 마음을 이르는 말.
- ex (AI Korean): 나한테 무슨 억하심정이라도 있는 거야?
  - en (AI): Do you have some kind of grudge against me or something?
- ex (AI Korean): 그가 왜 그렇게 나를 헐뜯는지 억하심정을 알 수 없다.
  - en (AI): I cannot fathom what is behind his running me down like that.

### 137. 복지부동 (伏地不動) [seongeo, stdict:147254, draft]

- en_gloss (AI): sitting tight; dodging responsibility
- en_def (AI): Avoiding one's assigned work or duties by keeping one's head down and doing nothing (lit. lie flat on the ground, not moving).
- ko_def (stdict, verbatim): 땅에 엎드려 움직이지 아니한다는 뜻으로, 주어진 일이나 업무를 처리하는 데 몸을 사림을 비유적으로 이르는 말.
- ex (stdict, verbatim): 잘못을 수정하려는 사람은 극소수이고 대부분 복지부동의 태도를 보이고 있다.
  - en (AI): Only a tiny few try to put their mistakes right, and most take an attitude of keeping their heads down.
- ex (stdict, verbatim): 개혁을 이끌어 나가야 할 사람들이 오히려 복지부동이다.
  - en (AI): The very people who should be leading the reform are sitting tight instead.

### 138. 구중궁궐 (九重宮闕) [seongeo, stdict:397934, draft]

- en_gloss (AI): the deep recesses of the royal palace
- en_def (AI): The inside of the palace where the king lives, shut off behind layer upon layer of gates (lit. a palace with nine layers).
- ko_def (stdict, verbatim): 겹겹이 문으로 막은 깊은 궁궐이라는 뜻으로, 임금이 있는 대궐 안을 이르는 말.
- ex (stdict, verbatim): 많은 궁녀들이 구중궁궐에서 쓸쓸히 늙어 갔다.
  - en (AI): Many court ladies grew old in the deep palace in loneliness.
- ex (AI Korean): 임금은 구중궁궐에 갇혀 바깥세상의 사정을 알지 못했다.
  - en (AI): Shut up in the inner palace, the king did not know what was happening in the world outside.

### 139. 동귀어진 (同歸於盡) [seongeo, opendict:1322592, draft]

- en_gloss (AI): mutual destruction; dying together with one's enemy
- en_def (AI): Ending things by dying together with another person (lit. return together to ruin).
- ko_def (opendict, verbatim): 파멸로 함께 돌아간다는 뜻으로, 다른 사람과 같이 죽음으로써 끝장을 냄을 이르는 말.
- ex (AI Korean): 그는 적과 동귀어진하겠다는 각오로 칼을 뽑았다.
  - en (AI): Resolved to perish together with the enemy, he drew his sword.
- ex (AI Korean): 협상이 깨지면 양쪽 모두 동귀어진의 길을 걷게 될 것이다.
  - en (AI): If the negotiations collapse, both sides will go down the path of mutual destruction.

### 140. 천리만리 (千里萬里) [seongeo, stdict:491241, draft]

- en_gloss (AI): extremely far away
- en_def (AI): A very long distance, a thousand or even ten thousand li away (lit. a thousand li, ten thousand li).
- ko_def (stdict, verbatim): 멀기가 천 리 또는 만 리나 된다는 뜻으로, 아주 먼 거리를 이르는 말.
- ex (stdict, verbatim): 네가 가는 곳이라면 천리만리라도 따라가겠다.
  - en (AI): I will follow you wherever you go, even if it is a thousand miles away.
- ex (AI Korean): 천리만리 떨어진 고향 생각에 밤새 잠을 이루지 못했다.
  - en (AI): Thinking of my hometown, so very far away, I could not sleep all night.

### 141. 경이원지 (敬而遠之) [seongeo, stdict:19086, draft]

- en_gloss (AI): keeping a respectful distance
- en_def (AI): Respecting someone but not getting close to them; also, appearing to respect while actually shunning them (lit. respect it, yet keep far from it).
- ko_def (stdict, verbatim): 공경하되 가까이하지는 않음.
- ex (AI Korean): 그 교수님은 학생들이 경이원지하는 분이라 가까이 가기 어렵다.
  - en (AI): That professor is someone the students keep a respectful distance from, so it is hard to approach.
- ex (AI Korean): 사람들은 그를 경이원지하며 속마음을 털어놓지 않았다.
  - en (AI): People kept their distance from him with wary respect and did not open up to him.

### 142. 창황망조 (蒼黃罔措) [seongeo, stdict:324383, draft]

- en_gloss (AI): flustered and at a loss
- en_def (AI): So hurried and pressed that one cannot do anything about it (lit. hurried and pale, not knowing what to do).
- ko_def (stdict, verbatim): 너무 급하여 어찌할 수가 없음.
- ex (AI Korean): 갑작스러운 화재에 사람들은 창황망조하며 뛰쳐나갔다.
  - en (AI): At the sudden fire, people rushed out in a panic, flustered and at a loss.
- ex (AI Korean): 준비도 못 했는데 손님이 들이닥쳐 창황망조했다.
  - en (AI): I hadn't prepared at all and guests burst in, so I was flustered and didn't know what to do.

### 143. 분골쇄신 (粉骨碎身) [seongeo, stdict:438357, draft]

- en_gloss (AI): working oneself to the bone
- en_def (AI): Making every effort with all one's heart, as though grinding one's bones to powder and breaking one's body (lit. pulverize the bones, shatter the body).
- ko_def (stdict, verbatim): 뼈를 가루로 만들고 몸을 부순다는 뜻으로, 정성으로 노력함을 이르는 말. 또는 그렇게 하여 뼈가 가루가 되고 몸이 부서짐.
- ex (AI Korean): 그는 나라를 위해 분골쇄신하겠다고 다짐했다.
  - en (AI): He vowed to work himself to the bone for his country.
- ex (AI Korean): 그는 분골쇄신의 노력으로 회사를 일으켜 세웠다.
  - en (AI): With tireless effort, as if grinding himself to dust, he built the company up.

### 144. 고육지책 (苦肉之策) [seongeo, stdict:25625, draft]

- en_gloss (AI): a desperate measure
- en_def (AI): A scheme one is forced to devise, at some cost to oneself, to get out of a difficult situation (lit. a plan of bitter flesh, i.e. hurting oneself).
- ko_def (stdict, verbatim): 자기 몸을 상해 가면서까지 꾸며 내는 계책이라는 뜻으로, 어려운 상태를 벗어나기 위해 어쩔 수 없이 꾸며 내는 계책을 이르는 말.
- ex (stdict, verbatim): 지겨운 가난을 면하기 위해 고육지책으로 개가해 간 게 오히려 고생문으로 자청해 들어간 꼴이 되었다.
  - en (AI): Remarrying as a last resort to escape dreary poverty turned out to be walking, of one's own accord, into a life of hardship.
- ex (stdict, verbatim): 정부는 가만히 놔뒀다가는 배춧값의 폭락이 뻔하기 때문에 농민들의 피해가 더 커지기 전에 공급량을 줄인다는 고육지책을 쓴 것이다.
  - en (AI): Since cabbage prices were bound to collapse if left alone, the government used the desperate measure of cutting supply before farmers' losses grew.

### 145. 백화요란 (百花燎亂) [seongeo, stdict:139096, draft]

- en_gloss (AI): a blaze of blossoms
- en_def (AI): All kinds of flowers blooming splendidly, as though ablaze (lit. a hundred flowers burning in profusion).
- ko_def (stdict, verbatim): 온갖 꽃이 불이 타오르듯이 피어 매우 화려함.
- ex (AI Korean): 봄 산에는 진달래와 철쭉이 백화요란하게 피어 있었다.
  - en (AI): Azaleas and royal azaleas bloomed in a blaze of color on the spring mountains.
- ex (AI Korean): 정원에 백화요란한 꽃들이 가득해 눈이 부실 지경이었다.
  - en (AI): The garden was so full of dazzling, blazing flowers that it was almost blinding.

### 146. 쾌도난마 (快刀亂麻) [seongeo, stdict:492773, draft]

- en_gloss (AI): cutting through a tangle decisively
- en_def (AI): Dealing with a hopelessly tangled matter clearly and powerfully (lit. a sharp knife through snarled hemp).
- ko_def (stdict, verbatim): 잘 드는 칼로 마구 헝클어진 삼 가닥을 자른다는 뜻으로, 어지럽게 뒤얽힌 사물을 강력한 힘으로 명쾌하게 처리함을 이르는 말.
- ex (stdict, verbatim): 신임 회장은 산적한 문제들을 쾌도난마로 처리했다.
  - en (AI): The new chairman dealt decisively with the mountain of problems.
- ex (AI Korean): 형사는 쾌도난마의 추리로 복잡한 사건을 해결했다.
  - en (AI): The detective cut through the tangled case with incisive reasoning.

### 147. 노류장화 (路柳牆花) [seongeo, stdict:65900, draft]

- en_gloss (AI): a roadside willow and wall flower (derogatory: a prostitute)
- en_def (AI): A willow by the road and a flower by the wall that anyone can pluck, used disparagingly of a prostitute or courtesan (lit. roadside willow, wall flower).
- ko_def (stdict, verbatim): 아무나 쉽게 꺾을 수 있는 길가의 버들과 담 밑의 꽃이라는 뜻으로, 창녀나 기생을 비유적으로 이르는 말.
- ex (AI Korean): 옛 시에서는 기생을 노류장화에 빗대어 노래하기도 했다.
  - en (AI): In old poems, courtesans were sometimes likened to the roadside willow and the flower by the wall.
- ex (AI Korean): 노류장화라는 표현에는 신분 차별적인 시각이 담겨 있다.
  - en (AI): The expression "roadside willow and wall flower" carries a class-discriminating outlook.

### 148. 비상지원 (飛霜之怨) [seongeo, stdict:157957, draft]

- en_gloss (AI): the bitterness of a wrongful imprisonment
- en_def (AI): Resentment over being unjustly jailed, from the story of Zou Yan, whose cry to heaven brought frost in midsummer (lit. the grievance of flying frost).
- ko_def (stdict, verbatim): 억울한 옥살이를 이르는 말. 중국 연(燕)나라의 추연(鄒衍)이 억울하게 하옥되었을 때 하늘을 우러르며 통곡하자 한여름인데도 서리가 내렸다는 고사에서 유래한다.
- ex (AI Korean): 그는 비상지원을 품은 채 옥에서 풀려났다.
  - en (AI): He was released from prison still harboring the bitterness of the wrongly jailed.
- ex (AI Korean): 억울하게 갇힌 선비의 비상지원이 시로 남았다.
  - en (AI): The bitter grievance of the scholar who had been unjustly jailed was left behind in a poem.

### 149. 이견대인 (利見大人) [seongeo, opendict:1314774, draft]

- en_gloss (AI): it is good to see a great person
- en_def (AI): It is beneficial to meet someone of outstanding wisdom and virtue who is worth emulating, a phrase from the Book of Changes (lit. it is advantageous to see a great person).
- ko_def (opendict, verbatim): 대인을 만나면 이롭다는 뜻으로, 지혜와 덕이 뛰어나 본받을 만한 사람을 만나는 것이 좋음을 이르는 말.
- ex (AI Korean): 이견대인이라 했으니 훌륭한 스승을 찾아가 가르침을 구해야 한다.
  - en (AI): As the phrase "it is good to see a great person" has it, you should seek out a fine teacher and ask for guidance.
- ex (AI Korean): 어려울 때일수록 이견대인의 마음으로 지혜로운 분을 찾아뵙는 것이 좋다.
  - en (AI): The harder the times, the better it is to go and call on a wise person, in the spirit of seeking out a great person.

### 150. 전인미답 (前人未踏) [seongeo, stdict:291593, draft]

- en_gloss (AI): untrodden; never reached before
- en_def (AI): A place no one has ever gone to, or a field no one has ever touched (lit. no one before has trodden).
- ko_def (stdict, verbatim): 이제까지 그 누구도 가 보지 못함.
- ex (AI Korean): 그는 전인미답의 고봉에 올랐다.
  - en (AI): He climbed a high peak that no one had ever reached.
- ex (AI Korean): 이 연구는 전인미답의 분야를 개척했다.
  - en (AI): This research broke new ground in a field no one had entered before.

### 151. 촌철살인 (寸鐵殺人) [seongeo, stdict:327689, draft]

- en_gloss (AI): a pithy, cutting remark
- en_def (AI): A brief remark that moves people or strikes at someone's weak point (lit. an inch of iron can kill a person).
- ko_def (stdict, verbatim): 한 치의 쇠붙이로도 사람을 죽일 수 있다는 뜻으로, 간단한 말로도 남을 감동하게 하거나 남의 약점을 찌를 수 있음을 이르는 말.
- ex (stdict, verbatim): 선배는 우리의 작품을 차례로 보며 촌철살인의 평을 내놓았다.
  - en (AI): The senior looked over our works one by one and gave sharply incisive comments.
- ex (AI Korean): 그의 촌철살인 한마디에 회의장이 조용해졌다.
  - en (AI): His one cutting remark silenced the meeting room.

### 152. 백전백패 (百戰百敗) [seongeo, stdict:138804, draft]

- en_gloss (AI): losing every time
- en_def (AI): Being beaten in every battle (lit. a hundred battles, a hundred defeats).
- ko_def (stdict, verbatim): 싸울 때마다 다 짐.
- ex (AI Korean): 그 팀은 올해 백전백패의 불명예를 안았다.
  - en (AI): The team took on the disgrace of losing every single game this year.
- ex (AI Korean): 준비 없이 시합에 나가면 백전백패다.
  - en (AI): If you go into a match without preparation, you'll lose every time.

### 153. 오합지중 (烏合之衆) [seongeo, stdict:239214, draft]

- en_gloss (AI): a disorderly rabble
- en_def (AI): A crowd gathered in haste, without rules or order (lit. a crowd gathered like crows).
- ko_def (stdict, verbatim): 까마귀가 모인 것처럼 질서가 없이 모인 병졸이라는 뜻으로, 임시로 모여들어서 규율이 없고 무질서한 병졸 또는 군중을 이르는 말.
- ex (AI Korean): 오합지중을 이끌고는 전쟁에서 이길 수 없다.
  - en (AI): You cannot win a war leading a disorderly rabble.
- ex (AI Korean): 훈련이 안 된 오합지중이라 적의 공격에 금세 흩어졌다.
  - en (AI): They were an untrained rabble and scattered at the enemy's first attack.

### 154. 풍찬노숙 (風餐露宿) [seongeo, stdict:360421, draft]

- en_gloss (AI): roughing it; enduring hardship far from home
- en_def (AI): Suffering great hardship while away from home (lit. eating the wind, sleeping in the dew).
- ko_def (stdict, verbatim): 바람을 먹고 이슬에 잠잔다는 뜻으로, 객지에서 많은 고생을 겪음을 이르는 말.
- ex (AI Korean): 독립운동가들은 풍찬노숙을 견디며 광복을 위해 싸웠다.
  - en (AI): The independence fighters endured exposure and hardship as they fought for liberation.
- ex (AI Korean): 그는 풍찬노숙하며 전국을 돌아다녔다.
  - en (AI): He wandered the whole country, roughing it all the way.

### 155. 고심참담 (苦心慘憺) [seongeo, stdict:24334, draft]

- en_gloss (AI): agonizing; torturous worry
- en_def (AI): Burning with anxiety and struggling hard, full of worry (lit. bitter heart, miserable and dismal).
- ko_def (stdict, verbatim): 몹시 마음을 태우며 애를 쓰면서 걱정을 함.
- ex (AI Korean): 고심참담 끝에 그는 사업을 접기로 결심했다.
  - en (AI): After agonizing over it, he decided to close down the business.
- ex (AI Korean): 아이의 병을 고치려고 고심참담하는 부모의 모습이 안타까웠다.
  - en (AI): It was painful to see the parents agonizing over how to cure their child's illness.

### 156. 여합부절 (如合符節) [seongeo, stdict:459604, draft]

- en_gloss (AI): fitting perfectly
- en_def (AI): Things matching exactly (lit. like the halves of a tally fitting together).
- ko_def (stdict, verbatim): 사물이 꼭 들어맞음.
- ex (AI Korean): 두 사람의 증언이 여합부절로 맞아떨어졌다.
  - en (AI): The two people's testimonies matched up perfectly.
- ex (AI Korean): 그의 예측은 결과와 여합부절이었다.
  - en (AI): His prediction fitted the outcome exactly.

### 157. 용혹무괴 (容或無怪) [seongeo, stdict:247682, draft]

- en_gloss (AI): not at all surprising
- en_def (AI): Nothing strange about it, even if it should happen (lit. even should it happen, nothing odd).
- ko_def (stdict, verbatim): 혹시 그런 일이 있더라도 괴이할 것이 없음.
- ex (AI Korean): 그 정도 실력이면 우승해도 용혹무괴다.
  - en (AI): With that level of skill, it wouldn't be surprising if he won.
- ex (AI Korean): 장마철에는 하루 종일 비가 와도 용혹무괴하다.
  - en (AI): In the rainy season, it would not be strange if it rained all day.

### 158. 인홀불견 (因忽不見) [seongeo, stdict:270563, draft]

- en_gloss (AI): vanishing suddenly
- en_def (AI): Being glimpsed for a moment and then suddenly gone (lit. suddenly, no longer seen).
- ko_def (stdict, verbatim): 언뜻 보이다가 갑자기 없어짐.
- ex (stdict, verbatim): 비방을 알려 준 도승은 밭둑길을 가고 있는 듯했으나 어느새 인홀불견이었다.
  - en (AI): The Taoist monk who told me the secret remedy seemed to be walking along the path between the fields, but in no time he had vanished.
- ex (AI Korean): 안개 속으로 걸어간 사내는 인홀불견이 되어 다시는 나타나지 않았다.
  - en (AI): The man who walked into the fog vanished suddenly and never appeared again.

### 159. 건곤일척 (乾坤一擲) [seongeo, stdict:13044, draft]

- en_gloss (AI): staking everything on one throw
- en_def (AI): Betting everything on a single decisive contest (lit. a single throw of heaven and earth).
- ko_def (stdict, verbatim): 천하를 두고 한 번에 모든 것을 건다는 뜻으로, 운명을 걸고 단판걸이로 승패를 겨룸을 이르는 말. 한유의 <과홍구(過鴻溝)>에 나오는 말이다.
- ex (AI Korean): 두 팀은 결승전에서 건곤일척의 승부를 펼쳤다.
  - en (AI): The two teams played an all-or-nothing match in the final.
- ex (AI Korean): 그는 건곤일척의 각오로 사업에 전 재산을 걸었다.
  - en (AI): With a do-or-die resolve, he staked his entire fortune on the business.

### 160. 공평무사 (公平無私) [seongeo, stdict:30681, draft]

- en_gloss (AI): fair and impartial
- en_def (AI): Being just, without any private favoritism (lit. fair, with no private interest).
- ko_def (stdict, verbatim): 공평하여 사사로움이 없음.
- ex (AI Korean): 심사위원들은 공평무사하게 점수를 매겼다.
  - en (AI): The judges scored fairly and without favoritism.
- ex (AI Korean): 재판관은 공평무사한 판결로 신뢰를 얻었다.
  - en (AI): The judge earned trust through impartial rulings.

### 161. 일석삼조 (一石三鳥) [seongeo, opendict:834528, draft]

- en_gloss (AI): killing three birds with one stone
- en_def (AI): Gaining three benefits at the same time (lit. one stone, three birds).
- ko_def (opendict, verbatim): 돌 한 개를 던져 새 세 마리를 잡는다는 뜻으로, 동시에 세 가지 이득을 봄을 이르는 말.
- ex (AI Korean): 운동도 하고 친구도 만나고 건강도 챙기니 일석삼조다.
  - en (AI): I exercise, meet friends and look after my health too, so it's killing three birds with one stone.
- ex (AI Korean): 이 프로그램은 학습과 놀이와 휴식을 한꺼번에 얻는 일석삼조의 효과가 있다.
  - en (AI): This program has a three-in-one effect: you get learning, play and rest all at once.

### 162. 백년지객 (百年之客) [seongeo, stdict:140924, draft]

- en_gloss (AI): son-in-law (as a lifelong honored guest)
- en_def (AI): A term for a son-in-law, who is always treated as a guest who must be received with care (lit. a guest for a hundred years).
- ko_def (stdict, verbatim): 한평생을 두고 늘 어려운 손님으로 맞이한다는 뜻으로, ‘사위01’를 이르는 말.
- ex (AI Korean): 장모님은 사위를 백년지객으로 극진히 대접하셨다.
  - en (AI): The mother-in-law treated her son-in-law with great courtesy, as a lifelong honored guest.
- ex (AI Korean): 옛말에 사위는 백년지객이라 하여 늘 조심스럽게 대했다.
  - en (AI): An old saying calls a son-in-law a guest of a hundred years, and he was always treated with care.

### 163. 종두득두 (種豆得豆) [seongeo, stdict:303808, draft]

- en_gloss (AI): you reap what you sow
- en_def (AI): If you plant beans, you get beans; results follow from their causes (lit. plant beans, get beans).
- ko_def (stdict, verbatim): 콩을 심으면 반드시 콩이 나온다는 뜻으로, 원인에 따라 결과가 생김을 이르는 말.
- ex (AI Korean): 종두득두라더니 열심히 공부한 만큼 성적이 나왔다.
  - en (AI): You reap what you sow, as they say: my grades matched how hard I studied.
- ex (AI Korean): 종두득두의 이치를 모르는 사람은 노력 없이 결과를 바란다.
  - en (AI): Those who don't understand that you reap what you sow expect results without effort.

### 164. 호연지기 (浩然之氣) [seongeo, stdict:504666, draft]

- en_gloss (AI): lofty, boundless spirit
- en_def (AI): A great, vast vital energy filling heaven and earth, or a free and expansive spirit (lit. vast, flowing spirit).
- ko_def (stdict, verbatim): 하늘과 땅 사이에 가득 찬 넓고 큰 원기. ≪맹자≫ <공손추(公孫丑)>의 상편에 나오는 말이다.
- ex (AI Korean): 청년들은 호연지기를 기르기 위해 산을 올랐다.
  - en (AI): Young people climbed mountains to cultivate a bold, expansive spirit.
- ex (AI Korean): 넓은 바다를 보니 가슴 속에 호연지기가 솟아올랐다.
  - en (AI): Seeing the wide sea, I felt a lofty, boundless spirit rise in my chest.

### 165. 백절불굴 (百折不屈) [seongeo, stdict:431173, draft]

- en_gloss (AI): indomitable; unyielding
- en_def (AI): Never giving in to any difficulty (lit. a hundred breaks, never bending).
- ko_def (stdict, verbatim): 어떠한 난관에도 결코 굽히지 않음.
- ex (AI Korean): 그는 백절불굴의 의지로 병마를 이겨 냈다.
  - en (AI): With an unyielding will, he overcame his illness.
- ex (AI Korean): 백절불굴의 정신이 있어야 큰일을 이룬다.
  - en (AI): You need an indomitable spirit to achieve great things.

### 166. 삼분천하 (三分天下) [seongeo, stdict:173885, draft]

- en_gloss (AI): the realm divided in three
- en_def (AI): A state of affairs in which a country is divided among three rulers or heroes (lit. divide the world into three).
- ko_def (stdict, verbatim): 한 나라를 세 사람의 군주나 영걸(英傑)이 나누어 차지함.
- ex (AI Korean): 삼국지는 삼분천하의 혼란을 배경으로 한다.
  - en (AI): The Romance of the Three Kingdoms is set against the turmoil of the realm divided in three.
- ex (AI Korean): 세 세력이 맞서 삼분천하의 형세를 이루었다.
  - en (AI): The three forces confronted each other and formed a situation of the realm divided in three.

### 167. 유유낙낙 (唯唯諾諾) [seongeo, stdict:263409, draft]

- en_gloss (AI): meek obedience
- en_def (AI): Obeying whatever one is told, always saying yes (lit. yes, yes, all right, all right).
- ko_def (stdict, verbatim): 명령하는 대로 순종함.
- ex (AI Korean): 그는 상사의 말이라면 유유낙낙 따르기만 한다.
  - en (AI): If it is something his boss says, he just meekly goes along with it.
- ex (AI Korean): 유유낙낙하는 부하들 사이에서 그는 외로웠다.
  - en (AI): He felt lonely among subordinates who just said yes to everything.

### 168. 상명하복 (上命下服) [seongeo, stdict:515814, draft]

- en_gloss (AI): orders from above, obedience below
- en_def (AI): The lower ranks obey whatever the upper ranks command, so the chain of command is clear (lit. orders from above, submission below).
- ko_def (stdict, verbatim): 위에서 명령하면 아래에서는 복종한다는 뜻으로, 상하 관계가 분명함을 이르는 말.
- ex (AI Korean): 군대는 상명하복의 질서가 엄격한 조직이다.
  - en (AI): The army is an organization with a strict order of command from above and obedience below.
- ex (AI Korean): 상명하복의 문화가 창의적인 의견을 막기도 한다.
  - en (AI): A culture of top-down orders and bottom-up obedience can stifle creative opinions.

### 169. 거국일치 (擧國一致) [seongeo, stdict:394354, draft]

- en_gloss (AI): the whole nation united as one
- en_def (AI): All the people of a country coming together as one (lit. the whole country in agreement).
- ko_def (stdict, verbatim): 온 국민이 뭉쳐서 하나가 됨.
- ex (stdict, verbatim): 거국일치의 확고한 결의로 통일을 위해 함께 나아가야 할 것이다.
  - en (AI): We shall have to go forward together toward unification with the firm resolve of a whole nation united as one.
- ex (AI Korean): 위기를 극복하려면 거국일치로 힘을 모아야 한다.
  - en (AI): To overcome the crisis, the whole nation must unite and pool its strength.

### 170. 경세제민 (經世濟民) [seongeo, stdict:19005, draft]

- en_gloss (AI): governing the world and relieving the people
- en_def (AI): Ruling the world well and saving the people from distress (lit. manage the world, relieve the people).
- ko_def (stdict, verbatim): 세상을 다스리고 백성을 구제함.
- ex (AI Korean): 선비는 경세제민의 뜻을 품고 학문에 정진했다.
  - en (AI): The scholar, aiming to govern the world and relieve the people, devoted effort to learning.
- ex (AI Korean): 경제는 본래 경세제민에서 나온 말이다.
  - en (AI): The word for economy originally came from the phrase meaning governing the world and relieving the people.

### 171. 일시동인 (一視同仁) [seongeo, stdict:271803, draft]

- en_gloss (AI): treating everyone alike with equal kindness
- en_def (AI): Treating all people equally and loving them the same, whether close or distant (lit. look on all with one eye, with the same benevolence).
- ko_def (stdict, verbatim): 멀고 가까운 사람을 친함에 관계없이 똑같이 대하여 준다는 뜻으로, 성인이 누구나 평등하게 똑같이 사랑함을 이르는 말. 한유의 <원인(原人)>에 나오는 말이다.
- ex (AI Korean): 성군은 일시동인의 마음으로 백성을 대했다.
  - en (AI): The wise king treated the people with equal kindness to all.
- ex (AI Korean): 선생님은 일시동인하여 모든 학생에게 똑같이 관심을 기울였다.
  - en (AI): The teacher treated everyone alike and gave all the students the same attention.

### 172. 일도양단 (一刀兩斷) [seongeo, stdict:272443, draft]

- en_gloss (AI): cutting in two with one stroke; decisive action
- en_def (AI): Cutting something in two with one blow of a sword; also, deciding a matter promptly without hesitation (lit. one blade, two pieces).
- ko_def (stdict, verbatim): 칼로 무엇을 대번에 쳐서 두 도막을 냄.
- ex (AI Korean): 그는 어려운 문제도 일도양단으로 결정을 내렸다.
  - en (AI): He made a decision even on a difficult problem in one decisive stroke.
- ex (AI Korean): 일도양단하는 결단력 덕분에 회사는 위기를 넘겼다.
  - en (AI): Thanks to the decisiveness of his clean cuts, the company got past the crisis.

### 173. 금의옥식 (錦衣玉食) [seongeo, stdict:405055, draft]

- en_gloss (AI): a life of luxury
- en_def (AI): A luxurious, extravagant life, in silk clothes and fine white rice (lit. brocade clothes, jade food).
- ko_def (stdict, verbatim): 비단옷과 흰쌀밥이라는 뜻으로, 호화스럽고 사치스러운 생활을 이르는 말.
- ex (AI Korean): 그는 금의옥식의 호사스러운 생활을 하며 살았다.
  - en (AI): He lived a life of luxury, in silk clothes and on the finest food.
- ex (AI Korean): 금의옥식을 누리던 귀족도 전쟁 앞에서는 속수무책이었다.
  - en (AI): Even nobles who had enjoyed a life of luxury were helpless in the face of war.

### 174. 무두무미 (無頭無尾) [seongeo, stdict:119542, draft]

- en_gloss (AI): without head or tail; incoherent
- en_def (AI): Having neither beginning nor end, so that it makes no sense (lit. no head, no tail).
- ko_def (stdict, verbatim): 머리도 꼬리도 없다는 뜻으로, 밑도 끝도 없음을 이르는 말.
- ex (AI Korean): 그의 변명은 무두무미하여 알아들을 수가 없었다.
  - en (AI): His excuse was so incoherent that I could not make sense of it.
- ex (AI Korean): 무두무미한 이야기에 사람들은 고개를 갸웃거렸다.
  - en (AI): People tilted their heads at the story that had neither beginning nor end.

### 175. 소양지판 (霄壤之判) [seongeo, stdict:447377, draft]

- en_gloss (AI): a world of difference
- en_def (AI): Things are enormously different from each other, as different as heaven and earth (lit. the gap between sky and earth).
- ko_def (stdict, verbatim): 하늘과 땅 사이의 차이라는 뜻으로, 사물들이 서로 엄청나게 다름을 이르는 말.
- ex (AI Korean): 두 도시의 생활 수준은 소양지판이다.
  - en (AI): The standard of living in the two cities is as different as heaven and earth.
- ex (AI Korean): 같은 재료로 만든 요리인데도 맛이 소양지판이었다.
  - en (AI): Though both dishes were made with the same ingredients, the taste was a world apart.

### 176. 폐포파립 (敝袍破笠) [seongeo, stdict:359267, draft]

- en_gloss (AI): shabby clothes
- en_def (AI): A shabby appearance, in worn-out clothes and a broken hat (lit. torn coat, ruined hat).
- ko_def (stdict, verbatim): 해어진 옷과 부서진 갓이란 뜻으로, 초라한 차림새를 비유적으로 이르는 말.
- ex (AI Korean): 그는 폐포파립의 차림으로 고향에 돌아왔다.
  - en (AI): He returned to his hometown in tattered clothes.
- ex (AI Korean): 폐포파립의 노인이 문 앞에서 밥을 청했다.
  - en (AI): An old man in rags asked for food at the door.

### 177. 무불통지 (無不通知) [seongeo, stdict:122469, draft]

- en_gloss (AI): knowing everything
- en_def (AI): Understanding every matter thoroughly, with nothing unknown (lit. nothing is not understood).
- ko_def (stdict, verbatim): 무슨 일이든지 환히 통하여 모르는 것이 없음.
- ex (AI Korean): 그 박사는 무불통지로 알려져 모르는 것이 없다고 한다.
  - en (AI): That scholar is known for knowing everything; they say nothing is left unknown.
- ex (AI Korean): 어떤 분야든 무불통지한 사람은 드물다.
  - en (AI): Few people know everything in every field.

### 178. 명견만리 (明見萬里) [seongeo, stdict:116444, draft]

- en_gloss (AI): farsightedness
- en_def (AI): Looking ten thousand li ahead, i.e. having very accurate and outstanding powers of observation and judgment (lit. see clearly ten thousand li).
- ko_def (stdict, verbatim): 만 리 앞을 내다본다는 뜻으로, 관찰력이나 판단력이 매우 정확하고 뛰어남을 이르는 말.
- ex (AI Korean): 명견만리의 안목으로 미래를 내다본 사람이 성공한다.
  - en (AI): Those who look into the future with far-seeing vision succeed.
- ex (AI Korean): 그 장군은 명견만리의 판단으로 전쟁의 흐름을 읽었다.
  - en (AI): That general read the course of the war with farsighted judgment.

### 179. 천언만어 (千言萬語) [seongeo, stdict:320147, draft]

- en_gloss (AI): a flood of words
- en_def (AI): An endless number of words spoken (lit. a thousand words, ten thousand sayings).
- ko_def (stdict, verbatim): 천 마디 만 마디 말이라는 뜻으로, 수없이 많이 하는 말을 이르는 말.
- ex (AI Korean): 천언만어를 쏟아부어도 그의 마음을 돌릴 수 없었다.
  - en (AI): No matter how many words I poured out, I couldn't change his mind.
- ex (AI Korean): 천언만어가 필요 없을 만큼 그의 표정이 모든 것을 말해 주었다.
  - en (AI): His expression said it all, so there was no need for a flood of words.

### 180. 일타쌍피 (一打雙皮) [seongeo, opendict:1369078, draft]

- en_gloss (AI): killing two birds with one stone
- en_def (AI): Gaining two benefits from a single action; from the card game go-stop, taking two junk cards with one card played (lit. one play, two junk cards).
- ko_def (opendict, verbatim): 고스톱에서 화투짝 한 장을 내고 피 두 장을 가져온다는 뜻으로, 한 가지 일을 하여 두 가지 이익을 봄을 이르는 말.
- ex (AI Korean): 출근길에 은행 일을 처리했으니 일타쌍피다.
  - en (AI): I took care of my bank errand on the way to work, so that was killing two birds with one stone.
- ex (AI Korean): 청소하면서 운동도 되니 일타쌍피가 따로 없다.
  - en (AI): I get exercise while cleaning, so it really is killing two birds with one stone.

### 181. 경적필패 (輕敵必敗) [seongeo, stdict:19267, draft]

- en_gloss (AI): underestimating the enemy means defeat
- en_def (AI): If you look down on your enemy, you are certain to lose (lit. slight the enemy, surely lose).
- ko_def (stdict, verbatim): 적을 얕보면 반드시 패함.
- ex (AI Korean): 경적필패라고 상대를 얕보다가 결국 지고 말았다.
  - en (AI): They say that underestimating the enemy means certain defeat, and sure enough, I looked down on my opponent and lost in the end.
- ex (AI Korean): 경적필패의 교훈을 잊지 말고 끝까지 최선을 다하자.
  - en (AI): Let's not forget the lesson that looking down on the enemy means defeat, and do our best to the end.

### 182. 득의만면 (得意滿面) [seongeo, stdict:416862, draft]

- en_gloss (AI): beaming with satisfaction
- en_def (AI): A face full of delight because things have gone as one wished (lit. pleased, the whole face full).
- ko_def (stdict, verbatim): 일이 뜻대로 이루어져 기쁜 표정이 얼굴에 가득함.
- ex (AI Korean): 합격 소식을 들은 그는 득의만면한 표정을 지었다.
  - en (AI): On hearing he had passed, he put on a beaming, triumphant expression.
- ex (AI Korean): 득의만면하던 그의 얼굴이 순식간에 굳어졌다.
  - en (AI): His face, which had been beaming with satisfaction, froze in an instant.

### 183. 일부종사 (一夫從事) [seongeo, stdict:270724, draft]

- en_gloss (AI): remaining faithful to one husband
- en_def (AI): Traditionally, a woman serving only one husband all her life, never remarrying (lit. serve one husband).
- ko_def (stdict, verbatim): 한 남편만을 섬김.
- ex (stdict, verbatim): 그 여자는 일부종사를 못 하고 다시 결혼했다.
  - en (AI): That woman did not stay with one husband for life but married again.
- ex (AI Korean): 옛날에는 여자에게 일부종사를 강요하는 풍습이 있었다.
  - en (AI): In the past there was a custom of forcing women to remain faithful to a single husband.

### 184. 천의무봉 (天衣無縫) [seongeo, stdict:487732, draft]

- en_gloss (AI): flawless and natural
- en_def (AI): Perfect in a natural, beautiful way, with no sign of contrivance, as a heavenly robe has no seams (lit. a heavenly garment has no seam).
- ko_def (stdict, verbatim): 천사의 옷은 꿰맨 흔적이 없다는 뜻으로, 일부러 꾸민 데 없이 자연스럽고 아름다우면서 완전함을 이르는 말. ≪태평광기≫의 곽한(郭翰)의 이야기에 나오는 말로, 주로 시가(詩歌)나 문장에 대하여 이르는 말이다.
- ex (stdict, verbatim): 아무리 뛰어난 작가의 시라 하여도 모두 천의무봉의 비단결만은 아니다.
  - en (AI): Even the poems of the finest writers are not all seamless, flawless silk.
- ex (AI Korean): 그 피아니스트의 연주는 천의무봉이라는 말이 어울렸다.
  - en (AI): The word "flawless" suited that pianist's performance.

### 185. 막역지우 (莫逆之友) [seongeo, stdict:109155, draft]

- en_gloss (AI): a bosom friend
- en_def (AI): A very close friend with whom there is no need for reserve (lit. a friend with whom one has no disagreement).
- ko_def (stdict, verbatim): 서로 거스름이 없는 친구라는 뜻으로, 허물없이 아주 친한 친구를 이르는 말.
- ex (stdict, verbatim): 그와는 어려서는 싸움도 많이 하였지만 뜻이 맞는 유일한 막역지우였다.
  - en (AI): He and I had many fights when we were young, but he was my one close friend who saw eye to eye with me.
- ex (AI Korean): 어려울 때 곁을 지켜 준 그는 평생의 막역지우였다.
  - en (AI): He, who stood by me in hard times, was a bosom friend for life.

### 186. 창가소부 (娼家笑婦) [seongeo, stdict:487986, draft]

- en_gloss (AI): prostitute (derogatory)
- en_def (AI): A derogatory term for a prostitute (lit. a woman who laughs in a brothel).
- ko_def (stdict, verbatim): 창기(娼妓)의 집에서 웃음을 파는 여자라는 뜻으로, ‘창녀’를 이르는 말.
- ex (AI Korean): 옛 문헌에는 창가소부라는 표현이 쓰였다.
  - en (AI): The expression "changga sobu" was used in old texts.
- ex (AI Korean): 그 소설은 창가소부의 삶을 동정 어린 시선으로 그렸다.
  - en (AI): The novel depicted the lives of prostitutes with a sympathetic eye.

### 187. 춘풍추우 (春風秋雨) [seongeo, stdict:493349, draft]

- en_gloss (AI): years gone by
- en_def (AI): The passage of time that has gone by (lit. spring breeze, autumn rain).
- ko_def (stdict, verbatim): 봄바람과 가을비라는 뜻으로, 지나간 세월을 이르는 말.
- ex (AI Korean): 춘풍추우를 견디며 지나온 세월을 돌이켜 보았다.
  - en (AI): I looked back on the years that had passed, enduring through spring breezes and autumn rains.
- ex (AI Korean): 춘풍추우 몇 해가 흘러 아이는 어른이 되었다.
  - en (AI): Spring breezes and autumn rains passed for a number of years, and the child grew up.

### 188. 혼정신성 (昏定晨省) [seongeo, stdict:502209, draft]

- en_gloss (AI): caring for one's parents morning and night
- en_def (AI): Making up one's parents' bed at night and asking after them at dawn, i.e. serving them well with filial devotion (lit. settle them at dusk, check on them at dawn).
- ko_def (stdict, verbatim): 밤에는 부모의 잠자리를 보아 드리고 이른 아침에는 부모의 밤새 안부를 묻는다는 뜻으로, 부모를 잘 섬기고 효성을 다함을 이르는 말.
- ex (AI Korean): 효자는 혼정신성을 게을리하지 않았다.
  - en (AI): The dutiful son never neglected caring for his parents morning and night.
- ex (AI Korean): 혼정신성의 예절은 요즘에는 보기 어려워졌다.
  - en (AI): The custom of tending to one's parents morning and night is now hard to find.

### 189. 여광여취 (如狂如醉) [seongeo, stdict:230061, draft]

- en_gloss (AI): as if mad or drunk
- en_def (AI): A state of having lost one's reason, as though crazed or intoxicated (lit. as if crazy, as if drunk).
- ko_def (stdict, verbatim): 미친 듯도 하고 취한 듯도 하다는 뜻으로, 이성을 잃은 상태를 비유적으로 이르는 말.
- ex (AI Korean): 우승이 확정되자 관중들은 여광여취로 환호했다.
  - en (AI): When the victory was confirmed, the crowd cheered as if in a frenzy.
- ex (AI Korean): 그는 여광여취의 상태에서 아무 말이나 내뱉었다.
  - en (AI): In a state of mad abandon, he blurted out whatever came to mind.

### 190. 신지무의 (信之無疑) [seongeo, stdict:203249, draft]

- en_gloss (AI): believing without a doubt
- en_def (AI): Trusting completely, without the slightest doubt (lit. believe it, with no doubt).
- ko_def (stdict, verbatim): 조금도 의심하지 아니하고 믿음.
- ex (stdict, verbatim): 모든 사람을 신지무의로 대하기는 굉장히 어려운 일이지요.
  - en (AI): It's very difficult to treat everyone with complete, unquestioning trust, isn't it?
- ex (AI Korean): 그는 친구의 말을 신지무의하고 믿었다.
  - en (AI): He believed his friend's words without the slightest doubt.

### 191. 전전반측 (輾轉反側) [seongeo, stdict:287308, draft]

- en_gloss (AI): tossing and turning
- en_def (AI): Lying down and turning from side to side, unable to sleep (lit. roll over and over, turn to the side).
- ko_def (stdict, verbatim): 누워서 몸을 이리저리 뒤척이며 잠을 이루지 못함.
- ex (AI Korean): 시험 결과가 걱정되어 밤새 전전반측했다.
  - en (AI): Worried about the exam results, I tossed and turned all night.
- ex (AI Korean): 전전반측하다가 새벽에야 겨우 잠이 들었다.
  - en (AI): After tossing and turning, I finally fell asleep only at dawn.

### 192. 함구불언 (緘口不言) [seongeo, stdict:363932, draft]

- en_gloss (AI): keeping one's mouth shut
- en_def (AI): Closing one's mouth and saying nothing (lit. sealed mouth, saying nothing).
- ko_def (stdict, verbatim): 입을 다물고 말을 하지 아니함.
- ex (AI Korean): 그는 기자들의 질문에 끝까지 함구불언으로 일관했다.
  - en (AI): He kept his mouth shut right through the reporters' questions.
- ex (AI Korean): 비밀을 지키기 위해 모두가 함구불언하기로 약속했다.
  - en (AI): Everyone promised to say nothing in order to keep the secret.

### 193. 황황망조 (遑遑罔措) [seongeo, stdict:505147, draft]

- en_gloss (AI): flustered and at a loss
- en_def (AI): In such a hurry that one does not know what to do and flounders (lit. hurried, hurried, not knowing what to do).
- ko_def (stdict, verbatim): 마음이 급하여 어찌할 줄을 모르고 허둥지둥함.
- ex (AI Korean): 갑자기 불이 나자 사람들은 황황망조하며 우왕좌왕했다.
  - en (AI): When a fire broke out, people ran about in confusion, flustered and at a loss.
- ex (AI Korean): 예상치 못한 질문에 그는 황황망조했다.
  - en (AI): At the unexpected question, he was flustered and did not know what to do.

### 194. 신상필벌 (信賞必罰) [seongeo, stdict:451402, draft]

- en_gloss (AI): rewarding merit and punishing guilt without fail
- en_def (AI): Giving rewards to those who have done well and punishments to those who have done wrong, fairly and strictly (lit. faithful reward, certain punishment).
- ko_def (stdict, verbatim): 공이 있는 자에게는 반드시 상을 주고, 죄가 있는 사람에게는 반드시 벌을 준다는 뜻으로, 상과 벌을 공정하고 엄중하게 하는 일을 이르는 말.
- ex (AI Korean): 조직을 이끌려면 신상필벌의 원칙이 지켜져야 한다.
  - en (AI): To lead an organization, the principle of reward and punishment without fail must be kept.
- ex (AI Korean): 신상필벌이 분명한 회사에서 일하고 싶다.
  - en (AI): I want to work at a company where reward and punishment are clearly applied.

### 195. 유감천만 (遺憾千萬) [seongeo, stdict:255318, draft]

- en_gloss (AI): extremely regrettable
- en_def (AI): Very sorry and disappointed, beyond measure (lit. regret ten-thousandfold).
- ko_def (stdict, verbatim): 섭섭하기 짝이 없음.
- ex (AI Korean): 이번 사고는 정말 유감천만이다.
  - en (AI): This accident is truly regrettable in the extreme.
- ex (AI Korean): 그는 약속을 어긴 것은 유감천만이라고 밝혔다.
  - en (AI): He stated that breaking the promise was deeply regrettable.

### 196. 지고지순 (至高至純) [seongeo, stdict:486845, draft]

- en_gloss (AI): supremely pure
- en_def (AI): Utterly high and pure, with nothing higher (lit. highest, purest).
- ko_def (stdict, verbatim): 더할 수 없이 높고 순수함.
- ex (stdict, verbatim): 두 남녀의 사랑은 지고지순 그 자체였다.
  - en (AI): The love between the man and woman was purity itself.
- ex (AI Korean): 어머니의 사랑은 지고지순하여 대가를 바라지 않는다.
  - en (AI): A mother's love is supremely pure and expects nothing in return.

### 197. 주지육림 (酒池肉林) [seongeo, stdict:482267, draft]

- en_gloss (AI): a lavish drinking feast
- en_def (AI): A luxurious banquet with abundant drink and meat (lit. a pond of wine, a forest of meat).
- ko_def (stdict, verbatim): 술로 연못을 이루고 고기로 숲을 이룬다는 뜻으로, 호사스러운 술잔치를 이르는 말. 중국 은나라 주왕이 못을 파 술을 채우고 숲의 나뭇가지에 고기를 걸어 잔치를 즐겼던 일에서 유래한다.
- ex (AI Korean): 폭군은 매일 밤 주지육림에 빠져 지냈다.
  - en (AI): The tyrant spent every night immersed in lavish feasting and drinking.
- ex (AI Korean): 연말 파티가 주지육림으로 변하는 것은 경계해야 한다.
  - en (AI): We should guard against the year-end party turning into a drunken orgy.

### 198. 황음무도 (荒淫無道) [seongeo, stdict:380624, draft]

- en_gloss (AI): debauched and depraved
- en_def (AI): Being lost in drink and sex and paying no heed to the duties of a decent person (lit. wanton and licentious, without the way).
- ko_def (stdict, verbatim): 주색에 빠져 사람으로서 마땅히 할 도리를 돌아보지 않는 면이 있음.
- ex (AI Korean): 황음무도한 왕은 결국 백성들의 손에 쫓겨났다.
  - en (AI): The debauched and depraved king was in the end driven out by his people.
- ex (AI Korean): 그는 황음무도한 생활 끝에 재산을 모두 잃었다.
  - en (AI): After a life of debauchery, he lost all his property.

### 199. 무아몽중 (無我夢中) [seongeo, stdict:426825, draft]

- en_gloss (AI): acting in a trance; in a dream
- en_def (AI): Acting without knowing it, as if in a dream, when the mind is fixed on one thing or the spirit has gone (lit. no self, in a dream).
- ko_def (stdict, verbatim): 자기를 모르고 꿈속에 있는 것 같다는 뜻으로, 마음이 외곬으로 쏠리거나 넋을 잃어 자기도 모르게 행동하는 지경을 이르는 말.
- ex (AI Korean): 그는 무아몽중에 길을 걷다가 신호를 놓쳤다.
  - en (AI): He was walking in a daze, lost in a dream, and missed the signal.
- ex (AI Korean): 무아몽중으로 달려온 그는 숨조차 제대로 쉬지 못했다.
  - en (AI): He ran there in a trance, unable even to breathe properly.

### 200. 일패도지 (一敗塗地) [seongeo, stdict:472026, draft]

- en_gloss (AI): crushing defeat
- en_def (AI): Being beaten so thoroughly that one can never rise again (lit. one defeat, brains smeared on the ground).
- ko_def (stdict, verbatim): 싸움에 한 번 패하여 간과 뇌가 땅바닥에 으깨어진다는 뜻으로, 여지없이 패하여 다시 일어날 수 없게 되는 지경에 이름을 이르는 말. 한고조 유방의 말로서 ≪사기≫의 <고조본기(高祖本紀)>에 나오는 말이다.
- ex (AI Korean): 그 군대는 첫 전투에서 일패도지하고 말았다.
  - en (AI): That army suffered a crushing defeat in the very first battle.
- ex (AI Korean): 일패도지의 아픔을 딛고 그는 다시 일어났다.
  - en (AI): He got up again, overcoming the pain of that crushing defeat.

### 201. 광대무변 (廣大無邊) [seongeo, stdict:33871, draft]

- en_gloss (AI): boundless; vast and without end
- en_def (AI): So wide and great that it has no limit (lit. broad, grand, without edge).
- ko_def (stdict, verbatim): 넓고 커서 끝이 없음.
- ex (AI Korean): 우주는 광대무변하여 그 끝을 알 수 없다.
  - en (AI): The universe is boundless, and its end cannot be known.
- ex (AI Korean): 광대무변한 초원이 눈앞에 펼쳐졌다.
  - en (AI): A vast, endless grassland spread out before us.

### 202. 산자수명 (山紫水明) [seongeo, stdict:170779, draft]

- en_gloss (AI): lovely scenery; purple mountains and clear waters
- en_def (AI): Beautiful landscape, with mountains tinged purple and clear water (lit. mountains purple, waters bright).
- ko_def (stdict, verbatim): 산은 자줏빛이고 물은 맑다는 뜻으로, 경치가 아름다움을 이르는 말.
- ex (AI Korean): 이곳은 산자수명한 경치로 이름난 고장이다.
  - en (AI): This is a region famous for its beautiful scenery of misty mountains and clear waters.
- ex (AI Korean): 산자수명의 풍광에 반해 이곳에 집을 지었다.
  - en (AI): Charmed by the picturesque landscape, I built a house here.

### 203. 언어도단 (言語道斷) [seongeo, stdict:225035, draft]

- en_gloss (AI): outrageous; beyond words
- en_def (AI): So absurd that one is lost for words, even when trying to speak (lit. the path of speech is cut off).
- ko_def (stdict, verbatim): 말할 길이 끊어졌다는 뜻으로, 어이가 없어서 말하려 해도 말할 수 없음을 이르는 말.
- ex (AI Korean): 거짓말을 해 놓고도 사과하지 않다니 언어도단이다.
  - en (AI): He lies and then does not even apologize; it is outrageous.
- ex (AI Korean): 그런 행동은 언어도단이라 할 말이 없다.
  - en (AI): Such behavior is beyond words; I have nothing to say.

### 204. 백사청송 (白沙靑松) [seongeo, stdict:429216, draft]

- en_gloss (AI): white sand and green pines
- en_def (AI): A beautiful seaside scene of white sand and green pine trees (lit. white sand, blue pines).
- ko_def (stdict, verbatim): 흰모래와 푸른 소나무라는 뜻으로, 바닷가의 아름다운 경치를 이르는 말.
- ex (AI Korean): 백사청송이 어우러진 해변을 따라 걸었다.
  - en (AI): We walked along a beach where white sand and green pines blend together.
- ex (AI Korean): 이 해안은 백사청송으로 이름난 명승지이다.
  - en (AI): This coast is a scenic spot famous for its white sand and green pines.

### 205. 공중누각 (空中樓閣) [seongeo, stdict:30604, draft]

- en_gloss (AI): a castle in the air
- en_def (AI): An idea or thing that has no basis or foundation (lit. a pavilion floating in the sky).
- ko_def (stdict, verbatim): 공중에 떠 있는 누각이라는 뜻으로, 아무런 근거나 토대가 없는 사물이나 생각을 비유적으로 이르는 말.
- ex (AI Korean): 현실성이 없는 그 계획은 공중누각에 불과하다.
  - en (AI): That unrealistic plan is nothing more than a castle in the air.
- ex (AI Korean): 기초 없이 쌓은 이론은 공중누각이 되기 쉽다.
  - en (AI): A theory built without foundations easily becomes a castle in the air.

### 206. 용두사미 (龍頭蛇尾) [seongeo, stdict:464940, draft]

- en_gloss (AI): a strong start and a weak finish; anticlimax
- en_def (AI): Starting out vigorously but ending in a whimper (lit. a dragon's head and a snake's tail).
- ko_def (stdict, verbatim): 용의 머리와 뱀의 꼬리라는 뜻으로, 처음은 왕성하나 끝이 부진한 현상을 이르는 말.
- ex (AI Korean): 야심 차게 시작한 사업이 용두사미로 끝나고 말았다.
  - en (AI): The business that began so ambitiously ended in anticlimax.
- ex (AI Korean): 용두사미가 되지 않으려면 끝까지 힘을 내야 한다.
  - en (AI): To avoid a big start and a weak finish, you have to keep up your effort to the end.

### 207. 이민위천 (以民爲天) [seongeo, stdict:268034, draft]

- en_gloss (AI): putting the people first
- en_def (AI): Valuing the people as one values heaven (lit. take the people as heaven).
- ko_def (stdict, verbatim): 백성을 하늘같이 소중히 여김.
- ex (AI Korean): 임금은 이민위천의 마음으로 나라를 다스렸다.
  - en (AI): The king ruled the country with the heart of regarding the people as heaven.
- ex (AI Korean): 정치인은 이민위천의 자세를 잊어서는 안 된다.
  - en (AI): A politician must not forget the attitude of putting the people first.

### 208. 충군애국 (忠君愛國) [seongeo, stdict:333814, draft]

- en_gloss (AI): loyalty to the king and love of country
- en_def (AI): Being thoroughly loyal to the ruler and loving one's country (lit. be loyal to the ruler, love the country).
- ko_def (stdict, verbatim): 임금에게 충성을 다하고 나라를 사랑함.
- ex (AI Korean): 그 장군은 충군애국의 정신으로 평생을 바쳤다.
  - en (AI): The general devoted a whole lifetime in a spirit of loyalty to the king and love of country.
- ex (AI Korean): 옛 교과서는 충군애국을 가장 중요한 덕목으로 가르쳤다.
  - en (AI): Old textbooks taught loyalty to the sovereign and love of country as the most important virtue.

### 209. 사상누각 (沙上樓閣) [seongeo, stdict:439961, draft]

- en_gloss (AI): a castle built on sand
- en_def (AI): Something that will not last long because its foundation is weak (lit. a pavilion on sand).
- ko_def (stdict, verbatim): 모래 위에 세운 누각이라는 뜻으로, 기초가 튼튼하지 못하여 오래 견디지 못할 일이나 물건을 이르는 말.
- ex (AI Korean): 기본기가 없는 실력은 사상누각이나 다름없다.
  - en (AI): Skill without the basics is no better than a castle built on sand.
- ex (AI Korean): 부실한 설계로 지은 건물은 사상누각처럼 무너졌다.
  - en (AI): The building, put up with a flimsy design, collapsed like a castle built on sand.

### 210. 신체발부 (身體髮膚) [seongeo, stdict:207255, draft]

- en_gloss (AI): one's whole body; body, hair and skin
- en_def (AI): The body as a whole (lit. body, hair, skin).
- ko_def (stdict, verbatim): 몸과 머리털과 피부라는 뜻으로, 몸 전체를 이르는 말.
- ex (stdict, verbatim): 우리 조상들은 신체발부는 부모에게서 받은 것이므로 이를 상하지 않게 하는 것이 효라고 여겼다.
  - en (AI): Our ancestors held that the body, hair and skin are received from one's parents, so keeping them unharmed is filial piety.
- ex (AI Korean): 신체발부는 수지부모라는 가르침에 따라 머리카락도 함부로 자르지 않았다.
  - en (AI): Following the teaching that the body, hair and skin are received from one's parents, they would not cut even their hair lightly.

### 211. 질풍신뢰 (疾風迅雷) [seongeo, stdict:487609, draft]

- en_gloss (AI): swift as storm and thunder
- en_def (AI): Things that change fast and fiercely, like a violent wind and a thunderclap (lit. fierce wind, sudden thunder).
- ko_def (stdict, verbatim): 심한 바람과 번개라는 뜻으로, 빠르고 심하게 변하는 상태를 이르는 말.
- ex (AI Korean): 군대는 질풍신뢰의 속도로 국경을 넘었다.
  - en (AI): The army crossed the border at lightning speed.
- ex (AI Korean): 질풍신뢰와 같은 변화에 적응하기가 쉽지 않다.
  - en (AI): It is not easy to adapt to change as swift and sudden as a gale and thunderclap.

### 212. 천병만마 (千兵萬馬) [seongeo, stdict:320745, draft]

- en_gloss (AI): a vast army
- en_def (AI): A great host of soldiers and war horses (lit. a thousand soldiers, ten thousand horses).
- ko_def (stdict, verbatim): 천 명의 군사와 만 마리의 군마라는 뜻으로, 아주 많은 수의 군사와 군마를 이르는 말.
- ex (AI Korean): 천병만마를 거느린 장군이 성 앞에 도착했다.
  - en (AI): The general, leading a vast army, arrived before the fortress.
- ex (AI Korean): 천병만마가 달리는 소리에 땅이 흔들렸다.
  - en (AI): The ground shook at the sound of the great army and its horses at the gallop.

### 213. 출장입상 (出將入相) [seongeo, stdict:334698, draft]

- en_gloss (AI): excelling as both general and minister
- en_def (AI): Serving as a general abroad and as a minister at home, i.e. mastering both military and civil affairs (lit. go out as a general, come in as a minister).
- ko_def (stdict, verbatim): 나가서는 장수가 되고 들어와서는 재상이 된다는 뜻으로, 문무를 다 갖추어 장상(將相)의 벼슬을 모두 지냄을 이르는 말.
- ex (AI Korean): 그 인물은 출장입상하여 나라의 기둥이 되었다.
  - en (AI): That figure served as both general and minister and became a pillar of the nation.
- ex (AI Korean): 출장입상의 재능을 갖춘 사람은 드물다.
  - en (AI): Few people have the talent to excel as both general and minister.

### 214. 함포고복 (含哺鼓腹) [seongeo, stdict:368225, draft]

- en_gloss (AI): well fed and content
- en_def (AI): Eating one's fill and patting one's belly, living happily with plenty to eat (lit. mouth full of food, drum the belly).
- ko_def (stdict, verbatim): 잔뜩 먹고 배를 두드린다는 뜻으로, 먹을 것이 풍족하여 즐겁게 지냄을 이르는 말.
- ex (AI Korean): 풍년이 들어 백성들이 함포고복하며 살았다.
  - en (AI): With a bumper harvest, the people lived well fed and content.
- ex (AI Korean): 함포고복의 태평성대를 누리던 시절이었다.
  - en (AI): It was a time when people enjoyed an era of peace and plenty.

### 215. 만만불가 (萬萬不可) [seongeo, stdict:110012, draft]

- en_gloss (AI): absolutely out of the question
- en_def (AI): Not right at all (lit. ten thousand, ten thousand times not allowed).
- ko_def (stdict, verbatim): 전혀 옳지 아니함.
- ex (AI Korean): 그런 요구는 만만불가이니 받아들일 수 없다.
  - en (AI): A demand like that is absolutely out of the question, so we cannot accept it.
- ex (AI Korean): 아이에게 위험한 일을 시키는 것은 만만불가다.
  - en (AI): Making a child do something dangerous is entirely unacceptable.

### 216. 만심환희 (滿心歡喜) [seongeo, stdict:110085, draft]

- en_gloss (AI): overjoyed
- en_def (AI): Being satisfied and extremely happy (lit. heart full, joyful).
- ko_def (stdict, verbatim): 만족하여 아주 기뻐함.
- ex (AI Korean): 그는 아들의 합격 소식에 만심환희했다.
  - en (AI): He was overjoyed at the news that his son had passed.
- ex (AI Korean): 만심환희하며 선물을 열어 보는 아이의 얼굴이 환했다.
  - en (AI): The face of the child, opening the present overjoyed, was radiant.

### 217. 만화방창 (萬化方暢) [seongeo, stdict:110159, draft]

- en_gloss (AI): everything bursting into life in spring
- en_def (AI): All living things sprouting, growing and flourishing luxuriantly on a warm spring day (lit. ten thousand things changing, just thriving).
- ko_def (stdict, verbatim): 따뜻한 봄날에 온갖 생물이 나서 자라 흐드러짐.
- ex (AI Korean): 봄이 되자 들과 산에 만화방창의 기운이 가득했다.
  - en (AI): With spring, the fields and mountains were filled with the energy of everything bursting into life.
- ex (AI Korean): 만화방창한 계절에는 어디를 가도 꽃이 만발한다.
  - en (AI): In the season when all things burst into life, flowers bloom everywhere you go.

### 218. 세궁역진 (勢窮力盡) [seongeo, stdict:444001, draft]

- en_gloss (AI): at the end of one's strength
- en_def (AI): Having lost momentum and run out of strength, unable to move (lit. force exhausted, strength spent).
- ko_def (stdict, verbatim): 기세가 꺾이고 힘이 다 빠져 꼼짝할 수 없게 됨.
- ex (stdict, verbatim): 나도 세궁역진의 상황이라 너를 도울 겨를이 없다.
  - en (AI): I am also at the end of my rope, so I have no time to help you.
- ex (AI Korean): 연패를 거듭한 팀은 세궁역진의 처지에 놓였다.
  - en (AI): The team, after repeated defeats, found itself exhausted of momentum and strength.

### 219. 영세불망 (永世不忘) [seongeo, stdict:454930, draft]

- en_gloss (AI): never to be forgotten
- en_def (AI): Not forgetting forever (lit. not forgetting through all ages).
- ko_def (stdict, verbatim): 영원히 잊지 아니함.
- ex (AI Korean): 베풀어 주신 은혜는 영세불망 잊지 않겠습니다.
  - en (AI): I will never forget the kindness you have shown me.
- ex (AI Korean): 영세불망의 비를 세워 그의 업적을 기렸다.
  - en (AI): They set up a memorial stone to be remembered forever and honored his achievements.

### 220. 와석종신 (臥席終身) [seongeo, stdict:245329, draft]

- en_gloss (AI): dying peacefully in bed
- en_def (AI): Living out one's natural lifespan and dying peacefully lying down (lit. lie on the mat and end one's life).
- ko_def (stdict, verbatim): 제명을 다하고 편안히 자리에 누워서 죽음.
- ex (AI Korean): 할아버지는 와석종신하시어 가족들이 지켜보는 가운데 눈을 감으셨다.
  - en (AI): Grandfather died peacefully in his bed, passing away with the family at his side.
- ex (AI Korean): 전란의 시대에는 와석종신이 큰 복이었다.
  - en (AI): In an age of war, dying peacefully in bed was a great blessing.

### 221. 유예미결 (猶豫未決) [seongeo, stdict:264004, draft]

- en_gloss (AI): dithering; unable to decide
- en_def (AI): Hesitating and being unable to make up one's mind (lit. hesitate, undecided).
- ko_def (stdict, verbatim): 망설여 결정을 짓지 못함.
- ex (AI Korean): 그는 유예미결인 채로 몇 달을 보냈다.
  - en (AI): He spent several months unable to make up his mind.
- ex (AI Korean): 유예미결하다가 좋은 기회를 놓쳤다.
  - en (AI): While dithering over the decision, I missed a good opportunity.

### 222. 전전불매 (輾轉不寐) [seongeo, stdict:287312, draft]

- en_gloss (AI): sleepless tossing and turning
- en_def (AI): Lying down, turning from side to side and unable to sleep (lit. toss and turn, not sleeping).
- ko_def (stdict, verbatim): 누워서 몸을 이리저리 뒤척이며 잠을 이루지 못함.
- ex (AI Korean): 걱정거리가 많아 그는 전전불매로 밤을 지새웠다.
  - en (AI): With so many worries, he spent the night tossing and turning without sleep.
- ex (AI Korean): 전전불매하는 밤이 길게만 느껴졌다.
  - en (AI): The nights spent tossing and turning without sleep felt only long.

### 223. 평온무사 (平穩無事) [seongeo, stdict:353548, draft]

- en_gloss (AI): peaceful and uneventful
- en_def (AI): Quiet and calm, with nothing happening (lit. calm and steady, without incident).
- ko_def (stdict, verbatim): 조용하고 평안하여 아무 일이 없음.
- ex (AI Korean): 이번 여행은 평온무사하게 끝났다.
  - en (AI): The trip ended peacefully, without any incident.
- ex (AI Korean): 그는 평온무사한 하루가 가장 큰 행복이라고 말한다.
  - en (AI): He says a peaceful, uneventful day is the greatest happiness.

### 224. 오만무례 (傲慢無禮) [seongeo, stdict:465544, draft]

- en_gloss (AI): arrogant and rude
- en_def (AI): Presumptuous and haughty in manner or behavior, and lacking in courtesy (lit. arrogant, without manners).
- ko_def (stdict, verbatim): 태도나 행동이 건방지거나 거만하여 예의(禮義)를 지키지 아니함.
- ex (AI Korean): 그의 오만무례한 태도에 모두가 불쾌해했다.
  - en (AI): Everyone was offended by his arrogant, rude attitude.
- ex (AI Korean): 오만무례하게 어른께 말대꾸를 하면 안 된다.
  - en (AI): You must not talk back to your elders in an arrogant and rude manner.

### 225. 권토중래 (捲土重來) [seongeo, stdict:400373, draft]

- en_gloss (AI): coming back stronger after a setback
- en_def (AI): Having failed once, regaining one's strength and returning to the attack (lit. roll up the dust and come again).
- ko_def (stdict, verbatim): 땅을 말아 일으킬 것 같은 기세로 다시 온다는 뜻으로, 한 번 실패하였으나 힘을 회복하여 다시 쳐들어옴을 이르는 말. 중국 당나라 두목의 <오강정시(烏江亭詩)>에 나오는 말로, 항우가 유방과의 결전에서 패하여 오강(烏江) 근처에서 자결한 것을 탄식한 말에서 유래한다.
- ex (AI Korean): 그 선수는 권토중래를 다짐하며 훈련에 몰두했다.
  - en (AI): The player threw themselves into training, vowing to come back stronger.
- ex (AI Korean): 권토중래의 기회를 노리던 야당이 선거에서 승리했다.
  - en (AI): The opposition party, which had been waiting for a chance to come back, won the election.

### 226. 유무상통 (有無相通) [seongeo, stdict:257564, draft]

- en_gloss (AI): sharing what one has with those who lack
- en_def (AI): Exchanging what one has for what one lacks, so that each makes up the other's need (lit. having and lacking, flowing to each other).
- ko_def (stdict, verbatim): 있는 것과 없는 것을 서로 융통함.
- ex (AI Korean): 이웃들은 유무상통하며 어려운 시절을 함께 견뎠다.
  - en (AI): The neighbors got through the hard times together, sharing what they had with those who lacked.
- ex (AI Korean): 유무상통의 정신으로 시작된 장터였다.
  - en (AI): It was a market that began in the spirit of exchanging what one has for what one lacks.

### 227. 삼종지도 (三從之道) [seongeo, stdict:172770, draft]

- en_gloss (AI): the three obediences (a woman's traditional duties)
- en_def (AI): Formerly, the three duties of a woman: to follow her father when young, her husband when married, and her son after her husband's death (lit. the way of the three followings).
- ko_def (stdict, verbatim): 예전에, 여자가 따라야 할 세 가지 도리를 이르던 말. 어려서는 아버지를, 결혼해서는 남편을, 남편이 죽은 후에는 자식을 따라야 하였다. ≪예기≫의 의례(儀禮) <상복전(喪服傳)>에 나오는 말이다.
- ex (AI Korean): 조선 시대에는 삼종지도를 여자의 도리로 가르쳤다.
  - en (AI): In the Joseon period the three obediences were taught as a woman's duty.
- ex (AI Korean): 삼종지도는 여성에게 순종을 강요한 낡은 관습이라는 비판을 받는다.
  - en (AI): The "three obediences" are criticized as an outdated custom that forced women into submission.

### 228. 하로동선 (夏爐冬扇) [seongeo, stdict:355449, draft]

- en_gloss (AI): a stove in summer, a fan in winter; out of season
- en_def (AI): Something unsuited to the time or occasion, like a stove in summer or a fan in winter (lit. summer stove, winter fan).
- ko_def (stdict, verbatim): 여름의 화로와 겨울의 부채라는 뜻으로, 격(格)이나 철에 맞지 아니함을 이르는 말. ≪논형(論衡)≫의 <봉우편(逢遇篇)>에 나오는 말이다.
- ex (AI Korean): 지금 와서 그런 제안을 하는 것은 하로동선이다.
  - en (AI): Making a proposal like that now is as pointless as a stove in summer.
- ex (AI Korean): 하로동선 같은 물건이라 쓸모가 없었다.
  - en (AI): It was something as out of season as a summer stove, and so useless.

### 229. 천하무쌍 (天下無雙) [seongeo, stdict:328069, draft]

- en_gloss (AI): peerless; unrivaled in the world
- en_def (AI): Having nothing in the world to compare with it (lit. no pair under heaven).
- ko_def (stdict, verbatim): 세상에서 그에 비길 만한 것이 없음.
- ex (AI Korean): 그는 천하무쌍의 검객으로 이름을 날렸다.
  - en (AI): He became famous as a swordsman without equal in the world.
- ex (AI Korean): 천하무쌍의 미인이라는 소문이 온 나라에 퍼졌다.
  - en (AI): The rumor spread through the whole country that she was a beauty without peer.

### 230. 오월동주 (吳越同舟) [seongeo, stdict:459299, draft]

- en_gloss (AI): enemies in the same boat
- en_def (AI): People who bear each other ill will finding themselves together, or having to cooperate (lit. Wu and Yue in the same boat).
- ko_def (stdict, verbatim): 서로 적의를 품은 사람들이 한자리에 있게 된 경우나 서로 협력하여야 하는 상황을 비유적으로 이르는 말. 중국 춘추 전국 시대에, 서로 적대시하는 오나라 사람과 월나라 사람이 같은 배를 탔으나 풍랑을 만나서 서로 단합하여야 했다는 데에서 유래한다. 출전은 ≪손자(孫子)≫의 <구지편(九地篇)>이다.
- ex (AI Korean): 정치적 라이벌이 한 배를 타게 되어 오월동주의 상황이 되었다.
  - en (AI): Political rivals ended up in the same boat, a situation of bitter enemies sharing a vessel.
- ex (AI Korean): 위기 앞에서는 오월동주라도 힘을 합쳐야 한다.
  - en (AI): In the face of a crisis, even sworn enemies in the same boat have to join forces.

### 231. 문일지십 (聞一知十) [seongeo, stdict:424625, draft]

- en_gloss (AI): grasping ten things from hearing one
- en_def (AI): Hearing one thing and inferring ten from it; extremely quick-witted (lit. hear one, know ten).
- ko_def (stdict, verbatim): 하나를 듣고 열 가지를 미루어 안다는 뜻으로, 지극히 총명함을 이르는 말. ≪논어≫의 <공야장편(公冶長篇)>에 나오는 말이다.
- ex (AI Korean): 그 학생은 문일지십이라 한 번 가르치면 모든 것을 이해한다.
  - en (AI): That student gets the gist of ten things from hearing one; teach them once and they understand everything.
- ex (AI Korean): 문일지십의 총명함으로 어린 나이에 이름을 알렸다.
  - en (AI): With a brilliance that grasps ten from hearing one, the child made a name at a young age.

### 232. 계계승승 (繼繼承承) [seongeo, stdict:398716, draft]

- en_gloss (AI): passed down through generation after generation
- en_def (AI): Descendants carrying on from one generation to the next, or a later generation taking over what an earlier one did (lit. succeed, succeed, inherit, inherit).
- ko_def (stdict, verbatim): 자자손손이 대를 이어 감.
- ex (AI Korean): 이 가게는 계계승승 이어 온 전통의 맛집이다.
  - en (AI): This shop is a restaurant of traditional taste, handed down generation after generation.
- ex (AI Korean): 가문의 비법은 계계승승 전해졌다.
  - en (AI): The family's secret recipe was passed down through the generations.

### 233. 골육상잔 (骨肉相殘) [seongeo, stdict:397331, draft]

- en_gloss (AI): kin killing kin
- en_def (AI): Close blood relatives hurting and killing each other (lit. bone and flesh harming each other).
- ko_def (stdict, verbatim): 가까운 혈족끼리 서로 해치고 죽임.
- ex (stdict, verbatim): 왕위 계승 문제를 둘러싸고 골육상잔의 참상이 빚어졌다.
  - en (AI): The horror of kin killing kin arose over the question of succession to the throne.
- ex (AI Korean): 형제가 칼을 겨누는 골육상잔의 비극은 되풀이되어서는 안 된다.
  - en (AI): The tragedy of brother turning on brother in fratricidal strife must never be repeated.

### 234. 이이제이 (以夷制夷) [seongeo, stdict:263332, draft]

- en_gloss (AI): pitting one enemy against another
- en_def (AI): Using one force to hold another in check (lit. use barbarians to control barbarians).
- ko_def (stdict, verbatim): 오랑캐로 오랑캐를 무찌른다는 뜻으로, 한 세력을 이용하여 다른 세력을 제어함을 이르는 말.
- ex (AI Korean): 그 나라는 이이제이의 전략으로 주변 강대국을 견제했다.
  - en (AI): That country checked the surrounding great powers by the strategy of playing one against another.
- ex (AI Korean): 이이제이의 계책으로 적들끼리 싸우게 만들었다.
  - en (AI): With the stratagem of using one foe against another, the enemies were set fighting among themselves.

### 235. 사분오열 (四分五裂) [seongeo, stdict:439725, draft]

- en_gloss (AI): falling to pieces; split into factions
- en_def (AI): Being torn into many pieces, or scattering in disorder (lit. four parts, five tears).
- ko_def (stdict, verbatim): 여러 갈래로 갈기갈기 찢어짐.
- ex (AI Korean): 내분으로 당이 사분오열되었다.
  - en (AI): The party was split apart by internal strife.
- ex (AI Korean): 지도자를 잃은 군대는 사분오열하여 흩어졌다.
  - en (AI): The army that lost its leader fell apart and scattered.

### 236. 불면불휴 (不眠不休) [seongeo, stdict:435555, draft]

- en_gloss (AI): working without sleep or rest
- en_def (AI): Working hard without resting at all (lit. no sleep, no rest).
- ko_def (stdict, verbatim): 자지도 않고 쉬지도 않는다는 뜻으로, 조금도 쉬지 않고 힘써 일함을 이르는 말.
- ex (AI Korean): 그는 불면불휴로 연구에 매달린 끝에 성과를 냈다.
  - en (AI): After working at his research without sleep or rest, he produced results.
- ex (AI Korean): 구조대는 불면불휴의 노력으로 생존자를 찾아냈다.
  - en (AI): The rescue team found survivors by working tirelessly without sleep or rest.

### 237. 취생몽사 (醉生夢死) [seongeo, stdict:330299, draft]

- en_gloss (AI): living in a drunken daze; frittering one's life away
- en_def (AI): Spending a whole life aimlessly in a haze, as if living and dying in a drunken dream (lit. live drunk, die dreaming).
- ko_def (stdict, verbatim): 술에 취하여 자는 동안에 꾸는 꿈 속에 살고 죽는다는 뜻으로, 한평생을 아무 하는 일 없이 흐리멍덩하게 살아감을 비유적으로 이르는 말.
- ex (AI Korean): 취생몽사로 세월을 보내서는 아무것도 이룰 수 없다.
  - en (AI): You cannot achieve anything by frittering your years away in a daze.
- ex (AI Korean): 그는 취생몽사의 삶을 청산하고 새 출발을 했다.
  - en (AI): He put an end to a life lived in a drunken stupor and made a fresh start.

### 238. 다정다한 (多情多恨) [seongeo, stdict:73429, draft]

- en_gloss (AI): full of both tenderness and sorrow
- en_def (AI): Having both much affection and much to grieve over (lit. much affection, much regret).
- ko_def (stdict, verbatim): 애틋한 정도 많고 한스러운 일도 많음.
- ex (AI Korean): 다정다한한 사람은 이별의 아픔도 더 깊게 느낀다.
  - en (AI): A person with both strong affection and strong sorrow feels the pain of parting more deeply.
- ex (AI Korean): 시인은 다정다한의 정서를 노래에 담았다.
  - en (AI): The poet put the emotion of tenderness mixed with sorrow into the song.

### 239. 갈건야복 (葛巾野服) [seongeo, stdict:387623, draft]

- en_gloss (AI): a recluse's humble attire
- en_def (AI): The coarse, plain clothes of a hermit or scholar living in retirement (lit. kudzu cap, rustic clothes).
- ko_def (stdict, verbatim): 갈건과 베옷이라는 뜻으로, 은사(隱士)나 처사(處士)의 거칠고 소박한 옷차림을 이르는 말.
- ex (AI Korean): 그는 갈건야복 차림으로 산속에 은거했다.
  - en (AI): He lived in seclusion in the mountains in a recluse's simple dress.
- ex (AI Korean): 갈건야복의 선비가 마을 어귀를 지나갔다.
  - en (AI): A scholar in plain hermit's clothes passed the entrance of the village.

### 240. 폐의파립 (敝衣破笠) [seongeo, stdict:499357, draft]

- en_gloss (AI): shabby, tattered clothes
- en_def (AI): A poor appearance, in worn-out clothes and a broken hat (lit. worn clothes, broken hat).
- ko_def (stdict, verbatim): 해어진 옷과 부서진 갓이란 뜻으로, 초라한 차림새를 비유적으로 이르는 말.
- ex (AI Korean): 그는 폐의파립의 행색으로 고향 마을에 돌아왔다.
  - en (AI): He came back to his home village looking shabby in tattered clothes.
- ex (AI Korean): 폐의파립을 한 거지가 길가에 앉아 있었다.
  - en (AI): A beggar in rags was sitting by the roadside.

### 241. 혼불부신 (魂不附身) [seongeo, stdict:377168, draft]

- en_gloss (AI): scared out of one's wits
- en_def (AI): Being so startled that one loses one's senses, as if the soul had scattered from the body (lit. the soul is not attached to the body).
- ko_def (stdict, verbatim): 혼백이 어지러이 흩어진다는 뜻으로, 몹시 놀라 넋을 잃음을 이르는 말.
- ex (AI Korean): 갑자기 나타난 멧돼지를 보고 그는 혼불부신이 되었다.
  - en (AI): When he saw the wild boar that suddenly appeared, he was scared out of his wits.
- ex (AI Korean): 천둥소리에 아이들이 혼불부신하여 울음을 터뜨렸다.
  - en (AI): At the clap of thunder, the children, scared out of their wits, burst into tears.

### 242. 흥진비래 (興盡悲來) [seongeo, stdict:508310, draft]

- en_gloss (AI): after pleasure comes sorrow
- en_def (AI): When joy runs its course, sorrow arrives, showing that the affairs of the world go in cycles (lit. when delight is exhausted, grief comes).
- ko_def (stdict, verbatim): 즐거운 일이 다하면 슬픈 일이 닥쳐온다는 뜻으로, 세상일은 순환되는 것임을 이르는 말.
- ex (AI Korean): 흥진비래라더니 즐겁던 여행이 사고로 끝났다.
  - en (AI): After joy comes sorrow, they say, and the cheerful trip ended in an accident.
- ex (AI Korean): 흥진비래의 이치를 알면 좋은 때에도 조심하게 된다.
  - en (AI): If you understand that after pleasure comes sorrow, you stay careful even in good times.

### 243. 궁흉극악 (窮凶極惡) [seongeo, stdict:397488, draft]

- en_gloss (AI): extremely wicked and cruel
- en_def (AI): Utterly vicious and malicious (lit. extreme wickedness, utmost evil).
- ko_def (stdict, verbatim): 몹시 흉측하고 악독함.
- ex (AI Korean): 궁흉극악한 범죄자에게 사형이 선고되었다.
  - en (AI): The death sentence was passed on the extremely wicked and cruel criminal.
- ex (AI Korean): 그 폭군은 궁흉극악한 짓을 서슴지 않았다.
  - en (AI): The tyrant did not hesitate to commit vile and cruel acts.

### 244. 명심불망 (銘心不忘) [seongeo, stdict:116033, draft]

- en_gloss (AI): engraving in one's heart and never forgetting
- en_def (AI): Fixing something deep in the heart and not forgetting it for a very long time (lit. engrave on the heart, not forget).
- ko_def (stdict, verbatim): 마음에 깊이 새겨 두어 오래오래 잊지 아니함.
- ex (AI Korean): 선생님의 가르침을 명심불망하겠습니다.
  - en (AI): I will engrave my teacher's lessons in my heart and never forget them.
- ex (AI Korean): 아버지의 유언을 명심불망하며 살아왔다.
  - en (AI): I have lived keeping my father's last words in my heart and never forgetting them.

### 245. 지원극통 (至冤極痛) [seongeo, stdict:316755, draft]

- en_gloss (AI): deep resentment and bitter anguish
- en_def (AI): The utmost feeling of having been wronged and of pain (lit. utmost injustice, extreme anguish).
- ko_def (stdict, verbatim): 지극히 원통함.
- ex (AI Korean): 억울한 누명을 쓰고 감옥에 갇힌 그는 지원극통에 몸부림쳤다.
  - en (AI): Falsely accused and shut up in prison, he writhed in deep resentment and anguish.
- ex (AI Korean): 지원극통한 심정을 어디에도 하소연할 데가 없었다.
  - en (AI): There was nowhere to pour out such bitter anguish.

### 246. 용호상박 (龍虎相搏) [seongeo, stdict:464729, draft]

- en_gloss (AI): a clash of titans
- en_def (AI): Two strong opponents fighting each other (lit. dragon and tiger strike each other).
- ko_def (stdict, verbatim): 용과 범이 서로 싸운다는 뜻으로, 강자끼리 서로 싸움을 이르는 말.
- ex (AI Korean): 결승전은 두 강팀의 용호상박이 될 것이다.
  - en (AI): The final will be a clash of titans between two strong teams.
- ex (AI Korean): 용호상박의 접전 끝에 승부가 갈렸다.
  - en (AI): The match was decided after a hard-fought contest between two powerful sides.

### 247. 백문이 불여일견 [sokdam, stdict:528347, draft]

- en_gloss (AI): seeing is believing
- en_def (AI): It is more certain to see something with your own eyes than merely to hear about it (lit. hearing a hundred times is no match for seeing once).
- ko_def (stdict, verbatim): 듣기만 하는 것보다는 직접 보는 것이 확실하다는 말.
- ex (AI Korean): 백문이 불여일견이라더니 직접 가서 보니 사진과는 느낌이 전혀 달랐다.
  - en (AI): Seeing is believing, as they say: going there myself, it felt completely different from the photos.
- ex (AI Korean): 백문이 불여일견이니 일단 현장에 가 보자.
  - en (AI): Seeing is believing, so let's go and look at the site first.

### 248. 은혜를 원수로 갚는다 [sokdam, stdict:531238, draft]

- en_gloss (AI): to repay kindness with harm
- en_def (AI): Doing harm to someone where one ought to thank them for their kindness (lit. repay a favor with enmity).
- ko_def (stdict, verbatim): 감사로써 은혜에 보답해야 할 자리에 도리어 해를 끼침을 이르는 말.
- ex (AI Korean): 도와준 사람을 모함하다니, 은혜를 원수로 갚는 격이다.
  - en (AI): To slander the person who helped you is to repay kindness with harm.
- ex (AI Korean): 은혜를 원수로 갚는다는 말이 그에게 꼭 들어맞는다.
  - en (AI): The saying about repaying kindness with harm fits him exactly.

### 249. 손이 발이 되도록 빌다 [sokdam, stdict:529753, draft]

- en_gloss (AI): to beg and plead desperately
- en_def (AI): Earnestly begging to be forgiven for a fault or mistake (lit. plead until one's hands become feet).
- ko_def (stdict, verbatim): 허물이나 잘못을 용서하여 달라고 간절히 빎을 비유적으로 이르는 말.
- ex (AI Korean): 그는 손이 발이 되도록 빌었지만 주인은 끝내 용서해 주지 않았다.
  - en (AI): He begged and pleaded until his hands were worn out, but the owner never forgave him.
- ex (AI Korean): 손이 발이 되도록 빌며 사정했더니 겨우 허락을 받았다.
  - en (AI): After pleading desperately for all I was worth, I finally got permission.

### 250. 귀신이 곡할 노릇이다 [sokdam, stdict:525727, draft]

- en_gloss (AI): baffling; beyond understanding
- en_def (AI): So strange and wondrous that nobody can fathom how it happened (lit. enough to make a ghost wail).
- ko_def (stdict, verbatim): 신기하고 기묘하여 그 속내를 알 수 없음을 비유적으로 이르는 말.
- ex (AI Korean): 문을 잠그고 나갔는데 물건이 사라지다니, 귀신이 곡할 노릇이다.
  - en (AI): I locked the door when I left and yet things have vanished; it is baffling.
- ex (AI Korean): 분명히 여기에 두었는데 없어졌으니 정말 귀신이 곡할 노릇이다.
  - en (AI): I'm sure I left it here, and now it's gone; it's truly beyond understanding.

### 251. 무른 메주 밟듯 [sokdam, opendict:603051, draft]

- en_gloss (AI): moving about with no difficulty (North Korean)
- en_def (AI): Going around freely and easily, with no difficulty at all; a North Korean expression (lit. like treading soft fermented-soybean blocks).
- ko_def (opendict, verbatim): 아무런 어려움 없이 쉽게 두루 돌아다니는 모양을 비유적으로 이르는 말.
- ex (AI Korean): 그는 무른 메주 밟듯 그 복잡한 시장 골목을 거침없이 누볐다.
  - en (AI): He moved through the crowded market alleys with ease, as if treading soft soybean blocks.
- ex (AI Korean): 길을 훤히 아는 그는 무른 메주 밟듯 산길을 오르내렸다.
  - en (AI): Knowing the way well, he went up and down the mountain path as easily as anything.

### 252. 사공이 많으면 배가 산으로 간다 [sokdam, stdict:522176, draft]

- en_gloss (AI): too many cooks spoil the broth
- en_def (AI): When many people each try to steer the boat their own way, it ends up on the mountain instead of the water; with nobody in charge, everyone pushing their own view gets nowhere (lit. with many boatmen the boat goes up the mountain).
- ko_def (stdict, verbatim): 여러 사람이 저마다 제 주장대로 배를 몰려고 하면 결국에는 배가 물로 못 가고 산으로 올라간다는 뜻으로, 주관하는 사람 없이 여러 사람이 자기주장만 내세우면 일이 제대로 되기 어려움을 비유적으로 이르는 말.
- ex (AI Korean): 사공이 많으면 배가 산으로 간다더니 의견이 너무 많아 일이 진척되지 않는다.
  - en (AI): Too many cooks spoil the broth, and with so many opinions, the work is making no progress.
- ex (AI Korean): 사공이 많으면 배가 산으로 가니 책임자를 한 명 정하자.
  - en (AI): With too many hands at the oar the boat ends up on the mountain, so let's name one person in charge.

### 253. 미주알고주알 캔다 [sokdam, stdict:521533, draft]

- en_gloss (AI): to pry into every detail
- en_def (AI): Looking into the inside story of a matter down to its smallest details (lit. dig out all the fine particulars).
- ko_def (stdict, verbatim): 일의 속사정을 속속들이 자세히 알아보는 경우를 비유적으로 이르는 말.
- ex (AI Korean): 남의 일을 미주알고주알 캔다고 해서 좋을 것은 없다.
  - en (AI): Nothing good comes of prying into every detail of other people's business.
- ex (AI Korean): 기자는 사건의 뒷이야기를 미주알고주알 캔다는 소문이 났다.
  - en (AI): The reporter was rumored to dig into every last detail of the story behind the case.

### 254. 입에 재갈을 물리다 [sokdam, stdict:531055, draft]

- en_gloss (AI): to gag; to silence
- en_def (AI): Figuratively, to stop someone from speaking out (lit. put a gag in the mouth).
- ko_def (stdict, verbatim): 함부로 입을 놀리지 못하게 함을 비유적으로 이르는 말.
- ex (AI Korean): 정부가 언론의 입에 재갈을 물리려 한다는 비판이 나왔다.
  - en (AI): There was criticism that the government was trying to gag the press.
- ex (AI Korean): 반대 의견의 입에 재갈을 물리는 회의는 의미가 없다.
  - en (AI): A meeting that gags dissenting opinions is meaningless.

### 255. 설마가 사람 죽인다 [sokdam, stdict:522498, draft]

- en_gloss (AI): "surely not" kills; never take it for granted
- en_def (AI): Counting on things not going wrong or hoping for luck leads to trouble, so one should guard against everything that could happen (lit. "surely not" kills a person).
- ko_def (stdict, verbatim): 그럴 리야 없을 것이라 마음을 놓거나 요행을 바라는 데에서 탈이 난다는 뜻으로, 요행을 바라지 말고 있을 수 있는 모든 것을 미리 예방해 놓아야 한다는 말.
- ex (AI Korean): 설마가 사람 죽인다고 하니 안전 점검을 미리 해 두자.
  - en (AI): They say "surely not" kills people, so let's do the safety checks in advance.
- ex (AI Korean): 설마가 사람 죽인다는 말처럼 대비를 소홀히 했다가 큰 피해를 입었다.
  - en (AI): As the saying "surely not" kills, they neglected to prepare and suffered great damage.

### 256. 까마귀 밥이 되다 [sokdam, stdict:525687, draft]

- en_gloss (AI): to die with no one to bury one
- en_def (AI): To die and be left behind with nobody to take care of one's remains (lit. become food for crows).
- ko_def (stdict, verbatim): 거두어 줄 사람이 없이 죽어 버려짐을 비유적으로 이르는 말.
- ex (AI Korean): 전쟁터에서 쓰러진 병사들은 까마귀 밥이 되고 말았다.
  - en (AI): The soldiers who fell on the battlefield ended up as food for the crows.
- ex (AI Korean): 돌봐 줄 사람 없이 외롭게 지내다간 까마귀 밥이 되기 십상이다.
  - en (AI): If you live alone with nobody to look after you, you are likely to end up with nobody to bury you.

### 257. 죽어도 눈을 감겠다 [sokdam, opendict:614786, draft]

- en_gloss (AI): I could die happy
- en_def (AI): An expression meaning one would have no regrets even in death (lit. I would close my eyes even in death).
- ko_def (opendict, verbatim): 죽어도 여한이 없다는 말.
- ex (AI Korean): 자식이 성공하는 모습을 보았으니 이제 죽어도 눈을 감겠다.
  - en (AI): I have seen my child succeed, so now I could die happy.
- ex (AI Korean): 이 소원만 이룬다면 죽어도 눈을 감겠다.
  - en (AI): If I can only fulfill this one wish, I could die content.

### 258. 가물에 콩씨 나듯 [sokdam, stdict:519181, draft]

- en_gloss (AI): few and far between
- en_def (AI): Things that turn up only rarely, here and there, like bean seeds sprouting sparsely in a drought (lit. like bean seeds coming up in a dry spell).
- ko_def (stdict, verbatim): 가뭄에는 심은 콩이 제대로 싹이 트지 못하여 드문드문 난다는 뜻으로, 어떤 일이나 물건이 어쩌다 하나씩 드문드문 있는 경우를 비유적으로 이르는 말.
- ex (AI Korean): 그 시골 마을에는 젊은이가 가물에 콩씨 나듯 드물게 보일 뿐이다.
  - en (AI): In that country village, young people are few and far between.
- ex (AI Korean): 좋은 소식은 가물에 콩씨 나듯 어쩌다 하나씩 들려왔다.
  - en (AI): Good news came in only rarely, one item at a time.

### 259. 과부 설움은 동무 과부가 안다 [sokdam, stdict:519772, draft]

- en_gloss (AI): only someone in the same plight understands
- en_def (AI): A person's difficulties are best understood by someone who has been through the same thing or is in a similar position (lit. a widow's sorrow is known by a fellow widow).
- ko_def (stdict, verbatim): 남의 곤란한 처지는 직접 그 일을 당해 보았거나 그와 비슷한 처지에 놓여 있는 사람이 잘 알 수 있음을 비유적으로 이르는 말.
- ex (AI Korean): 과부 설움은 동무 과부가 안다고, 같은 일을 겪은 친구만이 내 마음을 알아준다.
  - en (AI): Only one who has been in the same plight knows another's sorrow, and only a friend who has been through the same thing understands how I feel.
- ex (AI Korean): 과부 설움은 동무 과부가 안다더니 같은 처지의 이웃이 가장 큰 위로가 되었다.
  - en (AI): It is said that only someone in the same plight understands your sorrow, and a neighbor in the same situation was the greatest comfort.

### 260. 꿈자리가 사납더니 [sokdam, stdict:520096, draft]

- en_gloss (AI): (lament) no wonder things are going wrong
- en_def (AI): A saying uttered in a tone of lament when things do not go as wished and something gets in the way (lit. my sleeping-place was bad, and so...).
- ko_def (stdict, verbatim): 일이 뜻대로 되지 아니하고 방해되는 것이 끼어들 때 한탄조로 이르는 말.
- ex (AI Korean): 꿈자리가 사납더니 오늘은 되는 일이 하나도 없구나.
  - en (AI): I had a bad dream, and sure enough nothing is going right today.
- ex (AI Korean): 꿈자리가 사납더니 기어이 차를 놓치고 말았다.
  - en (AI): I had a bad dream, and sure enough I ended up missing my ride.

### 261. 그물코가 삼천이면 걸릴 날이 있다 [sokdam, stdict:519981, draft]

- en_gloss (AI): diligence will pay off in the end
- en_def (AI): If you work hard, you will get good results (lit. with three thousand net meshes, a day to catch something will come).
- ko_def (stdict, verbatim): 부지런히 일하면 좋은 결과를 얻을 수 있음을 비유적으로 이르는 말.
- ex (AI Korean): 그물코가 삼천이면 걸릴 날이 있다고 하니 포기하지 말고 계속 도전하자.
  - en (AI): They say that with three thousand meshes in the net, there is bound to be a catch, so let's not give up and keep trying.
- ex (AI Korean): 그물코가 삼천이면 걸릴 날이 있다더니 꾸준한 노력이 결국 결실을 맺었다.
  - en (AI): As the saying goes, with three thousand net meshes you will catch something sooner or later, and steady effort finally bore fruit.

### 262. 새우 싸움에 고래 등 터지랴 [sokdam, stdict:529285, draft]

- en_gloss (AI): the mighty are not harmed when the weak fight
- en_def (AI): However hard small, insignificant parties fight among themselves, a large, powerful one does not suffer for it (lit. would a whale's back burst over a shrimp fight?).
- ko_def (stdict, verbatim): 약하고 보잘것없는 것끼리 아무리 싸워도 크고 힘 있는 존재는 그 피해를 받지 아니함을 비유적으로 이르는 말.
- ex (AI Korean): 새우 싸움에 고래 등 터지랴, 작은 회사끼리 다투어 봐야 대기업은 끄떡도 없다.
  - en (AI): When shrimps fight, the whale's back doesn't burst: no matter how small companies quarrel, the big corporation is unmoved.
- ex (AI Korean): 약소국들의 다툼에 강대국은 새우 싸움에 고래 등 터지랴 하는 태도였다.
  - en (AI): In the disputes of the small nations, the great powers took the attitude that a shrimp fight won't burst a whale's back.

### 263. 어물전 망신은 꼴뚜기가 시킨다 [sokdam, stdict:523117, draft]

- en_gloss (AI): the worst of the group disgraces everyone
- en_def (AI): The most hopeless person among a group brings shame on the colleagues around them (lit. the baby squid disgraces the fish market).
- ko_def (stdict, verbatim): 지지리 못난 사람일수록 같이 있는 동료를 망신시킨다는 말.
- ex (AI Korean): 어물전 망신은 꼴뚜기가 시킨다더니 한 사람의 실수로 팀 전체가 욕을 먹었다.
  - en (AI): One bad apple spoils the bunch, as they say: one person's mistake brought blame on the whole team.
- ex (AI Korean): 어물전 망신은 꼴뚜기가 시킨다는데 우리 팀의 막내가 또 사고를 쳤다.
  - en (AI): They say the baby squid shames the whole fish market, and our team's youngest has caused another incident.

### 264. 피는 물보다 진하다 [sokdam, stdict:532902, draft]

- en_gloss (AI): blood is thicker than water
- en_def (AI): The bond of family feeling is deep (lit. blood is thicker than water).
- ko_def (stdict, verbatim): 혈육의 정이 깊음을 이르는 말.
- ex (AI Korean): 피는 물보다 진하다더니 형제는 끝내 서로를 외면하지 못했다.
  - en (AI): Blood is thicker than water, as they say, and in the end the brothers could not turn their backs on each other.
- ex (AI Korean): 피는 물보다 진하다고 하니 가족의 일이라면 모른 척할 수 없다.
  - en (AI): Blood is thicker than water, so I can't pretend not to see when it concerns family.

## Independent content review (content-review, 2026-11-24)

Reviewer: a separate agent from the drafter. Every P row, every example (Korean and English) of Part A and every line of
Part B was read; checked against the ko_def copied from stdict/opendict, the hanja and the Korean sentence.

**Part A — idiom_extra.tsv:** 263 approved, 1 rejected (264 items).
- Rejected: 이견대인 (利見大人, opendict) — a Book of Changes phrase, not a 성어 in current Korean use; the AI examples were contrived. Kept in the file as `rejected: ...` (the loader ships only `approved`; `problems()` and the test now accept `rejected: reason`).
- Doubtful items kept: 십상팔구 (a stdict entry, = 십중팔구, its examples are stdict's own), 불사이군 (忠臣不事二君, a real stdict 성어), 일타쌍피 (modern go-stop idiom, described as such), 무른 메주 밟듯 (opendict tags it 북한어, as the gloss says).
- Flagged English idiom equivalents checked: 은혜를 원수로 갚는다 (gloss now leads with "bite the hand that feeds you"), 천재일우 ("chance of a lifetime" — correct), 독불장군 ("stubborn loner" — correct), 환골탈태 (gloss + both senses correct), 유만부동 (both stdict senses covered; examples correct), 천의무봉 (gloss now "seamless perfection"), 호연지기 (correct).
- Sensitive items: 노류장화 / 창가소부 already labelled derogatory; 운우지정 relabelled "(literary euphemism)"; 일부종사 and 출가외인 labelled "(traditional)"; 삼종지도 already marked traditional with a critical example.
- Invented gender removed where the Korean names nobody (he/his/her for 사람, 배우, 가수, 코치, 공무원 ...).

Changes (old → new):

- `He is such a versatile actor that he handles any role he is given.` → `The actor is so versatile that any role given is handled well.`
- `Ten years after her debut, the singer underwent a complete transformation and presented music entirely unlike her earlier work.` → `Ten years after debuting, the singer underwent a complete transformation and presented music entirely unlike anything before.`
- `It is unbelievable that he committed such a vile act and shows no remorse.` → `It is unbelievable that someone could commit such a vile act and show no remorse.`
- `To torment a little child, he is truly a vicious person.` → `Anyone who would torment a little child is truly vicious.`
- `그녀는 합격 소식을 듣고 기쁨에 겨워 대성통곡을 했다. ⇥ ai ⇥ On hearing she had passed, she wept loudly, overcome with joy.` → `그녀는 친구의 사고 소식을 듣고 대성통곡을 했다. ⇥ ai ⇥ On hearing about her friend's accident, she burst into loud wailing.`
- `There is no point arguing over who was at fault this time; it is six of one, half a dozen of the other.` → `There is no point arguing over who was at fault this time; both sides are just as bad.`
- `The company met a new investor at the very last moment.` → `At a moment of desperate crisis, the company found a new investor.`
- `Not noticing that others dislike it, he chatters away as if no one else were around.` → `Not noticing that others dislike it, they chatter away as if no one else were around.`
- `With an arrogant disregard for others, he shouted in front of his elders.` → `With an arrogant disregard for others, they shouted in front of their elders.`
- `No matter how much I try to stop him he doesn't listen, so I won't say any more.` → `No matter how much I try to talk them out of it, it falls on deaf ears, so I won't say any more.`
- `Despite the string of defeats, the coach had faith in his players.` → `Despite the string of defeats, the coach kept faith in the players.`
- `He makes the mistake himself and then gets angry; the guilty blaming the innocent has its limits.` → `Making the mistake yourself and then getting angry: there are limits to turning the tables like that.`
- `The pupil's skill has gone beyond his teacher's, so it deserves to be called the pupil surpassing the master.` → `The pupil's skill has gone beyond the teacher's, so it truly is a case of the pupil surpassing the master.`
- `Just as the saying goes, the junior has broken the senior's record and outdone the master.` → `Just as the saying about the pupil surpassing the master goes, the junior has broken the senior's record.`
- `The official built up his wealth by using his public post for private gain and was punished.` → `The official grew rich by using a public post for private gain and was punished.`
- `Thanks to the decisiveness of his clean cuts, the company got past the crisis.` → `Thanks to clean, decisive judgment, the company got past the crisis.`
- `With that level of skill, it wouldn't be surprising if he won.` → `With that level of skill, it wouldn't be surprising if they won.`
- `That professor is someone the students keep a respectful distance from, so it is hard to approach.` → `That professor is someone the students keep at a respectful distance, so it is hard to get close.`
- `A loyal subject, the saying goes, does not serve two lords, and he kept faith even when the country fell.` → `Saying that a loyal subject does not serve two lords, they kept faith even when the country fell.`
- `The word "flawless" suited that pianist's performance.` → `That pianist's performance deserved to be called a seamless heavenly robe.`
- `flawless and natural ⇥ Perfect` → `seamless perfection; flawless and natural ⇥ Perfect`
- `He lies and then does not even apologize; it is outrageous.` → `Lying and then not even apologizing: it is outrageous.`
- `We should guard against the year-end party turning into a drunken orgy.` → `We should guard against the year-end party turning into an extravagant drinking binge.`
- `베풀어 주신 은혜는 영세불망 잊지 않겠습니다. ⇥ ai ⇥ I will never forget the kindness you have shown me.` → `베풀어 주신 은혜는 영세불망하겠습니다. ⇥ ai ⇥ I will never, ever forget the kindness you have shown me.`
- `He begged and pleaded until his hands were worn out, but the owner never forgave him.` → `He begged and pleaded, rubbing his hands together, but the owner never forgave him.`
- `As the saying "surely not" kills, they neglected to prepare and suffered great damage.` → `As the saying "surely not" kills goes, preparations were neglected and great damage followed.`
- `One bad apple spoils the bunch, as they say: one person's mistake brought blame on the whole team.` → `The weakest member shames the whole group, as they say: one person's mistake brought blame on the whole team.`
- ` ⇥ to repay kindness with harm ⇥ ` → ` ⇥ to bite the hand that feeds you; repay kindness with harm ⇥ `
- ` ⇥ remaining faithful to one husband ⇥ ` → ` ⇥ (traditional) remaining faithful to one husband ⇥ `
- ` ⇥ a married daughter belongs to her husband's family ⇥ ` → ` ⇥ (traditional) a married daughter is an outsider to her own family ⇥ `
- ` ⇥ sexual love; lovemaking ⇥ ` → ` ⇥ (literary euphemism) sexual union ⇥ `

**Part B — example_en_idioms.tsv:** all 1,051 rows read; 1,051 approved after 22 corrections (none rejected).
Corrections: invented or wrong subject/gender, mistranslated proverbs (개 머루 먹듯 dropped, 못 먹는 감 찔러나 본다 muddled), an added gloss in brackets, word-order slips.

Changes (old → new):

- `I heard Minjun left his wife for another woman?` → `I heard Minjun was seeing another woman behind his wife's back?`
- `but it was a gas leak that made you dizzy just to think about.` → `but it was a gas leak that is chilling just to think about.`
- `I want to grow old with you together, forever.` → `I want to spend my whole life with you, growing old together.`
- `prayed for the eternal immortality of our souls.` → `prayed for the immortality of our souls.`
- `is really tasteful and nice.` → `is really charming and lovely.`
- `and punished them without a word, not even asking why they were late.` → `and punished them indiscriminately, without even asking why they were late.`
- `please don't just get angry without a word; tell me` → `please don't just get angry before hearing what happened; tell me`
- `The rich man lived in luxury` → `The rich person lived in luxury`
- `The more the better, so the more the merrier.` → `The more the better, as they say, so the more you put in, the happier I'll be.`
- `Seung-gyu said he would quit drinking, but far from lasting three days, he didn't last a single day before drinking soju.` → `Seung-gyu vowed to quit drinking, but far from lasting three days, didn't last even a single day before having soju.`
- `like a dog that chased a chicken up onto the roof.` → `like a dog staring up at the roof after the chicken it chased has flown up there.`
- `Seeing a wealthy rich man, people thought he was born under a lucky star.` → `Seeing a man of great wealth, people thought he was born under a lucky star.`
- `odd rumors spread, the sort of sour grapes that poke at a persimmon one can't eat.` → `odd rumors spread, in the spirit of "if I can't eat the persimmon, I'll poke it."`
- `Yes, a rags-to-riches transformation, as if a loach had become a dragon.` → `Yes, a total transformation, as if a loach had become a dragon.`
- `he looked so frail from all his hardship that he seemed he would blow away if blown on and crumple if gripped.` → `he looked so frail from all his hardship that it seemed he would blow away if blown on and crumple if gripped.`
- `You actually know nothing, so stop pretending to know again.` → `You actually know nothing, so stop pretending to know again, like a dog gulping wild grapes without tasting them.`
- `let's scold Jisu sternly so that it never lies again.` → `let's scold Jisu sternly so that Jisu never lies again.`
- `Yumin's spouse had an affair, they divorced` → `Yumin's husband had an affair, they divorced`
- `I brought this bonus all to my mother-in-law.` → `I gave my whole bonus this time to my mother-in-law.`
- `Do you think one billion is a child's name in some neighborhood? (as if a billion were a trifle)` → `Do you think a billion is just some kid's name in the neighborhood?`
- `The swindler coaxed people with plausible words and then struck them, extorting money from them.` → `The swindler coaxed people with plausible words and then turned on them, extorting their money.`
- `Park Assistant Manager snatched the contract I had nearly closed.` → `Assistant Manager Park snatched away the contract I had nearly closed.`
