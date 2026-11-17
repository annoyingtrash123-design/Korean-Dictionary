# 속담 / 성어 corpus frequency

| corpus | source | eojeol (tokens) | Hangul syllables | segments | read | seconds |
|---|---|---|---|---|---|---|
| kowiki | https://dumps.wikimedia.org/kowiki/latest/kowiki-latest-pages-articles.xml.bz2 | 174,704,046 | 522,885,038 | 770,149 | 1423 MB | 291 |
| kowikisource | https://dumps.wikimedia.org/kowikisource/latest/kowikisource-latest-pages-articles.xml.bz2 | 48,344,284 | 136,389,852 | 22,846 | 150 MB | 43 |
| opensubtitles | https://object.pouta.csc.fi/OPUS-OpenSubtitles/v2018/mono/ko.txt.gz | 5,205,855 | 18,856,595 | 2,170,989 | 27 MB | 7 |

Score = mean over corpora of (full + partial) occurrences per million eojeol. `full` = the fixed stem up to the last predicate with any ending; `partial` = the quoted first half (e.g. 가는 말이 고와야) NOT followed by the rest. Proverbs with a fixed stem under 4 syllables (krdict) or 6 syllables (stdict/opendict only) are too generic and are not ranked (553 such).

## 속담 (proverbs)

9646 candidates, 722 with at least one hit, ranked 9093.

### Rank-frequency curve, all dictionaries (total raw occurrences across corpora)

| ranks | n | min total | median total | max total |
|---|---|---|---|---|
| 1-50 | 50 | 3 | 12.5 | 151 |
| 51-100 | 50 | 2 | 6 | 43 |
| 101-200 | 100 | 1 | 4 | 16 |
| 201-300 | 100 | 2 | 2 | 9 |
| 301-500 | 200 | 1 | 1 | 6 |
| 501-1000 | 500 | 0 | 0 | 3 |
| 1001+ | 8093 | 0 | 0 | 0 |

| rank | total occurrences | score (per million eojeol) |
|---|---|---|
| 100 | 6 | 0.074 |
| 300 | 4 | 0.013 |
| 500 | 1 | 0.007 |
| 1000 | 0 | 0.000 |

### Top 50, all dictionaries

