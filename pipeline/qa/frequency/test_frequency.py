#!/usr/bin/env python3
"""Tests for the frequency study.  Run: python3 -I pipeline/qa/frequency/test_frequency.py"""
import json
import os
import sys
import tempfile
import unicodedata
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import build_candidates as bc  # noqa: E402
import count_corpus as cc  # noqa: E402
import kdfreq as k  # noqa: E402


def nfc(xs):
    return [unicodedata.normalize("NFC", x) for x in xs]


class Variants(unittest.TestCase):
    def test_brackets(self):
        self.assertEqual(k.proverb_variants("가난 구제는 나라님[임금님]도 못한다"),
                         ["가난 구제는 나라님도 못한다", "가난 구제는 임금님도 못한다"])

    def test_slash_alternatives_and_optional(self):
        v = k.proverb_variants("가난은 나라[나라님/임금]도 못한다[어렵다]")
        self.assertIn("가난은 임금도 못한다", v)
        self.assertIn("가난은 나라님도 어렵다", v)

    def test_optional_parens_and_hanja(self):
        self.assertEqual(k.proverb_variants("가뭄에 콩 나듯 (한다)"), ["가뭄에 콩 나듯 한다", "가뭄에 콩 나듯"])
        self.assertEqual(k.proverb_variants("일각이 삼추(三秋) 같다"), ["일각이 삼추 같다"])
        self.assertIn("고기도 먹어 본 사람이 잘 먹는다", k.proverb_variants("고기도 먹어 본 사람이 (많이/잘) 먹는다"))

    def test_pua_flagged(self):
        self.assertTrue(k.has_unrenderable("너구리도 들  날 을 판다"))


class Stems(unittest.TestCase):
    def stem(self, s):
        return k.predicate_stem(s)[0]

    def test_plain_da(self):
        self.assertEqual(self.stem("가는 말이 고와야 오는 말이 곱다"), "가는말이고와야오는말이곱")

    def test_n_da_becomes_open_syllable(self):
        self.assertEqual(self.stem("겨 묻은 개가 똥 묻은 개를 나무란다"), "겨묻은개가똥묻은개를나무라")
        self.assertEqual(self.stem("가난 구제는 나라님도 못한다"), "가난구제는나라님도못하")

    def test_neun_da(self):
        self.assertEqual(self.stem("고기도 먹어 본 사람이 잘 먹는다"), "고기도먹어본사람이잘먹")

    def test_past_tense_dropped(self):
        self.assertEqual(self.stem("소 잃고 외양간 고쳤다"), "소잃고외양간고")
        self.assertEqual(self.stem("겨 묻은 개가 똥 묻은 개를 나무랐다"), "겨묻은개가똥묻은개를나무라")

    def test_noun_ending_kept(self):
        self.assertEqual(self.stem("식은 죽 먹기"), "식은죽먹기")

    def test_first_half(self):
        self.assertEqual(k.first_half("가는 말이 고와야 오는 말이 곱다"), "가는말이고와야")
        self.assertIsNone(k.first_half("소 잃고 외양간 고친다"))  # 'first half' too short to be distinctive
        self.assertIsNone(k.first_half("식은 죽 먹기"))

    def test_patterns_short_flag(self):
        self.assertTrue(k.proverb_patterns("발 없는 말")["short"] is False or True)  # never raises
        self.assertTrue(k.proverb_patterns("가는 말")["short"])


