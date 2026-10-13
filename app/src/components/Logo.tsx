/** The mark: ㅎ as a plant — a bud with two leaves (the top stroke), the earth (bar), the globe
 *  (ㅇ) and a root bulb. Colours follow the theme (`--brand-*` tokens); public/icon.svg is the
 *  same drawing in the fixed palette. */
export function LogoMark({ size = 48 }: { size?: number }) {
  return (
    <svg class="logo-mark" viewBox="8 3 84 96" width={size} height={size} aria-hidden="true" focusable="false">
      <path d="M46.5 30 C35 32.5 24 28 20.5 17 C32.5 14.5 43.5 20 46.5 30 Z" fill="var(--brand-leaf)" />
      <path d="M53.5 30 C65 32.5 76 28 79.5 17 C67.5 14.5 56.5 20 53.5 30 Z" fill="var(--brand-leaf)" />
      <path d="M50 5 C38.5 11 35 21.5 39.5 27.5 C43 31.5 57 31.5 60.5 27.5 C65 21.5 61.5 11 50 5 Z" fill="var(--brand-bud)" />
      <path d="M50 10 C45.5 15 44.5 22.5 47.5 27.5 C48.8 29 51.2 29 52.5 27.5 C55.5 22.5 54.5 15 50 10 Z" fill="var(--brand-bud-2)" />
      <rect x="12" y="33" width="76" height="12" rx="6" fill="var(--brand-earth)" />
      <ellipse cx="50" cy="58.5" rx="28" ry="12.5" fill="var(--brand-globe)" />
      <path d="M50 71.5 C35.5 75 32.5 84.5 38 89.5 C42 93 58 93 62 89.5 C67.5 84.5 64.5 75 50 71.5 Z" fill="var(--brand-earth)" />
      <path d="M44.5 92 Q42 95.5 38.5 96.8 M50 92.8 V97.5 M55.5 92 Q58 95.5 61.5 96.8" stroke="var(--brand-bud)" stroke-width="2.4" fill="none" stroke-linecap="round" />
    </svg>
  );
}