| # | text | src | kowiki | kowikisource | opensubtitles | total |
|---|---|---|---|---|---|---|
| 1 | 누워서 떡 먹기 | krdict,stdict | 5+0 | 0+0 | 16+0 | 21 |
| 2 | 호랑이도 제 말 하면 온다 | krdict,stdict | 3+0 | 2+0 | 10+0 | 15 |
| 3 | 밑져야 본전 | krdict,stdict | 6+0 | 7+0 | 6+0 | 19 |
| 4 | 눈에는 눈을 이에는 이를 | krdict,stdict | 53+0 | 0+0 | 5+0 | 58 |
| 5 | 누이 좋고 매부 좋다 | krdict,stdict | 0+0 | 2+0 | 6+0 | 8 |
| 6 | 주먹이 운다 | krdict,stdict | 148+0 | 2+0 | 1+0 | 151 |
| 7 | 갈수록 태산 | krdict,stdict | 2+0 | 2+0 | 5+0 | 9 |
| 8 | 등잔 밑이 어둡다 | krdict,stdict | 1+0 | 2+0 | 5+0 | 8 |
| 9 | 시작이 반이다 | krdict,stdict | 4+0 | 9+0 | 4+0 | 17 |
| 10 | 따 놓은 당상 | krdict,stdict | 1+0 | 0+0 | 5+0 | 6 |
| 11 | 보자 보자 하니까 얻어 온 장 한 번 더 뜬다 | stdict | 0+0 | 0+0 | 0+5 | 5 |
| 12 | 뒤를 돌아보고 울기보다는 앞을 바라보고 웃으랬다 | opendict | 0+16 | 0+19 | 0+2 | 37 |
| 13 | 주사위는 던져졌다 | krdict,stdict | 10+0 | 1+0 | 4+0 | 15 |
| 14 | 할 일이 없거든 오금이나 긁어라 | stdict | 0+0 | 0+2 | 0+4 | 6 |
| 15 | 우물 안 개구리 | krdict,stdict | 24+0 | 3+0 | 3+0 | 30 |
| 16 | 불난 집에 부채질한다 | krdict,stdict | 1+0 | 0+0 | 4+0 | 5 |
| 17 | 무소식이 희소식 | krdict,stdict | 0+0 | 0+0 | 4+0 | 4 |
| 18 | 꿀 먹은 벙어리 | krdict,stdict | 2+0 | 8+0 | 3+0 | 13 |
| 19 | 사람이 많으면 길이 열린다 | stdict | 0+8 | 0+6 | 0+3 | 17 |
| 20 | 수박 겉 핥기 | krdict,stdict | 2+0 | 7+0 | 3+0 | 12 |
| 21 | 백문이 불여일견不如一見 | stdict,opendict | 5+0 | 5+0 | 3+0 | 13 |
| 22 | 상갓집 개 | krdict,stdict | 0+0 | 34+0 | 0+0 | 34 |
| 23 | 아닌 밤중에 홍두깨 내밀듯 | krdict,stdict | 1+0 | 5+0 | 3+0 | 9 |
| 24 | 지성이면 감천 | krdict,stdict | 71+0 | 13+0 | 0+0 | 84 |
| 25 | 은혜를 원수로 갚는다 | stdict | 8+0 | 2+0 | 3+0 | 13 |
| 26 | 손이 발이 되도록 빌다 | stdict | 1+1 | 20+11 | 0+0 | 33 |
| 27 | 귀신이 곡할 노릇이다 | stdict | 2+0 | 3+0 | 3+0 | 8 |
| 28 | 새 발의 피 | krdict,stdict | 3+0 | 1+0 | 3+0 | 7 |
| 29 | 김칫국부터 마신다 | krdict,stdict | 2+0 | 1+0 | 3+0 | 6 |
| 30 | 말이 씨가 된다 | krdict,stdict | 2+0 | 0+0 | 3+0 | 5 |
| 31 | 꿩 먹고 알 먹는다 | krdict,stdict | 4+0 | 8+0 | 2+0 | 14 |
| 32 | 무자식 상팔자 | krdict,stdict | 58+0 | 1+0 | 1+0 | 60 |
| 33 | 열 길 물속은 알아도 한 길 사람의 속은 모른다 | krdict,stdict | 0+2 | 4+3 | 0+2 | 11 |
| 34 | 겉과 속이 다르다 | krdict,stdict | 22+0 | 1+0 | 2+0 | 25 |
| 35 | 그 아버지에 그 아들 | krdict,stdict | 12+0 | 2+0 | 2+0 | 16 |
| 36 | 산 넘어 산이다 | krdict,stdict | 15+0 | 1+0 | 2+0 | 18 |
| 37 | 하늘이 무너져도 솟아날 구멍이 있다 | krdict,stdict | 0+0 | 5+0 | 2+0 | 7 |
| 38 | 더도 말고 덜도 말고 늘 한가위만 같아라 | krdict | 0+14 | 0+1 | 0+2 | 17 |
| 39 | 더도 말고 덜도 말고 늘 가윗날만 같아라 | stdict | 0+14 | 0+1 | 0+2 | 17 |
| 40 | 속 빈 강정 | krdict,stdict | 3+0 | 3+0 | 2+0 | 8 |
| 41 | 옷이 날개라 | krdict,stdict | 8+0 | 1+0 | 2+0 | 11 |
| 42 | 밑 빠진 독에 물 붓기 | krdict,stdict | 1+0 | 2+0 | 2+0 | 5 |
| 43 | 무른 메주 밟듯 | opendict | 0+0 | 2+0 | 2+0 | 4 |
| 44 | 사공이 많으면 배가 산으로 간다 | stdict | 1+0 | 1+0 | 2+0 | 4 |
| 45 | 동에 번쩍 서에 번쩍 | krdict,stdict | 5+0 | 9+0 | 1+0 | 15 |
| 46 | 울며 겨자 먹기 | krdict,stdict | 31+0 | 11+0 | 0+0 | 42 |
| 47 | 입이 열 개라도 할 말이 없다 | krdict,stdict | 0+0 | 1+0 | 1+1 | 3 |
| 48 | 한 귀로 듣고 한 귀로 흘린다 | krdict,stdict | 0+0 | 1+0 | 2+0 | 3 |
| 49 | 계란으로 바위 치기 | krdict,stdict | 3+0 | 0+0 | 2+0 | 5 |
| 50 | 옳은 일을 하면 죽어도 옳은 귀신이 된다 | stdict | 0+1 | 0+0 | 0+2 | 3 |

### Rank-frequency curve, krdict entries only (654 ranked; ranks within this subset)

