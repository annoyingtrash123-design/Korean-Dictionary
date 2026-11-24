# Review: 쓰기 expressions and confusable words (`pipeline/data/writing_expressions.tsv`, `pipeline/data/confusables.tsv`)

AI-written (Claude), all `draft`. Nothing ships until a row's `review` column says `approved...`; shipped text is labelled AI in the app ("AI-written, reviewed"). 162 쓰기 expressions, 100 confusable groups. Every AI-written field is listed below, so the independent reviewer (content-review skill) can work from this sheet and then set the `review` column in the two TSV files (`approved`, or `rejected: reason`; correct the text in the TSV first if only a small fix is needed).

## What to check

### 쓰기 expressions (W…)

1. **expression**: a real, current written-register Korean expression; the `-` marks and `-(으)ㄴ/는` alternations are conjugated as written (ㄹ-stems, ㅂ-irregulars, 받침 choices).
2. **meaning**: the English is accurate and not wider or narrower than the Korean.
3. **register note**: the register claim is true (written / formal / formal polite), and the spoken equivalents it names to avoid really are the spoken counterparts. TOPIK 53 and 54 are written in 한다체; 51 and 52 (notices, e-mails) are 합쇼체 or 해요체 depending on the task. A note that says an expression "sounds casual" must not be false for a standard TOPIK essay.
4. **model sentences** (two per row): natural essay Korean, correct grammar, spelling and spacing, 한다체 (합쇼체 in the notices group), and the sentence really uses the expression of its row. The English translates it faithfully.
5. **link**: `g:` / `w:` points at the right dictionary or grammar entry (the build resolves it; an unresolved link is dropped and listed in the build log).
6. Group: the expression belongs to the function group it is filed under.

### Confusable groups (C…)

1. **words**: each really is confusable with the others; the × forms are spellings people actually write wrongly and are not valid spellings (check against 표준국어대사전 / 국립국어원 어문 규범).
2. **explanation**: the difference is stated correctly and completely enough; dictionary senses and the "rule of thumb" tests hold. A claim like "dictionaries now accept both" must be true for the current 표준국어대사전.
3. **sentences** (one per word): each uses its own word in its right sense and the minimal pair is truly contrasting; the English translates it faithfully.
4. **common mistake**: the wrong form named is a real mistake and the correction is right.
5. kind: spelling / meaning / grammar is the best fit.

## Look at these first (least certain)

- C003 개발 / 계발: whether 자기 개발 is accepted next to 자기 계발 (the note says so) and the 계발 = talents rule.
- C014 금세 / 금새: 금새 (price of an item) is rare; check the sense and that the example sentence is natural.
- C034 쫓다 / 좇다: the claim that standard usage prefers 좇다 for dreams and ideals.
- C031 부딪치다 / 부딪히다: the active/passive split; 어려움에 부딪히다 and 부딪치다 are both attested.
- C021 깨끗이 / 꼼꼼히 / 틈틈이: the 이/히 rule of thumb is simplified (ㅅ final or repeated noun: 이; other 하다 words: 히).
- C079 그러나 / 그런데, C049 너무 / 매우: register claims: 그런데 as a contrast marker, and 너무 (in speech 너무 also has a positive sense).
- C085 임대 / 임차, C086 권리 / 권한, C070 요구 / 요청: nuance statements.
- C043 -대 / -데: the -데 explanation (recollection, -더라) and that both are spoken.
- W049 -(으)ㄴ 바 있다, W016 필자는, W064 하지만: register notes about how a 54 grader sees them.
- every `g:` / `w:` link: the build resolves them against the pack; the build log lists the unresolved ones.

## 쓰기 expressions


### Introducing the topic

**W001** `-에 대해 살펴보고자 한다` · this essay sets out to look at … (stock opening of an introduction)

- register: Written, formal “(한다체)”. Typical first sentence of a 54 essay. Spoken counterpart “-에 대해 얘기해 볼게요” does not belong in an essay.
- 1: 이 글에서는 청소년의 스마트폰 사용 실태에 대해 살펴보고자 한다. / This essay sets out to look at how teenagers actually use smartphones.
- 2: 본론에서는 조기 영어 교육이 아동의 발달에 미치는 영향에 대해 살펴보고자 한다. / The main part looks at how early English education affects children's development.
- link: g:-고자 · review: draft

**W002** `-이/가 주목을 받고 있다` · … is attracting attention

- register: Written. Passive-style way to open on a trend; the spoken “-이/가 인기예요” sounds too casual.
- 1: 최근 재택근무가 새로운 근무 형태로 주목을 받고 있다. / Working from home has recently been attracting attention as a new way of working.
- 2: 환경을 생각하는 소비 방식이 젊은 층 사이에서 주목을 받고 있다. / Environmentally conscious consumption is drawing attention among young people.
- link: (none) · review: draft

**W003** `-이/가 사회적 문제로 대두되고 있다` · … is emerging as a social problem

- register: Written, formal. “대두되다” is almost only used in writing. In speech people say “-이/가 문제가 돼요”; do not write that in an essay.
- 1: 저출산이 심각한 사회적 문제로 대두되고 있다. / The low birth rate is emerging as a serious social problem.
- 2: 개인 정보 유출이 사회적 문제로 대두되고 있다. / Leaks of personal information are emerging as a social problem.
- link: w:대두되다 · review: draft

**W004** `-이/가 화두로 떠오르다` · … has become the hot topic

- register: Written, journalistic. “화두” = topic of discussion; avoid the spoken “-이/가 핫하다”.
- 1: 최근 인공지능의 활용 범위가 사회의 화두로 떠오르고 있다. / How widely artificial intelligence may be used has recently become a topic of public debate.
- 2: 고령화 사회의 노인 복지가 중요한 화두로 떠올랐다. / Welfare for the elderly in an ageing society has become a major topic.
- link: w:화두 · review: draft

**W005** `-에 대한 관심이 높아지고 있다` · interest in … is growing

- register: Written, formal. A safe, neutral opener; in speech people say “-에 관심이 많아졌어요”.
- 1: 건강한 식생활에 대한 관심이 높아지고 있다. / Interest in a healthy diet is growing.
- 2: 최근 들어 사회 전반에서 환경 보호에 대한 관심이 높아지고 있다. / In recent years interest in protecting the environment has been growing throughout society.
- link: (none) · review: draft

**W006** `오늘날 / 현대 사회에서는` · today / in modern society

- register: Written, formal. Sets the time frame at the start of an introduction; spoken equivalents “요즘 / 지금은” are too casual for an essay.
- 1: 오늘날 많은 사람들이 인터넷을 통해 정보를 얻는다. / Today many people obtain information through the internet.
- 2: 현대 사회에서는 개인의 정보를 보호하는 일이 점점 중요해지고 있다. / In modern society, protecting personal information is becoming ever more important.
- link: w:오늘날 · review: draft

**W007** `최근 들어` · in recent times

- register: Written, neutral to formal. Often paired with “-고 있다” or “-는 추세이다”. Spoken: “요즘”.
- 1: 최근 들어 일회용품 사용을 줄이려는 사람이 늘고 있다. / In recent years more people are trying to reduce their use of disposable items.
- 2: 최근 들어 혼자 사는 가구의 수가 빠르게 증가하고 있다. / The number of single-person households has been rising quickly in recent years.
- link: (none) · review: draft

**W008** `-는 추세이다` · there is a trend toward …

- register: Written, formal. Fits both the introduction and a graph description. The spoken “-는 추세예요 / -는 편이에요” does not suit an essay.
- 1: 결혼 연령이 점점 높아지는 추세이다. / There is a trend toward later marriage.
- 2: 도시로 인구가 집중되는 추세이다. / The population is tending to concentrate in cities.
- link: w:추세 · review: draft

**W009** `이에 대해 의견이 엇갈린다` · opinions are divided on this

- register: Written, formal. Sets up a debate before you take a side; “의견이 분분하다” is the stronger variant. Spoken “의견이 갈려요 / 의견이 서로 달라요” is too casual.
- 1: 학교에서 휴대 전화를 사용하는 것을 허용해야 하는지에 대해서는 의견이 엇갈린다. / Opinions are divided on whether mobile phones should be allowed at school.
- 2: 사형 제도를 유지해야 한다는 주장과 폐지해야 한다는 주장이 팽팽하게 맞서고 있다. / The argument for keeping the death penalty and the argument for abolishing it stand evenly matched.
- link: w:엇갈리다 · review: draft

**W010** `의견이 분분하다` · opinions differ widely

- register: Written, formal. Stronger than “엇갈리다”: many different views, not just two. Spoken “의견이 다 달라요” is too casual.
- 1: 이 문제의 해결 방법에 대해서는 전문가들 사이에서도 의견이 분분하다. / Even among experts, opinions on how to solve this problem differ widely.
- 2: 안락사를 허용해야 하는지를 두고 의견이 분분하다. / Opinions are sharply divided over whether euthanasia should be allowed.
- link: w:분분하다 · review: draft

**W011** `-을/를 둘러싸고 논란이 되고 있다` · … has become the subject of controversy

- register: Written, formal (news style). “둘러싸고” = surrounding / over. Spoken “말이 많아요 / 시끄러워요” is not for an essay.
- 1: 동물 실험의 윤리성을 둘러싸고 논란이 되고 있다. / The ethics of animal testing have become the subject of controversy.
- 2: 원격 수업의 효과를 둘러싸고 교육계에서 논란이 계속되고 있다. / Controversy over the effectiveness of remote classes continues in the education world.
- link: w:둘러싸다 · review: draft

**W012** `-이/가 필수적이 되었다` · … has become essential

- register: Written, formal. Sets up why the topic matters. Spoken “꼭 필요해요 / 없으면 안 돼요” must not appear in an essay.
- 1: 정보화 시대에는 정보를 올바르게 판단하는 능력이 필수적이 되었다. / In the information age the ability to judge information correctly has become essential.
- 2: 이제 외국어 능력은 사회생활에서 필수적인 요소가 되었다. / A foreign language is now an essential element of social life.
- link: w:필수적 · review: draft

**W013** `-이/가 중요한 의미를 지니다` · … carries an important meaning

- register: Written, formal. “지니다” is the written verb for "to carry/have" (spoken: “가지고 있다”).
- 1: 어린 시절의 독서 경험은 인격 형성에 중요한 의미를 지닌다. / Reading in childhood carries great significance for forming one's character.
- 2: 이 정책은 지역 균형 발전의 측면에서 중요한 의미를 지닌다. / This policy is significant for balanced regional development.
- link: w:지니다 · review: draft


### Stating a view

**W014** `-다고 생각한다` · I think that …

- register: Written “(한다체)”. The basic way to state an opinion; spoken “-(ㄴ/는)다고 생각해요 / -것 같아요” are not used in an essay.
- 1: 나는 학교 교육에서 예절 교육이 강화되어야 한다고 생각한다. / I think moral education should be strengthened in school education.
- 2: 나는 이 제도가 장기적으로 사회에 도움이 될 것이라고 생각한다. / I think that in the long run this system will benefit society.
- link: w:생각하다 · review: draft

**W015** `-다고 본다` · I take the view that …

- register: Written, a little firmer and more detached than “-다고 생각한다”. Spoken “-다고 봐요 / -것 같아요” do not belong in an essay.
- 1: 그러나 필자는 이러한 주장이 설득력이 부족하다고 본다. / However, I see this argument as lacking persuasive force.
- 2: 나는 개인의 노력만으로는 문제를 해결하기 어렵다고 본다. / I believe it is hard to solve the problem through individual effort alone.
- link: w:보다 · review: draft

**W016** `필자는 -다고 생각한다` · the writer thinks that …

- register: Written, formal. “필자” (the writer) or just “나는” is fine in TOPIK 54; do not mix with “저는” (polite) in the same essay. Spoken “저는 -것 같아요 / -다고 생각해요” is for conversation only.
- 1: 필자는 대중교통 이용을 장려하는 것이 가장 효과적인 해결책이라고 생각한다. / I think that encouraging public transport is the most effective solution.
- 2: 필자는 이 문제가 개인보다는 사회의 책임이라고 생각한다. / I believe this problem is the responsibility of society rather than of individuals.
- link: w:필자 · review: draft

**W017** `-라고 할 수 있다` · it can be said that …

- register: Written, formal. A measured way to make a claim; the spoken “-라고 할 수 있죠” is not for essays.
- 1: 독서는 인간의 사고력을 키우는 가장 기본적인 방법이라고 할 수 있다. / Reading can be called the most basic way to develop human thinking.
- 2: 이러한 변화는 사회 발전의 긍정적인 신호라고 할 수 있다. / This change can be called a positive sign of social progress.
- link: (none) · review: draft

**W018** `-(으)ㄴ/는 것으로 보인다` · it appears that …

- register: Written, formal. A hedge: use it when you are not sure rather than the spoken “-것 같아요”.
- 1: 이러한 현상은 앞으로도 한동안 계속될 것으로 보인다. / This phenomenon appears likely to continue for some time.
- 2: 조사 결과를 보면 소비자의 인식이 바뀌고 있는 것으로 보인다. / The survey results suggest that consumers' attitudes are changing.
- link: (none) · review: draft

**W019** `-는 것이 바람직하다` · it is desirable to …

- register: Written, formal. Recommendation without sounding like an order. Spoken “-는 게 좋아요 / -는 게 나아요” is too casual.
- 1: 학생들이 다양한 분야의 책을 읽는 것이 바람직하다. / It is desirable for students to read books from various fields.
- 2: 정부가 에너지 정책의 방향을 장기적인 관점에서 정하는 것이 바람직하다. / It is desirable for the government to set the direction of its energy policy from a long-term perspective.
- link: w:바람직하다 · review: draft

**W020** `-는 입장이다` · I take the position that …

- register: Written, formal. “입장” = stance; a clean way to open the body after a debate. Spoken “-는 입장이에요 / -편이에요” are not for essays.
- 1: 나는 조기 유학이 아이의 성장에 도움이 된다는 입장이다. / My position is that studying abroad early helps a child's growth.
- 2: 이에 대해 필자는 찬성하는 입장이다. / On this point I take the position of agreeing.
- link: w:입장 · review: draft

**W021** `-에 찬성한다 / -에 반대한다` · I am for / against …

- register: Written, formal. Short, clear stance for the thesis sentence. Spoken “찬성해요 / 반대해요” are not for essays.
- 1: 나는 대중교통 요금을 무료로 하는 정책에 찬성한다. / I am in favour of the policy of making public transport free.
- 2: 필자는 학교에서 교복을 의무로 입게 하는 것에 반대한다. / I am against making school uniforms compulsory.
- link: w:찬성하다 · review: draft

**W022** `-라는 의견에 동의한다` · I agree with the view that …

- register: Written, formal. Pair with a reason in the next sentence. Spoken “그 말에 동의해요 / 맞는 말이에요” are not for essays.
- 1: 나는 외국어 교육을 일찍 시작해야 한다는 의견에 동의한다. / I agree with the view that foreign-language education should start early.
- 2: 필자는 환경 보호를 위해 개인의 노력이 필요하다는 의견에 동의한다. / I agree with the view that individual effort is needed to protect the environment.
- link: w:동의하다 · review: draft

**W023** `-는 데 있다` · lies in …

- register: Written, formal. Names the key point: “-의 핵심은 -는 데 있다”. Spoken “-는 게 문제예요 / -는 거예요” is not for essays.
- 1: 문제의 핵심은 규칙이 지켜지지 않는다는 데 있다. / The heart of the problem is that the rules are not being followed.
- 2: 이 정책의 장점은 비용을 크게 줄일 수 있다는 데 있다. / The strength of this policy is that it can cut costs substantially.
- link: (none) · review: draft

**W024** `-에 달려 있다` · depends on …

- register: Written, formal. “문제의 해결은 -에 달려 있다”. Spoken “-에 달렸어요 / -에 따라 달라요” is too casual: write “-에 달려 있다” in an essay.
- 1: 환경 문제의 해결은 우리 모두의 실천에 달려 있다. / Solving environmental problems depends on the actions of all of us.
- 2: 사업의 성패는 소비자의 신뢰를 얻을 수 있는지에 달려 있다. / The success of the business depends on whether it can win consumers' trust.
- link: w:달리다 · review: draft

**W025** `-는 것은 사실이다` · it is true that …

- register: Written, formal. Concedes a fact before you argue against it (see also “-기는 하지만”). Spoken “-기는 해요 / 맞기는 해요” is too casual.
- 1: 인터넷이 생활을 편리하게 해 주는 것은 사실이다. / It is true that the internet makes life more convenient.
- 2: 경쟁이 사람을 성장시키는 면이 있는 것은 사실이다. / It is true that competition has a side that helps people grow.
- link: (none) · review: draft

**W026** `-(으)ㄹ 수 있다 / -(으)ㄹ 수도 있다` · can … / may even …

- register: Written. Softens a claim: 54 graders like cautious, balanced claims. The spoken “-ㄹ 수 있어요” changes only the ending.
- 1: 지나친 경쟁은 학생들에게 스트레스를 줄 수 있다. / Excessive competition can put students under stress.
- 2: 이러한 습관은 건강에 해로울 수도 있다. / Such a habit may even be harmful to health.
- link: g:-ㄹ 수 있다 · review: draft


### Giving reasons

**W027** `-기 때문이다` · the reason is that …

- register: Written, formal. Ends the sentence that gives the reason. Spoken counterparts “-거든요 / -잖아요 / -니까요” must not appear in an essay.
- 1: 이 정책이 필요한 이유는 사고를 예방할 수 있기 때문이다. / This policy is necessary because it can prevent accidents.
- 2: 많은 사람이 재택근무를 선호하는 것은 출퇴근 시간을 아낄 수 있기 때문이다. / Many people prefer working from home because they can save commuting time.
- link: g:-기 때문 · review: draft

**W028** `-기 때문에` · because …

- register: Written, formal. Goes mid-sentence; “-니까 / -아서” are spoken or conversational and weaker in an argument.
- 1: 전자책은 가볍고 휴대하기 편하기 때문에 많은 사람이 이용한다. / E-books are light and easy to carry, so many people use them.
- 2: 온라인 수업은 시간과 장소의 제약이 없기 때문에 직장인에게 유리하다. / Online classes have no limits of time or place, so they suit working people.
- link: g:-기 때문에 · review: draft

**W029** `-(으)ㄴ/는 이유는 -기 때문이다` · the reason … is that …

