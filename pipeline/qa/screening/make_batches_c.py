"""Rebuild the one-paste C batches (1,000 sentences, Korean definitions, GLOSS category).
python3 -I pipeline/qa/screening/make_batches_c.py OUT_DIR [C01 C02 ...]   (default: all)"""
import sys, os
D = os.path.dirname(os.path.abspath(__file__)); ROOT = os.path.abspath(os.path.join(D, '..', '..', '..'))
out = sys.argv[1]; want = set(sys.argv[2:])
rows = {}
for l in open(os.path.join(ROOT, 'pipeline/data/example_en.tsv'), encoding='utf-8'):
    if l.strip() and not l.startswith('#'):
        f = l.rstrip('\n').split('\t'); rows[tuple(f[:3])] = f
batches = {}
for l in open(os.path.join(D, 'batches_c1000.idmap.tsv'), encoding='utf-8'):
    if l.startswith('#'): continue
    rid, e, s, x, w, m, kd = l.rstrip('\n').split('\t'); batches.setdefault(rid[:3], []).append((rid, (e, s, x), w, m, kd))
tmpl = open(os.path.join(D, 'prompt_c.txt'), encoding='utf-8').read()
N = len(batches); os.makedirs(out, exist_ok=True)
clean = lambda t: t.replace('\t', ' ').replace('\n', ' ').replace(' | ', ' / ').strip()
for tag, items in sorted(batches.items()):
    if want and tag not in want: continue
    head = tmpl.replace('{TAG}', tag).replace('{N}', f'{N:02d}').replace('{COUNT}', str(len(items))).replace('{FIRST}', items[0][0]).replace('{LAST}', items[-1][0])
    data = '\n'.join(' | '.join([rid, clean(w), clean(m), clean(kd), clean(rows[k][3]), clean(rows[k][4])]) for rid, k, w, m, kd in items)
    open(os.path.join(out, f'Batch_{tag}_of_C{N:02d}.txt'), 'w', encoding='utf-8').write(head + f'=== DATA {tag} ===\n{data}\n=== END OF DATA {tag} ===\n')
print('wrote', len(want) or N, 'batch files to', out)