| ranks | n | min total | median total | max total |
|---|---|---|---|---|
| 1-50 | 50 | 2 | 11.5 | 151 |
| 51-100 | 50 | 1 | 3 | 13 |
| 101-200 | 100 | 1 | 3 | 16 |
| 201-300 | 100 | 1 | 1 | 4 |
| 301-500 | 200 | 0 | 0 | 1 |
| 501-1000 | 154 | 0 | 0 | 0 |
| 1001- | 0 | | | |

| rank | total occurrences | score (per million eojeol) |
|---|---|---|
| 100 | 1 | 0.064 |
| 300 | 1 | 0.002 |
| 500 | 0 | 0.000 |

### Top 50, krdict entries only (# = rank among all)

| # | text | src | kowiki | kowikisource | opensubtitles | total |
|---|---|---|---|---|---|---|
| 1 | 누워서 떡 먹기 | krdict,stdict | 5+0 | 0+0 | 16+0 | 21 |
| 2 | 호랑이도 제 말 하면 온다 | krdict,stdict | 3+0 | 2+0 | 10+0 | 15 |
| 3 | 밑져야 본전 | krdict,stdict | 6+0 | 7+0 | 6+0 | 19 |
| 4 | 눈에는 눈을 이에는 이를 | krdict,stdict | 53+0 | 0+0 | 5+0 | 58 |
| 5 | 누이 좋고 매부 좋다 | krdict,stdict | 0+0 | 2+0 | 6+0 | 8 |
| 6 | 주먹이 운다 | krdict,stdict | 148+0 | 2+0 | 1+0 | 151 |
| 7 | 갈수록 태산 | krdict,stdict | 2+0 | 2+0 | 5+0 | 9 |
| 8 | 등잔 밑이 어둡다 | krdict,stdict | 1+0 | 2+0 | 5+0 | 8 |
| 9 | 시작이 반이다 | krdict,stdict | 4+0 | 9+0 | 4+0 | 17 |
| 10 | 따 놓은 당상 | krdict,stdict | 1+0 | 0+0 | 5+0 | 6 |
| 13 | 주사위는 던져졌다 | krdict,stdict | 10+0 | 1+0 | 4+0 | 15 |
| 15 | 우물 안 개구리 | krdict,stdict | 24+0 | 3+0 | 3+0 | 30 |
| 16 | 불난 집에 부채질한다 | krdict,stdict | 1+0 | 0+0 | 4+0 | 5 |
| 17 | 무소식이 희소식 | krdict,stdict | 0+0 | 0+0 | 4+0 | 4 |
| 18 | 꿀 먹은 벙어리 | krdict,stdict | 2+0 | 8+0 | 3+0 | 13 |
| 20 | 수박 겉 핥기 | krdict,stdict | 2+0 | 7+0 | 3+0 | 12 |
| 22 | 상갓집 개 | krdict,stdict | 0+0 | 34+0 | 0+0 | 34 |
| 23 | 아닌 밤중에 홍두깨 내밀듯 | krdict,stdict | 1+0 | 5+0 | 3+0 | 9 |
| 24 | 지성이면 감천 | krdict,stdict | 71+0 | 13+0 | 0+0 | 84 |
| 28 | 새 발의 피 | krdict,stdict | 3+0 | 1+0 | 3+0 | 7 |
| 29 | 김칫국부터 마신다 | krdict,stdict | 2+0 | 1+0 | 3+0 | 6 |
| 30 | 말이 씨가 된다 | krdict,stdict | 2+0 | 0+0 | 3+0 | 5 |
| 31 | 꿩 먹고 알 먹는다 | krdict,stdict | 4+0 | 8+0 | 2+0 | 14 |
| 32 | 무자식 상팔자 | krdict,stdict | 58+0 | 1+0 | 1+0 | 60 |
| 33 | 열 길 물속은 알아도 한 길 사람의 속은 모른다 | krdict,stdict | 0+2 | 4+3 | 0+2 | 11 |
| 34 | 겉과 속이 다르다 | krdict,stdict | 22+0 | 1+0 | 2+0 | 25 |
| 35 | 그 아버지에 그 아들 | krdict,stdict | 12+0 | 2+0 | 2+0 | 16 |
| 36 | 산 넘어 산이다 | krdict,stdict | 15+0 | 1+0 | 2+0 | 18 |
| 37 | 하늘이 무너져도 솟아날 구멍이 있다 | krdict,stdict | 0+0 | 5+0 | 2+0 | 7 |
| 38 | 더도 말고 덜도 말고 늘 한가위만 같아라 | krdict | 0+14 | 0+1 | 0+2 | 17 |
| 40 | 속 빈 강정 | krdict,stdict | 3+0 | 3+0 | 2+0 | 8 |
| 41 | 옷이 날개라 | krdict,stdict | 8+0 | 1+0 | 2+0 | 11 |
| 42 | 밑 빠진 독에 물 붓기 | krdict,stdict | 1+0 | 2+0 | 2+0 | 5 |
| 45 | 동에 번쩍 서에 번쩍 | krdict,stdict | 5+0 | 9+0 | 1+0 | 15 |
| 46 | 울며 겨자 먹기 | krdict,stdict | 31+0 | 11+0 | 0+0 | 42 |
| 47 | 입이 열 개라도 할 말이 없다 | krdict,stdict | 0+0 | 1+0 | 1+1 | 3 |
| 48 | 한 귀로 듣고 한 귀로 흘린다 | krdict,stdict | 0+0 | 1+0 | 2+0 | 3 |
| 49 | 계란으로 바위 치기 | krdict,stdict | 3+0 | 0+0 | 2+0 | 5 |
| 51 | 여자가 한을 품으면 오뉴월에도 서리가 내린다 | krdict,stdict | 0+0 | 0+0 | 2+0 | 2 |
| 52 | 금이야 옥이야 | krdict,stdict | 34+0 | 9+0 | 0+0 | 43 |
| 53 | 고양이 앞에 쥐 | krdict,stdict | 0+0 | 9+0 | 1+0 | 10 |
| 54 | 눈 가리고 아웅 | krdict,stdict | 7+0 | 7+0 | 1+0 | 15 |
| 55 | 긁어 부스럼 | krdict,stdict | 3+0 | 8+0 | 1+0 | 12 |
| 60 | 순풍에 돛을 달다 | krdict,stdict | 2+0 | 15+0 | 0+0 | 17 |
| 61 | 금강산도 식후경 | krdict,stdict | 4+0 | 5+0 | 1+0 | 10 |
| 62 | 집도 절도 없다 | krdict,stdict | 3+0 | 5+0 | 1+0 | 9 |
| 63 | 하늘의 별 따기 | krdict,stdict | 9+0 | 3+0 | 1+0 | 13 |
| 65 | 병 주고 약 준다 | krdict,stdict | 3+0 | 4+0 | 1+0 | 8 |
| 69 | 개 눈에는 똥만 보인다 | krdict,stdict | 0+0 | 4+0 | 1+0 | 5 |
| 70 | 걱정도 팔자다 | krdict,stdict | 0+0 | 4+0 | 1+0 | 5 |