- register: Written, formal. A frame that keeps the reason explicit; keep the ending “-기 때문이다” (not “-거든요”). Spoken “-거든요 / -잖아요 / -니까요” must not be used in an essay.
- 1: 사람들이 도시로 몰리는 이유는 일자리가 많기 때문이다. / The reason people crowd into cities is that there are many jobs.
- 2: 이 방법이 효과적인 이유는 비용이 적게 들기 때문이다. / This method is effective because it costs little.
- link: (none) · review: draft

**W030** `-(으)로 인해` · because of …, due to …

- register: Written, formal. Takes a noun; in speech people say “-때문에 / -(으)로”.
- 1: 기후 변화로 인해 해마다 자연재해가 늘고 있다. / Because of climate change, natural disasters are increasing every year.
- 2: 과도한 업무로 인해 직장인의 스트레스가 커지고 있다. / Stress among office workers is growing because of excessive workloads.
- link: w:인하다 · review: draft

**W031** `-(으)로 말미암아` · owing to … (stronger, literary)

- register: Written, formal and a bit literary; use sparingly. Spoken “-아서 / -니까” are the everyday reason forms; this is a literary alternative for writing.
- 1: 산업화로 말미암아 도시의 인구가 빠르게 늘었다. / Owing to industrialisation, the urban population grew rapidly.
- 2: 잘못된 정보의 확산으로 말미암아 사회적 혼란이 생길 수 있다. / The spread of false information can cause social confusion.
- link: w:말미암다 · review: draft

**W032** `-(으)ㄴ/는 탓에` · because of … (unwelcome result)

- register: Written, neutral. Only for a negative result; use “덕분에” for a good one. Spoken “-아서 / -는 바람에” are conversational and weaker in an argument.
- 1: 교통량이 급증한 탓에 대기 오염이 심해졌다. / Air pollution has worsened because traffic volume rose sharply.
- 2: 운동할 시간이 부족한 탓에 현대인의 건강이 나빠지고 있다. / Modern people's health is getting worse for lack of time to exercise.
- link: g:-는 탓에 · review: draft

**W033** `-(으)ㄴ/는 덕분에` · thanks to … (good result)

- register: Written, neutral. The mirror of “탓에”: only for a favourable result. Spoken “-덕에 / -아서” are conversational.
- 1: 과학 기술이 발달한 덕분에 사람들의 수명이 늘어났다. / Thanks to advances in science and technology, human life expectancy has grown.
- 2: 교통이 편리해진 덕분에 지방에서도 도시의 문화를 즐길 수 있게 되었다. / Thanks to better transport, people can enjoy city culture even in the provinces.
- link: w:덕분 · review: draft

**W034** `-(으)므로` · since …, therefore …

- register: Written, formal; a stiff connective of reason that does not exist in speech (use “-니까 / -아서” there).
- 1: 환경 오염은 모든 사람에게 영향을 미치므로 함께 해결해야 한다. / Environmental pollution affects everyone, so we must solve it together.
- 2: 이 방법은 비용이 적게 드므로 많은 학교에서 활용할 수 있다. / This method costs little, so many schools can use it.
- link: g:-으므로 · review: draft

**W035** `-는 만큼` · given that …, in proportion to …

- register: Written, formal. Reason with an expectation attached: “중요한 문제인 만큼 신중히 해야 한다”. Spoken “-니까 / -잖아요” are too casual.
- 1: 개인 정보는 매우 중요한 만큼 철저히 보호해야 한다. / Since personal information is so important, it must be protected thoroughly.
- 2: 이 사업은 많은 사람에게 영향을 미치는 만큼 충분한 논의가 필요하다. / Given that this project affects many people, it needs sufficient discussion.
- link: g:-는 만큼 · review: draft

**W036** `-는 점에서` · in that …, from the point of view that …

- register: Written, formal. Gives the aspect that supports a claim. Spoken “-어서 / -라서” are too casual.
- 1: 이 제도는 비용을 줄일 수 있다는 점에서 의미가 있다. / This system is meaningful in that it can reduce costs.
- 2: 독서는 상상력을 키워 준다는 점에서 매우 유익하다. / Reading is very beneficial in that it nurtures the imagination.
- link: w:점 · review: draft

**W037** `-(으)ㄴ/는 데다가` · on top of that …

- register: Written, neutral. Adds a second reason of the same direction (good + good or bad + bad). The same form is used in speech; in an essay just keep the “한다체” ending and avoid the polite “-요”.
- 1: 대중교통은 요금이 저렴한 데다가 환경에도 도움이 된다. / Public transport is cheap and also helps the environment.
- 2: 도시 생활은 물가가 높은 데다가 주거 비용도 부담스럽다. / City life has high prices and heavy housing costs on top of that.
- link: g:-는 데다가 · review: draft

**W038** `-(으)ㄹ 수밖에 없다` · one cannot but …

- register: Written, formal. “어쩔 수 없다” is the spoken form; the writing form states the inevitable result.
- 1: 일자리가 부족하면 청년들이 도시로 떠날 수밖에 없다. / If jobs are scarce, young people have no choice but to leave for the cities.
- 2: 이 정책이 실패한다면 사회 전체의 부담이 커질 수밖에 없다. / If this policy fails, the burden on society as a whole will inevitably grow.
- link: g:-ㄹ 수밖에 없다 · review: draft


### Evidence and sources

**W039** `-에 따르면` · according to …

- register: Written, formal. Always followed by a report verb in the “-다고 한다 / -(으)ㄴ/는 것으로 나타났다” family. Spoken “-(ㄴ/는)대요 / -더라고요” for hearsay is too casual.
- 1: 통계청의 조사에 따르면 1인 가구가 해마다 늘고 있다고 한다. / According to a survey by Statistics Korea, one-person households are increasing every year.
- 2: 한 연구 결과에 따르면 운동은 스트레스를 줄이는 데 효과가 있다. / According to one study, exercise is effective in reducing stress.
- link: w:따르다 · review: draft

**W040** `-에 의하면` · according to … (reported or learned information)

- register: Written, formal. Interchangeable with “-에 따르면”; also common in news. Spoken “-(ㄴ/는)대요” is too casual.
- 1: 전문가들의 분석에 의하면 이러한 추세는 당분간 계속될 전망이다. / According to experts' analyses, this trend is expected to continue for a while.
- 2: 최근 발표된 보고서에 의하면 청소년의 수면 시간이 부족하다고 한다. / According to a recently published report, teenagers are not getting enough sleep.
- link: w:의하다 · review: draft

**W041** `-(으)로 알려져 있다` · it is known that …

- register: Written, formal. General knowledge or research; gives your claim backing without a named source. Spoken “-(ㄴ/는)대요 / -잖아요” are too casual.
- 1: 규칙적인 운동은 건강에 도움이 되는 것으로 알려져 있다. / Regular exercise is known to be good for health.
- 2: 이 방법은 학습 효과를 높이는 것으로 알려져 있다. / This method is known to improve learning effects.
- link: w:알려지다 · review: draft

**W042** `-(으)ㄴ/는 것으로 조사되었다` · it was found (in a survey) that …

- register: Written, formal. Typical for a survey result (53); passive voice; “-(으)ㄴ/는 것으로 나타났다” is the more common alternative. Spoken “-더라고요 / -던데요” must not appear in an essay.
- 1: 직장인 열 명 중 일곱 명이 스트레스를 받고 있는 것으로 조사되었다. / The survey found that seven out of ten office workers are under stress.
- 2: 청소년의 절반 이상이 하루 세 시간 넘게 스마트폰을 사용하는 것으로 조사되었다. / More than half of teenagers were found to use smartphones for over three hours a day.
- link: w:조사되다 · review: draft

**W043** `-(으)ㄴ/는 것으로 확인되었다` · it has been confirmed that …

- register: Written, formal. A stronger claim than “알려져 있다”: use it only for solid evidence. Spoken “-던데요 / -더라고요” are not for essays.
- 1: 실험 결과 이 방법이 학습 효과를 높이는 것으로 확인되었다. / The experiment confirmed that this method improves learning.
- 2: 새로운 치료법이 기존 방법보다 효과적인 것으로 확인되었다. / The new treatment was confirmed to be more effective than the existing one.
- link: w:확인되다 · review: draft

**W044** `조사 결과` · according to the survey results

- register: Written, formal. Opening phrase of a 53 sentence about a survey. Spoken “알아보니까 / 물어보니까” are too casual.
- 1: 조사 결과 응답자의 60%가 현재 직업에 만족하지 않는 것으로 나타났다. / The results showed that 60% of respondents are not satisfied with their current job.
- 2: 조사 결과 대부분의 학생이 온라인 수업을 선호하는 것으로 나타났다. / The survey showed that most students prefer online classes.
- link: w:조사 · review: draft

**W045** `-을/를 대상으로 조사한 결과` · a survey of … found that …

- register: Written, formal. Names who was asked; standard in 53. Spoken “물어봤더니 / 물어보니까” are too casual.
- 1: 20대 성인 남녀 1,000명을 대상으로 조사한 결과 절반 이상이 저축을 하지 않는 것으로 나타났다. / A survey of 1,000 men and women in their twenties found that over half do not save money.
- 2: 대학생을 대상으로 조사한 결과 취업 준비에 가장 큰 부담을 느끼는 요소는 경제적 문제였다. / A survey of university students found that financial problems weighed most heavily on their job preparations.
- link: w:대상 · review: draft

**W046** `-을/를 통해` · through …, by means of …

- register: Written, formal. “통해” is more formal than “-(으)로” and common in all writing. Spoken “-(으)로 / -해서” are plainer; “통해” is the written choice.
- 1: 사람들은 인터넷을 통해 다양한 정보를 쉽게 얻을 수 있다. / People can easily get various kinds of information through the internet.
- 2: 이러한 경험을 통해 학생들은 협동심을 기를 수 있다. / Through such experiences students can develop a spirit of cooperation.
- link: w:통하다 · review: draft

**W047** `-(이)라는 연구 결과가 있다` · there are studies showing that …

- register: Written, formal. Safe way to cite evidence without inventing a source in a 54 essay. Spoken “-(ㄴ/는)대요 / -잖아요” are too casual.
- 1: 충분한 수면이 학습 능력을 높인다는 연구 결과가 있다. / There are studies showing that sufficient sleep improves learning ability.
- 2: 책을 많이 읽는 아이가 어휘력이 높다는 연구 결과가 있다. / Studies show that children who read a lot have a richer vocabulary.
- link: w:연구 · review: draft

**W048** `-고 지적한다 / -고 주장한다` · point out / claim that …

- register: Written, formal. Report verbs for others' opinions; use “-다고 한다” for neutral reporting. Spoken “-(ㄴ/는)대요 / -라고 해요” is conversational; in writing use “-다고 한다” or these.
- 1: 전문가들은 일회용품 사용을 줄여야 한다고 지적한다. / Experts point out that the use of disposable items must be reduced.
- 2: 일부 학자들은 조기 교육이 아이의 창의력을 높인다고 주장한다. / Some scholars claim that early education boosts a child's creativity.
- link: w:지적하다 · review: draft

**W049** `-(으)ㄴ 바 있다` · has (have) …-ed, as is on record

- register: Written, formal. Reports a past fact or action; the spoken counterpart is “-(으)ㄴ 적이 있어요”.
- 1: 정부는 이미 이 문제에 대한 대책을 발표한 바 있다. / The government has already announced measures for this problem.
- 2: 여러 연구에서 비슷한 결과가 보고된 바 있다. / Similar results have been reported in several studies.
- link: (none) · review: draft

**W050** `실제로` · in fact, actually

- register: Written, neutral. Introduces concrete evidence for what you just claimed. Spoken “진짜로 / 정말로” are too casual for evidence.
- 1: 실제로 대중교통 이용률이 높은 도시는 대기 오염이 적다. / In fact, cities with high public transport use have less air pollution.
- 2: 실제로 많은 기업이 재택근무를 도입한 뒤 업무 효율이 높아졌다고 밝혔다. / In fact, many companies said work efficiency rose after they introduced working from home.
- link: w:실제로 · review: draft


### Adding information

**W051** `또한` · also, in addition

- register: Written, formal. The default additive connective of essays; spoken “그리고 / 또” are weaker and “근데” is wrong here.
- 1: 인터넷은 정보를 빠르게 전달한다. 또한 누구나 쉽게 의견을 표현할 수 있게 해 준다. / The internet delivers information quickly. It also lets anyone express an opinion easily.
- 2: 규칙적인 운동은 체력을 길러 준다. 또한 스트레스를 줄이는 데에도 도움이 된다. / Regular exercise builds stamina. It also helps reduce stress.
- link: w:또한 · review: draft

**W052** `-뿐만 아니라` · not only …, but also …

- register: Written, formal. “뿐만 아니라” can follow a noun or “-(으)ㄹ”; the spoken equivalent is “-(으)ㄹ 뿐 아니라” or “거기다”.
- 1: 독서는 지식을 쌓아 줄 뿐만 아니라 정서를 풍부하게 한다. / Reading not only builds knowledge but also enriches emotions.
- 2: 이 제도는 비용을 줄일 수 있을 뿐만 아니라 환경 보호에도 기여한다. / This system can not only cut costs but also contribute to protecting the environment.
- link: g:-을 뿐만 아니라 · review: draft

**W053** `아울러` · and also, along with this

- register: Written, formal. A step more formal than “또한”; often opens a sentence that adds a related point. Spoken “그리고 / 또” are too weak; avoid “그리고요”.
- 1: 정부는 지원금을 늘려야 한다. 아울러 지원 절차도 간소화해야 한다. / The government should increase subsidies. It should also simplify the application procedure.
- 2: 학교는 학생의 학업을 돌보아야 한다. 아울러 정서적인 어려움도 살펴야 한다. / Schools should look after students' studies. They should also pay attention to emotional difficulties.
- link: w:아울러 · review: draft

**W054** `게다가` · moreover, on top of that

- register: Written, neutral. Adds a stronger point; also used in speech, so keep it for the second, stronger reason.
- 1: 이 방법은 비용이 적게 든다. 게다가 효과도 빠르다. / This method costs little. On top of that, it works quickly.
- 2: 신문은 정보가 정확하다. 게다가 깊이 있는 분석을 접할 수 있다. / Newspapers are accurate. On top of that, they offer in-depth analysis.
- link: w:게다가 · review: draft

**W055** `더욱이` · furthermore, what is more

- register: Written, formal. Intensifies a case already made; “심지어” goes further to an extreme. Spoken “거기다 / 더구나요” are too casual.
- 1: 정보 보안은 개인에게 중요하다. 더욱이 기업에는 생존과 직결되는 문제이다. / Information security matters to individuals. What is more, for companies it is a matter of survival.
- 2: 이 정책은 예산이 많이 든다. 더욱이 효과도 분명하지 않다. / This policy costs a lot. Furthermore, its effect is not clear.
- link: w:더욱이 · review: draft

**W056** `나아가` · furthermore, going a step further

- register: Written, formal. Moves to the next, wider level (individual → society). Spoken “그리고 나서” is too casual.
- 1: 환경 교육은 학생의 생활 습관을 바꾼다. 나아가 사회 전체의 인식 변화로 이어진다. / Environmental education changes students' habits. It goes on to bring a shift in awareness across society.
- 2: 독서는 개인의 교양을 높인다. 나아가 사회의 지적 수준을 높이는 데 기여한다. / Reading raises an individual's cultivation. It goes further and contributes to raising a society's intellectual level.
- link: w:나아가다 · review: draft

**W057** `-와/과 더불어` · together with …, along with …

- register: Written, formal. Both "accompanied by" and "in parallel with". Spoken “같이 / 함께” are too plain here.
- 1: 경제 성장과 더불어 환경 문제도 심각해지고 있다. / Along with economic growth, environmental problems are becoming serious.
- 2: 인구 고령화와 더불어 의료비 부담이 커지고 있다. / Along with population ageing, the burden of medical costs is growing.
- link: w:더불다 · review: draft

**W058** `-는 동시에` · at the same time as …, while also …

- register: Written, formal. Two aspects of one thing at once. Spoken “-(으)면서 / -기도 해요” is too casual.
- 1: 인터넷은 편리함을 주는 동시에 개인 정보 유출의 위험도 안고 있다. / The internet gives convenience while also carrying the risk of personal data leaks.
- 2: 이 제도는 소비자를 보호하는 동시에 기업의 책임을 높인다. / This system protects consumers while also raising corporate responsibility.
- link: g:-는 동시에 · review: draft

**W059** `-(으)ㄹ 뿐 아니라` · not only … but also (variant)

- register: Written, formal. Same meaning as “뿐만 아니라”; the variant “뿐 아니라” is common when no noun precedes. No spoken difference in the form itself; keep the “한다체” ending and avoid the polite “-요”.
- 1: 그 지역은 교통이 편리할 뿐 아니라 자연환경도 뛰어나다. / That area is not only convenient to reach but also has an outstanding natural environment.
- 2: 운동은 몸을 건강하게 할 뿐 아니라 마음에도 안정을 준다. / Exercise not only keeps the body healthy but also gives peace of mind.
- link: g:-을 뿐만 아니라 · review: draft

**W060** `-기도 하다` · also …, … as well

- register: Written, neutral. Mentions a second function or fact with the same subject, often right after another verb. The form is the same in speech (“-기도 해요”); in an essay keep the “한다체” ending.
- 1: 스마트폰은 연락 수단이기도 하고 정보를 찾는 도구이기도 하다. / A smartphone is a means of contact and also a tool for finding information.
- 2: 휴식은 일의 능률을 높이기도 한다. / Rest also raises the efficiency of work.
- link: (none) · review: draft

**W061** `이와 함께` · along with this, in tandem

- register: Written, formal. Adds a parallel measure: “이와 함께 -도 필요하다”. Spoken “같이 / 함께 + -해요” is too casual.
- 1: 정부의 재정적 지원이 필요하다. 이와 함께 지역 사회의 참여도 중요하다. / Financial support from the government is needed. Along with it, participation by the local community is also important.
- 2: 학교는 환경 교육을 강화해야 한다. 이와 함께 가정에서의 실천도 병행되어야 한다. / Schools should strengthen environmental education. Together with this, practice at home should also go on.
- link: (none) · review: draft


### Contrast

**W062** `반면에 / -는 반면` · whereas, on the other hand

- register: Written, formal. Contrasts two things in one sentence (“-는 반면”) or across two “(반면에)”. Spoken “근데 / 그런데” do not mark contrast in an essay.
- 1: 도시는 일자리가 많은 반면 주거 비용이 높다. / Cities have plenty of jobs, whereas housing costs are high.
- 2: 온라인 쇼핑은 편리하다. 반면에 상품을 직접 확인할 수 없다는 단점이 있다. / Online shopping is convenient. On the other hand, it has the drawback that you cannot check the goods in person.
- link: g:-는 반면에 · review: draft

