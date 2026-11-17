#!/usr/bin/env python3
"""Give the LOCAL e2e data (app/public/data) a small `curriculum` table, so the home page's Today
carousel can be tested before the pipeline produces the real one (docs/SCOPE.md "curriculum").

Rewrites core.sqlite.gz.* and manifest.json in place (the folder is generated and git-ignored).
Does nothing when the core pack already has a curriculum table (the real one, or an earlier run).

  python3 scripts/e2e-curriculum.py [app/public/data]
"""
import gzip, hashlib, json, os, shutil, sqlite3, sys, tempfile

DATA = sys.argv[1] if len(sys.argv) > 1 else os.path.join(os.path.dirname(__file__), '..', 'app', 'public', 'data')
CHUNK = 20_000_000
SAMPLE_WORDS = 18
SAMPLE_GRAMMAR = 8


def main() -> None:
    man_path = os.path.join(DATA, 'manifest.json')
    man = json.load(open(man_path))
    core = next(p for p in man['packs'] if p['id'] == 'core')
    if (core.get('counts') or {}).get('curriculum'):
        print('core pack already has a curriculum table: nothing to do')
        return
    work = tempfile.mkdtemp(prefix='kd-curriculum-')
    db_path = os.path.join(work, 'core.sqlite')
    with open(db_path, 'wb') as out:
        for ch in core['chunks']:
            name = ch if isinstance(ch, str) else ch.get('file') or ch.get('name')
            with open(os.path.join(DATA, name), 'rb') as f:
                shutil.copyfileobj(f, out)
    # the chunks are one gzip stream cut into pieces
    raw = db_path + '.gz'
    os.rename(db_path, raw)
    with gzip.open(raw, 'rb') as src, open(db_path, 'wb') as dst:
        shutil.copyfileobj(src, dst)
    os.remove(raw)

    c = sqlite3.connect(db_path)
    if c.execute("SELECT 1 FROM sqlite_master WHERE name = 'curriculum'").fetchone():
        print('curriculum table present in the pack already (manifest not updated)')
        return
    c.execute("""CREATE TABLE curriculum (kind TEXT NOT NULL, level INTEGER NOT NULL, entry_id INTEGER, grammar_id INTEGER,
                 form TEXT NOT NULL, rank INTEGER NOT NULL)""")
    # real 고급 krdict words that have a full-sentence example with English, most common first
    words = c.execute("""SELECT id, headword FROM entries WHERE source = 'krdict' AND kind = 'word' AND level = 3
                         AND gloss IS NOT NULL AND gloss != '' AND pos IN ('noun', 'verb', 'adjective')
                         AND data LIKE '%"type":"sentence"%' ORDER BY rank LIMIT ?""", (SAMPLE_WORDS,)).fetchall()
    for i, (eid, hw) in enumerate(words):
        c.execute("INSERT INTO curriculum VALUES ('word', ?, ?, NULL, ?, ?)", (5 + i % 2, eid, hw, i))
    # real grammar rows whose linked entry has examples
    gram = c.execute("""SELECT g.id, g.entry_id, g.pattern FROM grammar g JOIN entries e ON e.id = g.entry_id
                        WHERE g.summary_en IS NOT NULL AND g.summary_en != '' AND e.data LIKE '%"type":"sentence"%'
                        ORDER BY g.level DESC, g.sort LIMIT ?""", (SAMPLE_GRAMMAR,)).fetchall()
    for i, (gid, eid, pat) in enumerate(gram):
        c.execute("INSERT INTO curriculum VALUES ('grammar', ?, ?, ?, ?, ?)", (5 + i % 2, eid, gid, pat, i))
    # an unmatched row, as the real table has
    c.execute("INSERT INTO curriculum VALUES ('word', 5, NULL, NULL, '(unmatched)', 99)")
    c.commit()
    c.close()
    print(f'inserted {len(words)} words + {len(gram)} grammar rows')

    sha = hashlib.sha256()
    size = 0
    with open(db_path, 'rb') as f:
        for blk in iter(lambda: f.read(1 << 20), b''):
            sha.update(blk)
            size += len(blk)
    gz_path = db_path + '.gz'
    with open(db_path, 'rb') as src, gzip.open(gz_path, 'wb', compresslevel=6) as dst:
        shutil.copyfileobj(src, dst)
    for ch in core['chunks']:
        name = ch if isinstance(ch, str) else ch.get('file') or ch.get('name')
        os.remove(os.path.join(DATA, name))
    chunks = []
    with open(gz_path, 'rb') as f:
        n = 0
        while True:
            blk = f.read(CHUNK)
            if not blk:
                break
            name = f'core.sqlite.gz.{n:03d}'
            with open(os.path.join(DATA, name), 'wb') as o:
                o.write(blk)
            chunks.append({'file': name, 'bytes': len(blk)})
            n += 1
    core['chunks'] = chunks
    core['bytes'] = size
    core['gz_bytes'] = os.path.getsize(gz_path)
    core['sha256'] = sha.hexdigest()
    core['version'] = sha.hexdigest()[:16]
    core.setdefault('counts', {})['curriculum'] = len(words) + len(gram) + 1
    json.dump(man, open(man_path, 'w'), ensure_ascii=False, indent=2)
    shutil.rmtree(work, ignore_errors=True)
    print('core pack rewritten:', core['version'], f'{len(chunks)} chunks')


if __name__ == '__main__':
    main()
