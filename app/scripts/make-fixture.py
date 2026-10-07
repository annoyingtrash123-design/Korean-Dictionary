#!/usr/bin/env python3
"""Build a small, contract-conformant fixture (core + stdict) into app/public/data/.

Mirrors what pipeline/ produces: <pack>.sqlite -> single gzip stream -> split into
<pack>.sqlite.gz.000 ... plus manifest.json.  Stdlib only.
"""
import gzip, hashlib, io, json, os, re, sqlite3, sys, tempfile, time

OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "public", "data")
CHUNK = int(os.environ.get("FIXTURE_CHUNK", 8 * 1024))  # small so resume is testable
VERSION = os.environ.get("FIXTURE_VERSION", "2025.01.fixture")

SCHEMA = """
CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT);
CREATE TABLE entries (
  id INTEGER PRIMARY KEY, headword TEXT NOT NULL, hw_norm TEXT NOT NULL, homonym INTEGER,
  hanja TEXT, pos TEXT, pron TEXT, source TEXT NOT NULL, lang TEXT NOT NULL, level INTEGER,
  rank INTEGER NOT NULL, kind TEXT NOT NULL, gloss TEXT, data TEXT NOT NULL);
CREATE INDEX entries_hw ON entries(hw_norm);
CREATE INDEX entries_rank ON entries(rank);
CREATE TABLE forms (form TEXT NOT NULL, entry_id INTEGER NOT NULL);
CREATE INDEX forms_form ON forms(form);
CREATE TABLE hanja_words (ch TEXT NOT NULL, entry_id INTEGER NOT NULL);
CREATE INDEX hanja_words_ch ON hanja_words(ch);
"""
CORE_ONLY = """
CREATE VIRTUAL TABLE entries_fts USING fts5(en, content='', tokenize='porter unicode61');
CREATE TABLE hanja_chars (ch TEXT PRIMARY KEY, readings TEXT, meaning_en TEXT, strokes INTEGER, radical TEXT, word_count INTEGER);
CREATE TABLE sentences (id INTEGER PRIMARY KEY, ko TEXT NOT NULL, en TEXT, source TEXT);
CREATE VIRTUAL TABLE sentences_fts USING fts5(ko, content='sentences', content_rowid='id', tokenize='trigram');
CREATE TABLE grammar (id INTEGER PRIMARY KEY, entry_id INTEGER, pattern TEXT NOT NULL, category TEXT NOT NULL, level INTEGER, summary_en TEXT, sort INTEGER);
"""

def norm(h): return re.sub(r"[-^ ·]", "", h)

def ex(ko, en=None, type="sentence"):
    d = {"ko": ko, "type": type}
    if en: d["en"] = en
    return d

def sense(gloss, d=None, ko=None, ex_=None, note=None, pattern=None, rel=None, pos=None):
    s = {"gloss": gloss}
    if pos: s["pos"] = pos
    if d: s["def"] = d
    if ko: s["ko_def"] = ko
    if note: s["note"] = note
    if pattern: s["pattern"] = pattern
    if ex_: s["examples"] = ex_
    if rel: s["rel"] = [{"type": t, "word": w} for t, w in rel]
    return s

