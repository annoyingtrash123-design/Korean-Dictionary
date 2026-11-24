#!/usr/bin/env python3
"""Give the LOCAL e2e data (app/public/data) a few `seongeo_zh` rows (the "Chinese & English" block of 성어 pages),
so the block can be tested without CC-CEDICT (fetched only in CI; the sandbox cannot reach it) and while every
English equivalent in pipeline/data/seongeo_en_equiv.tsv is still a draft (the pipeline ships only approved ones).
Nothing in the repository is marked approved: the rows exist only in the generated, git-ignored pack.

Sample rows (hanja as the dictionaries spell them):
  過猶不及 (과유불급)  Chinese + English equivalent, simplified differs from traditional
  他山之石 (타산지석)  Chinese only, simplified = traditional, two CEDICT definitions
  塞翁之馬 (새옹지마)  English equivalent only (CEDICT has 塞翁失馬, not an exact match), with a nuance note

A pack whose `seongeo_zh` already has rows (a CI build with CC-CEDICT) is left alone. Rewrites core.sqlite.gz.* and
manifest.json in place and records `counts.seongeo_zh_sample = 1` so the e2e spec knows the rows are these.

  python3 scripts/e2e-seongeo-zh.py [app/public/data]
"""
import importlib.util, json, os, sqlite3, sys

HERE = os.path.dirname(os.path.abspath(__file__))
DATA = sys.argv[1] if len(sys.argv) > 1 else os.path.join(HERE, '..', 'app', 'public', 'data')

spec = importlib.util.spec_from_file_location('e2e_curriculum', os.path.join(HERE, 'e2e-curriculum.py'))
cur = importlib.util.module_from_spec(spec)
sys.argv = [sys.argv[0], DATA]
spec.loader.exec_module(cur)

CREATE = """CREATE TABLE seongeo_zh (
  hanja TEXT PRIMARY KEY, headword TEXT NOT NULL,
  zh_trad TEXT, zh_simp TEXT, zh_pinyin TEXT, zh_en TEXT,
  en_equiv TEXT, en_note TEXT, en_ai INTEGER NOT NULL DEFAULT 0
) WITHOUT ROWID"""

ROWS = [
    ('過猶不及', '과유불급', '過猶不及', '过犹不及', 'guò yóu bù jí',
     ['to go too far is as bad as not going far enough (idiom, from Analects)'], 'too much of a good thing', None, 1),
    ('他山之石', '타산지석', '他山之石', '他山之石', 'tā shān zhī shí',
     ['lit. stones from other hills (may serve to polish jade) (idiom)',
      "fig. advice or criticism from others can help one to correct one's own shortcomings"], None, None, 0),
    ('塞翁之馬', '새옹지마', None, None, None, [], 'a blessing in disguise',
     'fits only one half: 새옹지마 says luck and misfortune keep trading places and cannot be judged in advance', 1),
]


def main() -> None:
    man_path = os.path.join(DATA, 'manifest.json')
    man = json.load(open(man_path))
    core = next(p for p in man['packs'] if p['id'] == 'core')
    counts = core.setdefault('counts', {})
    if counts.get('seongeo_zh_sample') or (counts.get('seongeo_zh') or 0) > 0:
        print('core pack already has seongeo_zh rows: nothing to do')
        return
    core, work, db_path = cur.unpack_core(DATA, man)
    c = sqlite3.connect(db_path)
    if not c.execute("SELECT 1 FROM sqlite_master WHERE name = 'seongeo_zh'").fetchone():
        c.execute(CREATE)
    c.execute('DELETE FROM seongeo_zh')
    for r in ROWS:
        c.execute('INSERT INTO seongeo_zh VALUES (?,?,?,?,?,?,?,?,?)', (*r[:5], json.dumps(r[5], ensure_ascii=False), *r[6:]))
    c.commit()
    c.close()
    counts['seongeo_zh'] = len(ROWS)
    counts['seongeo_zh_sample'] = 1
    print(f'inserted {len(ROWS)} sample seongeo_zh rows (e2e data only)')
    cur.repack_core(DATA, man, core, work, db_path)


if __name__ == '__main__':
    main()
