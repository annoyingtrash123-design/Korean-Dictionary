//! Korean deconjugation: from an inflected word / eojeol to plausible dictionary forms.
//!
//! The analyser does a best-first search over "peeling" rules. A node is a remaining string
//! plus a context bit-mask that says what the string is allowed to be:
//!
//! * `EO`   - an 아/어-form (e.g. `가`, `먹어`, `봐`, `추워`) whose stem must be recovered from the
//!            contracted last syllable;
//! * `NOUN` - a bare noun (after particles / copula);
//! * the `PLAIN | LDROP | IR*` bits - a verb/adjective stem, with the stem restorations that are
//!            allowed given the ending that was just removed (e.g. ㄹ-drop is only possible
//!            before ㄴ ㅂ ㅅ ㅁ; ㅂ/ㄷ/ㅅ/르/ㅎ restoration before vowel-initial endings).
//!
//! Every node is also a candidate (a stem + 다, or a noun). The search keeps the cheapest
//! analysis per lemma. Over-generation is expected: the caller filters against the database.

use crate::hangul::*;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

/// A dictionary-form candidate with a short English explanation of how it was derived.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Candidate {
    pub lemma: String,
    pub rule: String,
}

const MAX_CANDIDATES: usize = 40;
const MAX_EXPANSIONS: usize = 4000;
const MAX_DEPTH: usize = 7;
/// Rank credit per input character that has been peeled off.
const REMOVED_CREDIT: i32 = 6;

// ---- context bits -------------------------------------------------------------------------

const PLAIN: u16 = 1;
const LDROP: u16 = 2;
const IRB: u16 = 4;
const IRD: u16 = 8;
const IRS: u16 = 16;
const IRREU: u16 = 32;
const IRH: u16 = 64;
const STEMMASK: u16 = 127;
const NOUN: u16 = 256;
const EO: u16 = 512;

/// Consonant-initial ending: stem unchanged.
const C: u16 = PLAIN;
/// Vowel-initial ending (아/어/으X): all irregular stems possible.
const V: u16 = PLAIN | IRB | IRD | IRS | IRREU | IRH;
/// Ending starting with ㄴ ㅂ ㅅ ㅁ ㄹ / 시: ㄹ-drop (and ㅎ/ㅂ merging) possible.
const L: u16 = PLAIN | LDROP | IRH | IRB;
/// (으)-optional ending after a vowel or ㄹ stem (면, 려고, 니까...).
const M: u16 = PLAIN | IRH | IRB;
const ANY: u16 = 0;

type Step = (&'static str, &'static str); // (description, grammar pattern)

#[derive(Clone)]
struct Node {
    form: Vec<char>,
    ctx: u16,
    cost: i32,
    steps: Vec<Step>, // outermost (last-peeled) first
    notes: Vec<&'static str>,
}

fn ends(f: &[char], suf: &str) -> bool {
    let mut n = suf.chars().count();
    if n > f.len() {
        return false;
    }
    for c in suf.chars().rev() {
        n -= 1;
        if f[f.len() - suf.chars().count() + n] != c {
            return false;
        }
    }
    true
}

fn cat1(p: &[char], x: char) -> Vec<char> {
    let mut v = p.to_vec();
    v.push(x);
    v
}

fn syl(c: char, m: char) -> char {
    syllable(c, m, '\0').unwrap_or('?')
}

fn syl_f(c: char, m: char, f: char) -> char {
    syllable(c, m, f).unwrap_or('?')
}

// ---- tables -------------------------------------------------------------------------------

struct Suf {
    suf: &'static str,
    ctx: u16,
    cost: i32,
    desc: &'static str,
    pat: &'static str,
}

const fn sf(suf: &'static str, ctx: u16, cost: i32, desc: &'static str, pat: &'static str) -> Suf {
    Suf { suf, ctx, cost, desc, pat }
}