# ---- krdict -------------------------------------------------------------------------------
# (headword, hanja, pos, pron, level, kind, senses, forms, related, category, homonym)
KR = [
 ("학교", "學校", "noun", "학꾜", 1, "word", [
   sense("school", "A place where students are taught.", "학생들이 교사에게 교육을 받는 기관.", [
     ex("저는 학교에 갑니다.", "I go to school."), ex("학교 앞에서 만나요.", "Let's meet in front of the school."),
     ex("가: 학교에 어떻게 가요?\n나: 버스로 가요.", "A: How do you get to school?\nB: I go by bus.", "dialogue")],
     rel=[("synonym", "학원"), ("see also", "대학교")])],
   ["학교가", "학교는", "학교를"], [("derived", "초등학교"), ("derived", "대학교")], "교육 > 학교", None),
 ("학생", "學生", "noun", "학쌩", 1, "word", [
   sense("student", "A person who studies at a school.", "학교에 다니면서 공부하는 사람.", [ex("그는 대학교 학생이에요.", "He is a university student.")],
     rel=[("antonym", "선생님")])], ["학생이", "학생은"], [], "교육 > 사람", None),
 ("선생님", None, "noun", "선생님", 1, "word", [
   sense("teacher", "A person who teaches students.", "학생을 가르치는 사람.", [ex("선생님, 질문이 있어요.", "Teacher, I have a question.")],
     rel=[("antonym", "학생")])], [], [], "교육 > 사람", None),
 ("대학교", "大學校", "noun", "대학꾜", 1, "word", [sense("university", "A school for higher education.", "고등 교육을 하는 학교.", [ex("저는 서울 대학교에 다녀요.", "I attend Seoul University.")])], [], [], None, None),
 ("사람", None, "noun", "사람", 1, "word", [sense("person; people", "A human being.", "생각하고 말할 수 있는 동물.", [ex("저 사람은 누구예요?", "Who is that person?")])], ["사람이"], [], None, None),
 ("사랑", None, "noun", "사랑", 1, "word", [
   sense("love", "A strong feeling of affection for someone.", "어떤 사람을 아끼고 소중히 여기는 마음.", [ex("부모님의 사랑은 끝이 없어요.", "Parents' love is endless.")], rel=[("synonym", "애정"), ("antonym", "미움")])],
   ["사랑이", "사랑을"], [("derived", "사랑하다")], "감정", None),
 ("사랑하다", None, "verb", "사랑하다", 1, "word", [sense("to love", "To feel love for someone.", "누군가를 아끼고 소중히 여기다.", [ex("저는 당신을 사랑해요.", "I love you."), ex("그들은 서로 사랑했어요.", "They loved each other.")], pattern="1이 2를 사랑하다")],
   ["사랑해요", "사랑했어요", "사랑한다", "사랑합니다"], [], "감정", None),
 ("가다", None, "verb", "가다", 1, "word", [
   sense("to go", "To move from one place to another.", "어떤 장소에서 다른 장소로 이동하다.", [ex("학교에 가요.", "I go to school."), ex("어제 친구 집에 갔어요.", "I went to my friend's house yesterday.")], pattern="1이 2에 가다", rel=[("antonym", "오다")]),
   sense("to attend; to go (regularly)", "To regularly attend a place.", "어떤 곳에 다니다.", [ex("동생은 유치원에 가요.", "My younger sibling goes to kindergarten.")])],
   ["가", "가요", "갔어요", "갔다", "간다", "갑니다", "가서", "갈", "간"], [], "이동", None),
 ("오다", None, "verb", "오다", 1, "word", [sense("to come", "To move toward the speaker.", "말하는 사람 쪽으로 이동하다.", [ex("친구가 우리 집에 왔어요.", "My friend came to my house.")], rel=[("antonym", "가다")])], ["와요", "왔어요", "온다", "옵니다"], [], None, None),
 ("먹다", None, "verb", "먹따", 1, "word", [
   sense("to eat", "To take food into the mouth and swallow.", "음식을 입에 넣어 삼키다.", [ex("저는 밥을 먹어요.", "I eat rice."), ex("점심 먹었어요?", "Did you eat lunch?")], pattern="1이 2를 먹다", note="Honorific: 드시다 / 잡수시다"),
   sense("to be affected by (age, fear…)", "To get older.", "나이를 더하다.", [ex("나이를 먹으면 몸이 약해져요.", "As you age, your body gets weaker.")])],
   ["먹어요", "먹었어요", "먹는다", "먹습니다", "먹어서", "먹은", "먹을"], [("honorific", "드시다")], "음식", None),
 ("예쁘다", None, "adjective", "예쁘다", 1, "word", [sense("pretty; beautiful", "Pleasing to look at.", "생김새가 보기 좋게 아름답다.", [ex("꽃이 정말 예뻐요.", "The flowers are really pretty.")], rel=[("synonym", "아름답다"), ("antonym", "못생기다")])],
   ["예뻐요", "예뻤어요", "예쁜", "예쁘네요"], [], None, None),
 ("춥다", None, "adjective", "춥따", 1, "word", [sense("cold", "Having a low temperature (weather).", "대기의 온도가 낮다.", [ex("오늘은 너무 추워요.", "It is too cold today."), ex("어제는 추웠어요.", "It was cold yesterday.")], rel=[("antonym", "덥다")])],
   ["추워요", "추웠어요", "추운", "춥습니다"], [], "날씨", None),
 ("덥다", None, "adjective", "덥따", 1, "word", [sense("hot (weather)", "Having a high temperature.", "대기의 온도가 높다.", [ex("여름에는 너무 더워요.", "It is too hot in summer.")], rel=[("antonym", "춥다")])], ["더워요", "더웠어요"], [], None, None),
 ("물", None, "noun", "물", 1, "word", [sense("water", "A clear liquid that falls as rain.", "강, 바다 등을 이루는 액체.", [ex("물 좀 주세요.", "Please give me some water.")])], ["물이", "물을"], [], None, None),
 ("친구", "親舊", "noun", "친구", 1, "word", [sense("friend", "A person you are close to.", "가깝게 오래 사귄 사람.", [ex("제 친구는 한국 사람이에요.", "My friend is Korean.")])], ["친구가", "친구를"], [], None, None),
 ("한국", "韓國", "noun", "한국", 1, "word", [sense("Korea; South Korea", "A country in East Asia.", "동아시아의 나라.", [ex("저는 한국에 살아요.", "I live in Korea.")])], [], [], None, None),
 ("한국어", "韓國語", "noun", "한구거", 1, "word", [sense("Korean (language)", "The language of Korea.", "한국 사람이 쓰는 말.", [ex("한국어를 공부해요.", "I study Korean.")])], [], [], None, None),
 ("생일", "生日", "noun", "생일", 1, "word", [sense("birthday", "The day a person was born.", "태어난 날.", [ex("내일은 제 생일이에요.", "Tomorrow is my birthday.")])], [], [], None, None),
 ("인생", "人生", "noun", "인생", 2, "word", [sense("life (of a person)", "The period from birth to death.", "사람이 살아 있는 동안.", [ex("인생은 짧아요.", "Life is short.")])], [], [], None, None),
 ("애국심", "愛國心", "noun", "애국씸", 3, "word", [sense("patriotism", "Love for one's country.", "나라를 사랑하는 마음.", [ex("그는 애국심이 강해요.", "He is very patriotic.")])], [], [], None, None),
 ("일본", "日本", "noun", "일본", 1, "word", [sense("Japan", None, None, [ex("일본에 가 보고 싶어요.", "I want to visit Japan.")])], [], [], None, None),
 ("배", None, "noun", "배", 1, "word", [sense("ship; boat", "A vehicle that carries people on water.", "사람이나 짐을 싣고 물 위로 다니는 탈것.", [ex("배를 타고 섬에 갔어요.", "I took a boat to the island.")])], ["배가"], [], None, 1),
 ("배", None, "noun", "배", 1, "word", [sense("pear", "A round sweet fruit.", "과일의 하나.", [ex("배가 달고 맛있어요.", "The pear is sweet and delicious.")])], [], [], "음식 > 과일", 2),
 ("배", None, "noun", "배", 1, "word", [sense("belly; stomach", "The front part of the body.", "가슴 아래 부분.", [ex("배가 고파요.", "I'm hungry (my stomach is empty).")])], [], [], "신체", 3),
 ("고맙다", None, "adjective", "고맙따", 1, "word", [sense("thankful; grateful", "Feeling thanks.", "남이 도와주어 감사하다.", [ex("도와줘서 고마워요.", "Thanks for helping.")], rel=[("synonym", "감사하다")])], ["고마워요", "고마웠어요", "고마운"], [], None, None),
 ("공부하다", None, "verb", "공부하다", 1, "word", [sense("to study", "To learn.", "배우고 익히다.", [ex("도서관에서 공부해요.", "I study at the library.")])], ["공부해요", "공부했어요"], [], None, None),
 ("가다가", None, "adverb", "가다가", 3, "word", [sense("on the way", None, None, [ex("가다가 친구를 만났어요.", "I met a friend on the way.")])], [], [], None, None),
 ("눈", None, "noun", "눈", 1, "word", [sense("eye", "The organ of sight.", "보는 기관.", [ex("눈이 아파요.", "My eyes hurt.")]), sense("snow", "Frozen rain.", "얼어서 내리는 물.", [ex("눈이 와요.", "It's snowing.")])], ["눈이"], [], None, None),
 ("고생", "苦生", "noun", "고생", 2, "word", [sense("hardship; trouble", "Suffering through difficulty.", "어렵고 힘들게 지내는 일.", [ex("그동안 고생 많았어요.", "You've been through a lot.")])], [], [], None, None),
 ("눈치가 빠르다", None, "idiom", "눈치가 빠르다", 3, "idiom", [sense("to be quick to catch on", None, None, [ex("그는 눈치가 빨라요.", "He is perceptive.")])], [], [], None, None),
 ("원숭이도 나무에서 떨어진다", None, "proverb", "원숭이도 나무에서 떠러진다", None, "proverb", [sense("even monkeys fall from trees", "Even experts make mistakes.", "아무리 잘하는 사람도 실수할 때가 있다.", [])], [], [], None, None),
]
# grammar entries (krdict, kind=grammar) : (headword, level, category, summary, senses)
GR = [
 ("-아서/어서", 1, "Connective endings", "Because; and then (sequence)", [
    sense("because; so", "Attaches to a verb or adjective stem to give a reason.", "앞의 내용이 뒤의 내용의 이유나 원인임을 나타내는 연결 어미.", [ex("비가 와서 집에 있었어요.", "Because it rained, I stayed home."), ex("피곤해서 일찍 잤어요.", "I was tired so I went to bed early.")], pattern="V/A-아서/어서", note="Cannot be used with imperatives or propositives."),
    sense("and then (sequence)", None, "앞의 행동에 이어 뒤의 행동이 일어남을 나타내는 연결 어미.", [ex("시장에 가서 사과를 샀어요.", "I went to the market and bought apples.")])]),
 ("이/가", 1, "Particles", "Subject marking particle", [
    sense("subject particle", "Marks the subject of a sentence.", "문장의 주어임을 나타내는 조사.", [ex("친구가 와요.", "A friend is coming."), ex("비가 와요.", "It's raining.")], note="이 after a consonant, 가 after a vowel.")]),
 ("은/는", 1, "Particles", "Topic marking particle", [
    sense("topic particle", "Marks the topic or contrast.", "문장의 주제를 나타내는 조사.", [ex("저는 학생이에요.", "I am a student."), ex("커피는 좋아하지만 차는 안 마셔요.", "I like coffee but I don't drink tea.")], note="은 after a consonant, 는 after a vowel.")]),
 ("-고", 1, "Connective endings", "And; and then", [sense("and", "Connects two clauses.", "두 가지 이상의 일을 나열하는 연결 어미.", [ex("밥을 먹고 학교에 가요.", "I eat and go to school.")], pattern="V/A-고")]),
 ("-(으)ㄹ 거예요", 1, "Final endings", "Will; probably (future / conjecture)", [sense("will; going to", None, "미래의 일이나 추측을 나타내는 종결 어미.", [ex("내일 학교에 갈 거예요.", "I will go to school tomorrow.")])]),
 ("-았/었-", 1, "Pre-final endings", "Past tense", [sense("past tense", None, "과거를 나타내는 선어말 어미.", [ex("어제 영화를 봤어요.", "I watched a movie yesterday.")])]),
 ("-는", 2, "Nominal/adnominal endings", "Present adnominal (modifies a noun)", [sense("present adnominal", None, "현재의 동작을 나타내는 관형사형 어미.", [ex("제가 먹는 음식이에요.", "It's the food that I eat.")])]),
 ("-(으)면 좋겠다", 2, "Expressions", "I wish; it would be nice if", [sense("I hope; I wish", None, "바람을 나타내는 표현.", [ex("내일 날씨가 좋으면 좋겠어요.", "I hope the weather is good tomorrow.")])]),
 ("-님", 2, "Affixes", "Honorific suffix for people", [sense("honorific suffix", "Added to a name or title to show respect.", "높임을 나타내는 접미사.", [ex("사장님은 회의 중이세요.", "The president is in a meeting.")])]),
]
# ---- wikt / kengdic -----------------------------------------------------------------------
WK = [
 ("학교", "學校", "noun", "hak-kyo", [sense("school", None, None, [ex("우리 학교는 크다.", "Our school is big.")])], ["학교에서"], [("variant", "학굔")], "Sino-Korean word from 學校 (學 'study' + 校 'school')."),
 ("가다", None, "verb", "gada", [sense("to go", None, None, [ex("어디 가요?", "Where are you going?")]), sense("to leave; to depart", None, None), sense("(of a machine) to work", None, None)], ["갔다"], [], "From Middle Korean 가다."),
 ("사랑", None, "noun", "sarang", [sense("love; affection", None, None), sense("(archaic) a man's quarters in a traditional house", None, None, pos="noun")], [], [], None),
 ("김치", None, "noun", "gimchi", [sense("kimchi", None, None, [ex("김치를 좋아해요?", "Do you like kimchi?")])], ["김치가"], [], None),
 ("먹다", None, "verb", "meokda", [sense("to eat", None, None), sense("to drink (medicine)", None, None), sense("to accept (a bribe)", None, None)], [], [], None),
]
KE = [("학교", "學校", "noun", "school"), ("학생", "學生", "noun", "student; pupil"), ("한국말", "韓國말", "noun", "Korean language"),
      ("김밥", None, "noun", "gimbap; Korean seaweed rice roll"), ("애정", "愛情", "noun", "affection; love")]
