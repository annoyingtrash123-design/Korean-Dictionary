#!/usr/bin/env python3
"""Give the LOCAL e2e data (app/public/data) a `grammar_lesson` table with a few lessons *as if approved*, so
the lesson UI can be tested while every real lesson in pipeline/data/grammar_lessons/ is still a draft
(the pipeline ships only approved ones). Nothing in the repository is marked approved: the rows exist only
in the generated, git-ignored pack. Mirrors pipeline/src/grammar_lessons.rs (body without review, level,
form; compare patterns linked to their grammar entry).

Needs a pack built by the pipeline (real `curriculum` rows). Rewrites core.sqlite.gz.* and manifest.json
in place; a pack that already has `grammar_lesson` is left alone.

  python3 scripts/e2e-lessons.py [app/public/data]
"""
import importlib.util, json, os, re, sqlite3, shutil, sys

HERE = os.path.dirname(os.path.abspath(__file__))
DATA = sys.argv[1] if len(sys.argv) > 1 else os.path.join(HERE, '..', 'app', 'public', 'data')
LESSONS = os.path.join(HERE, '..', 'pipeline', 'data', 'grammar_lessons', '6.json')
# one lesson on a hanppuri entry, and the two homonym rows of one krdict entry (two lessons on one page)
FORMS = ['-노라면', '-거들랑1', '-거들랑2']

spec = importlib.util.spec_from_file_location('e2e_curriculum', os.path.join(HERE, 'e2e-curriculum.py'))
cur = importlib.util.module_from_spec(spec)
sys.argv = [sys.argv[0], DATA]
spec.loader.exec_module(cur)


def grammar_key(s: str) -> str:
    """pipeline/src/curriculum.rs `grammar_key`: no hyphens, spaces, digits, parentheses or commas."""
    s = re.sub(r'\([^)]*\)', '', s)
    return re.sub(r"[-–~,  '­0-9]", '', s)


def lesson_key(form_raw: str) -> str:
    return form_raw.strip().rstrip(', ')


def main() -> None:
    man_path = os.path.join(DATA, 'manifest.json')
    man = json.load(open(man_path))
    core = next(p for p in man['packs'] if p['id'] == 'core')
    counts = core.setdefault('counts', {})
    if counts.get('grammar_lesson'):
        print('core pack already has grammar_lesson: nothing to do')
        return
    core, work, db_path = cur.unpack_core(DATA, man)
    c = sqlite3.connect(db_path)
    exists = c.execute("SELECT 1 FROM sqlite_master WHERE name = 'grammar_lesson'").fetchone()
    if exists and c.execute('SELECT count(*) FROM grammar_lesson').fetchone()[0]:
        counts['grammar_lesson'] = c.execute('SELECT count(*) FROM grammar_lesson').fetchone()[0]
        json.dump(man, open(man_path, 'w'), ensure_ascii=False, indent=2)
        shutil.rmtree(work, ignore_errors=True)
        print('core pack already has lessons')
        return
    lessons = {l['form']: l for l in json.load(open(LESSONS))}
    cur_rows = {}
    for form, level, entry_id, grammar_id in c.execute("SELECT form, level, entry_id, grammar_id FROM curriculum WHERE kind = 'grammar'"):
        cur_rows[(level, lesson_key(form))] = (form, entry_id, grammar_id)
    links = {}
    for pattern, source, eid in c.execute('SELECT g.pattern, e.source, e.id FROM grammar g JOIN entries e ON e.id = g.entry_id ORDER BY g.id DESC'):
        for alt in pattern.split('/'):
            links[grammar_key(alt)] = (source, eid)
    if not exists:   # the pipeline's own table is there but empty while every lesson is a draft
        c.execute('CREATE TABLE grammar_lesson (level INTEGER NOT NULL, form TEXT NOT NULL, entry_id INTEGER, grammar_id INTEGER, body TEXT NOT NULL)')
        c.execute('CREATE INDEX grammar_lesson_entry ON grammar_lesson(entry_id)')
    n = 0
    for f in FORMS:
        row = cur_rows.get((6, f))
        if not row or f not in lessons or not row[1]:
            print('skipped (no curriculum row / entry):', f)
            continue
        body = {k: v for k, v in lessons[f].items() if k not in ('review', 'level', 'form')}
        for cmp in body['compare']:
            link = links.get(grammar_key(cmp['pattern']))
            if link:
                cmp['entry'] = {'source': link[0], 'id': link[1]}
        c.execute('INSERT INTO grammar_lesson VALUES (6, ?, ?, ?, ?)', (row[0], row[1], row[2], json.dumps(body, ensure_ascii=False)))
        n += 1
    c.commit()
    c.close()
    print(f'inserted {n} lessons (as if approved, e2e data only)')
    counts['grammar_lesson'] = n
    cur.repack_core(DATA, man, core, work, db_path)


if __name__ == '__main__':
    main()