/// Endings that attach to a bare stem (suffix stripped from the string).
static SUFFIXES: &[Suf] = &[
    // --- final endings
    sf("습니다", C, 10, "formal polite -습니다", "-ㅂ니다/습니다"),
    sf("습니까", C, 10, "formal polite question -습니까", "-ㅂ니까/습니까"),
    sf("으십시오", C, 10, "formal polite imperative -(으)십시오", "-(으)십시오"),
    sf("읍시다", C, 10, "formal propositive -읍시다 (let's)", "-(으)ㅂ시다"),
    sf("는다", L, 10, "plain present -는다/ㄴ다", "-ㄴ/는다"),
    sf("다", C, 8, "plain -다", "-다"),
    sf("자", C, 10, "propositive -자 (let's)", "-자"),
    sf("지요", C, 10, "-지요 (polite, confirming)", "-지요"),
    sf("죠", C, 10, "-죠 (polite, confirming)", "-지요"),
    sf("네요", C, 10, "-네요 (noticing)", "-네요"),
    sf("네", C, 10, "-네 (noticing)", "-네요"),
    sf("는군요", L, 10, "-는군요 (exclamation)", "-군요/는군요"),
    sf("군요", C, 10, "-군요 (exclamation)", "-군요/는군요"),
    sf("는구나", L, 10, "-는구나 (exclamation)", "-구나/는구나"),
    sf("구나", C, 10, "-구나 (exclamation)", "-구나/는구나"),
    sf("지", C, 10, "-지 (confirming / negation)", "-지"),
    sf("나요", C, 10, "-나요 (polite question)", "-나요"),
    sf("은가요", V, 10, "-(으)ㄴ가요 (polite question)", "-(으)ㄴ가요"),
    sf("냐", C, 10, "-냐 (question)", "-냐"),
    sf("으니", V, 10, "-(으)니 (since / question)", "-(으)니까"),
    sf("니", L, 10, "-(으)니 (since / question)", "-(으)니까"),
    sf("거든요", C, 10, "-거든요 (explaining)", "-거든요"),
    sf("거든", C, 10, "-거든 (if / explaining)", "-거든요"),
    sf("잖아요", C, 10, "-잖아요 (as you know)", "-잖아요"),
    sf("잖아", C, 10, "-잖아 (as you know)", "-잖아요"),
    sf("거라", C, 10, "imperative -거라", "-거라"),
    sf("오", L, 12, "formal -(으)오", "-(으)오"),
    sf("더라", C, 10, "retrospective -더라", "-더라"),
    sf("더니", C, 10, "retrospective -더니", "-더니"),
    sf("더군요", C, 10, "retrospective -더군요", "-더군요"),
    sf("더라도", C, 10, "-더라도 (even if)", "-더라도"),
    sf("던", C, 10, "retrospective adnominal -던", "-던"),
    // --- pre-final endings
    sf("겠", C, 8, "future/guess -겠-", "-겠-"),
    sf("시", L, 8, "honorific -(으)시-", "-(으)시-"),
    sf("았", V, 8, "past -았/었-", "-았/었-"),
    sf("었", V, 8, "past -았/었-", "-았/었-"),
    sf("였", V, 8, "past -였-", "-았/었-"),
    // --- connective endings
    sf("고서", C, 10, "-고서 (after)", "-고서"),
    sf("고", C, 10, "-고 (and / then)", "-고"),
    sf("지만", C, 10, "-지만 (but)", "-지만"),
    sf("는데", L, 10, "-는데 (background / but)", "-는데/(으)ㄴ데"),
    sf("은데", V, 10, "-(으)ㄴ데 (background / but)", "-는데/(으)ㄴ데"),
    sf("도록", C, 10, "-도록 (so that / until)", "-도록"),
    sf("게", C, 10, "-게 (adverbial / so that)", "-게"),
    sf("기에", C, 10, "-기에 (because)", "-기에"),
    sf("기", C, 10, "-기 (nominaliser)", "-기"),
    sf("음", V, 12, "-(으)ㅁ (nominaliser)", "-(으)ㅁ"),
    sf("으면서", V, 10, "-(으)면서 (while)", "-(으)면서"),
    sf("면서", M, 10, "-(으)면서 (while)", "-(으)면서"),
    sf("으면", V, 10, "-(으)면 (if)", "-(으)면"),
    sf("면", M, 10, "-(으)면 (if)", "-(으)면"),
    sf("으니까", V, 10, "-(으)니까 (because)", "-(으)니까"),
    sf("니까", L, 10, "-(으)니까 (because)", "-(으)니까"),
    sf("으려고", V, 10, "-(으)려고 (intending to)", "-(으)려고"),
    sf("려고", M, 10, "-(으)려고 (intending to)", "-(으)려고"),
    sf("으려면", V, 10, "-(으)려면 (if you intend to)", "-(으)려면"),
    sf("려면", M, 10, "-(으)려면 (if you intend to)", "-(으)려면"),
    sf("으러", V, 10, "-(으)러 (in order to go/come)", "-(으)러"),
    sf("러", M, 11, "-(으)러 (in order to go/come)", "-(으)러"),
    sf("으며", V, 10, "-(으)며 (and / while)", "-(으)며"),
    sf("며", M, 10, "-(으)며 (and / while)", "-(으)며"),
    sf("으므로", V, 10, "-(으)므로 (because)", "-(으)므로"),
    sf("므로", M, 10, "-(으)므로 (because)", "-(으)므로"),
    sf("을수록", V, 10, "-(으)ㄹ수록 (the more)", "-(으)ㄹ수록"),
    sf("자마자", C, 10, "-자마자 (as soon as)", "-자마자"),
    sf("다가", C, 10, "-다가 (while / and then)", "-다가"),
    sf("다면", C, 10, "-다면 (if)", "-다면"),
    sf("는다고", L, 10, "quotative -는다고", "-ㄴ/는다고"),
    sf("다고", C, 10, "quotative -다고", "-다고"),
    sf("라고", C, 10, "quotative -라고", "-라고"),
    sf("냐고", C, 10, "quotative -냐고", "-냐고"),
    sf("자고", C, 10, "quotative -자고", "-자고"),
    sf("는다는", L, 10, "quotative adnominal -는다는", "-다는"),
    sf("다는", C, 10, "quotative adnominal -다는", "-다는"),
    sf("는대요", L, 10, "reported -는대요", "-대요"),
    sf("대요", C, 10, "reported -대요", "-대요"),
    sf("래요", C, 10, "reported -래요", "-래요"),
    // --- adnominal
    sf("는지", L, 10, "-는지 (whether)", "-는지/(으)ㄴ지"),
    sf("은지", V, 10, "-(으)ㄴ지 (whether)", "-는지/(으)ㄴ지"),
    sf("는", L, 10, "present adnominal -는", "-는"),
    sf("은", V, 10, "adnominal -(으)ㄴ", "-(으)ㄴ"),
    sf("을", V, 10, "future adnominal -(으)ㄹ", "-(으)ㄹ"),
    // --- (으)ㄹ forms written with 을
    sf("을까요", V, 10, "-(으)ㄹ까요 (shall we / I wonder)", "-(으)ㄹ까요"),
    sf("을까", V, 10, "-(으)ㄹ까 (shall we / I wonder)", "-(으)ㄹ까요"),
    sf("을게요", V, 10, "-(으)ㄹ게요 (I will)", "-(으)ㄹ게요"),
    sf("을게", V, 10, "-(으)ㄹ게 (I will)", "-(으)ㄹ게요"),
    sf("을래요", V, 10, "-(으)ㄹ래요 (I want to)", "-(으)ㄹ래요"),
    sf("을래", V, 10, "-(으)ㄹ래 (I want to)", "-(으)ㄹ래요"),
    sf("을걸", V, 10, "-(으)ㄹ걸 (probably)", "-(으)ㄹ걸"),
    sf("을지", V, 10, "-(으)ㄹ지 (whether)", "-(으)ㄹ지"),
    sf("을거예요", V, 10, "future/guess -(으)ㄹ 거예요", "-(으)ㄹ 거예요"),
    sf("을거야", V, 10, "future/guess -(으)ㄹ 거야", "-(으)ㄹ 거예요"),
    sf("을겁니다", V, 10, "future/guess -(으)ㄹ 겁니다", "-(으)ㄹ 거예요"),
    sf("을텐데", V, 10, "-(으)ㄹ 텐데 (should be, but)", "-(으)ㄹ 텐데"),
    sf("을테니까", V, 10, "-(으)ㄹ 테니까 (since I will)", "-(으)ㄹ 테니까"),
    sf("을때", V, 10, "-(으)ㄹ 때 (when)", "-(으)ㄹ 때"),
    // --- expressions (stem-level)
    sf("고싶", C, 9, "-고 싶다 (want to)", "-고 싶다"),
    sf("고있", C, 9, "-고 있다 (progressive)", "-고 있다"),
    sf("고계시", C, 9, "-고 계시다 (honorific progressive)", "-고 있다"),
    sf("고나", C, 9, "-고 나서 (after doing)", "-고 나서"),
    sf("지않", C, 9, "-지 않다 (negation)", "-지 않다"),
    sf("지못하", C, 9, "-지 못하다 (cannot)", "-지 못하다"),
    sf("지마", C, 9, "-지 마세요 (don't)", "-지 마세요"),
    sf("지말", C, 9, "-지 말다 (don't)", "-지 마세요"),
    sf("게되", C, 9, "-게 되다 (come to be)", "-게 되다"),
    sf("게하", C, 9, "-게 하다 (make someone)", "-게 하다"),
    sf("기로하", C, 9, "-기로 하다 (decide to)", "-기로 하다"),
    sf("을수있", V, 9, "-(으)ㄹ 수 있다 (can)", "-(으)ㄹ 수 있다"),
    sf("을수없", V, 9, "-(으)ㄹ 수 없다 (cannot)", "-(으)ㄹ 수 있다"),
    sf("은적있", V, 9, "-(으)ㄴ 적이 있다 (have done)", "-(으)ㄴ 적이 있다"),
    sf("은적없", V, 9, "-(으)ㄴ 적이 없다 (have never done)", "-(으)ㄴ 적이 있다"),
    sf("으면안되", V, 9, "-(으)면 안 되다 (must not)", "-(으)면 안 되다"),
    sf("면안되", M, 9, "-(으)면 안 되다 (must not)", "-(으)면 안 되다"),
];

struct Join {
    jamo: char,
    rest: &'static str,
    ctx: u16,
    cost: i32,
    desc: &'static str,
    pat: &'static str,
}

const fn jn(jamo: char, rest: &'static str, cost: i32, desc: &'static str, pat: &'static str) -> Join {
    Join { jamo, rest, ctx: L, cost, desc, pat }
}