# ---- stdict -------------------------------------------------------------------------------
SD = [
 ("학교", "學校", "noun", "학꾜", [sense(None, None, "일정한 목적, 교과 과정, 설비, 제도 및 법규에 의하여 교사가 계속적으로 학생에게 교육을 실시하는 기관.", [ex("학교에 다니다.", None, "phrase")])], ["학교가", "학교의"], None),
 ("학생", "學生", "noun", "학쌩", [sense(None, None, "학교에 다니며 공부하는 사람.")], [], None),
 ("가다", None, "verb", "가다", [sense(None, None, "일정한 곳을 향하여 이동하다."), sense(None, None, "다른 곳으로 옮겨 가거나 떠나다.")], ["가", "갔다", "가서"], 1),
 ("먹다", None, "verb", "먹따", [sense(None, None, "음식 따위를 입을 통하여 배 속에 들여보내다.")], ["먹어", "먹은"], 1),
 ("사랑", None, "noun", "사랑", [sense(None, None, "어떤 사람이나 존재를 몹시 아끼고 소중히 여기는 마음."), sense(None, None, "남녀 간에 서로 그리워하는 마음.")], [], 1),
 ("사랑", "舍廊", "noun", "사랑", [sense(None, None, "집의 안채와 떨어져 있는, 바깥주인이 거처하며 손님을 접대하는 곳.")], [], 2),
 ("예쁘다", None, "adjective", "예쁘다", [sense(None, None, "생김새가 아름다워 눈으로 보기에 좋다.")], ["예뻐"], None),
 ("춥다", None, "adjective", "춥따", [sense(None, None, "대기의 온도가 낮거나 몸에 느끼는 기온이 낮다.")], ["추워", "추운"], None),
 ("학년", "學年", "noun", "항년", [sense(None, None, "학교에서 일 년간 배울 과정에 따라 구분한 단계.")], [], None),
 ("고등학교", "高等學校", "noun", "고등학꾜", [sense(None, None, "중학교 졸업자에게 중등 교육을 하는 학교.")], [], None),
 ("인생", "人生", "noun", "인생", [sense(None, None, "사람이 이 세상을 살아가는 일.")], [], None),
 ("애국", "愛國", "noun", "애국", [sense(None, None, "나라를 사랑함.")], [], None),
]
HANJA = {
 "學": ("학", "to study; learning", 16, "子"), "校": ("교", "school", 10, "木"), "生": ("생", "to live; to be born; raw", 5, "生"),
 "愛": ("애", "love; to love", 13, "心"), "人": ("인", "person", 2, "人"), "日": ("일", "sun; day", 4, "日"),
 "本": ("본", "root; origin; book", 5, "木"), "國": ("국", "country", 11, "囗"), "大": ("대", "big; great", 3, "大"),
 "苦": ("고", "bitter; suffering", 8, "艸"), "韓": ("한", "Korea; Han", 17, "韋"), "語": ("어", "language; words", 14, "言"),
 "親": ("친", "parent; intimate", 16, "見"), "舊": ("구", "old; former", 18, "臼"), "高": ("고", "tall; high", 10, "高"),
 "等": ("등", "class; rank; equal", 12, "竹"), "年": ("년", "year", 6, "干"), "心": ("심", "heart; mind", 4, "心"),
 "情": ("정", "feeling; emotion", 11, "心"), "廊": ("랑", "corridor", 12, "广"), "舍": ("사", "house; to give up", 8, "舌"),
}
SENT = [
 ("저는 학교에 갑니다.", "I go to school.", "tatoeba"), ("학교는 아홉 시에 시작해요.", "School starts at nine.", "tatoeba"),
 ("우리 학교는 서울에 있어요.", "Our school is in Seoul.", "tatoeba"), ("나는 학생이에요.", "I am a student.", "tatoeba"),
 ("그 학생은 공부를 열심히 해요.", "That student studies hard.", "tatoeba"), ("나는 매일 아침 밥을 먹어요.", "I eat rice every morning.", "tatoeba"),
 ("뭐 먹고 싶어요?", "What do you want to eat?", "tatoeba"), ("어제 학교에 가지 않았어요.", "I didn't go to school yesterday.", "tatoeba"),
 ("나는 한국어를 사랑해요.", "I love Korean.", "tatoeba"), ("사랑은 아름다운 것이다.", "Love is a beautiful thing.", "tatoeba"),
 ("오늘은 정말 춥네요.", "It's really cold today.", "tatoeba"), ("그녀는 정말 예쁘다.", "She is really pretty.", "tatoeba"),
 ("너는 어디에 가고 있니?", "Where are you going?", "tatoeba"), ("집에 가고 싶어요.", "I want to go home.", "tatoeba"),
 ("김치를 먹어 봤어요?", "Have you tried kimchi?", "tatoeba"),
]

