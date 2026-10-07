// Generates public/icon.svg and PNG icons (needs `sharp`). Run: npm run icons
import sharp from 'sharp';
import { writeFileSync, mkdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const out = join(dirname(fileURLToPath(import.meta.url)), '..', 'public');
mkdirSync(out, { recursive: true });
const svg = (pad = 0, radius = 96) => `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512">
<rect width="512" height="512" rx="${radius}" fill="#2f6f5e"/>
<text x="256" y="${256 + 6}" font-size="${300 - pad}" font-family="Noto Sans CJK KR, Noto Sans KR, Apple SD Gothic Neo, sans-serif" font-weight="700" fill="#faf8f5" text-anchor="middle" dominant-baseline="central">한</text>
</svg>`;
writeFileSync(join(out, 'icon.svg'), svg());
const png = (name, size, s) => sharp(Buffer.from(s)).resize(size, size).png().toFile(join(out, name));
await png('icon-512.png', 512, svg());
await png('icon-192.png', 192, svg());
await png('apple-touch-icon.png', 180, svg(0, 0));      // iOS rounds the corners itself
await png('icon-maskable-512.png', 512, svg(90, 0));    // full-bleed, glyph inside safe zone
console.log('icons written to', out);