/// Endings whose first consonant is written as the final consonant of the stem's last syllable
/// (갑니다 = 가 + ㅂ니다, 간 = 가 + ㄴ, 갈 = 가 + ㄹ).
static JOINS: &[Join] = &[
    jn('ㅂ', "니다", 10, "formal polite -ㅂ니다", "-ㅂ니다/습니다"),
    jn('ㅂ', "니까", 10, "formal polite question -ㅂ니까", "-ㅂ니까/습니까"),
    jn('ㅂ', "시다", 10, "formal propositive -ㅂ시다 (let's)", "-(으)ㅂ시다"),
    jn('ㅂ', "시오", 10, "formal imperative -(으)ㅂ시오", "-(으)십시오"),
    jn('ㄴ', "다고", 10, "quotative -ㄴ다고", "-ㄴ/는다고"),
    jn('ㄴ', "다는", 10, "quotative adnominal -ㄴ다는", "-다는"),
    jn('ㄴ', "다", 10, "plain present -ㄴ다", "-ㄴ/는다"),
    jn('ㄴ', "대요", 10, "reported -ㄴ대요", "-대요"),
    jn('ㄴ', "데", 10, "-(으)ㄴ데 (background / but)", "-는데/(으)ㄴ데"),
    jn('ㄴ', "가요", 10, "-(으)ㄴ가요 (polite question)", "-(으)ㄴ가요"),
    jn('ㄴ', "지", 10, "-(으)ㄴ지 (whether)", "-는지/(으)ㄴ지"),
    jn('ㄴ', "적있", 9, "-(으)ㄴ 적이 있다 (have done)", "-(으)ㄴ 적이 있다"),
    jn('ㄴ', "적없", 9, "-(으)ㄴ 적이 없다 (have never done)", "-(으)ㄴ 적이 있다"),
    jn('ㄴ', "", 11, "adnominal -(으)ㄴ", "-(으)ㄴ"),
    jn('ㄹ', "까요", 10, "-(으)ㄹ까요 (shall we / I wonder)", "-(으)ㄹ까요"),
    jn('ㄹ', "까", 10, "-(으)ㄹ까 (shall we / I wonder)", "-(으)ㄹ까요"),
    jn('ㄹ', "게요", 10, "-(으)ㄹ게요 (I will)", "-(으)ㄹ게요"),
    jn('ㄹ', "게", 10, "-(으)ㄹ게 (I will)", "-(으)ㄹ게요"),
    jn('ㄹ', "래요", 10, "-(으)ㄹ래요 (I want to)", "-(으)ㄹ래요"),
    jn('ㄹ', "래", 10, "-(으)ㄹ래 (I want to)", "-(으)ㄹ래요"),
    jn('ㄹ', "걸", 10, "-(으)ㄹ걸 (probably)", "-(으)ㄹ걸"),
    jn('ㄹ', "지", 10, "-(으)ㄹ지 (whether)", "-(으)ㄹ지"),
    jn('ㄹ', "거예요", 10, "future/guess -(으)ㄹ 거예요", "-(으)ㄹ 거예요"),
    jn('ㄹ', "거야", 10, "future/guess -(으)ㄹ 거야", "-(으)ㄹ 거예요"),
    jn('ㄹ', "겁니다", 10, "future/guess -(으)ㄹ 겁니다", "-(으)ㄹ 거예요"),
    jn('ㄹ', "텐데", 10, "-(으)ㄹ 텐데 (should be, but)", "-(으)ㄹ 텐데"),
    jn('ㄹ', "테니까", 10, "-(으)ㄹ 테니까 (since I will)", "-(으)ㄹ 테니까"),
    jn('ㄹ', "때", 10, "-(으)ㄹ 때 (when)", "-(으)ㄹ 때"),
    jn('ㄹ', "수있", 9, "-(으)ㄹ 수 있다 (can)", "-(으)ㄹ 수 있다"),
    jn('ㄹ', "수없", 9, "-(으)ㄹ 수 없다 (cannot)", "-(으)ㄹ 수 있다"),
    jn('ㄹ', "수록", 10, "-(으)ㄹ수록 (the more)", "-(으)ㄹ수록"),
    jn('ㄹ', "", 11, "future adnominal -(으)ㄹ", "-(으)ㄹ"),
    jn('ㅁ', "", 12, "-(으)ㅁ (nominaliser)", "-(으)ㅁ"),
];

#[derive(Clone, Copy, PartialEq)]
enum Cond {
    Any,
    Cons,
    Vowel,
    VowelOrL,
}

struct Part {
    suf: &'static str,
    cond: Cond,
    desc: &'static str,
    pat: &'static str,
}

const fn pt(suf: &'static str, cond: Cond, desc: &'static str, pat: &'static str) -> Part {
    Part { suf, cond, desc, pat }
}

/// Particles and copula endings: strip to get a noun.
static PARTICLES: &[Part] = &[
    pt("이", Cond::Cons, "subject particle 이", "-이/가"),
    pt("가", Cond::Vowel, "subject particle 가", "-이/가"),
    pt("은", Cond::Cons, "topic particle 은", "-은/는"),
    pt("는", Cond::Vowel, "topic particle 는", "-은/는"),
    pt("을", Cond::Cons, "object particle 을", "-을/를"),
    pt("를", Cond::Vowel, "object particle 를", "-을/를"),
    pt("에서", Cond::Any, "location particle 에서 (at / from)", "-에서"),
    pt("에게서", Cond::Any, "particle 에게서 (from a person)", "-에게서/한테서"),
    pt("한테서", Cond::Any, "particle 한테서 (from a person)", "-에게서/한테서"),
    pt("에게", Cond::Any, "particle 에게 (to a person)", "-에게/한테"),
    pt("한테", Cond::Any, "particle 한테 (to a person)", "-에게/한테"),
    pt("께서", Cond::Any, "honorific subject particle 께서", "-께서"),
    pt("께", Cond::Any, "honorific particle 께 (to)", "-께"),
    pt("에", Cond::Any, "particle 에 (at / to)", "-에"),
    pt("으로서", Cond::Cons, "particle 으로서 (as)", "-(으)로서"),
    pt("로서", Cond::VowelOrL, "particle 로서 (as)", "-(으)로서"),
    pt("으로써", Cond::Cons, "particle 으로써 (by means of)", "-(으)로써"),
    pt("로써", Cond::VowelOrL, "particle 로써 (by means of)", "-(으)로써"),
    pt("으로", Cond::Cons, "particle 으로 (toward / by)", "-(으)로"),
    pt("로", Cond::VowelOrL, "particle 로 (toward / by)", "-(으)로"),
    pt("과", Cond::Cons, "particle 과 (and / with)", "-와/과"),
    pt("와", Cond::Vowel, "particle 와 (and / with)", "-와/과"),
    pt("하고", Cond::Any, "particle 하고 (and / with)", "-하고"),
    pt("이랑", Cond::Cons, "particle 이랑 (and / with)", "-(이)랑"),
    pt("랑", Cond::Vowel, "particle 랑 (and / with)", "-(이)랑"),
    pt("이나", Cond::Cons, "particle 이나 (or / as many as)", "-(이)나"),
    pt("나", Cond::Vowel, "particle 나 (or / as many as)", "-(이)나"),
    pt("이라도", Cond::Cons, "particle 이라도 (even / at least)", "-(이)라도"),
    pt("라도", Cond::Vowel, "particle 라도 (even / at least)", "-(이)라도"),
    pt("이라고", Cond::Cons, "quotative 이라고", "-(이)라고"),
    pt("라고", Cond::Vowel, "quotative 라고", "-(이)라고"),
    pt("이란", Cond::Cons, "topic 이란 (what is)", "-(이)란"),
    pt("란", Cond::Vowel, "topic 란 (what is)", "-(이)란"),
    pt("이라서", Cond::Cons, "-이라서 (because it is)", "-(이)라서"),
    pt("라서", Cond::Vowel, "-라서 (because it is)", "-(이)라서"),
    pt("이든", Cond::Cons, "particle 이든 (whether)", "-(이)든"),
    pt("든", Cond::Vowel, "particle 든 (whether)", "-(이)든"),
    pt("도", Cond::Any, "particle 도 (also)", "-도"),
    pt("만", Cond::Any, "particle 만 (only)", "-만"),
    pt("의", Cond::Any, "possessive particle 의", "-의"),
    pt("부터", Cond::Any, "particle 부터 (from / starting)", "-부터"),
    pt("까지", Cond::Any, "particle 까지 (until / even)", "-까지"),
    pt("보다", Cond::Any, "particle 보다 (than)", "-보다"),
    pt("처럼", Cond::Any, "particle 처럼 (like)", "-처럼"),
    pt("같이", Cond::Any, "particle 같이 (like / together)", "-같이"),
    pt("마다", Cond::Any, "particle 마다 (every)", "-마다"),
    pt("조차", Cond::Any, "particle 조차 (even)", "-조차"),
    pt("밖에", Cond::Any, "particle 밖에 (nothing but)", "-밖에"),
    pt("들", Cond::Any, "plural marker 들", "-들"),
    pt("씩", Cond::Any, "distributive 씩 (each)", "-씩"),
    // copula
    pt("이에요", Cond::Any, "copula 이에요 (is)", "-이에요/예요"),
    pt("예요", Cond::Vowel, "copula 예요 (is)", "-이에요/예요"),
    pt("입니다", Cond::Any, "formal copula 입니다 (is)", "-입니다"),
    pt("입니까", Cond::Any, "formal copula question 입니까", "-입니다"),
    pt("이야", Cond::Cons, "casual copula 이야 (is)", "-이야/야"),
    pt("야", Cond::Vowel, "casual copula 야 (is)", "-이야/야"),
    pt("이죠", Cond::Any, "copula 이죠 (it is, right?)", "-이에요/예요"),
];