**W063** `그러나` · but, however

- register: Written, formal. The basic contrast connective; “하지만” is also fine but slightly less formal; “근데” is spoken.
- 1: 많은 사람이 이 제도를 지지한다. 그러나 부작용을 걱정하는 목소리도 적지 않다. / Many people support this system. However, quite a few voices worry about side effects.
- 2: 기술의 발달은 생활을 편리하게 했다. 그러나 새로운 문제도 가져왔다. / Technological progress has made life more convenient. However, it has also brought new problems.
- link: w:그러나 · review: draft

**W064** `하지만` · but, however

- register: Written, neutral. Less formal than “그러나” but fully acceptable in 54; avoid the spoken “근데 / 그렇지만 요”.
- 1: 이 방법은 효과가 빠르다. 하지만 비용이 많이 든다는 단점이 있다. / This method works quickly. However, it has the drawback of being costly.
- 2: 재택근무는 장점이 많다. 하지만 소통이 줄어든다는 문제가 있다. / Working from home has many advantages. However, it has the problem of reduced communication.
- link: w:하지만 · review: draft

**W065** `-와/과 달리` · unlike …, differing from …

- register: Written, formal. A noun-based way to set up a contrast; compare “이와 달리”. Spoken “-이랑 다르게 / -랑 달리” is too casual (“랑 / 이랑”).
- 1: 과거와 달리 오늘날에는 누구나 정보를 쉽게 얻을 수 있다. / Unlike in the past, anyone today can get information easily.
- 2: 종이책과 달리 전자책은 휴대하기가 편리하다. / Unlike paper books, e-books are easy to carry.
- link: w:달리 · review: draft

**W066** `-에 비해` · compared with …

- register: Written, formal. Comparison base: “A는 B에 비해 -다”. In speech people say “-보다”.
- 1: 도시는 농촌에 비해 문화 시설이 풍부하다. / Cities have richer cultural facilities than rural areas.
- 2: 지난해에 비해 온라인 수업을 선택한 학생이 크게 늘었다. / Compared with last year, many more students chose online classes.
- link: w:비하다 · review: draft

**W067** `-(으)ㄴ/는 데 비해` · whereas, compared with the fact that …

- register: Written, formal. Clause-level comparison; slightly weaker contrast than “-는 반면”. Spoken “-보다 / -는데” is too casual.
- 1: 선진국은 출산율이 낮아지고 있는 데 비해 일부 국가는 인구가 빠르게 늘고 있다. / While birth rates are falling in developed countries, the population is rising fast in some others.
- 2: 대도시는 일자리가 풍부한 데 비해 농촌은 일자리가 부족하다. / Big cities have plenty of jobs, whereas rural areas have too few.
- link: w:비하다 · review: draft

**W068** `이에 비해 / 이와 달리` · in comparison / by contrast

- register: Written, formal. Starts a sentence that sets a second group against the first (graphs, 53). Spoken “그런데 / 근데” do not mark a contrast in an essay.
- 1: 20대는 온라인 쇼핑을 자주 이용한다. 이에 비해 60대 이상은 이용 비율이 낮다. / People in their twenties often use online shopping. By comparison, use among people over sixty is low.
- 2: 도시 지역은 인구가 늘었다. 이와 달리 농촌 지역은 인구가 줄었다. / The population grew in urban areas. By contrast, it fell in rural areas.
- link: (none) · review: draft

**W069** `이와 반대로 / 이와는 대조적으로` · conversely / in contrast to this

- register: Written, formal. Marks a true opposite. Spoken “반대로 / 근데” are too casual; these two phrases mark a real opposite.
- 1: 편리한 교통 덕분에 도시 인구가 늘고 있다. 이와는 대조적으로 농촌 인구는 계속 줄고 있다. / Thanks to convenient transport the urban population is growing. In contrast, the rural population keeps shrinking.
- 2: 일부 학생은 경쟁을 통해 성장한다. 이와 반대로 어떤 학생은 경쟁 때문에 의욕을 잃는다. / Some students grow through competition. Conversely, others lose motivation because of it.
- link: w:대조적 · review: draft

**W070** `-는 데 반해` · whereas (stronger contrast)

- register: Written, formal. Sharper opposition than “비해”: the two facts point in opposite directions. Spoken “-는데 / -는 반면에요” are too casual.
- 1: 수출은 증가하는 데 반해 내수는 오히려 감소하고 있다. / While exports are rising, domestic demand is falling.
- 2: 젊은 층은 변화를 환영하는 데 반해 노년층은 변화를 부담스러워하는 경향이 있다. / Younger people welcome change, whereas older people tend to find it a burden.
- link: w:반하다 · review: draft

**W071** `오히려` · rather, on the contrary

- register: Written, neutral. Used when the result is the opposite of what was expected. Spoken “되레 / 도리어” are rarer; “오히려” is the standard written choice.
- 1: 일자리 지원 정책이 실업률을 낮추지 못하고 오히려 높였다. / The job-support policy did not lower the unemployment rate but actually raised it.
- 2: 지나친 간섭은 도움이 되기는커녕 오히려 아이의 자립심을 해친다. / Excessive interference does not help; on the contrary it damages a child's independence.
- link: w:오히려 · review: draft

**W072** `-기는커녕` · far from …, let alone …

- register: Written, neutral to formal. Strong denial and reversal; the spoken form is “-기는커녕” too, so keep to a formal context.
- 1: 이 정책은 문제를 해결하기는커녕 오히려 새로운 갈등을 낳았다. / Far from solving the problem, this policy gave rise to new conflicts.
- 2: 물가가 내리기는커녕 계속 오르고 있다. / Far from falling, prices keep rising.
- link: g:는커녕 · review: draft

**W073** `한편` · meanwhile, on the other hand

- register: Written, formal. Switches to another aspect or side; mild contrast, not a strong "however". Spoken “그런데 / 근데” are not for essays.
- 1: 도시화는 경제 성장을 이끌었다. 한편 지방의 인구 감소라는 문제도 낳았다. / Urbanisation drove economic growth. At the same time it gave rise to the problem of population decline in the provinces.
- 2: 많은 학생이 방학에 학원에 다닌다. 한편 직접 체험 활동에 참여하는 학생도 늘고 있다. / Many students attend cram schools in the holidays. Meanwhile, more are joining hands-on activities.
- link: w:한편 · review: draft


### Concession

**W074** `그럼에도 불구하고` · nevertheless, despite this

- register: Written, formal. Concedes a point and then goes on in the opposite direction. The shorter spoken “그래도 / 그런데도” is too casual here.
- 1: 이 제도는 많은 비판을 받았다. 그럼에도 불구하고 여전히 필요하다고 생각한다. / This system has been widely criticised. Nevertheless, I think it is still necessary.
- 2: 기술이 발달했다. 그럼에도 불구하고 환경 문제는 오히려 심각해지고 있다. / Technology has advanced. Nevertheless, environmental problems are in fact becoming more serious.
- link: g:에도 불구하고 · review: draft

**W075** `-음에도 불구하고` · despite …, in spite of …

- register: Written, formal. Attaches to a clause “(-ㅁ/음에도)” or a noun (“-에도 불구하고”). Spoken “그런데도 / -는데도” are too casual.
- 1: 정부가 여러 대책을 내놓았음에도 불구하고 집값은 계속 오르고 있다. / Despite the various measures the government introduced, house prices keep rising.
- 2: 높은 비용에도 불구하고 해외 유학을 선택하는 학생이 늘고 있다. / Despite the high cost, more students are choosing to study abroad.
- link: g:에도 불구하고 · review: draft

**W076** `-더라도` · even if …

- register: Written, neutral. Concedes a hypothetical; a very common essay concession. Spoken “-어도 / -아도” are plainer; “-더라도” is the standard in essays.
- 1: 시간이 오래 걸리더라도 올바른 방법을 선택해야 한다. / Even if it takes a long time, we must choose the right method.
- 2: 비용이 다소 들더라도 안전을 위한 투자는 필요하다. / Even if it costs somewhat, investment in safety is necessary.
- link: g:-더라도 · review: draft

**W077** `-(으)ㄹ지라도` · even if, even though

- register: Written, formal; slightly literary. For stronger "even if the worst happens". Spoken “-어도 / -더라도” are plainer; use “-ㄹ지라도” for a stronger written concession.
- 1: 결과가 좋지 않을지라도 도전하는 자세는 높이 평가받아야 한다. / Even if the result is not good, a willingness to take up challenges deserves high praise.
- 2: 일시적으로 손해를 볼지라도 장기적으로는 이익이 될 것이다. / Even if we suffer a loss temporarily, it will be to our advantage in the long run.
- link: g:-ㄹ지라도 · review: draft

**W078** `-기는 하지만` · it is … but …

- register: Written, neutral. Admit a point (“-기는 하다”), then give the opposite “(-지만)”: the standard "yes, but". Spoken “-기는 한데” is too casual.
- 1: 인터넷이 편리하기는 하지만 지나친 의존은 문제가 될 수 있다. / The internet is certainly convenient, but excessive reliance on it can become a problem.
- 2: 이 방법이 효과가 있기는 하지만 모든 경우에 적용할 수는 없다. / This method is effective, but it cannot be applied in every case.
- link: g:-기는 · review: draft

**W079** `물론 -지만` · of course …, but …

- register: Written, neutral. Concede then turn; shows balanced reasoning in 54. Spoken “물론이죠 / 맞는데” is too casual.
- 1: 물론 경쟁은 발전의 원동력이 되지만 지나치면 부작용을 낳는다. / Of course competition drives progress, but too much of it causes side effects.
- 2: 물론 온라인 수업이 편리하지만 직접 만나서 배우는 것과는 다르다. / Online classes are of course convenient, but they are not the same as learning face to face.
- link: w:물론 · review: draft

**W080** `비록 -지만 / 비록 -(으)ㄹ지라도` · even though … (with “비록”)

- register: Written, formal. “비록” signals the concession in advance and goes with “-지만 / -ㄹ지라도 / -더라도”. “비록” is used only in writing; in speech people say “아무리” or just “-지만”.
- 1: 비록 작은 실천이지만 이것이 모이면 큰 변화를 만들 수 있다. / Even though they are small acts, taken together they can bring about great change.
- 2: 비록 시간이 걸리겠지만 근본적인 해결책을 찾아야 한다. / Even though it will take time, we must find a fundamental solution.
- link: w:비록 · review: draft

**W081** `-(이)라고 해서 -는 것은 아니다` · just because …, it does not mean that …

- register: Written, formal. Rejects an assumption: a favourite 54 move before giving your view. Spoken “-다고 -는 건 아니에요” (“건” = “것은”) is too casual.
- 1: 경제가 성장했다고 해서 모든 국민이 행복해지는 것은 아니다. / Just because the economy has grown does not mean that every citizen becomes happy.
- 2: 기술이 발달했다고 해서 삶의 질이 저절로 높아지는 것은 아니다. / Technological progress does not automatically raise the quality of life.
- link: (none) · review: draft

**W082** `아무리 -어도` · no matter how …

- register: Written, neutral. Emphasises the extreme of a concession. The form is the same in speech; in an essay keep the “한다체” ending.
- 1: 아무리 바빠도 건강을 돌보는 일을 소홀히 해서는 안 된다. / No matter how busy we are, we must not neglect our health.
- 2: 아무리 기술이 발달해도 인간의 따뜻한 마음을 대신할 수는 없다. / No matter how advanced technology becomes, it cannot replace warm human feelings.
- link: g:-아도 · review: draft


### Cause and result

**W083** `따라서` · therefore, accordingly

- register: Written, formal. Draws a logical consequence; “그래서” is spoken, and “그러니까 / 그러니” is conversational; neither belongs in an argument.
- 1: 인구가 줄어들고 있다. 따라서 이에 대한 대책이 필요하다. / The population is declining. Therefore, measures against this are needed.
- 2: 대중교통은 환경에 부담을 덜 준다. 따라서 이용을 장려해야 한다. / Public transport puts less burden on the environment. Therefore its use should be encouraged.
- link: w:따라서 · review: draft

**W084** `그러므로` · therefore, so

- register: Written, formal. Slightly more bookish than “따라서”; both mark a conclusion from a reason. Do not confuse with “그럼으로(써)”. Spoken “그래서 / 그러니까” are conversational and not for an argument.
- 1: 건강은 한번 잃으면 되찾기 어렵다. 그러므로 평소에 관리하는 습관이 중요하다. / Health is hard to regain once lost. Therefore the habit of looking after it every day is important.
- 2: 정보는 힘이다. 그러므로 올바른 정보를 가려내는 능력을 길러야 한다. / Information is power. Therefore we must develop the ability to pick out correct information.
- link: w:그러므로 · review: draft

**W085** `그 결과` · as a result

- register: Written, formal. Links a result to a preceding cause; common in 53 to describe consequences. Spoken “그래서 / 그 때문에” are too casual.
- 1: 도시에 인구가 집중되었다. 그 결과 주택 가격이 급격히 올랐다. / The population became concentrated in cities. As a result, house prices rose sharply.
- 2: 투자가 꾸준히 늘었다. 그 결과 생산량이 두 배로 증가하였다. / Investment kept growing. As a result, output doubled.
- link: w:결과 · review: draft

**W086** `이로 인해 / 이 때문에` · because of this

- register: Written, formal. “이로 인해” is more formal than “이 때문에”; spoken “그래서 / 그 때문에” is weaker.
- 1: 기온이 상승하고 있다. 이로 인해 해수면도 높아지고 있다. / Temperatures are rising. Because of this, sea levels are rising as well.
- 2: 출산율이 낮아졌다. 이 때문에 학교의 학생 수가 크게 줄었다. / The birth rate has fallen. For this reason the number of pupils at schools has fallen sharply.
- link: (none) · review: draft

**W087** `이에 따라` · accordingly, in line with this

- register: Written, formal. A change that follows from the previous fact. Spoken “그래서” is too casual.
- 1: 온라인 쇼핑이 늘어났다. 이에 따라 오프라인 매장의 수는 줄고 있다. / Online shopping has grown. Accordingly, the number of physical shops is decreasing.
- 2: 고령 인구가 증가하고 있다. 이에 따라 의료 서비스에 대한 수요도 늘고 있다. / The elderly population is increasing. Accordingly, demand for medical services is also growing.
- link: (none) · review: draft

**W088** `-(으)ㅁ에 따라` · as …, in line with …

- register: Written, formal. A change that accompanies another change (as X rises, Y rises). Spoken “-(으)면 / -(으)니까” do not show that two things change together.
- 1: 소득이 증가함에 따라 소비 패턴도 변화하고 있다. / As incomes rise, consumption patterns are also changing.
- 2: 기술이 발달함에 따라 일자리의 종류도 달라지고 있다. / As technology advances, the kinds of jobs are also changing.
- link: w:따르다 · review: draft

**W089** `-을/를 초래하다` · bring about …, cause …

- register: Written, formal. For unwelcome results; do not use for good ones (use “가져오다”). Spoken “-게 만들어요 / -게 돼요” is too casual.
- 1: 무분별한 개발은 자연 환경의 파괴를 초래할 수 있다. / Indiscriminate development can bring about the destruction of the natural environment.
- 2: 잘못된 정책은 사회적 혼란을 초래한다. / A mistaken policy causes social confusion.
- link: w:초래하다 · review: draft

**W090** `-을/를 야기하다` · give rise to …, cause …

- register: Written, formal. Same as “초래하다”; used for problems, conflicts, confusion. Spoken “일으켜요 / 만들어요” are too casual.
- 1: 급격한 변화는 사회 구성원 사이에 갈등을 야기할 수 있다. / Sudden change can give rise to conflict among members of society.
- 2: 수면 부족은 집중력 저하를 야기한다. / Lack of sleep causes a drop in concentration.
- link: w:야기하다 · review: draft

**W091** `-을/를 낳다` · give rise to …, produce …

- register: Written, neutral to formal. Works for both good and bad outcomes. Spoken “만들어요 / 생겨요” are too casual.
- 1: 편리함의 추구가 새로운 사회 문제를 낳았다. / The pursuit of convenience gave rise to new social problems.
- 2: 한 사람의 작은 실천이 큰 변화를 낳을 수 있다. / One person's small act can produce great change.
- link: w:낳다 · review: draft

**W092** `-(으)로 이어지다` · lead to …

- register: Written, formal. Neutral about whether the result is good or bad. Spoken “-게 돼요 / 이어져요” is too casual.
- 1: 꾸준한 노력이 좋은 결과로 이어진다. / Steady effort leads to good results.
- 2: 사소한 갈등이 큰 분쟁으로 이어지는 경우가 많다. / Trivial conflicts often lead to major disputes.
- link: w:이어지다 · review: draft

**W093** `-에 영향을 미치다` · affect …, have an effect on …

- register: Written, formal. A staple of 54; “영향을 주다” is plainer. Spoken “영향 줘요 / 영향이 있어요” is too casual.
- 1: 수면 시간은 학업 성적에 큰 영향을 미친다. / Hours of sleep have a great effect on academic results.
- 2: 가정환경은 아이의 성격 형성에 영향을 미칠 수 있다. / The home environment can influence the formation of a child's character.
- link: w:영향 · review: draft

**W094** `-에 기여하다` · contribute to …

- register: Written, formal. For positive effects. Spoken “도움이 돼요 / 도움이 되죠” is too casual; “기여하다” is the written choice.
- 1: 대중교통 이용은 대기 오염을 줄이는 데 기여한다. / Using public transport contributes to reducing air pollution.
- 2: 이러한 노력은 사회 발전에 기여할 것이다. / Such efforts will contribute to the development of society.
- link: w:기여하다 · review: draft

**W095** `-(으)ㄹ 우려가 있다` · there is a risk that …

- register: Written, formal. Cautions about a possible bad outcome; the spoken “-(으)ㄹ까 봐 걱정이에요” is not used in essays.
- 1: 이대로 방치하면 문제가 더 심각해질 우려가 있다. / If it is left as it is, there is a risk that the problem will get worse.
- 2: 지나친 규제는 산업의 발전을 가로막을 우려가 있다. / Excessive regulation risks hindering the development of industry.
- link: w:우려 · review: draft

**W096** `-게 되다` · come to …, end up …

- register: Written, neutral. Reports a change of state or a result without naming an actor. Spoken “-게 됐어요” is too casual; write “-게 되었다”.
- 1: 인터넷의 보급으로 사람들은 더 쉽게 정보를 얻게 되었다. / With the spread of the internet, people came to obtain information more easily.
- 2: 사회가 변하면서 가족의 형태도 다양해지게 되었다. / As society changed, family forms came to be more varied.
- link: g:-게 되다 · review: draft

