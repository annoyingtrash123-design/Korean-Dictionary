"""Rebuild the one-paste screening batches (pipeline/qa/screening/README.md).
python3 -I pipeline/qa/screening/make_batches.py OUT_DIR [B01 B02 ...]   (default: all)"""
import sys, os
D = os.path.dirname(os.path.abspath(__file__)); ROOT = os.path.abspath(os.path.join(D, '..', '..', '..'))
out = sys.argv[1]; want = set(sys.argv[2:])
rows = {}
for l in open(os.path.join(ROOT, 'pipeline/data/example_en.tsv'), encoding='utf-8'):
    if l.strip() and not l.startswith('#'):
        f = l.rstrip('\n').split('\t'); rows[tuple(f[:3])] = f
batches = {}
for l in open(os.path.join(D, 'batches_2000.idmap.tsv'), encoding='utf-8'):
    if l.startswith('#'): continue
    rid, e, s, x, w, m = l.rstrip('\n').split('\t'); batches.setdefault(rid[:3], []).append((rid, (e, s, x), w, m))
head = open(os.path.join(D, 'prompt_template.txt'), encoding='utf-8').read()
N = len(batches); os.makedirs(out, exist_ok=True)
clean = lambda t: t.replace('\t', ' ').replace('\n', ' ').strip()
for tag, items in sorted(batches.items()):
    if want and tag not in want: continue
    h = head.replace('BATCH B01 of B23 (2000 rows, B01-0001 to B01-2000)', f'BATCH {tag} of B{N:02d} ({len(items)} rows, {items[0][0]} to {items[-1][0]})').replace('END B01 (checked N rows)', f'END {tag} (checked N rows)')
    data = '\n'.join(' | '.join([rid, w, m, clean(rows[k][3]), clean(rows[k][4])]) for rid, k, w, m in items)
    open(os.path.join(out, f'Batch_{tag}_of_B{N:02d}.txt'), 'w', encoding='utf-8').write(h + f'=== DATA {tag} ===\n{data}\n=== END OF DATA {tag} ===\n')
print('wrote', len(want) or N, 'batch files to', out)