/// Auxiliary verbs / expressions that sit on an 아/어-form (stem-level suffixes).
/// (suffix, description, pattern, cost)
static AUX_EO: &[(&str, &str, &str, i32)] = &[
    ("주", "-아/어 주다 (do for someone)", "-아/어 주다", 6),
    ("드리", "-아/어 드리다 (do for someone, humble)", "-아/어 주다", 6),
    ("보", "-아/어 보다 (try doing)", "-아/어 보다", 6),
    ("버리", "-아/어 버리다 (do completely)", "-아/어 버리다", 6),
    ("놓", "-아/어 놓다 (do in advance)", "-아/어 놓다", 6),
    ("두", "-아/어 두다 (do in advance)", "-아/어 놓다", 6),
    ("가", "-아/어 가다 (keep on)", "-아/어 가다", 7),
    ("오", "-아/어 오다 (come to)", "-아/어 오다", 7),
    ("내", "-아/어 내다 (do through)", "-아/어 내다", 7),
    ("있", "-아/어 있다 (state)", "-아/어 있다", 7),
    ("계시", "-아/어 계시다 (honorific state)", "-아/어 있다", 7),
    ("야하", "-아/어야 하다 (must)", "-아/어야 하다", 6),
    ("야되", "-아/어야 되다 (must)", "-아/어야 하다", 6),
    ("도되", "-아/어도 되다 (may)", "-아/어도 되다", 6),
    ("도괜찮", "-아/어도 괜찮다 (it's fine to)", "-아/어도 되다", 6),
    ("서는안되", "-아/어서는 안 되다 (must not)", "-아/어서는 안 되다", 6),
];

// ---- node helpers -------------------------------------------------------------------------

fn child(n: &Node, form: Vec<char>, ctx: u16, add: i32, step: Option<Step>, note: Option<&'static str>) -> Node {
    let mut steps = n.steps.clone();
    if let Some(s) = step {
        steps.insert(0, s);
    }
    let mut notes = n.notes.clone();
    if let Some(x) = note {
        notes.push(x);
    }
    Node { form, ctx, cost: n.cost + add, steps, notes }
}

fn cond_ok(prefix_last: char, cond: Cond) -> bool {
    match cond {
        Cond::Any => true,
        Cond::Cons => has_batchim(prefix_last),
        Cond::Vowel => !has_batchim(prefix_last),
        Cond::VowelOrL => !has_batchim(prefix_last) || final_of(prefix_last) == Some('ㄹ'),
    }
}