**W097** `-(으)ㄴ/는 이상` · since …, now that …

- register: Written, formal. Reason as a settled fact: “문제가 드러난 이상 해결해야 한다”. Spoken “-니까 / -었으니” is too casual.
- 1: 문제가 드러난 이상 하루빨리 대책을 마련해야 한다. / Now that the problem has come to light, measures must be prepared as soon as possible.
- 2: 사회의 구성원인 이상 공동체에 대한 책임을 다해야 한다. / As members of society, we must fulfil our responsibility to the community.
- link: g:-는 이상 · review: draft

**W098** `-(으)ㄹ 경우 / -(으)ㄴ다면` · if …, in case of …

- register: Written, formal. Conditions: “-(으)ㄹ 경우” for a case, “-(으)ㄴ다면” for a supposition. The spoken “-(으)면” is fine in writing too.
- 1: 이러한 상황이 계속될 경우 사회적 비용이 크게 늘어날 것이다. / If this situation continues, social costs will rise greatly.
- 2: 모든 사람이 조금씩 노력한다면 환경 문제를 해결할 수 있을 것이다. / If everyone makes a small effort, we will be able to solve the environmental problem.
- link: w:경우 · review: draft

**W099** `-지 않으면 -(으)ㄹ 것이다` · if we do not …, … will …

- register: Written, formal. A warning pattern for the close of an essay. Spoken “-(으)면 -(으)ㄹ 거예요 / -(으)ㄹ걸요” is too casual.
- 1: 지금 대책을 마련하지 않으면 문제는 더욱 심각해질 것이다. / If we do not prepare measures now, the problem will become even more serious.
- 2: 습관을 바꾸지 않으면 건강은 점점 나빠질 것이다. / If we do not change our habits, our health will get worse and worse.
- link: (none) · review: draft


### Graphs: introducing the data (53)

**W100** `-을/를 조사한 결과이다` · this is the result of a survey on …

- register: Written, formal “(한다체)”. Opening sentence of a 53 answer; name the topic, the group surveyed and the period. Spoken “조사해 봤어요” is too casual.
- 1: 이 그래프는 20대 직장인의 여가 활동을 조사한 결과이다. / This graph shows the results of a survey on leisure activities of office workers in their twenties.
- 2: 위 표는 최근 5년간 연령대별 인터넷 쇼핑 이용 현황을 조사한 결과이다. / The table above shows the results of a survey on online shopping use by age group over the last five years.
- link: w:조사 · review: draft

**W101** `-에 관한 설문 조사 결과를 나타낸 것이다` · this shows the results of a survey about …

- register: Written, formal. Alternative opening; “나타낸 것이다” is the written equivalent of "shows". Spoken “설문 조사 한 거예요” is too casual.
- 1: 이 그래프는 한국인의 독서 실태에 관한 설문 조사 결과를 나타낸 것이다. / This graph shows the results of a survey on reading habits in Korea.
- 2: 위 자료는 대학생의 취업 준비 방법에 관한 설문 조사 결과를 나타낸 것이다. / The data above show the results of a survey on how university students prepare for employment.
- link: w:설문 · review: draft

**W102** `-(으)ㄴ 것으로 나타났다` · it was found that …, the results show that …

- register: Written, formal. THE 53 verb: use it for every result; the spoken “-더라고요 / -인 것 같아요” must not appear.
- 1: 조사 결과 응답자의 45%가 영화 감상을 가장 선호하는 것으로 나타났다. / The survey showed that 45% of respondents prefer watching films most.
- 2: 2010년에 비해 2020년에는 재택근무를 하는 직장인이 세 배로 늘어난 것으로 나타났다. / Compared with 2010, the number of office workers working from home tripled by 2020.
- link: w:나타나다 · review: draft


### Graphs: changes and trends (53)

**W103** `-이/가 증가하다 / 늘어나다` · to increase, to grow

- register: Written, neutral. “증가하다” is the formal register of “늘어나다”; both are fine in a 53. Use “-(으)로” to give the new level. Spoken “늘었어요” is too casual; use “증가하였다” (or “늘었다” in “한다체”).
- 1: 도서 구입 비용이 2015년 30만 원에서 2020년 45만 원으로 증가하였다. / Spending on books rose from 300,000 won in 2015 to 450,000 won in 2020.
- 2: 해외로 나가는 여행객의 수가 해마다 늘어나고 있다. / The number of travellers going abroad is increasing every year.
- link: w:증가하다 · review: draft

**W104** `-이/가 감소하다 / 줄어들다` · to decrease, to fall

- register: Written, neutral. “감소하다” is the formal version of “줄어들다”; the spoken “줄었어요” is not used.
- 1: 신문 구독자 수가 2010년 이후 꾸준히 감소하고 있다. / The number of newspaper subscribers has been declining steadily since 2010.
- 2: 어린이의 수가 최근 10년 동안 약 20% 줄어들었다. / The number of children has fallen by roughly 20% over the last ten years.
- link: w:감소하다 · review: draft

**W105** `-이/가 상승하다 / 하락하다` · to rise / to fall (prices, rates, levels)

- register: Written, formal. Used with rates, prices, temperatures, ratios; not with people or things you can count. Spoken “올랐어요 / 떨어졌어요” is too casual; use “상승하였다 / 하락하였다”.
- 1: 쌀 가격이 지난해에 비해 15% 상승하였다. / The price of rice rose 15% compared with last year.
- 2: 청년 실업률은 2018년 이후 점차 하락하는 추세이다. / The youth unemployment rate has been gradually falling since 2018.
- link: w:상승하다 · review: draft

**W106** `급증하다 / 급감하다` · to surge / to plunge

- register: Written, formal. The “급-” prefix = sudden and large; use for the sharpest changes in a graph. Spoken “확 늘었어요 / 확 줄었어요” are too casual; avoid “확”, “팍”, “엄청”.
- 1: 온라인 쇼핑 이용자는 2020년 한 해 사이에 두 배로 급증하였다. / Online shoppers doubled in a single year, 2020.
- 2: 해외여행객 수는 2020년 한 해 동안 급감하였다. / The number of overseas travellers plunged in the single year 2020.
- link: w:급증하다 · review: draft

**W107** `대폭 / 크게 증가하다` · to increase sharply, greatly

- register: Written, formal. Adverbs that scale a change: “크게 / 대폭 / 꾸준히 / 소폭 / 다소”. Avoid the spoken “엄청 / 되게 / 많이”.
- 1: 스마트폰 이용 시간이 3년 만에 두 배로 대폭 증가하였다. / Time spent on smartphones increased sharply, doubling in just three years.
- 2: 전기 자동차의 판매량이 지난 5년간 크게 늘었다. / Sales of electric cars rose greatly over the last five years.
- link: w:대폭 · review: draft

**W108** `꾸준히 증가하다 / 증가세를 보이다` · to rise steadily / to show an upward trend

- register: Written, formal. “꾸준히” = steadily; “-세를 보이다” = shows a … trend (“증가세”, “감소세”, “상승세”, “하락세”). Spoken “계속 늘고 있어요” is too casual.
- 1: 1인 가구의 비율은 꾸준히 증가하여 2020년에는 30%를 넘어섰다. / The share of one-person households kept rising and exceeded 30% in 2020.
- 2: 노인 인구는 최근 10년간 지속적인 증가세를 보이고 있다. / The elderly population has shown a continuous upward trend over the last decade.
- link: w:꾸준히 · review: draft

**W109** `감소세를 보이다` · to show a downward trend

- register: Written, formal. Pair with a period: “최근 -년간 / 2015년 이후”. Spoken “계속 줄고 있어요” is too casual.
- 1: 담배 소비량은 2015년 이후 감소세를 보이고 있다. / Cigarette consumption has been falling since 2015.
- 2: 종이 신문의 발행 부수는 매년 감소세를 보였다. / Paper newspaper circulation showed a downward trend every year.
- link: w:감소세 · review: draft

**W110** `-(으)로 증가하다 / -에서 -(으)로` · to rise to / to rise from A to B

- register: Written, formal. State the start and end: “A에서 B로 증가/감소하였다”. This pattern is the core of every 53 sentence about change. Spoken “-에서 -(으)로 늘었어요” is too casual: write “-에서 -(으)로 증가하였다”.
- 1: 외국인 관광객은 2015년 1,300만 명에서 2019년 1,750만 명으로 증가하였다. / Foreign tourists rose from 13 million in 2015 to 17.5 million in 2019.
- 2: 흡연율은 1990년 35%에서 2020년 18%로 감소하였다. / The smoking rate fell from 35% in 1990 to 18% in 2020.
- link: g:-에서 · review: draft

**W111** `변화가 없다 / -을/를 유지하다` · to stay the same, to remain at a level

- register: Written, formal. “변화가 없다 / 비슷한 수준을 유지하다” = flat segments of a graph. Spoken “똑같아요 / 그대로예요” is too casual.
- 1: 2018년 이후 이용자 수는 비슷한 수준을 유지하고 있다. / Since 2018 the number of users has remained at a similar level.
- 2: 여성의 취업률은 지난 3년 동안 큰 변화 없이 50% 안팎을 유지하였다. / The employment rate of women stayed around 50% with little change over the past three years.
- link: w:유지하다 · review: draft

**W112** `-에 이르다 / -에 도달하다` · to reach …, to amount to …

- register: Written, formal. Marks a peak or a milestone figure. Spoken “-까지 갔어요 / -이 됐어요” is too casual.
- 1: 2023년에는 전체 인구의 20%에 이를 것으로 예상된다. / By 2023 it is expected to reach 20% of the whole population.
- 2: 온라인 쇼핑 거래액은 2020년에 처음으로 150조 원에 도달하였다. / The value of online shopping transactions reached 150 trillion won for the first time in 2020.
- link: w:이르다 · review: draft

**W113** `최고치를 기록하다` · to hit a record high

- register: Written, formal. “최고치” = highest value; “최저치를 기록하다” for a record low. Spoken “제일 많았어요” is too casual; write “최고치를 기록하였다”.
- 1: 수출액은 2021년에 사상 최고치를 기록하였다. / Exports hit a record high in 2021.
- 2: 실업률은 지난해 가장 낮은 수준을 기록하였다. / The unemployment rate hit its lowest level last year.
- link: w:최고치 · review: draft

**W114** `-(으)로 집계되었다` · was tallied at …, came to …

- register: Written, formal. For officially counted figures: “통계청 집계 / -(으)로 집계되었다”. Spoken “나왔어요” is too casual.
- 1: 지난해 국내 방문 외국인은 총 1,750만 명으로 집계되었다. / The total number of foreign visitors to Korea last year came to 17.5 million.
- 2: 올해 신입생 수는 지난해보다 5% 줄어든 것으로 집계되었다. / The number of new students this year was tallied as 5% lower than last year.
- link: w:집계되다 · review: draft

**W115** `-다가 다시 -다` · …, then … again

- register: Written, neutral. A change of direction within one sentence: “감소하다가 다시 증가하다”. Spoken “-었다가 다시 -었어요” is too casual.
- 1: 이용자 수는 2015년까지 증가하다가 이후 다시 감소하였다. / The number of users increased until 2015 and then declined again.
- 2: 물가는 한동안 하락하다가 올해 다시 상승하는 추세이다. / Prices fell for a while and then have trended upwards again this year.
- link: g:-다가 · review: draft

**W116** `-에 이어` · following …, after …

- register: Written, formal. For sequencing trends over several years or a ranking: “A에 이어 B가 증가하였다”. Spoken “-고 나서 / 그다음에” are too casual.
- 1: 2018년의 감소에 이어 2019년에도 판매량이 줄어들었다. / Following the decline of 2018, sales fell again in 2019.
- 2: 20대에 이어 30대의 이용률도 크게 높아졌다. / Following those in their twenties, use among those in their thirties also rose greatly.
- link: w:이어 · review: draft

**W117** `해마다 / 매년 / 지속적으로` · every year / continuously

- register: Written, neutral. Time adverbs for repeated change. Spoken “계속 / 점점” are too casual.
- 1: 해마다 어린이 놀이터가 줄고 있다. / Children's playgrounds are decreasing every year.
- 2: 소득이 지속적으로 늘면서 외식 비용도 증가하고 있다. / As income keeps rising, spending on eating out is also increasing.
- link: w:해마다 · review: draft


### Graphs: figures, shares, comparison (53)

**W118** `-을/를 차지하다` · to account for …, to make up …

- register: Written, formal. Share of a total: “전체의 -%를 차지하다”. The most common verb in a pie-chart description. Spoken “차지해요” / -% “정도예요” is too casual.
- 1: 20대가 전체 이용자의 40%를 차지하여 가장 높은 비율을 보였다. / People in their twenties made up 40% of all users, the highest share.
- 2: 독서가 취미인 사람은 전체의 15%를 차지하였다. / People whose hobby is reading accounted for 15% of the total.
- link: w:차지하다 · review: draft

**W119** `-에 달하다` · to amount to …, to reach as many as …

- register: Written, formal. Emphasises a large figure: “무려 -에 달하다” = as many as. Spoken “-이나 돼요” is too casual.
- 1: 인터넷 쇼핑 경험이 있는 응답자는 전체의 85%에 달하였다. / Respondents with online shopping experience came to 85% of the total.
- 2: 도시로 이동한 인구가 한 해에 100만 명에 달하는 것으로 나타났다. / The population that moved to cities came to one million in a single year.
- link: w:달하다 · review: draft

**W120** `-에 불과하다 / -에 그치다` · to be only …, to be a mere …

- register: Written, formal. Stresses that a figure is small. Spoken “-밖에 안 돼요” is too casual; use “-에 불과하다”.
- 1: 외국어를 구사할 수 있는 응답자는 10%에 불과하였다. / Respondents who could speak a foreign language were only 10%.
- 2: 전기차의 비율은 아직 전체의 3%에 그쳤다. / Electric cars still made up only 3% of the total.
- link: w:불과하다 · review: draft

**W121** `두 배 이상 / -배로` · more than double / -fold

- register: Written, formal. Multiples: “두 배”, “세 배”; “A는 B의 두 배이다” = A is twice B. Spoken “두 배나 돼요 / 두 배가 넘어요” is too casual.
- 1: 2020년의 온라인 수업 수강생은 2015년의 두 배 이상이다. / The number of online-class students in 2020 is more than double that in 2015.
- 2: 소득은 10년 만에 약 세 배로 늘었다. / Income grew roughly threefold in ten years.
- link: w:배 · review: draft

**W122** `절반 / 3분의 1 / -분의 -` · half / one third / fractions

- register: Written, formal. Fractions are read with “분의”: “3분의” 1, “4분의” 3. Spoken “반쯤 / 삼분의 일” are read aloud forms; in writing use “절반” and “3분의” 1.
- 1: 응답자의 절반 가량이 주 1회 이상 운동을 한다고 답하였다. / About half of the respondents said they exercise at least once a week.
- 2: 전체 가구의 3분의 1이 1인 가구인 것으로 나타났다. / One third of all households turned out to be one-person households.
- link: w:절반 · review: draft

**W123** `-가량 / 약 / 대략` · about, approximately

- register: Written, formal. “약” goes before the figure; “가량” follows it. Spoken “-쯤 / -정도” is too casual; “쯤” is spoken.
- 1: 응답자의 약 60%가 대중교통을 이용하는 것으로 나타났다. / About 60% of respondents were found to use public transport.
- 2: 한국인은 하루에 평균 두 시간가량 스마트폰을 사용한다. / Koreans use a smartphone for about two hours a day on average.
- link: w:가량 · review: draft

**W124** `-이/가 가장 높게 나타났다` · … was the highest

- register: Written, formal. Superlative in a graph: “가장 높게 / 낮게 나타났다”. Spoken “제일 높았어요” is too casual.
- 1: 연령대별로는 30대의 이용률이 가장 높게 나타났다. / By age group, people in their thirties had the highest rate of use.
- 2: 지역별로는 서울의 물가 상승률이 가장 낮게 나타났다. / By region, Seoul had the lowest rate of price increase.
- link: (none) · review: draft

**W125** `그 뒤를 이었다 / -(이)가 뒤를 이었다` · followed (in rank)

- register: Written, formal. Ranks the second and later items: “A가 가장 많았고 B가 그 뒤를 이었다”. Spoken “그다음은 -예요” is too casual.
- 1: 선호하는 여가 활동은 영화 감상이 가장 많았고 운동이 그 뒤를 이었다. / Watching films was the most preferred leisure activity, followed by exercise.
- 2: 이용 이유로는 편리함이 1위였으며 낮은 가격이 그 뒤를 이었다. / Convenience came first as a reason for using it, followed by low prices.
- link: w:뒤 · review: draft

**W126** `-순으로 나타났다` · in the order of …

- register: Written, formal. Lists items in rank order: “높은 순으로”, “많은 순으로”. Spoken “-순이에요” is too casual.
- 1: 응답은 건강, 경제적 안정, 가족의 순으로 나타났다. / Answers came in the order of health, economic security and family.
- 2: 이용률은 20대, 30대, 40대 순으로 높았다. / Rates of use were highest among those in their twenties, thirties and forties, in that order.
- link: w:순 · review: draft

**W127** `-보다 -이/가 더 많다 / 적다` · more / fewer than …

- register: Written, formal. Comparison between groups; “상대적으로” adds "relatively". Spoken “-보다 더 많아요” is too casual.
- 1: 남성보다 여성의 이용률이 상대적으로 더 높았다. / Use among women was relatively higher than among men.
- 2: 도시 지역은 농촌 지역보다 의료 시설이 훨씬 많다. / Urban areas have far more medical facilities than rural areas.
- link: w:상대적 · review: draft

**W128** `-을/를 웃돌다 / -을/를 밑돌다` · to exceed / to fall short of …

- register: Written, formal. Compares a figure with a benchmark (average, last year, target). Spoken “넘었어요 / 못 미쳤어요” is too casual.
- 1: 올해 수출액은 예상치를 웃돌았다. / This year's exports exceeded the forecast.
- 2: 청년의 투표율은 전체 평균을 밑돌았다. / The turnout of young people fell below the overall average.
- link: w:웃돌다 · review: draft

**W129** `-(으)ㅁ에 따라 / -(으)ㄹ수록` · as A rises, B …

- register: Written, formal. Links two figures that move together. The pattern is the same in speech; in an essay keep the “한다체” ending.
- 1: 연령이 높아짐에 따라 온라인 쇼핑 이용률은 낮아지는 것으로 나타났다. / As age rises, the rate of online shopping falls.
- 2: 소득 수준이 높을수록 문화생활에 쓰는 비용도 증가하였다. / The higher the income level, the more is spent on cultural activities.
- link: g:-을수록 · review: draft

