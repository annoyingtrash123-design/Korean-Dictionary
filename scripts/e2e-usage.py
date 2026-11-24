#!/usr/bin/env python3
"""Give the LOCAL e2e data (app/public/data) a few "Writing & usage" rows (`writing_expr` and `confusable`), so the two study
lists can be tested while every row of pipeline/data/writing_expressions.tsv and confusables.tsv is still a draft (the
pipeline ships only approved rows, so the real pack has none yet).
Nothing in the repository is marked approved: the rows exist only in the generated, git-ignored pack. They are the REAL
draft text of the data files (the first rows of four 쓰기 groups, six confusable groups), linked to entries the same way the
pipeline does (grammar rows by key, headwords by exact text).

A pack whose tables already have rows (a build with approved data) is left alone. Rewrites core.sqlite.gz.* and manifest.json
in place and records `counts.usage_sample = 1` so the e2e spec knows the rows are these.

  python3 scripts/e2e-usage.py [app/public/data]
"""
import csv, importlib.util, io, json, os, sqlite3, sys

HERE = os.path.dirname(os.path.abspath(__file__))
DATA = sys.argv[1] if len(sys.argv) > 1 else os.path.join(HERE, '..', 'app', 'public', 'data')
PIPE = os.path.join(HERE, '..', 'pipeline', 'data')

spec = importlib.util.spec_from_file_location('e2e_curriculum', os.path.join(HERE, 'e2e-curriculum.py'))
cur = importlib.util.module_from_spec(spec)
sys.argv = [sys.argv[0], DATA]
spec.loader.exec_module(cur)

WRITING_GROUPS = ['Introducing the topic', 'Giving reasons', 'Graphs: figures, shares, comparison (53)', 'Conclusion']
PER_GROUP = 3
CONFUSABLE_WORDS = ['바라다 | 원하다', '다르다 | 틀리다', '반드시 | 반듯이', '바람 | ×바램', '결재 | 결제', '-(으)로서 | -(으)로써']

CREATE = """
CREATE TABLE writing_expr (
  rank INTEGER PRIMARY KEY, id TEXT NOT NULL, grp TEXT NOT NULL, expr TEXT NOT NULL, meaning TEXT NOT NULL,
  register TEXT NOT NULL, ex TEXT NOT NULL, entry_id INTEGER, ai INTEGER NOT NULL DEFAULT 1
);
CREATE TABLE confusable (
  rank INTEGER PRIMARY KEY, id TEXT NOT NULL, kind TEXT NOT NULL, words TEXT NOT NULL,
  explanation TEXT NOT NULL, mistake TEXT NOT NULL, ai INTEGER NOT NULL DEFAULT 1
);"""


def tsv(name):
    lines = [l for l in open(os.path.join(PIPE, name), encoding='utf-8') if not l.startswith('#') and l.strip()]
    return list(csv.DictReader(io.StringIO(''.join(lines)), delimiter='\t', quoting=csv.QUOTE_NONE))


def grammar_key(s):
    """Same as pipeline/src/curriculum.rs grammar_key."""
    out, depth = '', 0
    for ch in s:
        if ch == '(':
            depth += 1
        elif ch == ')':
            depth = max(0, depth - 1)
        elif depth > 0:
            continue
        elif ch in "-–~,  '­" or ch.isdigit():
            continue
        else:
            out += ch
    return out


class Linker:
    def __init__(self, c):
        self.c = c
        self.grammar = {}
        for p, e in c.execute('SELECT pattern, entry_id FROM grammar WHERE entry_id IS NOT NULL ORDER BY id'):
            self.grammar.setdefault(grammar_key(p), e)

    def g(self, pattern):
        k = grammar_key(pattern)
        return self.grammar.get(k) or (self.grammar.get(k[:-1]) if k.endswith('에') else None)

    def w(self, hw):
        r = self.c.execute("SELECT id FROM entries WHERE headword = ? AND kind != 'grammar' ORDER BY (source = 'krdict') DESC, rank LIMIT 1", (hw,)).fetchone()
        return r[0] if r else None

    def link(self, spec):
        kind, _, v = spec.partition(':')
        return self.g(v) if kind == 'g' else self.w(v) if kind == 'w' else None

    def word(self, w):
        return self.g(w) if w.startswith('-') else self.w(w)


def main() -> None:
    man_path = os.path.join(DATA, 'manifest.json')
    man = json.load(open(man_path))
    core = next(p for p in man['packs'] if p['id'] == 'core')
    counts = core.setdefault('counts', {})
    if counts.get('usage_sample') or (counts.get('writing_expr') or 0) > 0 or (counts.get('confusable') or 0) > 0:
        print('core pack already has Writing & usage rows: nothing to do')
        return
    core, work, db_path = cur.unpack_core(DATA, man)
    c = sqlite3.connect(db_path)
    for t in ('writing_expr', 'confusable'):
        c.execute(f'DROP TABLE IF EXISTS {t}')
    c.executescript(CREATE)
    link = Linker(c)

    rows, n = [], {}
    for r in tsv('writing_expressions.tsv'):
        if r['group'] in WRITING_GROUPS and n.get(r['group'], 0) < PER_GROUP:
            n[r['group']] = n.get(r['group'], 0) + 1
            rows.append(r)
    rows.sort(key=lambda r: WRITING_GROUPS.index(r['group']))
    for i, r in enumerate(rows, 1):
        ex = [{'ko': r['ex1_ko'], 'en': r['ex1_en']}, {'ko': r['ex2_ko'], 'en': r['ex2_en']}]
        c.execute('INSERT INTO writing_expr VALUES (?,?,?,?,?,?,?,?,1)',
                  (i, r['id'], r['group'], r['expression'], r['meaning_en'], r['register_note'], json.dumps(ex, ensure_ascii=False), link.link(r['link']) if r['link'] else None))

    conf = {r['words']: r for r in tsv('confusables.tsv')}
    for i, key in enumerate(CONFUSABLE_WORDS, 1):
        r = conf[key]
        words = []
        for w, ko, en in zip(r['words'].split(' | '), r['ex_ko'].split(' | '), r['ex_en'].split(' | ')):
            wrong = w.startswith('×')
            w = w.lstrip('×')
            words.append({'w': w, 'wrong': wrong, 'ko': ko, 'en': en, 'entry': None if wrong else link.word(w)})
        c.execute('INSERT INTO confusable VALUES (?,?,?,?,?,?,1)', (i, r['id'], r['kind'], json.dumps(words, ensure_ascii=False), r['explanation_en'], r['mistake_en']))
    c.commit()
    c.close()
    counts['writing_expr'] = len(rows)
    counts['confusable'] = len(CONFUSABLE_WORDS)
    counts['usage_sample'] = 1
    print(f'inserted {len(rows)} sample writing_expr and {len(CONFUSABLE_WORDS)} confusable rows (e2e data only)')
    cur.repack_core(DATA, man, core, work, db_path)


if __name__ == '__main__':
    main()