/// 아/어-form -> candidate stems. `f` is the 해체 form (e.g. 먹어, 가, 봐, 추워, 예뻐, 그래).
fn eo_children(n: &Node, out: &mut Vec<Node>) {
    let f = &n.form;
    let len = f.len();
    if len == 0 {
        return;
    }
    let silent = n.ctx & EO != 0;
    let base = if silent { 0 } else { 16 };
    let step: Option<Step> = if silent { None } else { Some(("informal -아/어", "-아/어")) };
    let l = f[len - 1];
    let p = &f[..len - 1];
    macro_rules! push_ctx {
        ($stem:expr, $ctx:expr, $add:expr, $note:expr, $step:expr) => {{
            let stem: Vec<char> = $stem;
            if !stem.is_empty() {
                out.push(child(n, stem, $ctx, $add + base, $step, $note));
            }
        }};
    }
    macro_rules! push {
        ($stem:expr, $add:expr, $note:expr, $step:expr) => {
            push_ctx!($stem, V, $add, $note, $step)
        };
    }
    if l == '아' || l == '어' {
        if !p.is_empty() {
            push!(p.to_vec(), 0, None, step);
        }
        return;
    }
    if l == '여' {
        if !p.is_empty() {
            push!(p.to_vec(), 0, None, step);
        }
        push!(cat1(p, '이'), 1, None, step);
        return;
    }
    // 러-irregular: 이르러 -> 이르다, 푸르러 -> 푸르다
    if l == '러' && p.last() == Some(&'르') {
        push!(p.to_vec(), 1, Some("러-irregular"), step);
    }
    let d = match decompose(l) {
        Some(d) if d.fin == 0 => d,
        _ => return,
    };
    let c = CHOSEONG[d.initial as usize];
    let m = JUNGSEONG[d.medial as usize];
    let hc = matches!(c, 'ㄱ' | 'ㄹ' | 'ㅁ' | 'ㅇ' | 'ㄸ');
    match m {
        'ㅘ' => {
            if c == 'ㅇ' {
                push!(cat1(p, '오'), 0, None, step);
            } else {
                push!(cat1(p, syl(c, 'ㅗ')), 0, None, step);
                push!(cat1(p, syl_f(c, 'ㅗ', 'ㅎ')), 3, None, step); // 놔 -> 놓
            }
        }
        'ㅝ' => {
            if c == 'ㅇ' {
                push!(cat1(p, '우'), 0, None, step);
            } else {
                push!(cat1(p, syl(c, 'ㅜ')), 0, None, step);
            }
        }
        'ㅙ' => push!(cat1(p, syl(c, 'ㅚ')), 0, None, step),
        'ㅐ' => {
            if c == 'ㅎ' {
                push!(cat1(p, '하'), 0, None, step);
                push!(cat1(p, l), 4, None, step);
            } else {
                push!(cat1(p, l), 1, None, step);
            }
            if hc {
                push!(cat1(p, syl_f(c, 'ㅏ', 'ㅎ')), 3, Some("ㅎ-irregular"), step);
                push!(cat1(p, syl_f(c, 'ㅓ', 'ㅎ')), 3, Some("ㅎ-irregular"), step);
            }
            push!(cat1(p, syl(c, 'ㅓ')), 4, None, step); // 그래 -> 그러다
        }
        'ㅒ' => {
            if hc {
                push!(cat1(p, syl_f(c, 'ㅑ', 'ㅎ')), 2, Some("ㅎ-irregular"), step);
            }
            push!(cat1(p, l), 4, None, step);
        }
        'ㅔ' => {
            if c == 'ㅅ' {
                // (으)세요 = (으)시 + 어요
                push_ctx!(p.to_vec(), L, 0, None, Some(("honorific -(으)세요", "-(으)시-")));
                push!(cat1(p, '시'), 4, None, step);
            } else {
                push!(cat1(p, l), 4, None, step);
            }
            if hc {
                push!(cat1(p, syl_f(c, 'ㅓ', 'ㅎ')), 4, Some("ㅎ-irregular"), step);
            }
        }
        'ㅕ' if c != 'ㅇ' => {
            push!(cat1(p, syl(c, 'ㅣ')), 0, None, step);
            push!(cat1(p, l), 3, None, step);
        }
        'ㅓ' if c != 'ㅇ' => {
            push!(cat1(p, syl(c, 'ㅡ')), 0, Some("으-drop (ㅡ drops before 아/어)"), step);
            push!(cat1(p, l), 1, None, step);
            if c == 'ㅍ' {
                push!(cat1(p, '푸'), 1, Some("우-drop (ㅜ drops before 어)"), step);
            }
        }
        'ㅏ' if c != 'ㅇ' => {
            push!(cat1(p, l), 0, None, step);
            push!(cat1(p, syl(c, 'ㅡ')), 1, Some("으-drop (ㅡ drops before 아/어)"), step);
        }
        _ => {}
    }
}

/// ㅎ-irregular adjectives (빨갛 노랗 파랗 하얗 까맣 그렇 어떻 ...) have a stem syllable with
/// initial ㄱ ㄹ ㅁ ㅇ ㄸ and the vowel ㅏ ㅑ ㅓ before the dropped ㅎ.
fn ha_ok(d: Jamo) -> bool {
    matches!(JUNGSEONG[d.medial as usize], 'ㅏ' | 'ㅑ' | 'ㅓ')
        && matches!(CHOSEONG[d.initial as usize], 'ㄱ' | 'ㄹ' | 'ㅁ' | 'ㅇ' | 'ㄸ')
}

/// Restore irregular stems. Returns (stem, note, cost).
fn restore(f: &[char], mask: u16) -> Vec<(Vec<char>, &'static str, i32)> {
    let mut v = Vec::new();
    let n = f.len();
    if n == 0 {
        return v;
    }
    let last = f[n - 1];
    // 도오/추우 + 다 is rarely a word; the ㅂ-irregular reading (돕다, 춥다) is preferred
    let b_first = mask & IRB != 0
        && (last == '우' || last == '오')
        && n >= 2
        && decompose(f[n - 2]).map_or(false, |pd| pd.fin == 0);
    if mask & PLAIN != 0 {
        v.push((f.to_vec(), "", if b_first { 1 } else { 0 }));
    }
    let d = match decompose(last) {
        Some(d) => d,
        None => return v,
    };
    let with_last = |c: char| -> Vec<char> { cat1(&f[..n - 1], c) };
    if mask & LDROP != 0 && d.fin == 0 {
        if let Some(c) = with_final(last, 'ㄹ') {
            v.push((with_last(c), "ㄹ-irregular (ㄹ drops before ㄴ ㅂ ㅅ)", 2));
        }
    }
    if mask & IRB != 0 && (last == '우' || last == '오') && n >= 2 {
        let prev = f[n - 2];
        if decompose(prev).map_or(false, |pd| pd.fin == 0) {
            if let Some(c) = with_final(prev, 'ㅂ') {
                v.push((cat1(&f[..n - 2], c), "ㅂ-irregular (ㅂ becomes 우)", 0));
            }
        }
    }
    if mask & IRD != 0 && d.fin == 8 {
        if let Some(c) = with_final(last, 'ㄷ') {
            v.push((with_last(c), "ㄷ-irregular (ㄷ becomes ㄹ)", 2));
        }
    }
    if mask & IRS != 0 && d.fin == 0 && matches!(last, '지' | '나' | '이' | '그' | '부' | '저' | '자' | '아') {
        if let Some(c) = with_final(last, 'ㅅ') {
            v.push((with_last(c), "ㅅ-irregular (ㅅ drops)", 3));
        }
    }
    if mask & IRREU != 0 && last == '르' && n >= 2 && final_of(f[n - 2]) == Some('ㄹ') {
        if let Some(c) = without_final(f[n - 2]) {
            let mut s = f[..n - 2].to_vec();
            s.push(c);
            s.push('르');
            v.push((s, "르-irregular (ㄹ doubles)", 1));
        }
    }
    if mask & IRH != 0 && d.fin == 0 && ha_ok(d) {
        if let Some(c) = with_final(last, 'ㅎ') {
            v.push((with_last(c), "ㅎ-irregular (ㅎ drops)", 2));
        }
    }
    v
}

/// Multi-syllable / unambiguous particles: a noun candidate that still ends in one of these
/// can be stripped further, so it is ranked below the fully stripped noun.
fn ends_with_strong_particle(f: &[char]) -> bool {
    const STRONG: &[&str] = &[
        "에서", "에게서", "한테서", "에게", "한테", "께서", "께", "에", "부터", "까지", "보다", "처럼", "같이",
        "마다", "조차", "밖에", "하고",
    ];
    STRONG.iter().any(|p| f.len() > p.chars().count() && ends(f, p))
}

/// Stems that end in an auxiliary expression (먹고싶, 가고있, 먹지않 ...) are not lemmas.
fn ends_with_expression(stem: &[char]) -> bool {
    SUFFIXES.iter().any(|s| s.cost == 9 && stem.len() > s.suf.chars().count() && ends(stem, s.suf))
        || ["수있", "수없", "적있", "적없"].iter().any(|x| ends(stem, x))
}