**W130** `-(으)ㄹ 것으로 전망된다 / 예상된다` · is forecast to …

- register: Written, formal. Closing sentence of a 53: a forecast from the trend. Spoken “-(으)ㄹ 것 같아요” is too casual.
- 1: 이러한 추세는 앞으로도 계속될 것으로 전망된다. / This trend is expected to continue in the future.
- 2: 2030년에는 노인 인구가 전체의 25%를 넘을 것으로 예상된다. / By 2030 the elderly population is expected to exceed 25% of the total.
- link: w:전망되다 · review: draft


### Examples

**W131** `예를 들어 / 예를 들면` · for example

- register: Written, neutral. Introduces a concrete example; “예를 들어” is neutral and fine in an essay. The spoken “가령 / 이를테면” are acceptable too.
- 1: 환경을 위한 실천은 일상에서 시작할 수 있다. 예를 들어 일회용 컵 대신 개인 컵을 사용할 수 있다. / Acting for the environment can start in daily life. For example, you can use your own cup instead of a disposable one.
- 2: 다양한 방법이 있다. 예를 들어 도서관을 이용하거나 중고 책을 구입할 수 있다. / There are various ways. For example, you can use a library or buy second-hand books.
- link: w:예시 · review: draft

**W132** `-을/를 들 수 있다` · … can be cited (as an example)

- register: Written, formal. Closes a list of examples: A, B “등을 들 수 있다”. Spoken “-이 있어요 / -이에요” is too casual.
- 1: 대표적인 예로 재활용과 에너지 절약을 들 수 있다. / Recycling and saving energy can be cited as representative examples.
- 2: 이러한 문제의 원인으로 지나친 경쟁과 입시 중심의 교육을 들 수 있다. / Excessive competition and exam-centred education can be cited as causes of such problems.
- link: w:들다 · review: draft

**W133** `대표적인 예로` · as a representative example

- register: Written, formal. Picks the best-known example. Spoken “예를 들면 -요” is too casual.
- 1: 대표적인 예로 전자 결제 서비스를 꼽을 수 있다. / Electronic payment services can be picked out as a representative example.
- 2: 우리 생활을 바꾼 대표적인 예로 스마트폰을 들 수 있다. / The smartphone is a representative example of something that has changed our lives.
- link: w:대표적 · review: draft

**W134** `구체적으로 / 일례로` · specifically / as one example

- register: Written, formal. Moves from a claim to specifics. Spoken “예를 들면 / 가령” are plainer; “일례로” is the written choice.
- 1: 이 정책은 효과가 크다. 구체적으로 연간 비용이 20% 줄어들었다. / This policy has a big effect. Specifically, annual costs fell by 20%.
- 2: 일례로 서울에서는 자전거 이용자가 5년 사이에 두 배로 늘었다. / As one example, in Seoul the number of cyclists doubled in five years.
- link: w:구체적 · review: draft

**W135** `-는 바와 같이 / -(으)ㄹ 수 있듯이` · as …, as can be seen

- register: Written, formal. Points the reader back to an example or figure; spoken “-처럼” is too casual.
- 1: 앞서 살펴본 바와 같이 이러한 현상은 여러 원인이 겹쳐서 나타난다. / As we saw earlier, this phenomenon arises from several causes overlapping.
- 2: 위의 예에서 볼 수 있듯이 작은 습관이 큰 차이를 만든다. / As can be seen from the example above, small habits make a big difference.
- link: g:같이 · review: draft

**W136** `특히` · especially, in particular

- register: Written, neutral. Picks out the strongest case. Spoken “특별히 / 그중에서도” is too casual.
- 1: 환경 오염은 모든 사람에게 영향을 미치지만 특히 어린이에게 큰 피해를 준다. / Environmental pollution affects everyone but does particular harm to children.
- 2: 이 문제는 특히 도시 지역에서 심각하다. / This problem is particularly serious in urban areas.
- link: w:특히 · review: draft


### Necessity and proposals

**W137** `-어야 한다` · must, should

- register: Written, formal. The basic obligation: “한다체 -(아/어)야 한다”. Spoken “-(아/어)야 돼요” and “-(으)ㄹ 필요 있어요” sound casual in an essay.
- 1: 정부는 청년 일자리를 늘리기 위한 대책을 마련해야 한다. / The government must prepare measures to increase jobs for young people.
- 2: 우리는 환경을 보호하려는 노력을 일상에서부터 실천해야 한다. / We must put efforts to protect the environment into practice starting from daily life.
- link: g:-어야 하다 · review: draft

**W138** `-(으)ㄹ 필요가 있다` · there is a need to …

- register: Written, formal. Softer than “-어야 한다”; typical in recommendations. Spoken “-어야 돼요 / -어야 해요” is too casual.
- 1: 이를 해결하기 위해서는 사회적 합의를 이끌어 낼 필요가 있다. / To solve this, there is a need to draw out a social consensus.
- 2: 학생들이 스스로 생각하는 힘을 기를 수 있도록 교육 방식을 바꿀 필요가 있다. / The way of teaching needs to change so that students can develop the ability to think for themselves.
- link: w:필요 · review: draft

**W139** `-(으)ㄹ 필요가 없다` · there is no need to …

- register: Written, formal. Rejects an unnecessary measure. Spoken “-(으)ㄹ 필요 없어요” is too casual.
- 1: 이미 충분히 갖춰진 제도를 새로 만들 필요는 없다. / There is no need to create anew a system that is already adequately in place.
- 2: 이러한 변화를 지나치게 걱정할 필요는 없다. / There is no need to worry too much about this change.
- link: w:필요 · review: draft

**W140** `-는 방안을 모색해야 한다` · we should look for ways to …

- register: Written, formal. “모색하다” = to seek; typical conclusion verb. Spoken “방법을 찾아야 해요” is too casual.
- 1: 정부는 저출산 문제를 해결할 수 있는 방안을 모색해야 한다. / The government should look for ways to solve the problem of the low birth rate.
- 2: 기업은 직원의 일과 삶의 균형을 지킬 수 있는 방안을 모색해야 한다. / Companies should seek ways to protect their employees' work-life balance.
- link: w:모색하다 · review: draft

**W141** `-을/를 마련하다` · to prepare, to put in place

- register: Written, formal. “대책/방안/제도/기반을 마련하다”. Spoken “만들어야 해요 / 준비해야 해요” is too casual; “마련하다” is the written choice.
- 1: 이를 해결하기 위한 제도적 장치를 마련해야 한다. / Institutional mechanisms to resolve this must be put in place.
- 2: 정부는 노인 복지를 위한 구체적인 대책을 마련해야 한다. / The government must prepare concrete measures for the welfare of the elderly.
- link: w:마련하다 · review: draft

**W142** `-이/가 요구되다 / 필요하다` · … is required, … is needed

- register: Written, formal. Passive: “노력이 요구된다” sounds more formal than “필요하다”. Spoken “필요해요 / 해야 해요” is too casual.
- 1: 이 문제를 해결하기 위해서는 사회 구성원 모두의 노력이 요구된다. / To solve this problem, effort is required from all members of society.
- 2: 새로운 시대에는 변화에 적응하는 능력이 요구된다. / The new era requires the ability to adapt to change.
- link: w:요구되다 · review: draft

**W143** `-이/가 시급하다` · there is a pressing need / … is urgent

- register: Written, formal. “시급하다” = urgent; use it for the strongest recommendations. Spoken “급해요 / 빨리 해야 돼요” is too casual.
- 1: 환경 오염을 줄이기 위한 대책 마련이 시급하다. / Preparing measures to reduce environmental pollution is urgent.
- 2: 청소년 보호를 위한 법적 장치가 시급히 필요하다. / Legal safeguards for the protection of young people are urgently needed.
- link: w:시급하다 · review: draft

**W144** `-도록 해야 한다` · we should make sure that …

- register: Written, formal. Purpose or result: “-도록 + 하다/노력하다”. Spoken “-게 해야 돼요” is too casual.
- 1: 모든 학생이 공평하게 교육받을 수 있도록 해야 한다. / We should make sure that all students can be educated fairly.
- 2: 개인 정보가 유출되지 않도록 철저히 관리해야 한다. / Personal information must be managed thoroughly so that it does not leak.
- link: g:-도록 · review: draft

**W145** `-기 위해서는 -어야 한다` · in order to …, we must …

- register: Written, formal. Purpose clause + obligation: the backbone of a recommendation. Spoken “-려면 -어야 돼요” is too casual.
- 1: 목표를 이루기 위해서는 꾸준한 노력이 있어야 한다. / In order to achieve a goal, there must be steady effort.
- 2: 건강한 사회를 만들기 위해서는 개인과 정부가 함께 노력해야 한다. / To create a healthy society, individuals and the government must work together.
- link: g:-기 위해서 · review: draft


### Conclusion

**W146** `결론적으로` · in conclusion

- register: Written, formal. Opens the last paragraph of a 54 essay. The spoken “그래서 결국 / 어쨌든” is not used.
- 1: 결론적으로 나는 학교에서 환경 교육을 강화해야 한다고 생각한다. / In conclusion, I think environmental education should be strengthened in schools.
- 2: 결론적으로 기술의 발달은 우리 생활에 긍정적인 영향을 미친다고 할 수 있다. / In conclusion, it can be said that technological progress has a positive effect on our lives.
- link: w:결론적 · review: draft

**W147** `요컨대` · in short, to sum up

- register: Written, formal. Very concise summary: “요컨대” + the key claim. Spoken “한마디로 / 그러니까” is too casual.
- 1: 요컨대 문제의 해결은 개인의 노력과 사회의 지원이 함께 이루어질 때 가능하다. / In short, the problem can be solved only when individual effort and social support go together.
- 2: 요컨대 편리함과 안전은 서로 조화를 이루어야 한다. / In short, convenience and safety must be brought into harmony.
- link: w:요컨대 · review: draft

**W148** `이처럼 / 이와 같이` · as such, in this way

- register: Written, formal. Sums up the preceding paragraph before the conclusion. Spoken “이렇게 / 이래서” are too casual.
- 1: 이처럼 독서는 개인의 성장과 사회의 발전에 모두 도움이 된다. / In this way, reading helps both the growth of the individual and the development of society.
- 2: 이와 같이 환경 문제는 우리 모두의 노력이 필요한 과제이다. / Thus, the environmental problem is a task that needs the effort of all of us.
- link: (none) · review: draft

**W149** `이상에서 살펴본 바와 같이` · as examined above

- register: Written, formal. A longer, essay-like closing signpost. Spoken “지금까지 말한 대로” is too casual.
- 1: 이상에서 살펴본 바와 같이 청소년의 스마트폰 과의존은 사회적 대책이 필요한 문제이다. / As examined above, teenagers' over-dependence on smartphones is a problem that needs social measures.
- 2: 이상에서 살펴본 바와 같이 이 정책은 단기적으로는 어려움이 있으나 장기적으로는 도움이 된다. / As examined above, this policy has difficulties in the short term but is helpful in the long term.
- link: g:같이 · review: draft

**W150** `종합해 보면 / 요약하면` · to sum up, taken together

- register: Written, formal. Summarising several points. Spoken “정리하면 / 한마디로” is too casual.
- 1: 종합해 보면 대중교통 이용을 늘리는 것이 도시 문제 해결의 첫걸음이다. / Taken together, increasing the use of public transport is the first step in solving urban problems.
- 2: 요약하면 이 정책의 핵심은 비용을 줄이고 효율을 높이는 것이다. / To summarise, the heart of this policy is to cut costs and raise efficiency.
- link: w:종합하다 · review: draft

**W151** `결국` · in the end, after all

- register: Written, neutral. Points to the final outcome or underlying truth; “결국은” is also fine. Spoken “어쨌든 / 그냥” is too casual.
- 1: 결국 문제를 해결하는 것은 사람들의 작은 실천이다. / In the end, what solves the problem is people's small acts.
- 2: 결국 변화의 출발점은 개인의 인식 전환이다. / In the end, the starting point of change is a change in individual awareness.
- link: w:결국 · review: draft

**W152** `마지막으로 / 끝으로` · finally, lastly

- register: Written, neutral. Introduces the last point or recommendation of a conclusion. Spoken “마지막은 -예요” is too casual.
- 1: 마지막으로 정부와 기업의 지속적인 관심과 투자가 필요하다. / Lastly, continuous attention and investment from government and companies are needed.
- 2: 끝으로 개인의 작은 실천이 모여 큰 변화를 만든다는 점을 강조하고 싶다. / Finally, I would like to stress that small individual acts add up to create great change.
- link: w:마지막으로 · review: draft

**W153** `-어야 할 것이다` · (we) should …, it will be necessary to …

- register: Written, formal. Soft, authoritative closing: states what should be done, in the future form. Not “-어야 될 거예요”.
- 1: 앞으로는 개인과 사회가 함께 이 문제에 관심을 가져야 할 것이다. / In future, individuals and society should take an interest in this problem together.
- 2: 더 늦기 전에 우리 모두가 환경 보호에 앞장서야 할 것이다. / Before it is too late, all of us should take the lead in protecting the environment.
- link: g:-어야 하다 · review: draft

**W154** `-(으)ㄹ 것으로 기대된다` · it is hoped that …

- register: Written, formal. Hopeful ending line. Spoken “-(으)ㄹ 거 같아요 / 기대돼요” is too casual.
- 1: 이러한 노력이 계속된다면 문제가 점차 해결될 것으로 기대된다. / If such efforts continue, the problem is expected to be gradually solved.
- 2: 앞으로 이 제도가 정착되면 사회 전체에 긍정적인 변화가 나타날 것으로 기대된다. / Once the system takes root, positive change is expected to appear across society.
- link: w:기대되다 · review: draft

**W155** `-기를 바란다` · I hope that …

- register: Written, formal “(한다체)”. A closing wish. Spoken “-기를 바라요 / -면 좋겠어요” is too casual.
- 1: 이 글을 통해 많은 사람이 환경 문제에 관심을 가지게 되기를 바란다. / I hope that through this piece many people will come to take an interest in environmental problems.
- 2: 앞으로 더 많은 사람이 독서의 즐거움을 알게 되기를 바란다. / I hope that more people will come to know the joy of reading.
- link: w:바라다 · review: draft


### Notices and e-mails (51–52)

**W156** `-고자 합니다` · I would like to …, I am writing in order to …

- register: Formal polite “(합쇼체)”. For the practical texts of 51: notices, e-mails, announcements. Not “-(으)려고 해요”.
- 1: 이번 학기 학생회에서는 신입생 환영 행사를 열고자 합니다. / This semester the student council would like to hold a welcome event for new students.
- 2: 저희 동아리에서는 새로운 회원을 모집하고자 합니다. / Our club would like to recruit new members.
- link: g:-고자 · review: draft

**W157** `-시기 바랍니다` · please …

- register: Formal polite “(합쇼체)”. Polite request in a notice; “부탁드립니다” is also acceptable. Spoken “-세요 / -해 주세요” is too plain for a formal notice.
- 1: 참석하실 분은 이번 주 금요일까지 신청해 주시기 바랍니다. / Those who will attend are asked to apply by this Friday.
- 2: 문의 사항이 있으시면 아래 번호로 연락해 주시기 바랍니다. / If you have any questions, please contact the number below.
- link: w:바라다 · review: draft

**W158** `-(으)ㄹ 예정입니다` · is scheduled to …

- register: Formal polite “(합쇼체)”. Announces a plan or schedule: “일시와 장소는 -입니다”. Spoken “-(으)ㄹ 거예요 / -(으)ㄹ게요” is too casual in a notice.
- 1: 도서관은 다음 달 1일부터 새롭게 문을 열 예정입니다. / The library is scheduled to reopen from the 1st of next month.
- 2: 행사는 이번 주 토요일 오후 두 시에 시작될 예정입니다. / The event is scheduled to start at two o'clock this Saturday afternoon.
- link: w:예정 · review: draft

**W159** `-(으)ㄹ 수 있도록 해 주시면 감사하겠습니다` · I would be grateful if you could …

- register: Formal polite “(합쇼체)”. Polite request in an e-mail to a professor or a company. Spoken “해 주시면 좋겠어요 / -어 주세요” is too casual.
- 1: 신청서를 이번 주까지 제출할 수 있도록 해 주시면 감사하겠습니다. / I would be grateful if you could let me hand in the application by this week.
- 2: 자료를 미리 받아 볼 수 있도록 해 주시면 감사하겠습니다. / I would be grateful if I could receive the materials in advance.
- link: w:감사하다 · review: draft

**W160** `-게 되어 죄송합니다` · I am sorry that … had to …

- register: Formal polite “(합쇼체)”. Apology in an e-mail or notice; add “양해해 주시기 바랍니다” to ask for understanding. Spoken “미안해요 / 죄송해요” is too casual in an e-mail.
- 1: 개인 사정으로 약속 시간을 변경하게 되어 죄송합니다. / I apologise for having to change the appointment time because of personal circumstances.
- 2: 갑작스러운 사정으로 행사가 연기된 점 양해해 주시기 바랍니다. / Please understand that the event has been postponed for unforeseen reasons.
- link: w:양해 · review: draft

**W161** `-(으)ㄴ/는 것이다` · the fact is that …, it is that … (emphasising an explanation)

- register: Written, formal “(한다체)”. Typical for the closing line of a 52 passage that explains something; it turns the sentence into an explanation. Not spoken “-거든요”.
- 1: 따라서 잘 쉬는 것도 성공적인 일의 중요한 부분인 것이다. / So resting well is also an important part of work that succeeds.
- 2: 결국 습관이 사람을 만드는 것이다. / In the end, it is habit that makes the person.
- link: (none) · review: draft

**W162** `-(으)ㄴ/는 경향이 있다` · there is a tendency to …

- register: Written, formal “(한다체)”. General tendency, common in 52 text completion. Spoken “-는 경향이에요 / -는 편이에요” is too casual.
- 1: 사람들은 자신이 이미 가진 생각과 일치하는 정보를 더 쉽게 받아들이는 경향이 있다. / People tend to accept information that matches their existing ideas more readily.
- 2: 시간이 부족할수록 중요한 일을 미루는 경향이 있다. / The less time we have, the more we tend to put off important things.
- link: w:경향 · review: draft


## Confusable groups


**C001** [spelling] `반드시 | 반듯이`

- explanation: “반드시” is an adverb meaning “without fail, certainly”. “반듯이” comes from “반듯하다” and means “straight, neatly, upright”. They sound almost alike but have nothing in common in meaning.
- 반드시: 약속은 반드시 지켜야 한다. / A promise must be kept without fail.
- 반듯이: 허리를 반듯이 펴고 앉았다. / He sat with his back held straight.
- common mistake: Writing “반듯이” for “certainly”. If you can replace it with “꼭 / 틀림없이”, use “반드시”.
- review: draft