def build(path, pack):
    db = sqlite3.connect(path)
    db.executescript(SCHEMA + (CORE_ONLY if pack == "core" else ""))
    nid = [0]; counts = {"entries": 0}
    def add(source, lang, hw, hanja, pos, pron, level, kind, gloss, data, homonym, forms, rank):
        nid[0] += 1; i = nid[0]
        db.execute("INSERT INTO entries VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
                   (i, hw, norm(hw), homonym, hanja, pos, pron, source, lang, level, rank, kind, gloss, json.dumps(data, ensure_ascii=False)))
        for f in forms: db.execute("INSERT INTO forms VALUES (?,?)", (f, i))
        for ch in dict.fromkeys(hanja or ""):
            if "一" <= ch <= "鿿": db.execute("INSERT INTO hanja_words VALUES (?,?)", (ch, i))
        if pack == "core" and lang == "en" and gloss:
            fts = " ".join([gloss] + [s.get("def", "") for s in data["senses"]])
            db.execute("INSERT INTO entries_fts(rowid, en) VALUES (?,?)", (i, fts))
        counts["entries"] += 1
        return i
    def gl(senses): return "; ".join(s["gloss"] for s in senses if s.get("gloss"))[:120]
    if pack == "core":
        r = 100
        for hw, hj, pos, pron, lv, kind, ss, forms, rel, cat, hom in KR:
            r += 10 + (lv or 4) * 100
            add("krdict", "en", hw, hj, pos, pron, lv, kind, gl(ss),
                {"senses": ss, "related": [{"type": t, "word": w} for t, w in rel], **({"category": cat} if cat else {})}, hom, forms, r)
        gids = {}
        for hw, lv, cat, summ, ss in GR:
            r += 5
            gids[hw] = add("krdict", "en", hw, None, "ending" if "Particles" not in cat else "particle", None, lv, "grammar", summ,
                           {"senses": ss, "related": [], "category": cat}, None, [], 50 + r // 100)
        gs = 0
        for hw, lv, cat, summ, ss in GR:
            gs += 1
            db.execute("INSERT INTO grammar VALUES (?,?,?,?,?,?,?)", (gs, gids[hw], hw, cat, lv, summ, gs))
        for hw, hj, pos, rom, ss, forms, rel, etym in WK:
            add("wikt", "en", hw, hj, pos, None, None, "word", gl(ss),
                {"senses": ss, "related": [{"type": t, "word": w} for t, w in rel], **({"etym": etym} if etym else {})}, None, forms, 5000 + len(hw))
        for hw, hj, pos, g in KE:
            add("kengdic", "en", hw, hj, pos, None, None, "word", g, {"senses": [sense(g)]}, None, [], 9000)
        n = 0
        for ch, (rd, mean, st, rad) in HANJA.items():
            wc = db.execute("SELECT COUNT(*) FROM hanja_words WHERE ch=?", (ch,)).fetchone()[0]
            db.execute("INSERT INTO hanja_chars VALUES (?,?,?,?,?,?)", (ch, rd, mean, st, rad, wc))
        for ko, en, src in SENT:
            n += 1
            db.execute("INSERT INTO sentences VALUES (?,?,?,?)", (n, ko, en, src))
            db.execute("INSERT INTO sentences_fts(rowid, ko) VALUES (?,?)", (n, ko))
        counts.update(sentences=n, grammar=gs, hanja_chars=len(HANJA))
    else:
        for hw, hj, pos, pron, ss, forms, hom in SD:
            add("stdict", "ko", hw, hj, pos, pron, None, "word", None, {"senses": ss, "related": []}, hom, forms, 20000 + len(hw))
    db.executemany("INSERT INTO meta VALUES (?,?)", [("pack", pack), ("version", VERSION), ("built_at", time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())),
        ("counts", json.dumps(counts)), ("sources", json.dumps(["fixture"]))])
    db.commit(); db.execute("VACUUM"); db.close()
    return counts