## 성어 (four-character idioms)

5282 candidates, 1685 with at least one hit, ranked 5282.

### Rank-frequency curve, all dictionaries (total raw occurrences across corpora)

| ranks | n | min total | median total | max total |
|---|---|---|---|---|
| 1-50 | 50 | 42 | 305 | 12330 |
| 51-100 | 50 | 34 | 132 | 399 |
| 101-200 | 100 | 10 | 73.5 | 190 |
| 201-300 | 100 | 2 | 35 | 117 |
| 301-500 | 200 | 1 | 16 | 55 |
| 501-1000 | 500 | 2 | 4 | 26 |
| 1001+ | 4282 | 0 | 0 | 5 |

| rank | total occurrences | score (per million eojeol) |
|---|---|---|
| 100 | 155 | 0.475 |
| 300 | 20 | 0.123 |
| 500 | 8 | 0.050 |
| 1000 | 5 | 0.010 |

### Top 50, all dictionaries

| # | text | src | kowiki | kowikisource | opensubtitles | total |
|---|---|---|---|---|---|---|
| 1 | 국가대표 | stdict | 12298 | 19 | 13 | 12330 |
| 2 | 돌연변이 | krdict | 3712 | 22 | 163 | 3897 |
| 3 | 의사소통 | krdict,stdict | 3878 | 244 | 40 | 4162 |
| 4 | 만장일치 | krdict,stdict | 3679 | 60 | 13 | 3752 |
| 5 | 과소평가 | stdict | 521 | 14 | 73 | 608 |
| 6 | 공작부인 | stdict | 785 | 467 | 3 | 1255 |
| 7 | 불가사의 | stdict | 969 | 66 | 38 | 1073 |
| 8 | 자급자족 | stdict | 696 | 81 | 14 | 791 |
| 9 | 자유자재 | krdict,stdict | 620 | 63 | 15 | 698 |
| 10 | 심사숙고 | krdict,stdict | 135 | 18 | 31 | 184 |
| 11 | 양성평등 | krdict,stdict | 177 | 257 | 4 | 438 |
| 12 | 고군분투 | stdict | 761 | 10 | 13 | 784 |
| 13 | 배은망덕 | krdict,stdict | 74 | 17 | 31 | 122 |
| 14 | 사각지대 | krdict | 339 | 58 | 15 | 412 |
| 15 | 부정부패 | krdict,stdict | 730 | 58 | 2 | 790 |
| 16 | 자살행위 | stdict | 30 | 18 | 26 | 74 |
| 17 | 통행금지 | krdict,stdict | 320 | 33 | 14 | 367 |
| 18 | 근친상간 | krdict | 673 | 19 | 4 | 696 |
| 19 | 기하급수 | krdict | 543 | 5 | 9 | 557 |
| 20 | 남녀노소 | stdict | 322 | 84 | 7 | 413 |
| 21 | 공동생활 | krdict | 153 | 149 | 4 | 306 |
| 22 | 횡설수설 | krdict,stdict | 91 | 15 | 19 | 125 |
| 23 | 난공불락 | krdict,stdict | 148 | 8 | 18 | 174 |
| 24 | 설상가상 | stdict | 519 | 17 | 5 | 541 |
| 25 | 자포자기 | krdict,stdict | 130 | 50 | 13 | 193 |
| 26 | 승승장구 | krdict | 400 | 20 | 8 | 428 |
| 27 | 다재다능 | stdict | 515 | 16 | 4 | 535 |
| 28 | 우유부단 | krdict,stdict | 269 | 26 | 9 | 304 |
| 29 | 전전긍긍 | krdict,stdict | 66 | 56 | 11 | 133 |
| 30 | 유일무이 | krdict,stdict | 397 | 18 | 5 | 420 |
| 31 | 막무가내 | krdict,stdict | 93 | 25 | 13 | 131 |
| 32 | 명실상부 | stdict | 396 | 58 | 0 | 454 |
| 33 | 왈가왈부 | krdict,stdict | 20 | 6 | 16 | 42 |
| 34 | 십중팔구 | stdict | 30 | 27 | 13 | 70 |
| 35 | 일편단심 | stdict | 161 | 54 | 6 | 221 |
| 36 | 만신창이 | krdict,stdict | 79 | 6 | 13 | 98 |
| 37 | 동서남북 | stdict | 282 | 70 | 0 | 352 |
| 38 | 무궁무진 | krdict,stdict | 85 | 30 | 10 | 125 |
| 39 | 공명정대 | krdict,stdict | 83 | 35 | 9 | 127 |
| 40 | 일방통행 | krdict,stdict | 200 | 21 | 7 | 228 |
| 41 | 전대미문 | stdict | 155 | 39 | 6 | 200 |
| 42 | 약육강식 | stdict | 52 | 21 | 11 | 84 |
| 43 | 개과천선 | krdict,stdict | 244 | 14 | 6 | 264 |
| 44 | 임시방편 | krdict,stdict | 141 | 3 | 10 | 154 |
| 45 | 기회균등 | krdict | 108 | 76 | 3 | 187 |
| 46 | 천진난만 | stdict | 216 | 36 | 4 | 256 |
| 47 | 자기중심 | krdict,stdict | 224 | 15 | 6 | 245 |
| 48 | 유명무실 | krdict,stdict | 265 | 57 | 0 | 322 |
| 49 | 시종일관 | krdict | 228 | 30 | 4 | 262 |
| 50 | 속수무책 | krdict,stdict | 200 | 17 | 6 | 223 |