fn emit(n: &Node, root_len: usize, best: &mut HashMap<String, (i32, usize, String, Vec<&'static str>)>, seq: &mut usize) {
    // analyses that consumed more of the input rank first: partial peelings are usually junk
    let credit = REMOVED_CREDIT * root_len.saturating_sub(n.form.len()) as i32;
    let mut put = |lemma: String, extra: i32, notes: Vec<&'static str>, steps: &[Step]| {
        let cost = n.cost + extra - credit;
        let mut parts: Vec<String> = Vec::new();
        for nt in &notes {
            if !nt.is_empty() {
                parts.push((*nt).to_string());
            }
        }
        for (desc, _) in steps.iter() {
            if !desc.is_empty() {
                parts.push((*desc).to_string());
            }
        }
        let rule = if parts.is_empty() { "dictionary form".to_string() } else { parts.join(" + ") };
        let pats: Vec<&'static str> = steps.iter().map(|s| s.1).filter(|p| !p.is_empty()).collect();
        let e = best.entry(lemma).or_insert((i32::MAX, 0, String::new(), Vec::new()));
        if cost < e.0 {
            *seq += 1;
            *e = (cost, *seq, rule, pats);
        }
    };
    let f = &n.form;
    if f.is_empty() {
        return;
    }
    if n.ctx & NOUN != 0 {
        // a noun that still ends in a clear particle is probably not fully stripped
        let extra = if ends_with_strong_particle(f) { 15 } else { 0 };
        put(f.iter().collect(), extra, n.notes.clone(), &n.steps);
    }
    if n.ctx & STEMMASK != 0 {
        for (stem, note, add) in restore(f, n.ctx & STEMMASK) {
            let last = *stem.last().unwrap();
            if final_of(last) == Some('ㅆ') && last != '있' {
                continue;
            }
            if matches!(last, '으' | '습' | '읍') || !is_hangul_char(last) || ends_with_expression(&stem) {
                continue;
            }
            let mut notes = n.notes.clone();
            if !note.is_empty() {
                notes.push(note);
            }
            let mut lemma: String = stem.iter().collect();
            lemma.push('다');
            let pen = if ends_with_strong_particle(&stem) { 15 } else { 0 };
            put(lemma, add + pen, notes.clone(), &n.steps);
            // 공부하다 / 공부되다 -> also the noun 공부
            if stem.len() >= 2 && (last == '하' || last == '되') && note.is_empty() {
                let noun: String = stem[..stem.len() - 1].iter().collect();
                let mut nn = notes.clone();
                nn.push(if last == '하' { "noun + 하다" } else { "noun + 되다" });
                put(noun, add + 1, nn, &n.steps);
            }
            // 학생이다 -> 학생
            if stem.len() >= 2 && last == '이' && note.is_empty() {
                let noun: String = stem[..stem.len() - 1].iter().collect();
                let mut nn = notes.clone();
                nn.push("noun + copula 이다");
                put(noun, add + 2, nn, &n.steps);
            }
        }
    }
}

/// A dictionary-looking form (ends in 다) is never an informal 아/어-form.
fn last_char_ok_for_root_eo(f: &[char]) -> bool {
    f.last() != Some(&'다')
}

fn expand(n: &Node, is_root: bool, out: &mut Vec<Node>) {
    if n.ctx & EO != 0 {
        eo_children(n, out);
        return;
    }
    let f = &n.form;
    let len = f.len();
    if len == 0 {
        return;
    }
    if is_root && last_char_ok_for_root_eo(f) {
        eo_children(n, out);
    }
    let last = f[len - 1];

    // particles / copula -> noun
    for p in PARTICLES {
        let sl = p.suf.chars().count();
        if len > sl && ends(f, p.suf) && cond_ok(f[len - sl - 1], p.cond) {
            out.push(child(n, f[..len - sl].to_vec(), NOUN, 10, Some((p.desc, p.pat)), None));
        }
    }

    // plain suffixes
    for s in SUFFIXES {
        let sl = s.suf.chars().count();
        if len > sl && ends(f, s.suf) {
            out.push(child(n, f[..len - sl].to_vec(), s.ctx, s.cost, Some((s.desc, s.pat)), None));
        }
    }

    // endings merged into the last syllable's batchim
    for j in JOINS {
        let rl = j.rest.chars().count();
        if len > rl && ends(f, j.rest) {
            let pc = f[len - rl - 1];
            if final_of(pc) == Some(j.jamo) && !(j.jamo == 'ㅂ' && matches!(pc, '습' | '읍')) {
                if let Some(base) = without_final(pc) {
                    out.push(child(n, cat1(&f[..len - rl - 1], base), j.ctx, j.cost, Some((j.desc, j.pat)), None));
                }
            }
        }
    }

    // 아니다 (negative copula): 아니에요 -> 아니
    if ends(f, "아니에요") {
        out.push(child(n, f[..len - 2].to_vec(), C, 10, Some(("polite copula -에요", "-이에요/예요")), None));
    }

    // contracted past: 갔 = 가 + 았, 했 = 하 + 였, 먹었(handled by suffix), 셨 = 시 + 었
    if final_of(last) == Some('ㅆ') && !matches!(last, '있' | '겠' | '았' | '었') {
        if let Some(base) = without_final(last) {
            out.push(child(n, cat1(&f[..len - 1], base), EO, 8, Some(("past -았/었-", "-았/었-")), None));
        }
    }

    // epenthetic 으 (먹으시 -> 먹)
    if last == '으' && len >= 2 && has_batchim(f[len - 2]) {
        out.push(child(n, f[..len - 1].to_vec(), V, 3, None, None));
    }

    // 아/어-taking endings
    for (suf, desc, pat) in [
        ("요", "polite -아요/어요", "-아요/어요"),
        ("서", "-아서/어서 (because / and then)", "-아서/어서"),
        ("도", "-아도/어도 (even if)", "-아도/어도"),
        ("야", "-아야/어야 (only if / must)", "-아야/어야"),
        ("라", "imperative -아라/어라", "-아라/어라"),
    ] {
        if len > 1 && ends(f, suf) {
            out.push(child(n, f[..len - 1].to_vec(), EO, 10, Some((desc, pat)), None));
        }
    }
    if len > 1 && last == '요' {
        // polite 요 after other endings (먹죠요?, 학교요)
        out.push(child(n, f[..len - 1].to_vec(), ANY, 10, Some(("polite -요", "-요")), None));
        out.push(child(n, f[..len - 1].to_vec(), NOUN, 25, Some(("polite -요", "-요")), None));
    }

    // auxiliary verbs on an 아/어-form (stem-level)
    if n.ctx & STEMMASK != 0 {
        for (suf, desc, pat, cost) in AUX_EO {
            let sl = suf.chars().count();
            if len > sl && ends(f, suf) {
                out.push(child(n, f[..len - sl].to_vec(), EO, *cost, Some((desc, pat)), None));
            }
        }
        // -아/어하다 (좋아하다, 싫어하다, 미워하다)
        if last == '하' && len >= 2 {
            if let Some(d) = decompose(f[len - 2]) {
                if d.fin == 0 && matches!(JUNGSEONG[d.medial as usize], 'ㅏ' | 'ㅓ' | 'ㅕ' | 'ㅘ' | 'ㅝ' | 'ㅐ' | 'ㅔ') {
                    out.push(child(n, f[..len - 1].to_vec(), EO, 11, Some(("-아/어하다 (to feel / act …)", "-아/어하다")), None));
                }
            }
        }
    }

    // adverbials: 많이 -> 많다, 조용히 -> 조용하다
    if len > 1 && last == '이' {
        out.push(child(n, f[..len - 1].to_vec(), C, 12, Some(("adverbial -이", "-이")), None));
    }
    if len > 1 && last == '히' {
        out.push(child(n, cat1(&f[..len - 1], '하'), C, 12, Some(("adverbial -히", "-히")), None));
    }
}

