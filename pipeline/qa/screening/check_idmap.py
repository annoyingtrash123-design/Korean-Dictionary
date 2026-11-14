"""Guard: every idmap row's meaning/뜻 must be THIS sense's gloss and ko_def, taken from the krdict entry
whose example at (sense, ex) is the sentence itself. Idioms/proverbs share the parent word's ext_id, and
entries.gloss merges all senses — both once leaked into the batches (C01–C03).
python3 -I pipeline/qa/screening/check_idmap.py CORE_SQLITE"""
import json, sqlite3, sys
from pathlib import Path
here = Path(__file__).resolve().parent
by = {}
for e, d in sqlite3.connect(sys.argv[1]).execute("select ext_id, data from entries where source='krdict'"):
    by.setdefault(e, []).append(json.loads(d).get('senses', []))
ko = {}
for l in open(here.parent.parent / 'data/example_en.tsv', encoding='utf-8'):
    if l.strip() and not l.startswith('#'):
        f = l.rstrip('\n').split('\t'); ko[tuple(f[:3])] = f[3]
def meaning(h):
    t = h.get('gloss') or h.get('def', '')
    return t if len(t) <= 120 else t[:119].rstrip() + '…'

bad = []
for l in open(here / 'batches_c1000.idmap.tsv', encoding='utf-8'):
    if l.startswith('#'): continue
    rid, e, s, x, w, m, kd = l.rstrip('\n').split('\t'); s, x = int(s), int(x)
    hit = [ss[s] for ss in by.get(e, []) if s < len(ss) and x < len(ss[s].get('examples', []))
           and ss[s]['examples'][x].get('ko') == ko.get((e, str(s), str(x)))]
    if len(hit) != 1 or hit[0].get('ko_def') != kd or meaning(hit[0]) != m:
        bad.append(rid)
print(f'{len(bad)} bad rows', bad[:20]); sys.exit(1 if bad else 0)