### Rank-frequency curve, krdict entries only (241 ranked; ranks within this subset)

| ranks | n | min total | median total | max total |
|---|---|---|---|---|
| 1-50 | 50 | 41 | 225 | 4162 |
| 51-100 | 50 | 13 | 91 | 287 |
| 101-200 | 100 | 6 | 37 | 126 |
| 201-300 | 41 | 0 | 9 | 20 |
| 301-500 | 0 | | | |
| 501-1000 | 0 | | | |
| 1001- | 0 | | | |

| rank | total occurrences | score (per million eojeol) |
|---|---|---|
| 100 | 18 | 0.308 |

### Top 50, krdict entries only (# = rank among all)

| # | text | src | kowiki | kowikisource | opensubtitles | total |
|---|---|---|---|---|---|---|
| 2 | 돌연변이 | krdict | 3712 | 22 | 163 | 3897 |
| 3 | 의사소통 | krdict,stdict | 3878 | 244 | 40 | 4162 |
| 4 | 만장일치 | krdict,stdict | 3679 | 60 | 13 | 3752 |
| 9 | 자유자재 | krdict,stdict | 620 | 63 | 15 | 698 |
| 10 | 심사숙고 | krdict,stdict | 135 | 18 | 31 | 184 |
| 11 | 양성평등 | krdict,stdict | 177 | 257 | 4 | 438 |
| 13 | 배은망덕 | krdict,stdict | 74 | 17 | 31 | 122 |
| 14 | 사각지대 | krdict | 339 | 58 | 15 | 412 |
| 15 | 부정부패 | krdict,stdict | 730 | 58 | 2 | 790 |
| 17 | 통행금지 | krdict,stdict | 320 | 33 | 14 | 367 |
| 18 | 근친상간 | krdict | 673 | 19 | 4 | 696 |
| 19 | 기하급수 | krdict | 543 | 5 | 9 | 557 |
| 21 | 공동생활 | krdict | 153 | 149 | 4 | 306 |
| 22 | 횡설수설 | krdict,stdict | 91 | 15 | 19 | 125 |
| 23 | 난공불락 | krdict,stdict | 148 | 8 | 18 | 174 |
| 25 | 자포자기 | krdict,stdict | 130 | 50 | 13 | 193 |
| 26 | 승승장구 | krdict | 400 | 20 | 8 | 428 |
| 28 | 우유부단 | krdict,stdict | 269 | 26 | 9 | 304 |
| 29 | 전전긍긍 | krdict,stdict | 66 | 56 | 11 | 133 |
| 30 | 유일무이 | krdict,stdict | 397 | 18 | 5 | 420 |
| 31 | 막무가내 | krdict,stdict | 93 | 25 | 13 | 131 |
| 33 | 왈가왈부 | krdict,stdict | 20 | 6 | 16 | 42 |
| 36 | 만신창이 | krdict,stdict | 79 | 6 | 13 | 98 |
| 38 | 무궁무진 | krdict,stdict | 85 | 30 | 10 | 125 |
| 39 | 공명정대 | krdict,stdict | 83 | 35 | 9 | 127 |
| 40 | 일방통행 | krdict,stdict | 200 | 21 | 7 | 228 |
| 43 | 개과천선 | krdict,stdict | 244 | 14 | 6 | 264 |
| 44 | 임시방편 | krdict,stdict | 141 | 3 | 10 | 154 |
| 45 | 기회균등 | krdict | 108 | 76 | 3 | 187 |
| 47 | 자기중심 | krdict,stdict | 224 | 15 | 6 | 245 |
| 48 | 유명무실 | krdict,stdict | 265 | 57 | 0 | 322 |
| 49 | 시종일관 | krdict | 228 | 30 | 4 | 262 |
| 50 | 속수무책 | krdict,stdict | 200 | 17 | 6 | 223 |
| 51 | 전무후무 | krdict,stdict | 186 | 37 | 4 | 227 |
| 53 | 위풍당당 | krdict,stdict | 158 | 5 | 8 | 171 |
| 54 | 파란만장 | krdict,stdict | 320 | 13 | 2 | 335 |
| 55 | 불협화음 | krdict | 389 | 10 | 0 | 399 |
| 56 | 과잉보호 | krdict,stdict | 40 | 3 | 11 | 54 |
| 57 | 기상천외 | krdict | 187 | 44 | 2 | 233 |
| 60 | 기진맥진 | krdict,stdict | 51 | 49 | 5 | 105 |
| 62 | 학술회의 | krdict,stdict | 235 | 42 | 0 | 277 |
| 63 | 임기응변 | krdict,stdict | 127 | 31 | 4 | 162 |
| 66 | 부국강병 | krdict | 153 | 58 | 0 | 211 |
| 67 | 시대착오 | krdict | 231 | 23 | 1 | 255 |
| 68 | 백발백중 | krdict,stdict | 68 | 20 | 6 | 94 |
| 69 | 자업자득 | krdict,stdict | 32 | 0 | 9 | 41 |
| 70 | 반신반의 | krdict,stdict | 56 | 49 | 3 | 108 |
| 71 | 인신공격 | krdict,stdict | 128 | 18 | 4 | 150 |
| 72 | 박수갈채 | krdict,stdict | 92 | 9 | 6 | 107 |
| 74 | 망연자실 | krdict,stdict | 67 | 14 | 6 | 87 |