def pack_file(sqlite_path, pack, required):
    raw = open(sqlite_path, "rb").read()
    buf = io.BytesIO()
    with gzip.GzipFile(fileobj=buf, mode="wb", mtime=0, compresslevel=9) as g: g.write(raw)
    gz = buf.getvalue()
    name = f"{pack}.sqlite.gz"
    chunks = []
    for k, off in enumerate(range(0, len(gz), CHUNK)):
        cn = f"{name}.{k:03d}"
        open(os.path.join(OUT, cn), "wb").write(gz[off:off + CHUNK]); chunks.append(cn)
    return {"id": pack, "required": required, "file": f"{pack}.sqlite", "bytes": len(raw), "gz_bytes": len(gz),
            "sha256": hashlib.sha256(raw).hexdigest(), "chunks": chunks}

def main():
    os.makedirs(OUT, exist_ok=True)
    for f in os.listdir(OUT): os.remove(os.path.join(OUT, f))
    packs = []
    with tempfile.TemporaryDirectory() as tmp:
        for pack, req in (("core", True), ("stdict", False)):
            p = os.path.join(tmp, f"{pack}.sqlite")
            counts = build(p, pack)
            m = pack_file(p, pack, req); m["counts"] = counts; packs.append(m)
    manifest = {"version": VERSION, "packs": packs}
    json.dump(manifest, open(os.path.join(OUT, "manifest.json"), "w"), indent=1)
    open(os.path.join(OUT, "ATTRIBUTION.md"), "w").write("# Attribution (fixture)\n\nSynthetic fixture data for development only.\n")
    print("wrote", OUT, [(p["id"], p["bytes"], p["gz_bytes"], len(p["chunks"])) for p in packs])

if __name__ == "__main__": main()