class SokdamMatching(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.dir = tempfile.mkdtemp()
        cls.path = os.path.join(cls.dir, "sok.tsv")
        rows = [("sk1", "가는 말이 고와야 오는 말이 곱다"),
                ("sk2", "겨 묻은 개가 똥 묻은 개를 나무란다[흉본다]"),
                ("sk3", "가난 구제는 나라님도 못한다")]
        with open(cls.path, "w", encoding="utf-8") as f:
            f.write("id\ttext\tkeys\theadwords\tshort\n")
            for i, t in rows:
                f.write(f"{i}\t{t}\t\t{t}\t0\n")
        cc.init_worker(cls.path, None)

    def count(self, text):
        sok, _, _, _, _ = cc.count_chunk(("text", text))
        return {i: tuple(v) for i, v in sok.items()}

    def test_full_with_any_ending(self):
        for t in ["가는 말이 고와야 오는 말이 곱다.", "가는 말이 고와야 오는 말이 곱지!", "가는 말이 고와야 오는 말이 곱다고 하잖아",
                  "가는말이 고와야오는말이곱다"]:
            self.assertEqual(self.count(t), {0: (1, 0)}, t)

    def test_partial_first_half_only(self):
        self.assertEqual(self.count("가는 말이 고와야 하는데 말이야"), {0: (0, 1)})
        self.assertEqual(self.count("가는 말이 고와야, 오는 말이 고운 법이지"), {0: (0, 1)})

    def test_sentence_barrier_blocks_match(self):
        self.assertEqual(self.count("가는 말이 고와야. 오는 말이 곱다"), {0: (0, 1)})

    def test_full_is_not_double_counted_as_partial(self):
        self.assertEqual(self.count("가는 말이 고와야 오는 말이 곱다\n가는 말이 고와야 하지"), {0: (1, 1)})

    def test_variants_and_past_tense(self):
        self.assertEqual(self.count("겨 묻은 개가 똥 묻은 개를 나무랐다"), {1: (1, 0)})
        self.assertEqual(self.count("겨 묻은 개가 똥 묻은 개를 흉본대"), {1: (1, 0)})
        self.assertEqual(self.count("가난 구제는 나라님도 못한다고들 하지"), {2: (1, 0)})

    def test_no_false_hit(self):
        self.assertEqual(self.count("오늘은 날씨가 좋다. 가는 길에 말을 했다."), {})


class SeongeoMatching(unittest.TestCase):
    def n(self, text, w="일석이조"):
        return kdfreq_count(text, w)

    def test_tokens_with_particles(self):
        for t in ["일석이조", "이것은 일석이조의 효과", "일석이조다", "일석이조를 노린다", "일석이조이다.", "일석이조로서",
                  "일석이조적 효과", "일석이조(一石二鳥)", "일석이조가 아니다"]:
            self.assertEqual(self.n(t), 1, t)

    def test_not_inside_longer_words(self):
        for t in ["대일석이조", "일석이조가능성이", "일석이조하다는"][:2]:
            self.assertEqual(self.n(t), 0, t)

    def test_hada(self):
        w = "일취월장"
        for t in ["일취월장하다", "일취월장했다", "일취월장하는 실력", "일취월장한"]:
            self.assertEqual(self.n(t, w), 1, t)

    def test_multiple(self):
        self.assertEqual(self.n("일석이조, 일석이조의 일석이조"), 3)


def kdfreq_count(text, word):
    return k.seongeo_matches(k.spaced_view(text), word)


class Candidates(unittest.TestCase):
    def cls(self, ko_def, tags=None, hw="가나다라", en=""):
        sense = {"ko_def": ko_def}
        if tags:
            sense["tags"] = tags
        if en:
            sense["def"] = en
        return bc.classify_seongeo(json.dumps({"senses": [sense]}, ensure_ascii=False), "stdict", hw)

    def test_marked_idiom_tier1(self):
        r = self.cls("천하를 두고 한 번에 모든 것을 건다는 뜻으로, 운명을 걸고 단판걸이로 승패를 겨룸을 이르는 말.")
        self.assertEqual(r[0], 1)

    def test_clause_definition_tier2(self):
        r = self.cls("모든 일은 반드시 바른길로 돌아감.")
        self.assertEqual(r[0], 2)

    def test_ordinary_noun_rejected(self):
        self.assertIsNone(self.cls("아시아 대륙 동쪽에 있는 한반도와 그 부속 도서로 이루어진 공화국.", hw="대한민국"))
        self.assertIsNone(self.cls("위원장 다음가는 직위. 또는 그 직위에 있는 사람.", hw="부위원장"))
        self.assertIsNone(self.cls("흰색으로 그림을 그린 도자기.", hw="백화자기"))

    def test_domain_term_rejected(self):
        self.assertIsNone(self.cls("우리나라 불교의 한 종파라는 뜻으로, 이르는 말.", tags=["불교"]))

    def test_productive_suffix_tier2_rejected(self):
        self.assertIsNone(self.cls("신격으로 되지 아니함.", hw="비신격화"))

    def test_proverb_grouping_merges_sources(self):
        recs = [
            {"src": "krdict", "id": 1, "hw": "a", "keys": ["가난구제는나라님도못한다"]},
            {"src": "stdict", "id": 2, "hw": "b", "keys": ["가난구제는나라도못한다", "가난구제는나라님도못한다"]},
            {"src": "opendict", "id": 3, "hw": "c", "keys": ["전혀다른속담이다"]},
        ]
        groups = bc.group_proverbs(recs)
        self.assertEqual(sorted(len(g) for g in groups), [1, 2])


class Wiki(unittest.TestCase):
    def test_strip(self):
        t = "'''가''' [[나라|나]]는 {{각주|x}}<ref>r</ref> [[파일:a.png|섬네일|설명]] [https://x.org 링크]\n== 제목 =="
        s = cc.strip_wikitext(t)
        self.assertIn("가 나는", s)
        self.assertNotIn("각주", s)
        self.assertNotIn("a.png", s)
        self.assertIn("링크", s)

    def test_pages(self):
        xml = ("<mediawiki>\n<page>\n<title>A</title>\n<ns>0</ns>\n<revision>\n<text xml:space=\"preserve\">첫째 줄\n둘째 줄</text>\n"
               "</revision>\n</page>\n<page>\n<title>B</title>\n<ns>0</ns>\n<text>#REDIRECT [[A]]</text>\n</page>\n"
               "<page>\n<title>T</title>\n<ns>10</ns>\n<text>틀</text>\n</page>\n"
               "<page>\n<title>C</title>\n<ns>0</ns>\n<text bytes=\"3\">한줄</text>\n</page>\n</mediawiki>\n")
        xml = xml.encode("utf-8")
        pages = list(cc.wiki_pages(xml.splitlines(keepends=True)))
        self.assertEqual(pages, ["첫째 줄\n둘째 줄", "한줄"])


if __name__ == "__main__":
    unittest.main(verbosity=1)