**C002** [spelling] `결재 | 결제`

- explanation: “결재” (決裁) is approval of a plan or document by a person with authority. “결제” (決濟) is settling a payment or an account. In everyday life they are mixed up most with card payments.
- 결재: 부장님께 보고서를 올려 결재를 받았다. / I got the report approved by my department head.
- 결제: 카드로 대금을 결제했다. / I paid the bill by card.
- common mistake: Writing “카드 결재” for a card payment; it is always “카드 결제”.
- review: draft

**C003** [spelling] `개발 | 계발`

- explanation: “개발” (開發) is to develop something new or to open up land, resources and technology. “계발” (啓發) is to develop what is already inside a person: talent, potential, intelligence.
- 개발: 회사는 새로운 소프트웨어를 개발했다. / The company developed new software.
- 계발: 학생들의 창의력을 계발하는 교육이 필요하다. / Education that develops students' creativity is needed.
- common mistake: Using “개발” for talents and “계발” for products. “자기 계발” (self-improvement) is the set phrase; many people also write “자기 개발”, which dictionaries now accept too, but in a test stay with “계발” for abilities.
- review: draft

**C004** [spelling] `지양 | 지향`

- explanation: “지양” (止揚) means to avoid or stop doing something (the thing you give up). “지향” (志向) means to aim at something (the direction you head in). They are near opposites but sound alike.
- 지양: 일회용품 사용은 지양해야 한다. / The use of disposable items should be avoided.
- 지향: 우리 학교는 학생 중심의 교육을 지향한다. / Our school aims for student-centred education.
- common mistake: Writing “과소비를 지향하다” to mean avoid overspending. The thing you avoid goes with “지양”, the goal with “지향”.
- review: draft

**C005** [spelling] `혼동 | 혼돈 | 혼란`

- explanation: “혼동” is to mix up two things and take one for the other. “혼돈” is chaos, a state with no order (often of the world or society). “혼란” is confusion or disorder that people feel or cause.
- 혼동: 두 단어의 뜻을 혼동하기 쉽다. / It is easy to confuse the meanings of the two words.
- 혼돈: 전쟁 직후 사회는 혼돈 상태에 빠졌다. / Right after the war, society fell into chaos.
- 혼란: 갑작스러운 발표로 시장에 혼란이 일어났다. / The sudden announcement caused confusion in the market.
- common mistake: Writing “혼돈하다” for “to confuse (one thing for another)”; the verb is “혼동하다”. “혼돈” is a noun for chaos.
- review: draft

**C006** [spelling] `일체 | 일절`

- explanation: Both are written 一切. “일체” means “everything, all”. “일절” means “not at all, entirely” and is used only with a negative or a ban.
- 일체: 이 행사에 드는 비용 일체는 회사가 부담한다. / The company bears all the costs of this event.
- 일절: 이곳에서는 흡연을 일절 금지한다. / Smoking is strictly forbidden here.
- common mistake: Using “일체” with a negative (“일체 안 된다”). With a negative or a ban, say “일절”.
- review: draft

**C007** [spelling] `체제 | 체계 | 체재`

- explanation: “체제” (體制) is the structure of a country or an organisation: “정치 체제”, “경제 체제”. “체계” (體系) is a set of things arranged in a logical order: “교육 체계”, “체계적으로”. “체재” (滯在) is staying in a place away from home for a while.
- 체제: 그 나라는 민주주의 체제를 유지하고 있다. / That country maintains a democratic system of government.
- 체계: 학습한 내용을 체계 있게 정리해야 한다. / You have to organise what you learn in a systematic way.
- 체재: 그는 출장으로 한 달간 파리에 체재했다. / He stayed in Paris for a month on a business trip.
- common mistake: Writing “체제적으로” for “systematically”; the adverb is “체계적으로”.
- review: draft

**C008** [spelling] `실재 | 실제`

- explanation: “실재” (實在) is real existence, being there in fact: “실재하다”. “실제” (實際) means the real situation or what actually happens, as against theory or rumour: “실제로”, “실제 상황”.
- 실재: 신화에 나오는 동물은 실재하지 않는다. / The animals in myths do not exist.
- 실제: 실제로 가 보니 소문과 달랐다. / When I actually went there, it was different from the rumours.
- common mistake: Writing “실재로” for “in fact”; the adverb is “실제로”. “실재” is the verb root “실재하다” (to exist).
- review: draft

**C009** [spelling] `재고 | 제고`

- explanation: “재고” (再考) is to think again and reconsider. “제고” (提高) is to raise a level, quality or awareness.
- 재고: 이 계획은 재고할 필요가 있다. / This plan needs to be reconsidered.
- 제고: 서비스의 질을 제고하기 위해 노력해야 한다. / We must make efforts to raise the quality of service.
- common mistake: Writing “재고” for “raise” (“질을 재고하다”). Raising something is “제고”. Another “재고” (在庫, stock in the warehouse) is a third word.
- review: draft

**C010** [spelling] `갱신 | 경신`

- explanation: Both come from 更新. “갱신” means renewing or updating something so that it continues: “계약 갱신”, “여권 갱신”. “경신” means breaking a record by setting a new one: “기록 경신”.
- 갱신: 이번 달에 운전면허를 갱신해야 한다. / I have to renew my driving licence this month.
- 경신: 선수가 세계 기록을 경신했다. / The athlete broke the world record.
- common mistake: Writing “기록 갱신” for breaking a record (common in news); strictly it is “기록 경신”.
- review: draft

**C011** [spelling] `부문 | 부분`

- explanation: “부문” is a category or field into which something is divided: “연기 부문”, “경제 부문”. “부분” is a part of a whole: “앞부분”, “일부분”.
- 부문: 그는 연기 부문에서 상을 받았다. / He won an award in the acting category.
- 부분: 이 글의 앞부분은 이해하기 쉽다. / The first part of this text is easy to understand.
- common mistake: Writing “부분별 수상자” for the winners by category; the divisions of a field are “부문”.
- review: draft

**C012** [spelling] `한참 | 한창`

- explanation: “한참” is a fairly long time or distance (“한참 동안”). “한창” is the height of something, its peak (“한창 -고 있다”, “한창때”).
- 한참: 그는 한참 동안 아무 말도 하지 않았다. / He did not say a word for quite a while.
- 한창: 지금은 벚꽃이 한창이다. / The cherry blossoms are now in full bloom.
- common mistake: Writing “한창 동안” for “for a long time”. A span of time is “한참 동안”.
- review: draft

**C013** [spelling] `바람 | ×바램`

- explanation: The noun from “바라다” (to wish) is “바람”. “바램” is a very common wrong form. The verb stem is “바라-”, so “바라요”, not “×바래요”.
- 바람: 나의 가장 큰 바람은 가족의 건강이다. / My greatest wish is my family's health.
- ×바램: (wrong spelling, no sentence)
- common mistake: Writing “바램” for “wish” and “바래요” for “I hope”. Correct: “바람”, “바라요”.
- review: draft

**C014** [spelling] `금세 | 금새`

- explanation: “금세” is short for “금시에” and means “soon, in no time”. “금새” is a different, rare word meaning the price of an item.
- 금세: 아이는 눕자마자 금세 잠이 들었다. / The child fell asleep in no time after lying down.
- 금새: 시장마다 채소의 금새가 조금씩 다르다. / The price of vegetables differs a little from market to market.
- common mistake: Writing “금새” for “soon”. “금방” and “금세” both mean “right away”; “금새” is not one of them.
- review: draft

**C015** [spelling] `며칠 | ×몇 일`

- explanation: There is no word “몇 일”. “며칠” is the one word used both for “how many days” and “what date”.
- 며칠: 며칠 동안 비가 계속 내렸다. / It kept raining for several days.
- ×몇 일: (wrong spelling, no sentence)
- common mistake: Writing “몇 일” (“오늘이 몇 일이에요?”). Always “며칠”.
- review: draft

**C016** [spelling] `오랜만 | 오랫동안`

- explanation: “오랜만” means “after a long time” (a shortened form of “오래간만”) and goes with “에”. “오랫동안” means “for a long time” and is written as one word with a “ㅅ” in the middle.
- 오랜만: 오랜만에 고향 친구를 만났다. / I met a hometown friend after a long time.
- 오랫동안: 그는 오랫동안 한국에서 살았다. / He has lived in Korea for a long time.
- common mistake: Writing “×오랫만에” (with “ㅅ”). The correct form is “오랜만에”.
- review: draft

**C017** [spelling] `설레다 | ×설레이다`

- explanation: The verb is “설레다” (to flutter with excitement); the noun is “설렘”. Writing “설레이다” or “설레임” is a common mistake.
- 설레다: 첫 출근을 앞두고 가슴이 설렌다. / My heart flutters on the eve of my first day at work.
- ×설레이다: (wrong spelling, no sentence)
- common mistake: Writing “가슴이 설레인다” or “설레임”. Correct: “설렌다”, “설렘”.
- review: draft

**C018** [spelling] `헤매다 | ×헤메다`

- explanation: The verb is “헤매다” (to wander about, to be lost). Its stem is “헤매-”: “헤맨다”, “헤맸다”.
- 헤매다: 길을 몰라 한참을 헤맸다. / I didn't know the way and wandered around for a long time.
- ×헤메다: (wrong spelling, no sentence)
- common mistake: Writing “헤메다 / 헤멨다”. Correct: “헤매다 / 헤맸다”.
- review: draft

**C019** [spelling] `삼가다 | ×삼가하다`

- explanation: “삼가다” is a verb in itself (to refrain, to be careful). Do not add “-하다”. The adverb “삼가” (respectfully) is a different word.
- 삼가다: 공공장소에서는 큰 소리로 말하는 것을 삼가시기 바랍니다. / Please refrain from talking loudly in public places.
- ×삼가하다: (wrong spelling, no sentence)
- common mistake: Writing “삼가하다 / 삼가해 주십시오”. Correct: “삼가다 / 삼가 주십시오”.
- review: draft

**C020** [spelling] `무릅쓰다 | ×무릎쓰다`

- explanation: “무릅쓰다” means to endure something difficult or dangerous and go on. It has nothing to do with “무릎” (the knee).
- 무릅쓰다: 그는 위험을 무릅쓰고 아이를 구했다. / He risked danger to save the child.
- ×무릎쓰다: (wrong spelling, no sentence)
- common mistake: Writing “무릎쓰고”. Correct: “무릅쓰고”.
- review: draft

**C021** [spelling] `깨끗이 | 꼼꼼히 | 틈틈이`

- explanation: Adverb endings follow a rule. After a stem with a “ㅅ” final consonant “(깨끗하다)” or a reduplicated noun (“틈틈”, “곳곳”, “집집”) the ending is “-이”. After other stems that end in “-하다” (“꼼꼼하다”, “조용하다”) it is “-히”.
- 깨끗이: 손을 깨끗이 씻었다. / I washed my hands clean.
- 꼼꼼히: 계약서를 꼼꼼히 읽어 보았다. / I read the contract carefully.
- 틈틈이: 틈틈이 책을 읽는 습관이 필요하다. / You need the habit of reading in your spare moments.
- common mistake: Writing “깨끗히” or “꼼꼼이”. Rule of thumb: “ㅅ” final or repeated noun → “-이”; other “-하다” words → “-히”.
- review: draft

**C022** [spelling] `비율 | 확률`

- explanation: After a vowel or “ㄴ”, the suffix 率 is written “율” (“비율”, “이자율”, “백분율”). After any other consonant, it is “률” (“확률”, “성공률”, “출석률”).
- 비율: 전체에서 여성이 차지하는 비율이 높다. / The proportion of women in the total is high.
- 확률: 내일 비가 올 확률은 30%이다. / The probability of rain tomorrow is 30%.
- common mistake: Writing “비률 / 확율”. The “률/율” rule depends on the sound before it (vowel or “ㄴ → 율”).
- review: draft

**C023** [spelling] `어떻게 | 어떡해`

- explanation: “어떻게” is the adverb “how”. “어떡해” is a contraction of “어떻게 해” and stands on its own as “what should I do?”.
- 어떻게: 이 문제를 어떻게 풀어야 할까? / How should I solve this problem?
- 어떡해: 지갑을 잃어버렸는데 어떡해! / I lost my wallet, what do I do!
- common mistake: Writing “어떻게!” as an exclamation of distress or “어떡게”. Distress is “어떡해”.
- review: draft

**C024** [spelling] `웬 | 왠지`

- explanation: “웬” (from “어찌 된”) modifies a noun: “웬일”, “웬 사람”. “왠지” is a contraction of “왜인지” and means “for some reason”. “왠” only appears in “왠지”.
- 웬: 이게 웬 일이니? / What on earth is this?
- 왠지: 오늘은 왠지 기분이 좋다. / For some reason I feel good today.
- common mistake: Writing “왠일” (correct: “웬일”) or “웬지” (correct: “왠지”).
- review: draft

**C025** [spelling] `안 | 않다`

- explanation: “안” is a short form of “아니” and comes before a verb or adjective: “안 가다”. “않다” is a verb that follows “-지”: “가지 않다”. Both give the same negation.
- 안: 나는 커피를 안 마신다. / I don't drink coffee.
- 않다: 나는 커피를 마시지 않는다. / I don't drink coffee.
- common mistake: Writing “않 먹었다” or “먹지 안 았다”. “않-” only appears in “-지 않다”; the short “안” goes before the verb.
- review: draft

**C026** [spelling] `되 | 돼`

- explanation: “돼” is the contraction of “되어”. A quick test: replace with “하 / 해”. If “하” fits, write “되”; if “해” fits, write “돼”.
- 되: 그는 꿈을 이루어 의사가 되었다. / He fulfilled his dream and became a doctor.
- 돼: 이제 그만 가도 돼. / You can leave now.
- common mistake: Writing “안 되요 / 되서 / 되요”. Correct: “안 돼요 / 돼서 / 돼요”.
- review: draft

**C027** [spelling] `안되다 | 안 되다`

- explanation: “안되다” (one word) is an adjective: to feel sorry for, or to go badly (“일이 안되다”, “안됐다”). “안 되다” (spaced) is the negation of “되다”: not become, not be allowed.
- 안되다: 그 소식을 들으니 참 안됐다. / I felt very sorry to hear that news.
- 안 되다: 이곳에서는 사진을 찍으면 안 된다. / You must not take photos here.
- common mistake: Writing “사진을 찍으면 안되요”. “Not allowed” is “안 돼요” (two words).
- review: draft

**C028** [spelling] `못하다 | 못 하다`

- explanation: “못하다” (one word) means “to be bad at” (“노래를 못하다”) or “cannot at all”. “못 하다” (spaced) means that someone cannot do it because of circumstances.
- 못하다: 그는 노래를 못해서 노래방에 가지 않는다. / He is bad at singing, so he does not go to karaoke.
- 못 하다: 몸이 아파서 숙제를 못 했다. / I could not do my homework because I was sick.
- common mistake: Mixing the two spellings. Rule of thumb: skill or level → “못하다”; cannot because of circumstances → “못 하다”.
- review: draft

**C029** [spelling] `-ㄴ 지 | -ㄴ지`

- explanation: “-(으)ㄴ 지” (spaced; “지” is a noun for time) means “since …”: “온 지 3년”. “-(으)ㄴ지” (attached) is an ending for an indirect question or doubt: “맞는지 모르다”.
- -ㄴ 지: 한국에 온 지 3년이 되었다. / It has been three years since I came to Korea.
- -ㄴ지: 이것이 맞는지 확인해 보았다. / I checked whether this is right.
- common mistake: Writing “온지 3년이 되었다” or “맞는 지 확인했다”. “Since” needs a space; “whether” does not.
- review: draft

**C030** [spelling] `받치다 | 받히다 | 바치다`

- explanation: “받치다” is to hold up from below (“우산을 받치다”). “받히다” is the passive of “받다”: to be butted or hit. “바치다” is to offer or devote (“목숨을 바치다”).
- 받치다: 책 밑에 받침을 받쳐 놓았다. / I propped a stand underneath the book.
- 받히다: 그는 소에게 받혀 크게 다쳤다. / He was gored by a bull and badly hurt.
- 바치다: 그는 평생을 연구에 바쳤다. / He devoted his whole life to research.
- common mistake: Writing “소에게 받쳤다” (should be “받혔다”) or “목숨을 받쳤다” (should be “바쳤다”).
- review: draft

**C031** [spelling] `부딪치다 | 부딪히다`

- explanation: “부딪치다” is active: to strike something, or two things strike each other. “부딪히다” is passive: to be struck, or to run into something such as an obstacle (“차에 부딪히다”, “난관에 부딪히다”).
- 부딪치다: 두 사람이 모퉁이에서 서로 부딪쳤다. / The two people bumped into each other at the corner.
- 부딪히다: 사업은 시작부터 큰 어려움에 부딪혔다. / The business ran into great difficulties from the start.
- common mistake: Writing “어려움에 부딪쳤다” where you mean you ran into difficulty. The thing that stops you hits you: “부딪히다”.
- review: draft

**C032** [spelling] `걷잡다 | 겉잡다`

- explanation: “걷잡다” is to stop or hold back something that is spreading; it appears in “걷잡을 수 없다” (cannot be stopped). “겉잡다” is to estimate roughly (“겉잡아 말하다”).
- 걷잡다: 불길이 걷잡을 수 없이 번졌다. / The fire spread out of control.
- 겉잡다: 이 일은 겉잡아도 사흘은 걸린다. / This job will take at least three days, even at a rough estimate.
- common mistake: Writing “겉잡을 수 없다”; it is always “걷잡을 수 없다”.
- review: draft

**C033** [spelling] `띄다 | 띠다`

- explanation: “띄다” is to catch the eye (“눈에 띄다”) or to leave a gap (“간격을 띄다”). “띠다” is to wear or carry something with you: “미소를 띠다”, “붉은빛을 띠다”, “임무를 띠다”.
- 띄다: 거리에서 눈에 띄는 간판을 발견했다. / I spotted a conspicuous sign on the street.
- 띠다: 그의 얼굴은 미소를 띠고 있었다. / His face wore a smile.
- common mistake: Writing “미소를 띄다” or “눈에 띠다”. The verb that carries a colour or an expression is “띠다”.
- review: draft

**C034** [spelling] `쫓다 | 좇다`

