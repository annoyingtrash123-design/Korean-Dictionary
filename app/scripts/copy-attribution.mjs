// Bundle the pipeline's ATTRIBUTION.md into the app so the About section works offline.
// Keeps the committed copy when ../pipeline is not available (e.g. a checkout of app/ only).
import { copyFileSync, existsSync, mkdirSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const src = join(root, '..', 'pipeline', 'ATTRIBUTION.md');
const dst = join(root, 'src', 'assets', 'ATTRIBUTION.md');
if (existsSync(src)) { mkdirSync(dirname(dst), { recursive: true }); copyFileSync(src, dst); console.log('attribution: copied from pipeline/'); }
else console.log('attribution: pipeline/ATTRIBUTION.md not found, keeping the committed copy');
