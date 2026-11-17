"""Shared helpers for the 속담/성어 frequency study: headword normalisation and the matchers.

Pure stdlib, imported by build_candidates.py, count_corpus.py and the tests. Run with `python3 -I`
(the callers put this directory on sys.path explicitly).

Corpus views
------------
* compact view (for 속담): every run of non-Hangul characters that is not a sentence terminator is
  deleted (spaces, commas, quotes ...), sentence terminators (. ? ! … newline) become a '|' barrier,
  and the result is decomposed to conjoining jamo (NFD).  Spacing variants of the same proverb then
  coincide, and endings can be matched by prefix because 곱+다 / 곱+지 / 곱+고 share the stem; the
  jamo level also lets a stem ending in an open syllable match past-tense forms
  (나무라 ~ 나무랐다: ㅏ + ㅆ).
* spaced view (for 성어): Hangul runs separated by single spaces ('|' for terminators) so a 4-syllable
  token can be checked for a left word boundary and for what follows it in the same eojeol.
"""
import re
import unicodedata

HANGUL = "가-힣"
_NONHANGUL_RUN = re.compile(f"[^{HANGUL}.?!…。\n|]+")
_TERM = re.compile(r"[.?!…。\n|]+")
_MULTI_BARRIER = re.compile(r"\|+")


def compact_view(text):
    """Corpus text -> compact string (Hangul syllables and '|' barriers only)."""
    t = _NONHANGUL_RUN.sub("", text)
    t = _TERM.sub("|", t)
    return t


def spaced_view(text):
    """Corpus text -> Hangul runs separated by one space; '|' marks a sentence barrier."""
    t = _TERM.sub("|", text)
    t = re.sub(f"[^{HANGUL}|]+", " ", t)
    return t


def jamo(s):
    """Hangul syllables -> conjoining jamo (NFD); everything else unchanged."""
    return unicodedata.normalize("NFD", s)


# ---------------------------------------------------------------- headword normalisation (속담)

_HANJA_PAREN = re.compile(r"\([一-鿿-\s]+\)")
_PUA = re.compile(r"[-]")
MAX_VARIANTS = 24


def has_unrenderable(hw):
    """Private-use glyphs (missing characters in the source) make an entry unmatchable."""
    return bool(_PUA.search(hw))


def _expand_brackets(s):
    """'나라[나라님/임금]도' -> ['나라도', '나라님도', '임금도'] (the alternatives replace the token part
    that precedes the bracket; the part of the eojeol after it is shared)."""
    m = re.search(r"\[([^\[\]]*)\]", s)
    if not m:
        return [s]
    head, alts, tail = s[: m.start()], m.group(1), s[m.end():]
    cut = head.rfind(" ") + 1  # the base word is the stretch since the last space
    pre, base = head[:cut], head[cut:]
    out = []
    for a in [base] + [x.strip() for x in alts.split("/") if x.strip()]:
        out.extend(_expand_brackets(pre + a + tail))
        if len(out) > MAX_VARIANTS:
            break
    return out


def _expand_optional(s):
    """'눈(을) 가리고' -> ['눈을 가리고', '눈 가리고'] (full form first)."""
    m = re.search(r"\(([^()]*)\)", s)
    if not m:
        return [s]
    head, opt, tail = s[: m.start()], m.group(1), s[m.end():]
    out = []
    for o in (opt, ""):
        out.extend(_expand_optional(head + o + tail))
        if len(out) > MAX_VARIANTS:
            break
    return out


def _slash_choices(s):
    """'(많이/잘) 먹는다' optional groups with '/' inside parens; also bare 'a/b' inside brackets are
    handled by _expand_brackets.  Parens with '/' become separate alternatives plus the empty one."""
    m = re.search(r"\(([^()]*/[^()]*)\)", s)
    if not m:
        return [s]
    head, opt, tail = s[: m.start()], m.group(1), s[m.end():]
    out = []
    for o in [x.strip() for x in opt.split("/")] + [""]:
        out.extend(_slash_choices(head + o + tail))
        if len(out) > MAX_VARIANTS:
            break
    return out


def strip_key(s):
    """Normalised key: Hangul syllables only (no spaces, punctuation, digits, Latin)."""
    return re.sub(f"[^{HANGUL}]", "", s)


def proverb_variants(headword):
    """All surface variants of a dictionary proverb headword as spaced strings, canonical first.
    Hanja glosses in parentheses are dropped; [alternatives] and (optional parts) are expanded
    (capped at MAX_VARIANTS)."""
    s = _HANJA_PAREN.sub("", headword).strip().rstrip(".")
    s = re.sub(r"\s+", " ", s)
    variants = []
    for a in _slash_choices(s):
        for b in _expand_brackets(a):
            for c in _expand_optional(b):
                variants.append(re.sub(r"\s+", " ", c).strip())
    seen, out = set(), []
    for v in variants:
        k = strip_key(v)
        if k and k not in seen:
            seen.add(k)
            out.append(v)
    return out[:MAX_VARIANTS]


# ---------------------------------------------------------------- stems and partials (속담)

# sentence-final endings that may be stripped to expose the predicate stem, longest first
_FINAL = ["이로다", "이로군", "이라고", "이더라", "이다", "이라", "이요", "이야", "이지", "이니", "이냐",
          "구나", "더라", "로다", "니라", "다", "라", "요", "냐", "랴", "까", "지", "네", "군", "니"]