- explanation: “쫓다” is to chase after or drive away (“도둑을 쫓다”, “파리를 쫓다”). “좇다” is to follow or pursue something abstract such as a dream, an ideal, or someone's advice.
- 쫓다: 경찰이 도망가는 도둑을 쫓았다. / The police chased the fleeing thief.
- 좇다: 그는 오로지 돈만을 좇아 살아왔다. / He has lived pursuing nothing but money.
- common mistake: Writing “꿈을 쫓다”. For goals, ideals and opinions, standard Korean prefers “좇다”, but “쫓다” is widely used.
- review: draft

**C035** [meaning] `바라다 | 원하다`

- explanation: “바라다” is to hope for something that depends on events or on other people; it often takes “-기를 바라다”. “원하다” is to want something and ask for it; its object is a thing, a person or a state you could actually get.
- 바라다: 나는 모든 사람이 행복하기를 바란다. / I hope that everyone will be happy.
- 원하다: 그는 안정적인 직업을 원한다. / He wants a stable job.
- common mistake: Writing “바래요 / 바램” (the stem is “바라-”: “바라요”, “바람”) and “-기를 원하다” for a hope that depends on others, where “-기를 바라다” is more natural.
- review: draft

**C036** [meaning] `다르다 | 틀리다`

- explanation: “다르다” means “different”: A is not the same as B. “틀리다” means “wrong, incorrect”, or something going wrong. They are one of the most common mix-ups in speech.
- 다르다: 두 사람의 생각은 서로 다르다. / The two people's ideas differ from each other.
- 틀리다: 이 문제의 답은 틀렸다. / The answer to this question is wrong.
- common mistake: Saying “생각이 틀리다” for “my opinion is different”. Use “틀리다” only for right/wrong.
- review: draft

**C037** [meaning] `맞추다 | 맞히다`

- explanation: “맞추다” is to fit or adjust things to one another: “시간에 맞추다”, “입을 맞추다”, “답을 맞춰 보다” (compare answers). “맞히다” is to get something right or to hit a target: “정답을 맞히다”.
- 맞추다: 우리는 약속 시간에 맞추어 도착했다. / We arrived to fit in with the appointed time.
- 맞히다: 그는 퀴즈의 정답을 모두 맞혔다. / He got all the answers in the quiz right.
- common mistake: Writing “정답을 맞췄다” for getting the correct answer; it is “맞혔다”. “답을 맞춰 보다” (check answers against each other) is correct.
- review: draft

**C038** [meaning] `가르치다 | 가리키다`

- explanation: “가르치다” is to teach (someone a subject). “가리키다” is to point at something with a finger or an arrow. There is no word “가르키다”.
- 가르치다: 선생님은 학생들에게 한국어를 가르친다. / The teacher teaches Korean to the students.
- 가리키다: 그는 손가락으로 지도의 한 지점을 가리켰다. / He pointed to a spot on the map with his finger.
- common mistake: Writing “가르키다” or “손가락으로 가르치다”. Pointing is “가리키다”.
- review: draft

**C039** [meaning] `늘이다 | 늘리다`

- explanation: “늘이다” is to make longer by stretching: “고무줄을 늘이다”. “늘리다” is to increase quantity or size: “수를 늘리다”, “재산을 늘리다”.
- 늘이다: 그는 고무줄을 길게 늘여 보았다. / He stretched the elastic band out long.
- 늘리다: 정부는 일자리를 늘리겠다고 약속했다. / The government promised to increase the number of jobs.
- common mistake: Writing “바지 길이를 늘렸다” (should be “늘였다”) or “인원을 늘였다” (should be “늘렸다”).
- review: draft

**C040** [meaning] `부치다 | 붙이다`

- explanation: “부치다” is to send (mail, money, a parcel) and also to fry (“전을 부치다”). “붙이다” is to stick on or attach, and also to add or start (“불을 붙이다”, “이름을 붙이다”).
- 부치다: 어머니께 소포를 부쳤다. / I sent a parcel to my mother.
- 붙이다: 벽에 포스터를 붙였다. / I stuck a poster on the wall.
- common mistake: Writing “편지를 붙였다” or “우표를 부쳤다”; it is “편지를 부쳤다” and “우표를 붙였다”.
- review: draft

**C041** [meaning] `이따가 | 있다가`

- explanation: “이따가” is an adverb meaning “later, in a while”. “있다가” is “있다” + “-다가”: stay for a while and then do something else.
- 이따가: 이따가 다시 연락드리겠습니다. / I will contact you again later.
- 있다가: 집에 있다가 친구의 전화를 받고 나갔다. / I was at home when I got a call from a friend and went out.
- common mistake: Writing “있다가 연락할게” for “later”. If you can say “나중에”, it is “이따가”; if “있다 + 가다 / 오다 …” follows, it is “있다가”.
- review: draft

**C042** [meaning] `그러므로 | 그럼으로(써)`

- explanation: “그러므로” is a conjunction meaning “therefore” (“그러하다 + -므로”). “그럼으로(써)” means “by doing so”, “by that means”; it is “그렇게 하다 + -(으)ㅁ으로써”.
- 그러므로: 시간은 한정되어 있다. 그러므로 계획이 필요하다. / Time is limited. Therefore a plan is necessary.
- 그럼으로(써): 그는 매일 연습했다. 그럼으로써 실력을 크게 높였다. / He practised every day, and by doing so he raised his skill greatly.
- common mistake: Writing “그럼으로 인해 …” for “therefore”. Use “그러므로 / 따라서” for results and “그럼으로써” for “by doing that”.
- review: draft

**C043** [meaning] `-대 | -데`

- explanation: “-(ㄴ/는)대” is the contraction of “-다고 해”: I report what someone said. “-데” is the contraction of “-더라”: I report what I saw or heard myself. Both are spoken endings.
- -대: 친구가 내일 비가 온대. / My friend says it will rain tomorrow.
- -데: 어제 가 보니까 그 집 음식이 맛있데. / When I went there yesterday, their food was good.
- common mistake: Using “-데” for something you were told. Hearsay is “-대”; what you witnessed is “-데”. Neither is for an essay.
- review: draft

**C044** [meaning] `계기 | 기회`

- explanation: “계기” is the trigger or turning point that sets something in motion. “기회” is a chance or an opportunity that you can take.
- 계기: 그 책이 그가 작가가 되는 계기가 되었다. / That book became the trigger for his becoming a writer.
- 기회: 이번 방학은 외국어를 배울 좋은 기회이다. / These holidays are a good chance to learn a foreign language.
- common mistake: Writing “계기를 얻다” for “get a chance”. A chance is “기회를 얻다”; a “계기” is what makes something happen.
- review: draft

**C045** [meaning] `경험 | 체험`

- explanation: “경험” is the experience you accumulate through doing or living, including past knowledge. “체험” is a particular direct experience, mostly a hands-on event.
- 경험: 그는 오랜 해외 근무 경험이 있다. / He has long experience of working overseas.
- 체험: 학생들은 농촌에서 일주일 동안 생활을 체험했다. / The students experienced life in the countryside for a week.
- common mistake: Writing “체험이 풍부하다” for rich experience at work. A long record is “경험”; a one-off activity is “체험”.
- review: draft

**C046** [meaning] `비판 | 비난`

- explanation: “비판” is to analyse and judge something critically, giving reasons. “비난” is to blame or condemn someone, often with no intention to improve.
- 비판: 학자들은 정부의 정책을 비판했다. / Scholars criticised the government's policy.
- 비난: 그는 실수 때문에 사람들의 비난을 받았다. / He was blamed by people because of the mistake.
- common mistake: Writing “건설적인 비난”. A reasoned critique is “비판”.
- review: draft

**C047** [meaning] `효과 | 효율 | 효능`

- explanation: “효과” is the result or effect something has. “효율” is efficiency: output compared with input. “효능” is the efficacy of a substance such as a medicine or herb.
- 효과: 이 약은 두통에 효과가 있다. / This medicine is effective for headaches.
- 효율: 새로운 방식으로 업무 효율을 높였다. / The new method raised work efficiency.
- 효능: 인삼의 효능이 과학적으로 확인되었다. / The efficacy of ginseng was confirmed scientifically.
- common mistake: Writing “효율이 있다” for “be effective”. “Effective” is “효과가 있다”; “efficient” is “효율적이다”.
- review: draft

**C048** [meaning] `보전 | 보존 | 보관`

- explanation: “보전” is to keep something safe and intact so that it is not damaged: “환경 보전”, “재산 보전”. “보존” is to keep something as it is, in its original state: “문화재 보존”, “음식 보존”. “보관” is to keep something in a safe place: “귀중품 보관”.
- 보전: 우리는 자연환경을 보전해야 한다. / We must keep the natural environment safe.
- 보존: 문화재를 원래의 모습 그대로 보존한다. / Cultural properties are preserved in their original form.
- 보관: 귀중품은 금고에 보관해 주십시오. / Please keep valuables in the safe.
- common mistake: Writing “환경 보존” for protecting the environment against damage. “보전” protects, “보존” keeps unchanged.
- review: draft

**C049** [meaning] `너무 | 매우`

- explanation: “너무” means “too much, excessively” and traditionally carries a negative sense. “매우” means “very” and is neutral. In an essay use “매우 / 아주 / 상당히” for a high degree.
- 너무: 이 가방은 너무 무거워서 들 수 없다. / This bag is too heavy to lift.
- 매우: 이 가방은 매우 가볍다. / This bag is very light.
- common mistake: Writing “너무 좋다” in a formal essay. The standard meaning of “너무” is excess, so use “매우” instead.
- review: draft

**C050** [meaning] `다시 | 또`

- explanation: “다시” is “again”, doing something once more from the start or resuming after stopping. “또” is “again, once more” for repetition, and also “and, in addition”.
- 다시: 시험에 떨어졌지만 다시 도전했다. / I failed the exam but took up the challenge again.
- 또: 어제도 늦었는데 오늘 또 늦었다. / I was late yesterday and I was late again today.
- common mistake: Writing “또 시작했다” for “started over after a stop”; “다시” fits better.
- review: draft

**C051** [meaning] `이미 | 벌써`

- explanation: “이미” is “already”, stating a completed fact. “벌써” is “already” with a feeling of surprise that something came earlier than expected.
- 이미: 내가 도착했을 때 회의는 이미 끝나 있었다. / When I arrived, the meeting had already ended.
- 벌써: 벌써 열두 시라니 믿기지 않는다. / I can't believe it is already twelve o'clock.
- common mistake: Using “벌써” in a neutral report. For a plain fact in an essay, “이미” is the safe choice.
- review: draft

**C052** [meaning] `낫다 | 낳다 | 낮다`

- explanation: “낫다” is to recover or to be better than: “병이 낫다”, “더 낫다”. “낳다” is to give birth or to produce a result. “낮다” is low.
- 낫다: 감기가 다 나았다. / My cold has completely gone.
- 낳다: 그녀는 올해 첫째 아이를 낳았다. / She gave birth to her first child this year.
- 낮다: 이 지역은 평균 소득이 낮다. / Average income in this area is low.
- common mistake: Writing “감기가 낳았다” or “병이 낮다”. Remember: “낳” (with “ㅎ”) is to give birth; “낫” (with “ㅅ”) is to recover.
- review: draft

**C053** [meaning] `부수다 | 부시다`

- explanation: “부수다” is to smash into pieces: “문을 부수다”. “부시다” is dazzling to the eyes: “눈이 부시다”.
- 부수다: 아이가 장난감을 부수고 말았다. / The child ended up smashing the toy.
- 부시다: 눈이 부시게 맑은 하늘이었다. / The sky was so clear it dazzled the eyes.
- common mistake: Writing “눈이 부수다” or “문을 부시다”.
- review: draft

**C054** [meaning] `썩이다 | 썩히다`

- explanation: “썩이다” is used almost only in “속을 썩이다”: to cause someone worry. “썩히다” is to let something rot or go to waste: “재능을 썩히다”.
- 썩이다: 아들이 부모의 속을 썩이고 있다. / The son is causing his parents worry.
- 썩히다: 좋은 재능을 썩히지 말아야 한다. / You should not let a good talent go to waste.
- common mistake: Writing “속을 썩히다”. With “속” (heart, feelings), it is “썩이다”.
- review: draft

**C055** [meaning] `거치다 | 걸치다`

- explanation: “거치다” is to pass through or go by way of, and to go through a process: “서울을 거쳐”, “심사를 거쳐”. “걸치다” is to extend across a period or area, or to hang something over: “3년에 걸쳐”, “어깨에 걸치다”.
- 거치다: 우리는 서울을 거쳐 부산으로 갔다. / We went to Busan by way of Seoul.
- 걸치다: 이 사업은 3년에 걸쳐 진행되었다. / This project was carried out over three years.
- common mistake: Writing “3년을 거쳐” (should be “3년에 걸쳐”) or “심사를 걸쳐” (should be “심사를 거쳐”).
- review: draft

**C056** [meaning] `다리다 | 달이다`

- explanation: “다리다” is to iron clothes. “달이다” is to boil down herbs or medicine until the essence comes out.
- 다리다: 셔츠를 다려서 입고 나갔다. / I ironed my shirt, put it on and went out.
- 달이다: 어머니는 약초를 달여 주셨다. / My mother brewed herbs for me.
- common mistake: Writing “약을 다리다” or “옷을 달이다”.
- review: draft

**C057** [meaning] `매다 | 메다`

- explanation: “매다” is to tie: “끈을 매다”, “넥타이를 매다”. “메다” is to carry on the shoulder: “가방을 메다”.
- 매다: 그는 신발 끈을 단단히 맸다. / He tied his shoelaces tightly.
- 메다: 그는 어깨에 배낭을 메고 걸었다. / He walked with a backpack on his shoulder.
- common mistake: Writing “가방을 매다” or “넥타이를 메다”.
- review: draft

**C058** [meaning] `안치다 | 앉히다`

- explanation: “안치다” is to put food such as rice in a pot to cook. “앉히다” is to make someone sit down.
- 안치다: 쌀을 씻어 솥에 안쳤다. / I washed the rice and put it in the pot.
- 앉히다: 엄마는 아이를 의자에 앉혔다. / The mother sat the child on the chair.
- common mistake: Writing “밥을 앉히다”. Cooking rice is “밥을 안치다”.
- review: draft

**C059** [meaning] `조리다 | 졸이다`

- explanation: “조리다” is to simmer meat, fish or vegetables in seasoning until the liquid is almost gone: “생선을 조리다”. “졸이다” is to boil down a liquid, or “(마음을)” to feel anxious: “국물을 졸이다”, “마음을 졸이다”.
- 조리다: 생선을 간장에 조려서 반찬으로 먹었다. / I simmered the fish in soy sauce and ate it as a side dish.
- 졸이다: 합격자 발표를 기다리며 마음을 졸였다. / I waited anxiously for the results to be announced.
- common mistake: Writing “마음을 조리다”. Anxiety is always “마음을 졸이다”.
- review: draft

**C060** [meaning] `저리다 | 절이다`

- explanation: “저리다” is to feel a numb, tingling or aching sensation (“다리가 저리다”). “절이다” is to pickle or salt vegetables or fish.
- 저리다: 오래 앉아 있었더니 다리가 저린다. / My legs are tingling from sitting so long.
- 절이다: 배추를 소금물에 절였다. / I salted the cabbage in brine.
- common mistake: Writing “배추를 저리다”. Pickling is “절이다”.
- review: draft

**C061** [meaning] `벌리다 | 벌이다`

- explanation: “벌리다” is to open wide or to create a gap: “입을 벌리다”, “간격을 벌리다”. “벌이다” is to start or carry out something, such as a business, a party or a fight: “사업을 벌이다”, “잔치를 벌이다”.
- 벌리다: 아이가 입을 크게 벌렸다. / The child opened his mouth wide.
- 벌이다: 그는 새로운 사업을 벌였다. / He started a new business.
- common mistake: Writing “잔치를 벌리다”. For starting an event, a business or a fight it is “벌이다”.
- review: draft

**C062** [meaning] `드러내다 | 드러나다`

- explanation: “드러내다” is a transitive verb: to bring something into view on purpose. “드러나다” is intransitive: to become visible by itself.
- 드러내다: 그는 자신의 감정을 드러내지 않았다. / He did not show his feelings.
- 드러나다: 진실이 마침내 드러났다. / The truth was finally revealed.
- common mistake: Writing “감정이 드러내다” or “진실을 드러났다”. The one who acts takes “을/를 드러내다”; the thing that appears takes “이/가 드러나다”.
- review: draft

**C063** [meaning] `비추다 | 비치다`

- explanation: “비추다” is transitive: to shine light on something, or to reflect. “비치다” is intransitive: light shines, or something shows through.
- 비추다: 달빛이 조용히 길을 비추었다. / The moonlight quietly lit the road.
- 비치다: 창문으로 햇빛이 비친다. / Sunlight shines through the window.
- common mistake: Writing “햇빛이 방을 비치다”. The sun that lights something “비추다”; the light that comes in “비치다”.
- review: draft

**C064** [meaning] `나타내다 | 나타나다`

- explanation: “나타내다” is transitive: to show or express something. “나타나다” is intransitive: to appear or become evident. In a 53 answer both are used for “shows”: the graph “나타내다”, the result “나타나다”.
- 나타내다: 이 그래프는 연도별 변화를 나타낸다. / This graph shows the changes by year.
- 나타나다: 조사 결과 이러한 경향이 나타났다. / The survey results showed this tendency.
- common mistake: Writing “그래프가 나타났다” or “결과를 나타났다”. Active subject + object → “나타낸다”; result as subject → “나타났다”.
- review: draft

**C065** [meaning] `지나다 | 지내다`

- explanation: “지나다” is to pass: time or places go past. “지내다” is to spend time or to live in some way.
- 지나다: 벌써 한 달이 지났다. / A month has already passed.
- 지내다: 요즘 어떻게 지내십니까? / How are you doing these days?
- common mistake: Writing “잘 지나세요?” for “how are you?”. Life is “지내다”.
- review: draft

**C066** [meaning] `잊다 | 잃다`

- explanation: “잊다” is to forget: it is gone from the memory. “잃다” is to lose something: it is gone from your possession.
- 잊다: 약속 시간을 잊고 말았다. / I forgot the appointed time.
- 잃다: 그는 사고로 지갑을 잃었다. / He lost his wallet in an accident.
- common mistake: Writing “지갑을 잊어버렸다” for losing a thing in the street; it is “잃어버렸다”. Only memory is “잊다”.
- review: draft

**C067** [meaning] `안 | 못`

- explanation: “안” is a plain negative of choice or fact: not doing. “못” says one cannot do it because of ability or circumstances.
- 안: 나는 매운 음식을 안 먹는다. / I don't eat spicy food.
- 못: 나는 매운 음식을 못 먹는다. / I can't eat spicy food.
- common mistake: Writing “못 예쁘다”. “못” goes with action verbs; adjectives are negated with “안”.
- review: draft

