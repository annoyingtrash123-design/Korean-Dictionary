# Catalogue pinning report

Source: `pipeline/texts/raw/_discover.json` (GitHub Actions discovery run). Only titles that appear in that file were used.

Counts: kept 52, repinned 29, sections set 4, no match 28 (total 113)

Notes: `uncertain` is a list in the schema, so no-match entries carry `uncertain = [..., "source_title"]` plus a `# discover: no match (...)` comment above `[[text]]`. Section pins for long works exceed ~60 KB where no single subpage fits (hanjungnok 1권 75 KB, pyohaerok 卷01 66 KB, mujeong 1장~20장 129 KB).

| id | decision | chosen title(s) and host | evidence |
|---|---|---|---|
| chunhyangjeon | no match | (unchanged) 열녀춘향수절가 | 열녀춘향수절가 not found; only encyclopedia hits |
| honggildongjeon | repinned | 홍길동전 [ko.wikisource] prefer_edition=경판 | 79799 B; "화셜 됴션국 셰종됴 시졀의 ᄒᆞᆫ ᄌᆡ샹이 이시니 셩은 홍이오 명은 뫼라" |
| simcheongjeon | kept | 심청전 [ko.wikisource] | 138423 B; "## 상권 심쳥젼권지상이라 송나라 말년의 황주 도화동의 ᄒᆞᆫ 사ᄅᆞᆷ이" |
| heungbujeon | no match | (unchanged) 흥부전 | 흥부전 is a versions index; only edition target (경판 25장본) is a 354-byte stub |
| tokkijeon | no match | (unchanged) 토끼전 | 토끼전 is a bare 주부전/수궁전 link page (682 bytes); no text page in results |
| bakssijeon | kept | 박씨전 [ko.wikisource] | 123074 B; "아 동방 션죠ᄃᆡ왕 직위 쵸의 ᄒᆞᆫ양 셩ᄂᆡ의 ᄒᆞᆫ ᄌᆡ상이 잇스되 " |
| guunmong | kept | 구운몽 [ko.wikisource] | 210136 B; "## 상권 ## 원문 구운몽 샹 천하 명산이 다셧시 잇스니 동의는 동악 " |
| sassinamjeonggi | no match | (unchanged) 사씨남정기 | no ko/zh page or relevant search hit |
| hanjungnok | sections set | 한중록 [ko.wikisource] sections=1권 | 481 B; "1권 2권 3권 4권 5권 6권 ## 라이선스" |
| mangbokssajeopogi | kept | 만복사저포기 [ko.wikisource] | 28360 B; "(전라도) 남원에 양생이 살고 있었는데, 일찍이 어버이를 잃은 데다 아직" |
| iseang_gyujangjeon | no match | (unchanged) 이생규장전 | only the zh 金鰲新話 collection (61 KB; heading for this story not shown) - needs a human to pick heading |
| chwiyububerugjeong | no match | (unchanged) 취유부벽정기 | no page found; zh 金鰲新話 collection exists but not surfaced for this work |
| namyeombujuji | no match | (unchanged) 남염부주지 | no page found; zh 金鰲新話 collection exists but not surfaced for this work |
| yonggungbuyeonrok | no match | (unchanged) 용궁부연록 | only the zh 金鰲新話 collection (61 KB; heading for this story not shown) - needs a human to pick heading |
| jeongeupsa | no match | (unchanged) 정읍사 | no 정읍사 page; prefix hits are unrelated 정진사전 |
| cheongsanbyeolgok | kept | 청산별곡 [ko.wikisource] | 2900 B; "살어리 살어리랏다 靑山(쳥산)애 살어리랏다. 멀위랑 ᄃᆞ래랑 먹고 靑山애" |
| gasiri | kept | 가시리 [ko.wikisource] | 1530 B; "가시리 가시리잇고 나ᄂᆞᆫ ᄇᆞ리고 가시리잇고 나ᄂᆞᆫ :위 증즐가 날러" |
| gujiga | kept | 구지가 [ko.wikisource] | 626 B; "거북아 거북아 머리를 내밀어라 내밀지 않으면 구워 먹으리 ## 라이선스" |
| hwangjoga | kept | 황조가 [ko.wikisource] | 705 B; "펄펄 나는 저 꾀꼬리는 암수가 서로 노니는데 외로울사 이내 몸은 뉘와 함" |
| cheoyongga | no match | (unchanged) 처용가 | 처용가 exists but is a 1191-byte license-only stub |
| dansimga | kept | 단심가 [ko.wikisource] | 1250 B; "## 청구영언 ## 가곡원류 (우조 두거) ## 가곡원류 (계면조 두거)" |
| hayeoga | kept | 하여가 [ko.wikisource] | 1021 B; "## 청구영언 ## 가곡원류 ## 해석 이런들 어떠하며 저런들 어떠하리오" |
| giljae_hoegoga | repinned | 회고가 [ko.wikisource] prefer_edition=길재 | 1032 B; "## 청구영언 ## 가곡원류 ## 저작권" |
| seongsammun_sijo | kept | 이 몸이 죽어 가서 [ko.wikisource] | 599 B; "## 청구영언 ## 가곡원류 ## 저작권" |
| hwangjini_byeokgyesu | kept | 청산리 벽계수야 [ko.wikisource] | 989 B; "## 시조 ## 청구영언 ## 가곡원류 ## 한시 靑山裏碧溪水(청산리벽계" |
| hwangjini_dongjitdal | kept | 동짓달 기나긴 밤을 [ko.wikisource] | 824 B; "## 청구영언 ## 가곡원류 (우조 초삭대엽) ## 가곡원류 (여창 우조" |
| isunsin_hansando | kept | 한산섬 달 밝은 밤에 [ko.wikisource] | 604 B; "## 청구영언 ## 가곡원류 ## 저작권" |
| kimsangheon_sijo | kept | 가노라 삼각산아 [ko.wikisource] | 1028 B; "가노라 三角山아 다시 보쟈 漢江水야 故國山川을 ᄯᅥᄂᆞ고쟈 ᄒᆞ랴마ᄂᆞᆫ" |
| ouga | kept | 오우가 -> 산중신곡 [ko.wikisource] | 5656 B; "山산中듕新신曲곡 ## 만흥 山산水슈間간바회아래뛰집을짓노라ᄒᆞ니 그모른ᄂᆞ" |
| eobusasiga | kept | 어부사시사 [ko.wikisource] | 18477 B; "## 春츈 압개예안개것고뒫뫼희ᄒᆡ비췬다 ᄇᆡ떠라ᄇᆡ떠라 밤믈은거의디고낟믈" |
| gwandongbyeolgok | no match | (unchanged) 관동별곡 | 송강가사/관동별곡 found but only a 446-byte header template, no text |
| samimingok | no match | (unchanged) 사미인곡 | 송강가사/사미인곡 found but only a 538-byte header template, no text |
| chuyaujung | repinned | 추야우중 [ko.wikisource] | 743 B; "秋夜雨中 (추야우중) 가을 밤 비 내리는 중에 ## 원문 ## 현대어 가" |
| gyeokhwangsoseo | kept | 檄黃巢書 [zh.wikisource] | 3782 B; "廣明二年七月八日，諸道都統檢校太尉高某告黃巢： 夫守正修常，曰道臨危制變，曰權智" |
| dongmyeongwangpyeon | repinned | 東國李相國全集/卷三 [zh.wikisource] sections=東明王篇 | 28414 B; "## 古律詩 ## 東明王篇幷序 世多說東明王神異之事。雖愚夫騃婦。亦頗能說其事" |
| seulgyeonseol | kept | 蝨犬說 [zh.wikisource] | 999 B; "客有謂予曰：＂昨晚見一不逞男子以大棒子椎遊犬而殺者，勢甚可哀，不能無痛心．自是誓" |
| yangbanjeon | kept | 양반전 -> 연암집/제8권 별집 [ko.wikisource] | 38358 B; "## 自序 友居倫季。 匪厥疎卑。 如土於行。 寄王四時。 親義別叙。 非信奚爲" |
| heosaengjeon | repinned | 열하일기/옥갑야화 [ko.wikisource] | 16045 B; "## 玉匣夜話 行還至玉匣, 與諸裨連牀夜語。 燕京舊時風俗淳厚, 譯輩雖萬金能" |
| mokminsimseo | repinned | 목민심서 [ko.wikisource] sections=서,율기 | 596 B; "서 부임(赴任) 율기(律己) 봉공(奉公) 애민(愛民) 이전(吏典) 호전(" |
| dangun_samgukyusa | kept | 三國遺事/卷第一 [zh.wikisource] | 15749 B; "東晉中宗建虎（丁丑）大興（戊寅）四明帝永昌（壬午）大寧（癸未）三顯宗咸和（丙戌）" |
| ondal_samguksagi | repinned | 三國史記/卷45 [zh.wikisource] | 18414 B (prefix hit under najeon_war); "## 乙巴素 乙巴素，高句麗人也" - chapter holds 온달; heading 溫達 not shown so no sections (unsure) |
| hwawanggye | no match | (unchanged) 三國史記/卷第四十六 | 三國史記/卷第四十六 not found and no 권46 page in results |
| songin | repinned | 송인 [ko.wikisource] | 891 B; "## 원문 ## 현대어 비 갠 긴 둑에는 풀빛이 짙은데 그대 보내는 남포" |
| balhaego_seo | repinned | 발해고/발해고서 [ko.wikisource] | 9552 B; "원문 번역문 遼界全地 及倂東北諸夷 起于唐玄宗癸丑 亡于後唐莊宗丙戌 傳世三十" |
| yeolha_dogangrok | repinned | 열하일기/도강록 [ko.wikisource] alt=熱河日記/卷01 | 75572 B; "## 머리말 起辛未, 止乙酉。 自鴨綠江, 至遼陽。 十五日。 曷爲後三庚子。" |
| yeolha_hojil | no match | (unchanged) 熱河日記/虎叱 | 호질 not a separate page; no hit shows it (it sits inside 관내정사, not evidenced) |
| yeolha_iyagudoha | no match | (unchanged) 熱河日記/一夜九渡河記 | only whole-chapter 열하일기/산장잡기 (ko, 17 KB) shown, heading 一夜九渡河記 not in evidence |
| uisanmundap | repinned | 담헌서/내집 4권/보유/의산문답 [ko.wikisource] | 43754 B; "## 毉山問答 子虛子隱居讀書三十年。竆天地之化。究性命之微。極五行之根。達三敎" |
| hongdaeyong_yeongi | no match | (unchanged) 湛軒燕記 | 담헌서/외집 7-10권 (연기) are ~265-byte stubs; no usable 燕記 text |
| pyohaerok | sections set | 漂海錄 [zh.wikisource] sections=卷01 | 433 B; "卷一 卷二 卷三 分类: 朝鮮典籍 分类: 燕行錄" |
| bukhakui_seo | repinned | 北學議 [zh.wikisource] sections=序 | 121200 B; "## 序 余幼時，慕崔孤雲、趙重峰之爲人，慨然有異世執鞭之願，孤雲，爲唐進士，東" |
| najeon_war | no match | (unchanged) 三國史記/卷第七 | 三國史記/卷第七 not found; no 卷07 page in results |
| euljimundeok_sijo | repinned | 三國史記/卷44 [zh.wikisource] sections=乙支文德 | 18766 B (prefix hit under najeon_war); "## 乙支文德 乙支文德，未詳其世系" |
| seohui_damphan | kept | 高麗史/卷九十四 [zh.wikisource] | 44366 B; "## 徐熙訥、恭 徐熙，小字廉允，內議令弼子也。性嚴恪。光宗十一年，年十八擢甲科" |
| seonjo_myeongwonbyeong | no match | (unchanged) 宣祖實錄 | 宣祖實錄 not found; only 宣祖修正實錄 year pages (different text) or 644 KB year pages |
| injo_samjeondo | no match | (unchanged) 仁祖實錄 | 仁祖實錄 not found; no relevant hit |
| anjunggeun_dongyang | kept | 동양평화론 [ko.wikisource] | 22666 B; "## 서문(序文) 대저 합치면 성공하고 흩어지면 패망한다는 것은 만고에 " |
| hunminjeongeum_haerye | kept | 訓民正音 -> 훈민정음 [ko.wikisource] | 1290 B; "## 라이선스" |
| hunminjeongeum_jeonginji | no match | (unchanged) 訓民正音/解例/鄭麟趾序 | 訓民正音/解例/鄭麟趾序 not found; no 解例 page in results |
| hunminjeongeum_eonhae | no match | (unchanged) 훈민정음 언해 | 훈민정음언해 exists but is a 775-byte license-only stub |
| sejong_sillok_changje | no match | (unchanged) 世宗實錄/卷一百二 | 世宗實錄/卷一百二 not found; only 400+ KB year pages |
| dongnip_sinmun_changgan | no match | (unchanged) 독립신문 창간호 논설 | 독립신문/1896년/4월/7일 found but only a 593-byte header template, no text |
| gimi_declaration | kept | 기미독립선언서 -> 3·1독립선언서 [ko.wikisource] | 22020 B; "## 원문 宣言書 吾等은 玆에 我 朝鮮의 獨立國임과 朝鮮人의 自主民임을 " |
| daehan_declaration | no match | (unchanged) 대한독립선언서 | 대한독립선언서 found but only a 542-byte header template, no text |
| imsi_heonjang | kept | 대한민국 임시헌장 -> 대한민국임시헌장 (제1호) [ko.wikisource] | 7120 B; "## 원문 ## 大韓民國 臨時憲章 宣布文 神人一致로 中外協應하야 漢城에 " |
| joseon_hyeongmyeong | kept | 조선혁명선언 [ko.wikisource] | 23542 B; "## 1 강도 일본이 우리의 국호를 없이 하며, 우리의 정권을 빼앗으며," |
| joseondongnip_ui_seo | repinned | 조선 독립의 서 [ko.wikisource] | 9554 B; "朝鮮獨立에 對한 感想槪要 此書는 獄中에 게신 我代表者가 日人 檢事總長의 " |
| constitution_rok | kept | 대한민국 헌법 -> 대한민국헌법 (제10호) [ko.wikisource] | 51239 B; "## 전문 유구한 역사와 전통에 빛나는 우리 대한국민은 3·1운동으로 건" |
| constitution_1948 | kept | 제헌헌법 -> 대한민국헌법 (제1호) [ko.wikisource] | 27400 B; "## 전문 유구한 역사와 전통에 빛나는 우리들 대한국민은 기미 삼일운동으" |
| udhr_korean | repinned | 세계인권선언 [ko.wikisource] | 12782 B; "## 전문 모든 인류 구성원의 천부의 존엄성과 동등하고 양도할 수 없는 " |
| jindallaekkot | repinned | 진달래꽃 (시집)/진달래꽃 [ko.wikisource] | 1114 B; "현대어 진달래꽃 나 보기가 역겨워 가실 때에는 말없이 고이 보내드리오리다" |
| sanyuhwa | repinned | 진달래꽃 (시집)/산유화 [ko.wikisource] | 1184 B; "현대어 산유화 산에는 꽃 피네 꽃이 피네 갈 봄 여름없이 꽃이 피네 산에" |
| chohon | repinned | 진달래꽃 (시집)/초혼 [ko.wikisource] | 996 B; "산산이 부서진 이름이어! 허공중(虛空中)에 헤어진 이름이어! 불러도 주인" |
| meonhuil | kept | 먼 후일 -> 진달래꽃 (시집)/먼 후일 [ko.wikisource] | 1182 B; "현대어 먼 후일 먼 훗날 당신이 찾으시면 그때에 내 말이 `잊었노라' 당" |
| nimui_chimmuk | repinned | 님의 침묵/님의 침묵 [ko.wikisource] | 2023 B; "## 님의沈默 ## 현대어 임은 갔습니다. 아아, 사랑하는 나의 임은 갔" |
| alsuopseoyo | repinned | 님의 침묵/알 수 없어요 [ko.wikisource] alt=알수 없어요 | 1375 B; "## 알ㅅ수업서요 ## 현대어 바람도 없는 공중에 수직의 파문을 내이며 " |
| narutbaewa_haengin | repinned | 님의 침묵/나룻배와 행인 [ko.wikisource] | 1058 B; "## 나루ㅅ배와行人 ## 현대어 나는 나룻배 당신은 행인(行人) 당신은 " |
| seosi | no match | (unchanged) 서시 | only empty/header-only 하늘과 바람과 별과 시 subpages (402-425 bytes) |
| byeolhenunbam | no match | (unchanged) 별 헤는 밤 | only empty/header-only 하늘과 바람과 별과 시 subpages (290-404 bytes) |
| jahwasang | no match | (unchanged) 자화상 | only empty/header-only 윤동주 subpages (289-402 bytes); other hits are other authors |
| swipge_sseuieojin_si | kept | 쉽게 씌어진 시 [ko.wikisource] | 1058 B; "창밖에 밤비가 속살거려 육첩방(六疊房)은 남의 나라 시인이란 슬픈 천명(" |
| cheongpodo | kept | 청포도 [ko.wikisource] | 819 B; "내 고장 칠월은 청포도가 익어가는 시절 이 마을 전설이 주절이주절이 열리" |
| gwangya | repinned | 광야 [ko.wikisource] prefer_edition=이육사 | 766 B; "까마득한 날에 하늘이 처음 열리고 어데 닭 우는 소리 들렸으랴 모든 산맥" |
| jeoljeong | kept | 절정 [ko.wikisource] | 622 B; "매운 계절(季節)의 채찍에 갈겨 마침내 북방(北方)으로 휩쓸려오다 하늘도" |
| bbaeatgin_deule | kept | 빼앗긴 들에도 봄은 오는가 [ko.wikisource] | 3976 B; "원문 ᄲᅢ앗긴들에도, 봄은오는가 지금은 남의ᄯᅡᆼ―ᄲᅢ앗긴들에도 봄은오는" |
| naui_chimsil | kept | 나의 침실로 [ko.wikisource] | 3098 B; "「마돈나」 지금은 밤도, 모든 목거지에, 다니로라 피곤(疲困)하야 돌아겨" |
| hyangsu | repinned | 향수/향수 [ko.wikisource] | 1296 B; "넓은 벌 동쪽 끝으로 옛이야기 지줄대는 실개천이 회돌아 나가고, 얼룩백이" |
| yurichang | repinned | 향수/유리창1 [ko.wikisource] | 628 B; "유리(琉璃)에 차고 슬픈것이 어린거린다. 열없이 붙어서서 입김을 흐리우니" |
| moran | repinned | 영랑시선/모란이 피기까지는 [ko.wikisource] | 822 B; "모란이 피기까지는 나는 아즉 나의봄을 기둘리고 있을테요 모란이 뚝뚝 떠러" |
| ogamdo | repinned | 오감도 [ko.wikisource] sections=오감도 시제1호 | 31436 B; "## 烏瞰圖 詩第一號 / 오감도 시제1호 조선중앙일보, 1934년 7월 " |
| haeeguseo | kept | 해에게서 소년에게 [ko.wikisource] | 2604 B; "처……ㄹ썩, 처……ㄹ썩, 척, 쏴……아. 때린다 부순다 무너 버린다. 태" |
| unsu_joeun_nal | kept | 운수 좋은 날 [ko.wikisource] | 25123 B; "🙝🙟 새침하게 흐린 품이 눈이 올 듯하더니 눈은 아니 오고 얼다가 만 비" |
| bicheo | kept | 빈처 [ko.wikisource] | 34026 B; "🙝🙟 "그것이 어째 없을까?" 아내가 장문을 열고 무엇을 찾더니 입안말로" |
| dongbaekkkot | kept | 동백꽃 [ko.wikisource] | 16938 B; "오늘도 또 우리 수탉이 막 쫓기었다. 내가 점심을 먹고 나무를 하러 갈 " |
| bombom | repinned | 봄봄 [ko.wikisource] | 27180 B; "“장인님! 인제 저…….” 내가 이렇게 뒤통수를 긁고, 나이가 찼으니 성" |
| memilkkot | kept | 메밀꽃 필 무렵 [ko.wikisource] | 20468 B; "여름장이란 애시당초에 글러서, 해는 아직 중천에 있건만 장판은 벌써 쓸쓸" |
| beongeoriu_samryong | no match | (unchanged) 벙어리 삼룡이 | 벙어리 삼룡이 exists but is a 337-byte license-only stub |
| mullebanga | kept | 물레방아 [ko.wikisource] | 30464 B; "🙝🙟 덜컹덜컹 홈통에 들었다가 다시 쏟아져 흐르는 물이 육중한 물레방아를" |
| gamja | kept | 감자 [ko.wikisource] | 15236 B; "🙝🙟 싸움, 간통, 살인, 도둑, 구걸, 징역, 이 세상의 모든 비극과 " |
| baettaragi | kept | 배따라기 [ko.wikisource] | 30778 B; "🙝🙟 좋은 일기이다. 좋은 일기라도, 하늘에 구름 한 점 없는 - 우리 " |
| talchulgi | kept | 탈출기 [ko.wikisource] | 21553 B; "## 1 김군! 수삼 차 편지는 반갑게 받았다. 그러나 나는 한 번도 회" |
| taepyeongcheonha | sections set | 태평천하 [ko.wikisource] sections=제1장,제15장 | 990 B; "## 목차 윤직원 영감 귀택지도(歸宅之圖) 무임승차 기술 서양국 명창대회" |
| chisuk | kept | 치숙 [ko.wikisource] | 33616 B; "우리 아저씨 말이지요? 아따 저 거시키, 한참 당년에 무엇이냐 그놈의 것" |
| nalgae | kept | 날개 [ko.wikisource] | 54601 B; "‘박제가 되어 버린 천재’를 아시오? 나는 유쾌하오. 이런 때 연애까지가" |
| mujeong | sections set | 무정 [ko.wikisource] sections=1장~20장 | 960 B; "## 목차 1장~20장 21장~40장 41장~60장 61장~80장 81장" |
| bsagam | kept | B사감과 러브레터 [ko.wikisource] | 13215 B; "🙝🙟 C여학교에서 교원 겸 기숙사 사감 노릇을 하는 B여사라면 딱장대요 " |
| nakyeop_taeuumyeo | kept | 낙엽을 태우면서 [ko.wikisource] | 6298 B; "가을이 깊어지면 나는 거의 매일 같이 뜰의 낙엽을 긁어모으지 않으면 안 " |
| gwontae | kept | 권태 [ko.wikisource] | 25770 B; "## 1 어서, 차라리 어두워버리기나 했으면 좋겠는데─벽촌의 여름날은 지" |
| dareul_ssoda | kept | 달을 쏘다 [ko.wikisource] | 4274 B; "번거롭던 사위(四圍)가 잠잠해지고 시계 소리가 또렷하나 보니 밤은 저윽이" |
| sihon | kept | 시혼 [ko.wikisource] | 14573 B; "1 적어도 平凡한 가운데서는 物의 正體를 보지 못하며, 慣習的 行爲에서는" |
| eorini_chanmi | kept | 어린이 찬미 [ko.wikisource] | 11181 B; "어린이가 잠을 잔다. 내 무릎 앞에 편안히 누워서 낮잠을 달게 자고 있다" |
| mannyeon_shatsu | repinned | 만년 셔츠 [ko.wikisource] | 14671 B; "## 1 박물 시간이었다. “이 없는 동물이 무엇인지 아는가?” 선생님이" |
| wangjawa_jebi | repinned | 사랑의 선물/왕자와 제비 [ko.wikisource] | 20362 B; "이른봄, 꽃 피기 전이었습니다. 말랐던 버드나무 가지가 파릇파릇하여질 때" |
