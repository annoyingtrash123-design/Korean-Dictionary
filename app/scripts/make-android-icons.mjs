// Generates the Android launcher icons (legacy + adaptive) and splash images from public/ icons.
// Run from app/: node scripts/make-android-icons.mjs   (writes into android/app/src/main/res)
import sharp from 'sharp';
import { readdirSync, writeFileSync, mkdirSync } from 'node:fs';
import { join } from 'node:path';

const BG = '#294d38';
const res = 'android/app/src/main/res';
const maskable = 'public/icon-maskable-512.png';
const svg = 'public/icon.svg';
const dens = { mdpi: 1, hdpi: 1.5, xhdpi: 2, xxhdpi: 3, xxxhdpi: 4 };

const roundMask = (s) => Buffer.from(`<svg xmlns="http://www.w3.org/2000/svg" width="${s}" height="${s}"><circle cx="${s / 2}" cy="${s / 2}" r="${s / 2}"/></svg>`);

for (const [d, m] of Object.entries(dens)) {
  const dir = join(res, `mipmap-${d}`);
  const legacy = Math.round(48 * m);
  const fg = Math.round(108 * m);
  // legacy square icon: the rounded-corner SVG artwork
  await sharp(svg, { density: 384 }).resize(legacy, legacy).png().toFile(join(dir, 'ic_launcher.png'));
  // legacy round icon: maskable art clipped to a circle
  await sharp(maskable).resize(legacy, legacy)
    .composite([{ input: roundMask(legacy), blend: 'dest-in' }]).png().toFile(join(dir, 'ic_launcher_round.png'));
  // adaptive foreground (108dp canvas, art inside the safe zone), transparent outside
  const inner = Math.round(fg * 0.78);
  await sharp(maskable).resize(inner, inner)
    .extend({ top: Math.floor((fg - inner) / 2), bottom: Math.ceil((fg - inner) / 2), left: Math.floor((fg - inner) / 2), right: Math.ceil((fg - inner) / 2), background: { r: 0, g: 0, b: 0, alpha: 0 } })
    .png().toFile(join(dir, 'ic_launcher_foreground.png'));
}

writeFileSync(join(res, 'values/ic_launcher_background.xml'),
  `<?xml version="1.0" encoding="utf-8"?>\n<resources>\n    <color name="ic_launcher_background">${BG}</color>\n</resources>\n`);

// plain splash: background colour with the icon centred; keep each file's existing size
const rgb = { r: 0x29, g: 0x4d, b: 0x38 };
for (const dir of readdirSync(res).filter((n) => n === 'drawable' || /^drawable-(land|port)-/.test(n))) {
  const file = join(res, dir, 'splash.png');
  let w, h;
  try { ({ width: w, height: h } = await sharp(file).metadata()); } catch { continue; }
  const s = Math.round(Math.min(w, h) * 0.28);
  const icon = await sharp(svg, { density: 384 }).resize(s, s).png().toBuffer();
  await sharp({ create: { width: w, height: h, channels: 3, background: rgb } })
    .composite([{ input: icon, gravity: 'centre' }]).png().toFile(file);
}
console.log('android icons + splash written');
