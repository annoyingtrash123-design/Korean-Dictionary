"""FrequencyWords ("word count" per line) -> {word: rank}."""
from __future__ import annotations

# Common verb/adjective endings used to credit a stem for frequent conjugated forms.
ENDINGS = frozenset("""
어 아 여 어요 아요 여요 었 았 였 었어 았어 였어 었어요 았어요 었다 았다 었는데 았는데
고 지 지만 지는 지요 면 으면 서 어서 아서 니 으니 네 네요 자 게 도록 는 은 을 ㄴ ㄹ
는데 은데 을까 을게 을래 을 거 을거 는다 ㄴ다 습니다 ㅂ니다 세요 으세요 십시오 으십시오 려고 으려고
ㄹ 려 러 으러 며 으며 기 음 ㅁ 다 냐 니까 으니까 어도 아도 어야 아야 어라 아라 거나 다가 더니 던 ㄹ까
""".split())


def parse(path):
    """Yield (word, count) in file order (rank 1 first)."""
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            parts = line.strip().rsplit(" ", 1)
            if len(parts) != 2:
                continue
            try:
                yield parts[0], int(parts[1])
            except ValueError:
                continue


def load_ranks(path) -> dict[str, int]:
    ranks: dict[str, int] = {}
    for i, (w, _) in enumerate(parse(path), 1):
        ranks.setdefault(w, i)
    return ranks


def stem_ranks(ranks: dict[str, int]) -> dict[str, int]:
    """For verb/adjective stems: best rank of a frequent form = stem + ending.

    Handles plain stems (먹 + 어요) and 하-contractions (공부하 + 였 -> 공부했)."""
    out: dict[str, int] = {}
    for w, r in ranks.items():
        n = len(w)
        for k in range(1, n):
            stem, rem = w[:k], w[k:]
            if rem in ENDINGS:
                if out.get(stem, 1 << 30) > r:
                    out[stem] = r
            if rem[:1] in ("해", "했") and k >= 1:
                s2 = stem + "하"
                tail = rem[1:]
                if tail == "" or tail in ENDINGS:
                    if out.get(s2, 1 << 30) > r:
                        out[s2] = r
    return out
