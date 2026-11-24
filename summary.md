# Hanja certainty report

Owner rule: if in doubt, no hanja (only keep what a Korean dictionary confirms).

## Stdict-based fill (`hanja_fill_report.tsv`): 627 fills

| certainty | n |
|---|---|
| low | 497 |
| high | 130 |

### by certainty / rule / evidence

| certainty | rule | source | evidence | n |
|---|---|---|---|---|
| low | kengdic | kengdic | cedict | 391 |
| low | derived | kengdic | english_overlap | 105 |
| high | kengdic | kengdic | opendict | 63 |
| high | derived | kengdic | stdict_derived | 39 |
| high | kengdic | kengdic | opendict+cedict | 14 |
| high | derived | kengdic | opendict_derived | 6 |
| high | homograph | krdict | def_score>=0.5 | 4 |
| high | derived | krdict | stdict_derived | 2 |
| high | unique | krdict | def_score>=0.5 | 2 |
| low | unique | krdict | def_score<0.5 | 1 |

kept: {'true': 627}

## kengdic fallback (`hanja_kengdic_fallback.tsv`): 303 candidates

| verdict | certainty | evidence | n |
|---|---|---|---|
| kept | high | stdict_hanja_no_native | 180 |
| dropped |  |  | 106 |
| kept | high | stdict_best_match | 13 |
| kept | low | stdict_silent | 3 |
| kept | high | reviewed_keep | 1 |

## 40 random LOW fills (stdict-based)