# tense/aspect/modal syllables that sit between stem and ending
_TENSE = ["었", "았", "였", "겠", "시"]
# connective endings that close a clause (a quoted "first half" ends on one of these)
_CONNECT_STRICT = re.compile(
    r"(?:아야|어야|여야|면|으면|니|으니|니까|으니까|아서|어서|여서|도록|는데|은데|다가|거든|"
    r"듯이|려면|으려면|지만|으나|라도|아도|어도|여도|수록|거나|든지|더니|다면|라면|다고|고|야)$")

MIN_STEM_SYLLABLES = 4     # a stem shorter than this is too generic to count (flagged, not ranked)
MIN_PARTIAL_SYLLABLES = 6  # a quoted first half must be at least this long to be distinctive


def _batchim(ch):
    code = ord(ch) - 0xAC00
    return code % 28 if 0 <= code < 11172 else 0


def _open(ch):
    """Syllable without its final consonant."""
    code = ord(ch) - 0xAC00
    return chr(0xAC00 + code - code % 28)


def predicate_stem(variant):
    """Fixed part of a proverb for the 'full' match: the compact text without its sentence-final ending.
    Returns (stem_compact, had_ending)."""
    c = strip_key(variant)
    if not c:
        return "", False
    for e in _FINAL:
        if c.endswith(e) and len(c) - len(e) >= 3:
            c = c[: -len(e)]
            had = True
            break
    else:
        return c, False
    # 먹는다 / 한다 / 먹었다 / 나무랐다: expose the bare stem so any ending follows
    if c.endswith("는") and e == "다":
        c = c[:-1]
    elif e == "다" and _batchim(c[-1]) == 4:  # ㄴ: 한다, 란다, 된다 -> 하, 라, 되
        c = c[:-1] + _open(c[-1])
    else:
        while c and c[-1] in _TENSE and len(c) > 3:
            c = c[:-1]
        if c and _batchim(c[-1]) == 20 and len(c) > 3:
            # ㅆ: 갔 -> 가 is safe for ㅏ; other vowels are contractions (쳤 = 치+었) we cannot undo, so the
            # syllable is dropped and the stem stays a conservative prefix
            o = _open(c[-1])
            c = c[:-1] + o if (ord(o) - 0xAC00) // 28 % 21 == 0 else c[:-1]
    return c, had


def first_half(variant):
    """The part of a multi-eojeol proverb that people quote alone: everything up to and including the
    first non-final eojeol that carries a connective ending (고와야, 가자니, 보면 ...).  None when the
    proverb has no such split or the half would be too short to be distinctive."""
    words = re.sub(r"[^가-힣 ]", "", variant).split()
    for i, w in enumerate(words[:-1]):
        if i >= 1 and _CONNECT_STRICT.search(w):
            half = "".join(words[: i + 1])
            if len(half) >= MIN_PARTIAL_SYLLABLES:
                return half
            continue
    return None


def proverb_patterns(headword):
    """-> dict(keys=[normalised variant keys], full=[jamo stems], partial=[jamo halves], short=bool).
    `full` stems are matched anywhere in the compact corpus; `partial` halves likewise but are only
    counted where no full match starts at the same position."""
    variants = proverb_variants(headword)
    keys, full, partial = [], [], []
    for v in variants:
        keys.append(strip_key(v))
        stem, _ = predicate_stem(v)
        if stem and len(stem) >= MIN_STEM_SYLLABLES:
            full.append(jamo(stem))
        h = first_half(v)
        if h:
            partial.append(jamo(h))
    full = list(dict.fromkeys(full))
    partial = [p for p in dict.fromkeys(partial) if p not in full]
    return {"keys": keys, "full": full, "partial": partial,
            "short": not full}


# ---------------------------------------------------------------- 성어 matching

# what may follow a 4-syllable 성어 inside its eojeol: particles, copula, 하다 forms, 적 forms
_SEONGEO_TAIL = re.compile(
    r"^(?:"
    r"|[이가은는을를의에도만과와로다요야]|였[가-힣]{0,2}"
    r"|에서|에게|에는|에도|에만|에서는|에서도|으로|으로는|으로도|으로서|으로써|로서|로써|까지|부터|처럼|만큼|보다|밖에|마저|조차|이나|이란|이라|이라고|이라는|이라며|이며|이고|이다|이었|이었다|입니다|인|인데|일|임|이지|이면|이야|"
    r"라고|라는|라며|란|"
    r"하[는고게며여면지기다]|했[가-힣]{0,3}|해[가-힣]{0,3}|한|할|함|하[가-힣]{0,2}|"
    r"적|적인|적으로|적이|적이다|적인데|적으로는|적이고|적임|적인|적이었|"
    r"스[럽러][가-힣]{0,3}"
    r")$")


def seongeo_tail_ok(rest):
    """`rest` = the Hangul that follows the 4 syllables up to the end of the eojeol."""
    return len(rest) <= 6 and bool(_SEONGEO_TAIL.match(rest))


def seongeo_matches(spaced, word):
    """Count occurrences of `word` in a spaced-view string as a token with optional particles /
    하다 / 적 after it.  The left side must be a word boundary (start, space or barrier)."""
    n = 0
    start = 0
    L = len(word)
    while True:
        i = spaced.find(word, start)
        if i < 0:
            return n
        start = i + 1
        if i > 0 and spaced[i - 1] not in " |":
            continue
        j = i + L
        while j < len(spaced) and spaced[j] not in " |":
            j += 1
        if seongeo_tail_ok(spaced[i + L: j]):
            n += 1