struct Analysis {
    lemma: String,
    rule: String,
    pats: Vec<&'static str>,
    cost: i32,
    seq: usize,
}

fn analyze(input: &str) -> Vec<Analysis> {
    let form: Vec<char> = input.chars().filter(|c| is_hangul_char(*c) || is_han(*c)).collect();
    if form.is_empty() || !form.iter().any(|c| is_syllable(*c)) {
        return Vec::new();
    }
    let input_clean: String = form.iter().collect();
    let root_len = form.len();
    let root = Node { form, ctx: ANY, cost: 0, steps: Vec::new(), notes: Vec::new() };
    let mut best: HashMap<String, (i32, usize, String, Vec<&'static str>)> = HashMap::new();
    let mut seq = 0usize;
    let mut arena: Vec<Node> = vec![root];
    let mut heap: BinaryHeap<Reverse<(i32, usize)>> = BinaryHeap::new();
    heap.push(Reverse((0, 0)));
    let mut visited: HashMap<(Vec<char>, u16), i32> = HashMap::new();
    let mut expansions = 0usize;
    while let Some(Reverse((_, idx))) = heap.pop() {
        let node = arena[idx].clone();
        let key = (node.form.clone(), node.ctx);
        if let Some(&c) = visited.get(&key) {
            if c <= node.cost {
                continue;
            }
        }
        visited.insert(key, node.cost);
        emit(&node, root_len, &mut best, &mut seq);
        expansions += 1;
        if expansions > MAX_EXPANSIONS || node.steps.len() >= MAX_DEPTH {
            continue;
        }
        let mut kids = Vec::new();
        expand(&node, idx == 0, &mut kids);
        for k in kids {
            arena.push(k);
            let i = arena.len() - 1;
            heap.push(Reverse((arena[i].cost, i)));
        }
    }
    let mut v: Vec<Analysis> = best
        .into_iter()
        .filter(|(lemma, _)| *lemma != input_clean)
        .map(|(lemma, (cost, seq, rule, pats))| Analysis { lemma, rule, pats, cost, seq })
        .collect();
    v.sort_by(|a, b| (a.cost, a.seq, &a.lemma).cmp(&(b.cost, b.seq, &b.lemma)));
    v.truncate(MAX_CANDIDATES);
    v
}

/// Plausible dictionary forms for an inflected word, most likely first, deduplicated,
/// excluding the input itself, capped at 40.
pub fn deconjugate(input: &str) -> Vec<Candidate> {
    analyze(input).into_iter().map(|a| Candidate { lemma: a.lemma, rule: a.rule }).collect()
}

/// Grammar patterns (as listed in the grammar reference, e.g. `-아서/어서`) that appear in the
/// most plausible analyses of the input.
pub fn grammar_hints(input: &str) -> Vec<String> {
    let mut hints: Vec<String> = Vec::new();
    for a in analyze(input).into_iter().take(3) {
        for p in a.pats {
            if !hints.iter().any(|h| h == p) {
                hints.push(p.to_string());
            }
        }
    }
    hints
}

#[cfg(test)]
mod tests {
    use super::*;

    /// (input, expected lemma) - the lemma must be among the first 5 candidates.
    const CASES: &[(&str, &str)] = &[
        // polite -아요/어요, contractions
        ("먹어요", "먹다"), ("가요", "가다"), ("와요", "오다"), ("봐요", "보다"), ("줘요", "주다"),
        ("해요", "하다"), ("돼요", "되다"), ("마셔요", "마시다"), ("켜요", "켜다"), ("써요", "쓰다"),
        ("커요", "크다"), ("서요", "서다"), ("내요", "내다"), ("자요", "자다"), ("타요", "타다"),
        ("앉아요", "앉다"), ("좋아요", "좋다"), ("살아요", "살다"), ("배워요", "배우다"), ("기다려요", "기다리다"),
        ("가져와요", "가져오다"), ("가져요", "가지다"), ("있어요", "있다"), ("없어요", "없다"), ("싫어요", "싫다"),
        // 으-drop
        ("예뻐요", "예쁘다"), ("아파요", "아프다"), ("바빠요", "바쁘다"), ("나빠요", "나쁘다"), ("기뻐요", "기쁘다"),
        ("슬퍼요", "슬프다"), ("배고파요", "배고프다"),
        // ㅂ irregular
        ("추워요", "춥다"), ("더워요", "덥다"), ("어려워요", "어렵다"), ("쉬워요", "쉽다"), ("매워요", "맵다"),
        ("가까워요", "가깝다"), ("무거워요", "무겁다"), ("도와요", "돕다"), ("고마워요", "고맙다"),
        ("아름다워요", "아름답다"), ("추운", "춥다"), ("추우면", "춥다"), ("도운", "돕다"), ("추웠어요", "춥다"),
        ("도왔어요", "돕다"),
        // ㄷ irregular
        ("들어요", "듣다"), ("들어요", "들다"), ("걸어요", "걷다"), ("물어요", "묻다"), ("들으면", "듣다"),
        ("들은", "듣다"), ("들었어요", "듣다"),
        // ㅅ irregular
        ("지어요", "짓다"), ("나아요", "낫다"), ("이어요", "잇다"), ("지은", "짓다"), ("지었어요", "짓다"),
        // 르 irregular
        ("몰라요", "모르다"), ("불러요", "부르다"), ("빨라요", "빠르다"), ("달라요", "다르다"), ("올라요", "오르다"),
        ("흘러요", "흐르다"), ("잘라요", "자르다"), ("골라요", "고르다"), ("몰랐어요", "모르다"), ("불렀어요", "부르다"),
        // ㅎ irregular
        ("빨개요", "빨갛다"), ("노래요", "노랗다"), ("파래요", "파랗다"), ("하얘요", "하얗다"), ("까매요", "까맣다"),
        ("그래요", "그렇다"), ("이래요", "이렇다"), ("어때요", "어떻다"), ("하얀", "하얗다"), ("빨간", "빨갛다"),
        ("그런", "그렇다"), ("이런", "이렇다"), ("파란", "파랗다"), ("노란", "노랗다"), ("까만", "까맣다"),
        ("어떤", "어떻다"), ("빨가면", "빨갛다"), ("그래서", "그렇다"),
        // ㄹ drop
        ("사는", "살다"), ("만드세요", "만들다"), ("삽니다", "살다"), ("압니다", "알다"), ("아는", "알다"),
        ("만든", "만들다"), ("먼", "멀다"), ("긴", "길다"), ("노는", "놀다"), ("팝니다", "팔다"),
        ("우는", "울다"), ("사세요", "살다"), ("사니까", "살다"), ("살면", "살다"),
        // 러 / 우 irregular
        ("이르러", "이르다"), ("퍼요", "푸다"),
        // past
        ("갔어요", "가다"), ("먹었어요", "먹다"), ("왔어요", "오다"), ("봤어요", "보다"), ("줬어요", "주다"),
        ("했어요", "하다"), ("됐어요", "되다"), ("마셨어요", "마시다"), ("썼어요", "쓰다"), ("예뻤어요", "예쁘다"),
        ("아팠어요", "아프다"), ("좋았어요", "좋다"), ("있었어요", "있다"), ("없었어요", "없다"), ("갔다", "가다"),
        ("먹었다", "먹다"), ("갔던", "가다"), ("먹었는데", "먹다"),
        // formal
        ("갑니다", "가다"), ("먹습니다", "먹다"), ("합니다", "하다"), ("좋습니다", "좋다"), ("감사합니다", "감사하다"),
        ("먹습니까", "먹다"), ("갑니까", "가다"),
        // 하다 nouns
        ("공부했어요", "공부하다"), ("공부했어요", "공부"), ("공부해요", "공부하다"), ("공부하고", "공부하다"),
        ("사랑해요", "사랑하다"), ("공부합니다", "공부"),
        // connectives
        ("먹고", "먹다"), ("가서", "가다"), ("먹어서", "먹다"), ("추워서", "춥다"), ("먹으면", "먹다"),
        ("가면", "가다"), ("먹으니까", "먹다"), ("가니까", "가다"), ("먹지만", "먹다"), ("가는데", "가다"),
        ("먹는데", "먹다"), ("좋은데", "좋다"), ("먹으려고", "먹다"), ("가려고", "가다"), ("먹으면서", "먹다"),
        ("가면서", "가다"), ("먹도록", "먹다"), ("먹게", "먹다"), ("먹기", "먹다"), ("먹음", "먹다"), ("감", "가다"),
        ("먹어도", "먹다"), ("먹어야", "먹다"),
        // adnominals
        ("먹는", "먹다"), ("먹은", "먹다"), ("먹을", "먹다"), ("간", "가다"), ("갈", "가다"), ("먹던", "먹다"),
        ("좋은", "좋다"),
        // sentence endings
        ("먹자", "먹다"), ("먹지", "먹다"), ("먹네", "먹다"), ("먹네요", "먹다"), ("가죠", "가다"), ("먹죠", "먹다"),
        ("가는군요", "가다"), ("좋군요", "좋다"), ("먹구나", "먹다"), ("먹어라", "먹다"), ("갑시다", "가다"),
        ("가십시오", "가다"), ("가세요", "가다"), ("드세요", "드시다"), ("계세요", "계시다"), ("주무세요", "주무시다"),
        ("먹으세요", "먹다"), ("먹으십시오", "먹다"), ("가셨어요", "가다"), ("할게요", "하다"), ("갈까요", "가다"),
        ("먹을까요", "먹다"), ("간대요", "가다"),
        // expressions
        ("먹을 거예요", "먹다"), ("갈 거예요", "가다"), ("먹고 싶어요", "먹다"), ("가고 있어요", "가다"),
        ("먹어 주세요", "먹다"), ("해 주세요", "하다"), ("먹어 봐요", "먹다"), ("먹어야 해요", "먹다"),
        ("가야 해요", "가다"), ("먹을 수 있어요", "먹다"), ("갈 수 없어요", "가다"), ("먹어도 돼요", "먹다"),
        ("먹지 않아요", "먹다"), ("먹지 마세요", "먹다"), ("도와주세요", "돕다"), ("도와주세요", "도와주다"),
        // particles / copula
        ("학교에서", "학교"), ("학교를", "학교"), ("학생이", "학생"), ("친구가", "친구"), ("책을", "책"),
        ("한국에", "한국"), ("친구와", "친구"), ("친구하고", "친구"), ("친구랑", "친구"), ("선생님께", "선생님"),
        ("한국어는", "한국어"), ("집으로", "집"), ("학교로", "학교"), ("서울에서는", "서울"), ("학생이에요", "학생"),
        ("의사예요", "의사"), ("학생입니다", "학생"), ("친구들", "친구"), ("학교까지", "학교"), ("학교처럼", "학교"),
        ("책도", "책"), ("한국의", "한국"), ("학생이었어요", "학생"), ("의사였어요", "의사"), ("학생이다", "학생"),
        ("친구에게", "친구"), ("친구한테", "친구"), ("학교에는", "학교"),
        // adverbs
        ("많이", "많다"), ("조용히", "조용하다"), ("쉽게", "쉽다"), ("크게", "크다"),
    ];

    fn top5(input: &str) -> Vec<String> {
        deconjugate(input).into_iter().take(5).map(|c| c.lemma).collect()
    }

    #[test]
    fn expected_lemma_in_top5() {
        assert!(CASES.len() >= 100, "need at least 100 cases, have {}", CASES.len());
        let mut failures = Vec::new();
        for (input, lemma) in CASES {
            let t = top5(input);
            if !t.iter().any(|l| l == lemma) {
                failures.push(format!("{input} -> {lemma}: got {:?}", deconjugate(input).iter().take(8).map(|c| c.lemma.clone()).collect::<Vec<_>>()));
            }
        }
        assert!(failures.is_empty(), "{} of {} failed:\n{}", failures.len(), CASES.len(), failures.join("\n"));
    }

    #[test]
    fn expected_lemma_anywhere_in_candidates() {
        for (input, lemma) in CASES {
            assert!(deconjugate(input).iter().any(|c| c.lemma == *lemma), "{input} -> {lemma}");
        }
    }

    #[test]
    fn excludes_input_dedupes_and_caps() {
        for (input, _) in CASES {
            let r = deconjugate(input);
            assert!(r.len() <= MAX_CANDIDATES);
            let clean: String = input.chars().filter(|c| !c.is_whitespace()).collect();
            assert!(!r.iter().any(|c| c.lemma == clean), "{input} returned itself");
            let mut seen = std::collections::HashSet::new();
            for c in &r {
                assert!(seen.insert(c.lemma.clone()), "{input}: duplicate {}", c.lemma);
                assert!(!c.rule.is_empty());
            }
        }
    }

    #[test]
    fn non_korean_input_gives_nothing() {
        assert!(deconjugate("").is_empty());
        assert!(deconjugate("school").is_empty());
        assert!(deconjugate("123").is_empty());
    }

    #[test]
    fn rule_explanations() {
        let r = deconjugate("갔어요");
        assert_eq!(r[0].lemma, "가다");
        assert_eq!(r[0].rule, "past -았/었- + polite -아요/어요");
        let r = deconjugate("추워요");
        let c = r.iter().find(|c| c.lemma == "춥다").unwrap();
        assert!(c.rule.contains("ㅂ-irregular"), "{}", c.rule);
        let r = deconjugate("학교에서");
        assert_eq!(r[0].lemma, "학교");
        assert!(r[0].rule.contains("에서"));
    }

    #[test]
    fn grammar_hints_report_patterns() {
        let h = grammar_hints("갔어요");
        assert!(h.contains(&"-았/었-".to_string()), "{h:?}");
        assert!(h.contains(&"-아요/어요".to_string()), "{h:?}");
        let h = grammar_hints("먹으면");
        assert!(h.contains(&"-(으)면".to_string()), "{h:?}");
        let h = grammar_hints("학교에서");
        assert!(h.contains(&"-에서".to_string()), "{h:?}");
        assert!(grammar_hints("school").is_empty());
    }

    #[test]
    fn performance_is_bounded() {
        let t = std::time::Instant::now();
        for _ in 0..3 {
            deconjugate("먹고싶었는데요");
            deconjugate("공부하고있었어요");
        }
        assert!(t.elapsed().as_millis() < 5000, "{:?}", t.elapsed());
    }
}