| headword | hanja | source | rule | score | core gloss / definition | stdict definition | evidence |
|---|---|---|---|---|---|---|---|
| 신사상 | 新思想 | kengdic | kengdic | 1.00 | new Thought | 새로운 사상. | cedict |
| 흡입기 | 吸入器 | kengdic | kengdic | 1.00 | an inhaler | 약물을 안개나 기체 상태로 하여 기도나 폐에 들어가도록 하는 기구. 마취나 호흡 기관 질환의 치료에 쓴다. | cedict |
| 기숙생 | 寄宿生 | kengdic | kengdic | 1.00 | boarder | 기숙사에서 사는 학생. 또는 기숙하는 학생. | cedict |
| 충적층 | 沖積層 | kengdic | kengdic | 1.00 | alluvium | 비교적 최근에 하천의 활동에 의하여 자갈, 모래, 진흙 따위가 쌓여 이루어진, 아직 굳지 아니한 퇴적층. | cedict |
| 근호 | 根號 | kengdic | kengdic | 1.00 | radical | 거듭제곱근을 나타내는 기호. 제곱근(<img style="vertical-align: middle;" src='http://stdmgt.korean.go.kr:8899/dictionary/compilation/popup/formula.do?latex=%5Csqrt%7B%5C%2C+%5… | cedict |
| 퇴폐시키다 | 頹廢시키다 | kengdic | derived | 1.00 | debauch, demoralize |  | english_overlap |
| 간헐 | 間歇 | kengdic | kengdic | 1.00 | intermitence | 얼마 동안의 시간 간격을 두고 되풀이하여 일어났다 쉬었다 함. | cedict |
| 금홍석 | 金紅石 | kengdic | kengdic | 1.00 | rutile | 이산화 타이타늄으로 이루어진 산화 광물. 정방 정계에 속하며, 기둥 모양으로 된 결정으로, 붉은 갈색, 회색, 푸른색, 검은색 따위를 띤다. 변성암이나 화성암 따위에 들어 있다. | cedict |
| 격리시키다 | 隔離시키다 | kengdic | derived | 1.00 | insulate, isolate |  | english_overlap |
| 집행인 | 執行人 | kengdic | kengdic | 1.00 | an executor | 실제로 시행하는 사람. | cedict |
| 선회시키다 | 旋回시키다 | kengdic | derived | 1.00 | wheel |  | english_overlap |
| 투기자 | 投機者 | kengdic | kengdic | 1.00 | a speculator | 시세 변동을 이용하여 요행으로 큰 이익을 얻으려는 사람. | cedict |
| 회귀선 | 回歸線 | kengdic | kengdic | 1.00 | tropic | 적도를 중심으로 하여 남북 각 23도 27분을 지나는 위선(緯線). 북쪽을 북회귀선 또는 하지선, 남쪽을 남회귀선 또는 동지선이라 한다. 북회귀선상에서는 하지에, 남회귀선상에서는 동지에 태양이 머리 위에 온다. 위도 23도 27분은 적도가 지구의 공전 궤도면과 이루는 각이다. | cedict |
| 뇌막염 | 腦膜炎 | kengdic | kengdic | 1.00 | brain fever; meningitis | 수막의 염증. 열이 나며, 뇌척수액의 압력이 올라가기 때문에, 심한 두통·구역질·목이 뻣뻣해지는 증상이 나타난다. | cedict |
| 나자식물 | 裸子植物 | kengdic | kengdic | 1.00 | gymnosperm | 밑씨가 씨방 안에 있지 않고 드러나 있는 식물. 가루받이 때 꽃가루가 밑씨 위에 바로 붙는다. 꽃잎은 없으며, 줄기에는 형성층이 발달하였으나 물관이 없고 헛물관을 갖는다. 소나무, 소철, 잣나무, 전나무, 은행나무 따위가 있다. | cedict |
| 축거 | 軸距 | kengdic | kengdic | 1.00 | wheelbase | 자동차의 앞바퀴 중심과 뒷바퀴 중심 사이의 거리. 이것이 길면 차의 안전성이 좋고, 짧으면 차의 회전 반경이 작아서 짧게 돌 수 있다. | cedict |
| 일광욕실 | 日光浴室 | kengdic | kengdic | 1.00 | a solarium, a sunroom | 일광욕을 하기 위하여 벽을 유리로 만든 방. | cedict |
| 감화원 | 感化院 | kengdic | kengdic | 1.00 | a reformatory, a penitentiary | 보호 처분을 받은 청소년을 수용하여 생각과 감정을 좋은 방향으로 변화시키는 기관. | cedict |
| 극피동물 | 棘皮動物 | kengdic | kengdic | 1.00 | echinoderm | 극피동물문의 동물을 통틀어 이르는 말. | cedict |
| 십자형 | 十字形 | kengdic | kengdic | 1.00 | crisscross, in a cross shape | ‘十’ 자와 같은 모양. | cedict |
| 충혈시키다 | 充血시키다 | kengdic | derived | 1.00 | congest, inflame |  | english_overlap |
| 유선염 | 乳腺炎 | kengdic | kengdic | 1.00 | mastitis | 젖꼭지에 생긴 상처로 화농균이 침입하여 일어나는 젖샘의 염증. 젖샘이 부어 빨개지고 통증이 심하다. | cedict |
| 활용시키다 | 活用시키다 | kengdic | derived | 1.00 | to inflect, conjugate |  | english_overlap |
| 석뇌유 | 石腦油 | kengdic | kengdic | 1.00 | naphtha | 150~220℃에서 증류하여 얻는 중질의 가솔린. | cedict |
| 상호작용하다 | 相互作用하다 | kengdic | derived | 1.00 | interact; interplay |  | english_overlap |
| 절삭 | 切削 | kengdic | kengdic | 1.00 | machine cutting | 금속 따위를 자르거나 깎음. | cedict |
| 분자식 | 分子式 | kengdic | kengdic | 1.00 | molecular formula | 분자 하나에 들어 있는 각 원소의 종류와 개수를 나타내는 화학식. 물의 분자식은 ‘H2O’, 이산화 탄소의 분자식은 ‘CO2’ 따위로 나타낸다. | cedict |
| 형석 | 螢石 | kengdic | kengdic | 1.00 | fluorite | 플루오린화 칼슘으로 이루어진 할로겐 광물. 등축 정계에 속하며, 육면체 또는 팔면체의 결정이나 덩어리 모양으로 난다. 유리 광택을 내고 무색투명하나 불순물에 따라 누런색·녹색·녹청색 따위를 띠며, 약하고 무르다. 플루오린의 원료, 유리 공업 재료, 야금제(冶金劑) 따위로 쓴다. | cedict |
| 금어초 | 金魚草 | kengdic | kengdic | 1.00 | antirrhinum; snapdragon | 현삼과의 여러해살이풀. 높이는 20~80cm이며, 잎은 어긋나거나 마주나고 갸름한 피침 모양이다. 여름에서 가을까지 흰색, 붉은색, 노란색, 보라색 따위의 금붕어 입처럼 생긴 꽃이 줄기 끝에 이삭을 이루며 핀다. 관상용이며 지중해 연안이 원산지이다. | cedict |
| 척수염 | 脊髓炎 | kengdic | kengdic | 1.00 | myelitis | 몸 밖의 병원체에 의하여 척수에 생기는 여러 가지 염증. 초기에는 발열·팔다리 통증 외에 이완 마비가 일어나며, 점차 운동·지각 마비나 대소변 배설 장애 따위의 증상이 나타난다. | cedict |
| 겸자 | 鉗子 | kengdic | kengdic | 1.00 | forceps; clamp | 날이 서지 않은 가위 모양으로 생긴 외과 수술 기구. 조직이나 기관을 집어서 누르거나 고정하는 데에 쓴다. 분만 집게, 지혈 집게 따위가 있다. | cedict |
| 실재시키다 | 實在시키다 | kengdic | derived | 1.00 | to substantialize |  | english_overlap |
| 낙담시키다 | 落膽시키다 | kengdic | derived | 1.00 | to dampen the spirits of, discourage, dishearten |  | english_overlap |
| 아마포 | 亞麻布 | kengdic | kengdic | 1.00 | cambric, linen | 아마(亞麻)의 실로 짠 얇은 직물을 통틀어 이르는 말. 굵은 실로 짠 것은 양복감으로 쓰고, 가는 실로 짠 것은 셔츠, 손수건, 실내 장식품 따위를 만드는 데 쓴다. | cedict |
| 여순 | 旅順 | kengdic | kengdic | 1.00 | port Arthur | → 뤼순. | cedict |
| 백류석 | 白榴石 | kengdic | kengdic | 1.00 | leucite | 산화 칼륨을 특히 많이 함유한 알칼리 현무암 속에서 나는 규산염 광물. 등축 정계에 속하며 흰색 또는 회색을 띤다. | cedict |
| 출석자 | 出席者 | kengdic | kengdic | 1.00 | an attendant, an attendee, the turnout | 출석한 사람. | cedict |
| 정오표 | 正誤表 | kengdic | kengdic | 1.00 | corrigendum | 출판물 따위에서, 잘못된 글자나 문구를 바로잡아 만든 일람표. | cedict |
| 건락소 | 乾酪素 | kengdic | kengdic | 1.00 | casein | 인단백질의 하나. 포유류의 젖 속에 들어 있는 단백질의 80%를 차지한다. 보통 우유에 산을 가한 다음 침전시켜 얻는다. 식사 대용품·의약품·접착제·인조 섬유를 만드는 데 쓴다. | cedict |
| 부패시키다 | 腐敗시키다 | kengdic | derived | 1.00 | corrupt, empoison, cause to rot |  | english_overlap |

## 40 random LOW fills (kengdic fallback)

| headword | hanja | source | gloss | evidence | verdict |
|---|---|---|---|---|---|
| 한쌍 | —雙 | wikt | a pair, a couple | stdict_silent | kept |
| 운영자 | 運營者 | wikt | operator, administrator, manager | stdict_silent | kept |
| 대결산 | 大決算 | krdict | settlement of accounts; balancing accounts | stdict_silent | kept |

## 40 random HIGH fills (stdict-based)

| headword | hanja | source | rule | score | core gloss / definition | stdict definition | evidence |
|---|---|---|---|---|---|---|---|
| 운동학 | 運動學 | kengdic | kengdic | 1.00 | kinematics | 힘과 운동의 관계는 생각하지 아니하고 물체의 운동만을 수학적으로 기술하는 학문. 역학의 한 분야이다. | opendict+cedict |
| 낙뢰하다 | 落雷하다 | kengdic | derived | 1.00 | thunderstrike | 벼락이 떨어지다. | stdict_derived |
| 견인기 | 牽引機 | kengdic | kengdic | 1.00 | a retractor | 어떤 물체를 끌어당기는 기구. | opendict |
| 군론 | 群論 | kengdic | kengdic | 1.00 | group theory | 군의 이론과 응용에 관하여 연구하는 학문. 수학의 한 분야이다. | opendict+cedict |
| 접종물 | 接種物 | kengdic | kengdic | 1.00 | inoculum | 미생물 배양이나 조직 배양에서, 배양액에 접종할 때 이용하는, 살아 있는 미생물이나 세포가 들어 있는 물질. | opendict |
| 신결석 | 腎結石 | kengdic | kengdic | 1.00 | renal calculus | 콩팥에 오줌 속의 염류의 결정 또는 결석이 생기는 질환. 발작성의 복통이 때때로 일어나며 혈뇨와 돌을 배설하게 되는데, 육식을 잘하는 노인에게는 요산 염석(尿酸鹽石)이, 채식을 하는 어린아이에게는 수산 염석(蓚酸鹽石)이 흔히 발생한다. | opendict |
| 부갑상선 | 副甲狀腺 | kengdic | kengdic | 1.00 | parathyroid gland | ‘부갑상샘’의 전 용어. | opendict+cedict |
| 저혈압증 | 低血壓症 | kengdic | kengdic | 1.00 | hypotension | 혈압이 정상 수치보다 낮은 증상. 최저 혈압이 90mmHg에 미치지 아니하는 경우이다. 의학적으로는 혈압이 낮아서 동맥피가 충분히 장기(臟器)로 순환되기 어려운 상태를 이른다. 피로감, 나른함, 두통, 어깨 결림 따위가 나타난다. | opendict |
| 항행하다 | 航行하다 | kengdic | derived | 1.00 | navigate | 배나 비행기 따위를 타고 항로 또는 궤도를 다니다. | stdict_derived |
| 왕진하다 | 往診하다 | kengdic | derived | 1.00 | visit | 의사가 병원 밖의 환자가 있는 곳으로 가서 진료하다. | stdict_derived |
| 숙영하다 | 宿營하다 | kengdic | derived | 1.00 | lie | 군대가 훈련이나 전쟁을 수행하기 위하여 병영 밖에서 머물러 지내다. 노영하는 일, 사영하는 일, 촌락에서 노영하는 일 따위를 이른다. | stdict_derived |
| 결정화하다 | 結晶化하다 | kengdic | derived | 1.00 | crystallize | 용액이나 융해물 따위로부터 결정이 이루어지다. 또는 결정을 이루다. | stdict_derived |
| 환원법 | 還元法 | kengdic | kengdic | 1.00 | reduction | 정언적 삼단 논법에서, 제2·제3·제4격을 제1격으로 고치는 방법. 직접 환원법과 간접 환원법이 있다. | opendict |
| 윤작하다 | 輪作하다 | kengdic | derived | 1.00 | rotate crops | 같은 주제나 소재로 여러 작가가 돌아가며 글을 쓰다. | stdict_derived |
| 분파적 | 分派的 | kengdic | derived | 1.00 | divisive | 여러 갈래로 나뉘는 성격이 있는 것. | stdict_derived |
| 동시성 | 同時性 | kengdic | kengdic | 1.00 | simultaneity, synchronicity, synchronism | 시간적 간격을 초월하여 종교적 실존이나 순환하는 문화 현상이 영원한 곳에서 되풀이되거나 대면하는 일. 키르케고르의 기독교 사상에 있어서 가장 중요한 개념의 하나로, 그리스도와 기독교 신자가 역사의 시간을 넘어서 동시에 결부되는 것과 같은 현상을 이른다. | opendict |
| 만곡하다 | 彎曲하다 | kengdic | derived | 1.00 | curve | 활 모양으로 굽다. | stdict_derived |
| 호흡률 | 呼吸率 | kengdic | kengdic | 1.00 | respiratory quotient (ratio) | 생물이 일정한 시간에 흡수한 산소의 부피로 같은 시간에 배출한 이산화 탄소의 부피를 나눈 값. | opendict |
| 용골 | 龍骨 | kengdic | kengdic | 1.00 | keel | 선박 바닥의 중앙을 받치는 길고 큰 재목. 이물에서 고물에 걸쳐 선체를 받치는 기능을 한다. | opendict+cedict |
| 대망 | 待望 | krdict | homograph | 0.60 | 많은 사람들이 몹시 기다리고 바라는 것. | 기다리고 바람. | def_score>=0.5 |
| 어교 | 魚膠 | kengdic | kengdic | 1.00 | fish glue | 민어의 부레를 끓여서 만든 풀. 교착력이 강하여 목기(木器)를 붙이는 데 많이 쓴다. | opendict |
| 재시험하다 | 再試驗하다 | kengdic | derived | 1.00 | reexamine, retest | 두 번 시험을 치다. | stdict_derived |
| 건위제 | 健胃劑 | kengdic | kengdic | 1.00 | stomachic | 위장을 튼튼하게 하는 약제. 소화액의 분비를 왕성하게 하고 위장의 운동을 촉진시켜서 소화·흡수 작용을 돕는다. | opendict |
| 근전도 | 筋電圖 | kengdic | kengdic | 1.00 | electromyogram | 근육의 움직임에 따라 발생하는 전류의 변화를 기록하는 그래프. | opendict |
| 굴곡하다 | 屈曲하다 | kengdic | derived | 1.00 | to wind, to turn, to inflect | 이리저리 굽어 꺾여 있다. | stdict_derived |
| 관리도 | 管理圖 | kengdic | kengdic | 1.00 | control chart | 관리를 잘하기 위하여 필요한 사항을 적거나 그린 도표나 그래프를 통틀어 이르는 말. | opendict |
| 서술적 | 敍述的 | kengdic | derived | 1.00 | declarative, descriptive, predicative |  | opendict_derived |
| 격벽 | 隔壁 | kengdic | kengdic | 1.00 | bulkhead, partition wall, septum | 벽을 사이에 둠. | opendict |
| 유지질 | 類脂質 | kengdic | kengdic | 1.00 | lipoid | 지방과 성질이 비슷하나 구조가 다른 화합물을 통틀어 이르는 말. 물에는 녹지 않으나 에테르, 클로로포름, 벤젠 따위의 유기 용매에는 녹는다. 레시틴, 콜레스테롤, 에고스테롤 따위가 있다. | opendict |
| 폭격수 | 爆擊手 | kengdic | kengdic | 1.00 | bombardier | 폭격기에서 폭탄을 떨어뜨리는 일을 맡아 하는 사람. | opendict |
| 효소원 | 酵素原 | kengdic | kengdic | 1.00 | a zymogen | 비활성 전구물질로, 생체에서 만들어져 분비된 다음 다른 물질의 작용을 받고서 비로소 참된 효소가 되는 일부 효소를 이르는 말. | opendict |
| 수량화 | 數量化 | kengdic | kengdic | 1.00 | to quantifie and count things | 지능, 생활 수준, 사회적 태도처럼 수량으로 직접 나타내기 어려운 현상에 대하여 간접적으로 수량을 부여함. | opendict |
| 철침착증 | 鐵沈着症 | kengdic | kengdic | 1.00 | siderosis | 진폐증의 하나. 철이 함유되어 있는 미립자를 오랫동안 흡입함으로써 철분이 폐에 침착하여 일어난다. | opendict |
| 수소화하다 | 水素化하다 | kengdic | derived | 1.00 | to hydrogenate |  | opendict_derived |
| 휴교하다 | 休校하다 | kengdic | derived | 1.00 | recess | 학교가 학생을 가르치는 업무를 한동안 쉬다. | stdict_derived |
| 등편각선 | 等偏角線 | kengdic | kengdic | 1.00 | isogonic | 자기(磁氣) 자오선의 방위각이 같은 여러 개의 지점을 지도 위에 이은 선. | opendict |
| 동선 | 動線 | krdict | homograph | 0.52 | 어떤 일을 할 때 사람이 움직이는 거리나 방향을 나타내는 선. | 건축물의 내외부에서, 사람이나 물건이 어떤 목적이나 작업을 위하여 움직이는 자취나 방향을 나타내는 선. | def_score>=0.5 |
| 무진히 | 無盡히 | krdict | derived | 1.00 | 끝이 없을 만큼 매우. | 다함이 없을 만큼 매우. | stdict_derived |
| 다육성 | 多肉性 | kengdic | kengdic | 1.00 | fleshiness | 식물의 잎과 줄기, 열매 따위에 살이 많은 특성. | opendict |
| 원자가 | 原子價 | kengdic | kengdic | 1.00 | atomic value, valence | 원자의 원자량 또는 원자단의 원자량 총량을 화학 당량으로 나눈 수. 보통 수소를 기준으로 하여 수소 원자 n개와 치환 또는 결합하는 원소의 것을 n가라고 한다. | opendict |

## 40 random HIGH fills (kengdic fallback)

| headword | hanja | source | gloss | evidence | verdict |
|---|---|---|---|---|---|
| 자영업자 | 自營業者 | wikt | business owner | stdict_hanja_no_native | kept |
| 아래층 | 아래層 | wikt | downstairs | stdict_hanja_no_native | kept |
| 자주색 | 紫朱色 | wikt | violet, purple (color) | stdict_hanja_no_native | kept |
| 저음 | 低音 | wikt | low note, low pitch | stdict_hanja_no_native | kept |
| 면류관 | 冕旒冠 | wikt | royal crown | stdict_hanja_no_native | kept |
| 패가망신 | 敗家亡身 | wikt | ruining oneself and one's own family | stdict_hanja_no_native | kept |
| 양쪽 | 兩쪽 | wikt | both sides, both parties | stdict_hanja_no_native | kept |
| 팔각형 | 八角形 | krdict | octagon | stdict_best_match | kept |
| 무책임 | 無責任 | krdict | irresponsibility | stdict_best_match | kept |
| 융통성 | 融通性 | wikt | flexibility | stdict_hanja_no_native | kept |
| 매달 | 每달 | wikt | monthly | stdict_hanja_no_native | kept |
| 위층 | 위層 | wikt | upstairs | stdict_hanja_no_native | kept |
| 사무직 | 事務職 | wikt | office job, white-collar job | stdict_hanja_no_native | kept |
| 하숙집 | 下宿집 | wikt | boarding house | stdict_hanja_no_native | kept |
| 북쪽 | 北쪽 | wikt | north, northward | stdict_hanja_no_native | kept |
| 외할머니 | 外할머니 | wikt | maternal grandmother | stdict_hanja_no_native | kept |
| 해체 | 解體 | wikt | The informal, non-polite speech level in Korean. | stdict_hanja_no_native | kept |
| 육개장 | 肉개醬 | wikt | yukgaejang, a spicy Korean dish made of beef, powdered red pepper, and fern (or mung bean sprout). | stdict_hanja_no_native | kept |
| 대칭 | 對稱 | krdict | symmetry | stdict_best_match | kept |
| 복사본 | 複寫本 | wikt | copy (as opposed to the original) | stdict_hanja_no_native | kept |
| 생명체 | 生命體 | wikt | life, lifeform | stdict_hanja_no_native | kept |
| 가두 | 街頭 | wikt | street | stdict_hanja_no_native | kept |
| 야인 | 野人 | wikt | unrefined person; uncivilized person | stdict_hanja_no_native | kept |
| 노란색 | 노란色 | wikt | yellow (color) | stdict_hanja_no_native | kept |
| 인문 | 人文 | wikt | the humanities | stdict_hanja_no_native | kept |
| 칫솔 | 齒솔 | wikt | toothbrush | stdict_hanja_no_native | kept |
| 맞은편 | 맞은便 | wikt | other side | stdict_hanja_no_native | kept |
| 장모님 | 丈母님 | wikt | honorific of 장모 (jangmo, “mother-in-law, wife's mother”) | stdict_hanja_no_native | kept |
| 번째 | 番째 | wikt | a dependent noun used to express the order or relative position of an object; a construction equivalent to ordinal numb… | stdict_hanja_no_native | kept |
| 시아버지 | 媤아버지 | wikt | father-in-law (husband's father) | stdict_hanja_no_native | kept |
| 하악 | 下顎 | wikt | a hiss, while hissing | stdict_hanja_no_native | kept |
| 손수건 | 손手巾 | wikt | handkerchief, hankie | stdict_hanja_no_native | kept |
| 동생 | 同生 | krdict | brother; sister | reviewed_keep | kept |
| 노포 | 老鋪 | wikt | a shop with a long history | stdict_hanja_no_native | kept |
| 웅장 | 雄壯 | wikt | magnificence, splendor, grandeur | stdict_hanja_no_native | kept |
| 모함 | 謀陷 | wikt | slander; entrapping | stdict_hanja_no_native | kept |
| 처자식 | 妻子息 | wikt | one's wife and children; one's family | stdict_hanja_no_native | kept |
| 청설모 | 靑鼠毛 | wikt | a red squirrel, Sciurus vulgaris | stdict_hanja_no_native | kept |
| 증식 | 增殖 | wikt | proliferation | stdict_hanja_no_native | kept |
| 자료실 | 資料室 | krdict | reference room | stdict_best_match | kept |