**C068** [meaning] `작다 | 적다`

- explanation: “작다” is small in size or scale: a room, a voice, a plan. “적다” is few or little in number or amount: people, money, rain. Their opposites are “크다” and “많다”.
- 작다: 이 방은 크기가 작다. / This room is small in size.
- 적다: 시험에 참석한 사람의 수가 적다. / The number of people who attended the exam is small.
- common mistake: Writing “사람이 작다” for few people or “크기가 적다” for small size.
- review: draft

**C069** [meaning] `구별 | 구분`

- explanation: “구별” is to tell two things apart by their differences. “구분” is to divide something into categories.
- 구별: 쌍둥이를 구별하기가 어렵다. / It is hard to tell the twins apart.
- 구분: 책을 주제별로 구분하여 정리했다. / I sorted the books into groups by subject.
- common mistake: Using the two as one; “구별” is about spotting a difference, “구분” about dividing into groups.
- review: draft

**C070** [meaning] `요구 | 요청`

- explanation: “요구” is a demand, strong and asserting a right. “요청” is a request, polite and asking for help or action.
- 요구: 노동자들은 임금 인상을 요구했다. / The workers demanded a wage increase.
- 요청: 도움을 요청하는 편지를 보냈다. / I sent a letter asking for help.
- common mistake: Using “요구” for a polite request. Use “요청” (or “부탁”) when you ask, “요구” when you insist.
- review: draft

**C071** [meaning] `원인 | 이유`

- explanation: “원인” is the cause that produces a result or event. “이유” is the reason that explains an action or an opinion.
- 원인: 경찰이 사고의 원인을 조사하고 있다. / The police are investigating the cause of the accident.
- 이유: 그가 떠난 이유를 알 수 없다. / I cannot tell why he left.
- common mistake: Writing “사고의 이유” for the cause of an accident and “결정의 원인” for the reason of a decision.
- review: draft

**C072** [meaning] `해결 | 해소`

- explanation: “해결” is to solve a problem or conflict. “해소” is to dissolve or relieve something unpleasant such as stress, dissatisfaction or tension.
- 해결: 우리는 문제를 해결할 방법을 찾았다. / We found a way to solve the problem.
- 해소: 운동은 스트레스를 해소하는 데 도움이 된다. / Exercise helps to relieve stress.
- common mistake: Writing “스트레스를 해결하다”. The condition goes away with “해소”; the problem is closed with “해결”.
- review: draft

**C073** [meaning] `개선 | 개정 | 개혁`

- explanation: “개선” is to make something better, step by step: “환경 개선”. “개정” is to revise an official text such as a law or a book. “개혁” is a fundamental, large-scale reform of a system.
- 개선: 근로 환경을 개선해야 한다. / Working conditions must be improved.
- 개정: 국회는 관련 법률을 개정하였다. / The National Assembly revised the related law.
- 개혁: 정부는 교육 제도를 개혁하겠다고 밝혔다. / The government said it would reform the education system.
- common mistake: Using “개정” for any change. “개정” is only for a formal text such as a law; a gradual improvement is “개선” and a whole system is “개혁”.
- review: draft

**C074** [meaning] `강조 | 강요`

- explanation: “강조” is to emphasise a point so that it stands out. “강요” is to force someone to do something.
- 강조: 선생님은 독서의 중요성을 강조했다. / The teacher stressed the importance of reading.
- 강요: 부모가 아이에게 공부를 강요해서는 안 된다. / Parents must not force their children to study.
- common mistake: Mixing the two because both have “강”. They share a sound only.
- review: draft

**C075** [meaning] `영향 | 효과`

- explanation: “영향” is influence: a change one thing causes in another, good or bad. “효과” is the intended, usually good effect of an action.
- 영향: 부모의 태도는 아이에게 큰 영향을 준다. / Parents' attitudes have a great influence on a child.
- 효과: 운동은 다이어트에 효과가 있다. / Exercise is effective for dieting.
- common mistake: Writing “효과를 받다”. To be influenced is “영향을 받다”; “효과” is something you have (“효과가 있다”).
- review: draft

**C076** [meaning] `대한 | 관한 | 위한`

- explanation: “-에 대한” points at the target of a feeling or action (“관심”, “태도”, “비판”). “-에 관한” is about a subject or topic (“책”, “연구”, “자료”). “-을/를 위한” is for the benefit or purpose of someone.
- 대한: 환경 문제에 대한 관심이 높아졌다. / Interest in environmental problems has grown.
- 관한: 그는 한국 역사에 관한 책을 읽었다. / He read a book about Korean history.
- 위한: 건강을 위한 운동을 시작했다. / I started exercising for the sake of my health.
- common mistake: Writing “환경에 관한 관심” (target of interest → “대한”) or “아이를 위한 연구” for a study about children (topic → “에 관한”).
- review: draft

**C077** [meaning] `때문에 | 덕분에 | 탓에`

- explanation: All three give a cause. “때문에” is neutral. “덕분에” gives the cause of a good result (thanks to). “탓에” gives the cause of a bad result (because of, blame).
- 때문에: 폭설 때문에 길이 막혔다. / The road was blocked because of the heavy snow.
- 덕분에: 친구 덕분에 시험에 합격했다. / Thanks to my friend, I passed the exam.
- 탓에: 내 부주의 탓에 일이 잘못되었다. / The matter went wrong because of my carelessness.
- common mistake: Writing “덕분에” for a bad result (“사고가 나서 덕분에 다쳤다”) or “탓에” for a good one.
- review: draft

**C078** [meaning] `-기 위해 | -기 때문에`

- explanation: “-기 위해(서)” expresses purpose: the aim of an action (to do …). “-기 때문에” expresses cause: why something is so. They cannot be exchanged.
- -기 위해: 건강을 지키기 위해 매일 운동한다. / I exercise every day in order to stay healthy.
- -기 때문에: 건강이 나빠졌기 때문에 매일 운동한다. / I exercise every day because my health has got worse.
- common mistake: Writing “-기 위해” after an adjective or a past tense. Purpose clauses go with verbs and cannot take “-았/었”.
- review: draft

**C079** [meaning] `그러나 | 그런데`

- explanation: “그러나” (like “하지만”) marks a clear contrast with the previous sentence. “그런데” changes the subject or adds a new twist, with only a mild contrast, and is more common in speech.
- 그러나: 이 방법은 효과가 있다. 그러나 비용이 많이 든다. / This method is effective. However, it costs a lot.
- 그런데: 이 방법은 효과가 있다. 그런데 이 방법을 어떻게 적용할 수 있을까? / This method is effective. By the way, how could it be applied?
- common mistake: Using “그런데” where a clear opposition is meant. In 54 a firm contrast should be “그러나 / 하지만 / 반면에”.
- review: draft

**C080** [meaning] `비교 | 대조 | 대비`

- explanation: “비교” is to compare things to see similarities and differences. “대조” is to set side by side to check or to stress a difference: “원본과 대조하다”, “대조적이다”. “대비” is “compared to” in numbers (“전년 대비”), or preparing for something (a separate word: “대비하다”).
- 비교: 두 제품의 가격을 비교해 보았다. / I compared the prices of the two products.
- 대조: 사본을 원본과 대조해 오류를 찾았다. / I checked the copy against the original and found the errors.
- 대비: 올해 매출은 전년 대비 10% 늘었다. / This year's sales rose 10% compared with last year.
- common mistake: Writing “전년 비교 10% 증가” in a graph description. The set phrase is “전년 대비 10% 증가”.
- review: draft

**C081** [meaning] `변화 | 변동 | 변경`

- explanation: “변화” is a change in the nature or state of something. “변동” is a fluctuation, especially in numbers such as prices or rates. “변경” is an alteration someone makes to a plan or a setting.
- 변화: 사회 구조에 큰 변화가 생겼다. / A great change has occurred in the social structure.
- 변동: 환율의 변동이 심해지고 있다. / Fluctuations in the exchange rate are growing larger.
- 변경: 회의 일정이 변경되었다. / The schedule of the meeting has been changed.
- common mistake: Writing “일정이 변화되었다” for a changed appointment; planned changes are “변경”.
- review: draft

**C082** [meaning] `사용 | 이용 | 활용`

- explanation: “사용” is to use a thing, money or a word. “이용” is to use a facility or service, or to take advantage of someone. “활용” is to make good use of resources, knowledge or skills.
- 사용: 이 단어는 일상에서 자주 사용된다. / This word is often used in daily life.
- 이용: 많은 시민이 대중교통을 이용한다. / Many citizens use public transport.
- 활용: 학습한 내용을 실생활에 활용해 보세요. / Try to put what you have learnt to use in real life.
- common mistake: Writing “대중교통을 사용하다”; services and places are “이용하다”.
- review: draft

**C083** [meaning] `발견 | 발명`

- explanation: “발견” is to find something that already existed but was unknown. “발명” is to invent something that did not exist before.
- 발견: 그는 새로운 별을 발견했다. / He discovered a new star.
- 발명: 에디슨은 전구를 발명했다. / Edison invented the light bulb.
- common mistake: Writing “불을 발명했다” for the discovery of fire (it was a discovery). The thing already existed in nature → “발견”.
- review: draft

**C084** [meaning] `제공 | 공급`

- explanation: “제공” is to give or offer something, often information, service or a meal, without asking for a price. “공급” is to supply goods or services in quantity, as in supply and demand.
- 제공: 학교는 학생들에게 점심을 제공한다. / The school provides lunch for the students.
- 공급: 전력 공급이 갑자기 중단되었다. / The power supply was suddenly cut off.
- common mistake: Writing “수요와 제공” for supply and demand. The economic term is “수요와 공급”.
- review: draft

**C085** [meaning] `임대 | 임차`

- explanation: “임대” is to lend a property out and receive rent (the owner's side). “임차” is to borrow a property and pay rent (the tenant's side).
- 임대: 그는 건물을 임대하여 수입을 얻고 있다. / He earns income by renting out a building.
- 임차: 우리 회사는 사무실을 임차해 사용하고 있다. / Our company rents and uses an office.
- common mistake: Using “임대” for the tenant's side. “임대” = the owner, “임차” = the tenant.
- review: draft

**C086** [meaning] `권리 | 권한`

- explanation: “권리” is a right that a person is entitled to. “권한” is authority that comes with a position or a role.
- 권리: 모든 국민은 교육을 받을 권리가 있다. / Every citizen has the right to receive education.
- 권한: 이 문제는 담당자의 권한 밖의 일이다. / This matter is outside the person in charge's authority.
- common mistake: Writing “교육을 받을 권한” for a basic right.
- review: draft

**C087** [meaning] `절약 | 저축`

- explanation: “절약” is to use less so as not to waste: “전기를 절약하다”. “저축” is to put money aside for the future: “은행에 저축하다”.
- 절약: 에너지를 절약하는 습관이 필요하다. / We need the habit of saving energy.
- 저축: 그는 매달 월급의 일부를 저축한다. / He saves part of his salary every month.
- common mistake: Writing “전기를 저축하다” for saving electricity.
- review: draft

**C088** [meaning] `소비 | 소모`

- explanation: “소비” is to use up goods, money or energy as a consumer. “소모” is to wear out or use up time, energy or materials.
- 소비: 가계의 소비가 줄고 있다. / Household consumption is falling.
- 소모: 이 일은 시간과 체력을 많이 소모한다. / This work wastes a lot of time and strength.
- common mistake: Writing “소모자” for a consumer. The consumer is “소비자”.
- review: draft

**C089** [meaning] `어리다 | 젊다`

- explanation: “어리다” is for a child or someone not yet grown up (young in age). “젊다” is young as an adult, in the prime of life, not old.
- 어리다: 동생은 아직 어려서 혼자 갈 수 없다. / My younger sibling is still young and cannot go alone.
- 젊다: 그는 나이는 많지만 생각이 젊다. / He is old in years but young in thinking.
- common mistake: Using “어리다” for a twenty-year-old adult. For adults it is “젊다”.
- review: draft

**C090** [meaning] `주의 | 조심`

- explanation: “주의” is attention and caution; it often stands in notices “(주의하십시오)”. “조심” is being careful in one's behaviour, with a more everyday tone.
- 주의: 바닥이 미끄러우니 주의하십시오. / The floor is slippery, so be careful.
- 조심: 길이 미끄러우니 조심해서 걸어라. / The road is slippery, so walk carefully.
- common mistake: Using “조심하십시오” in a formal notice; “주의하십시오” is the standard sign-board wording.
- review: draft

**C091** [meaning] `익숙하다 | 능숙하다`

- explanation: “익숙하다” is to be familiar with or used to something. “능숙하다” is to be skilled and good at something.
- 익숙하다: 나는 이곳의 생활에 익숙하다. / I am used to life here.
- 능숙하다: 그는 한국어를 능숙하게 말한다. / He speaks Korean fluently.
- common mistake: Saying “한국어에 익숙하다” to mean good at Korean. “익숙하다” only says it is not strange to you.
- review: draft

**C092** [meaning] `정확하다 | 확실하다`

- explanation: “정확하다” is accurate: with no error. “확실하다” is certain: sure, with no doubt.
- 정확하다: 이 계산은 정확하다. / This calculation is accurate.
- 확실하다: 그가 올 것이라는 것은 확실하다. / It is certain that he will come.
- common mistake: Using “정확하다” for “sure”. “확실하다” means beyond doubt.
- review: draft

**C093** [meaning] `적절하다 | 적당하다`

- explanation: “적절하다” fits the situation or purpose exactly (“적절한 표현”). “적당하다” is suitable or moderate; it can also mean “just enough”, or “half-hearted” (“적당히 하다”).
- 적절하다: 이 상황에는 그 표현이 적절하다. / That expression is appropriate for this situation.
- 적당하다: 소금을 적당히 넣어야 맛이 좋다. / You must add a suitable amount of salt for it to taste good.
- common mistake: Using “적당히” in a formal essay for “appropriately”; it can sound like “carelessly”. Use “적절히”.
- review: draft

**C094** [grammar] `-(으)로서 | -(으)로써`

- explanation: “-(으)로서” marks a status, role or qualification (as a student, as a citizen). “-(으)로써” marks a tool, material, method or a point in time (by means of, with).
- -(으)로서: 학생으로서 지켜야 할 규칙이 있다. / There are rules to observe as a student.
- -(으)로써: 대화로써 갈등을 해결할 수 있다. / Conflicts can be solved by means of dialogue.
- common mistake: Writing “학생으로써 책임을 다하다”. If you can say “as a …”, it is “-로서”; if you can say “by using …”, it is “-로써”.
- review: draft

**C095** [grammar] `-든지 | -던지`

- explanation: “-든지” (also “-든”) shows a choice, “whichever, whether … or …”. “-던지” is for a past recollection or uncertain reason: “-던지” in “얼마나 … 던지”.
- -든지: 가든지 말든지 마음대로 해라. / Go or don't go, do as you like.
- -던지: 어제는 얼마나 추웠던지 손이 곱을 정도였다. / It was so cold yesterday that my hands went numb.
- common mistake: Writing “무엇을 먹던지 좋다” for “whatever you eat is fine”. Choice is “먹든지”; “-던지” recalls the past.
- review: draft

**C096** [grammar] `-(으)러 | -(으)려고`

- explanation: “-(으)러” is used only before “가다 / 오다 / 다니다” and says the purpose of the movement. “-(으)려고” says an intention and works with any verb, and can also be used with a past tense in the main clause.
- -(으)러: 친구를 만나러 카페에 갔다. / I went to the café to meet a friend.
- -(으)려고: 시험에 합격하려고 열심히 공부했다. / I studied hard in order to pass the exam.
- common mistake: Writing “시험에 합격하러 공부했다”. “-러” needs a movement verb after it.
- review: draft

**C097** [grammar] `-(으)ㅁ | -기`

- explanation: Both turn a verb into a noun. “-(으)ㅁ” names a result or a state and is common in formal writing, notices and titles. “-기” names an action or an activity (“읽기”, “쓰기”) and goes with “위해”, “때문에” and expectations.
- -(으)ㅁ: 그는 이 방법이 효과적임을 보여 주었다. / He showed that this method is effective.
- -기: 매일 달리기는 건강에 도움이 된다. / Running every day is good for health.
- common mistake: Using “-음” for an activity as a subject (“달림은 건강에 좋다”); a named activity is “-기” “(달리기)”.
- review: draft

**C098** [grammar] `-(으)ㄴ데 | -(으)ㄴ 데`

- explanation: “-(으)ㄴ/는데” (attached) is a connective: background or contrast. “-(으)ㄴ/는 데” (spaced; “데” is a noun meaning “place, point, respect”) is a noun phrase: “in doing …”, “where …”.
- -(으)ㄴ데: 비가 오는데 우산이 없다. / It is raining and I have no umbrella.
- -(으)ㄴ 데: 이 일을 끝내는 데 두 시간이 걸렸다. / It took two hours to finish this job.
- common mistake: Dropping the space in “끝내는 데 두 시간이 걸렸다” (here “데” means “in doing”). If “데” could be replaced by “것”, it is the noun and needs the space.
- review: draft

**C099** [grammar] `에게 | 한테 | 께`

- explanation: “에게” is the standard written particle for an animate indirect object. “한테” is the spoken form. “께” is the honorific form for someone to be respected.
- 에게: 선생님은 학생들에게 숙제를 내 주었다. / The teacher gave the students homework.
- 한테: 어제 친구한테 선물을 받았다. / I received a present from a friend yesterday.
- 께: 할아버지께 편지를 드렸다. / I wrote a letter to my grandfather.
- common mistake: Using “한테” in an essay; in 53/54 write “에게”. Never use “에게” or “한테” for the one you respect.
- review: draft

**C100** [grammar] `-아서 | -(으)니까 | -기 때문에`

- explanation: All three give a reason. “-아서/어서” is neutral and cannot be followed by a command or suggestion. “-(으)니까” is a more personal reason and can be followed by a command or suggestion. “-기 때문에” is the formal written reason for essays.
- -아서: 비가 와서 행사가 취소되었다. / The event was cancelled because it rained.
- -(으)니까: 비가 오니까 우산을 가져가세요. / Take an umbrella, since it is raining.
- -기 때문에: 비가 오기 때문에 행사가 취소되었다. / The event was cancelled because of the rain.
- common mistake: Using “-니까” in an essay; essay reasons are “-기 때문에 / -(으)므로”. “-아서” cannot be followed by a request (“비가 와서 우산을 가져가세요” is wrong).
- review: draft
