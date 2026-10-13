/** The 한 mark: ㅎ as a plant — a bud with two leaves (top stroke), the earth (bar), the globe
 *  (ㅇ) and a root bulb below — with ㅏ and ㄴ as rounded strokes. Colours follow the theme
 *  (`--brand-*` tokens); public/icon.svg is the same drawing in the fixed palette. */
export function LogoMark({ size = 48 }: { size?: number }) {
  return (
    <svg class="logo-mark" viewBox="8 4 82 86" width={size} height={size} aria-hidden="true" focusable="false">
      <path d="M31 23 C23.5 21.5 23 13 31 6.5 C39 13 38.5 21.5 31 23 Z" fill="var(--brand-bud)" />
      <path d="M30.5 23.5 C24 24.5 17.5 21.5 15.5 14.5 C22.5 14 28.5 17 30.5 23.5 Z" fill="var(--brand-leaf)" />
      <path d="M31.5 23.5 C38 24.5 44.5 21.5 46.5 14.5 C39.5 14 33.5 17 31.5 23.5 Z" fill="var(--brand-leaf)" />
      <path d="M12.5 27.6 C22 25.4 40 25.4 49.5 27.6 C51.6 28.1 51.6 31.4 49.5 31.9 C40 34.1 22 34.1 12.5 31.9 C10.4 31.4 10.4 28.1 12.5 27.6 Z" fill="var(--brand-earth)" />
      <ellipse cx="31" cy="41.3" rx="13.5" ry="6.6" fill="var(--brand-stroke)" />
      <path d="M31 49.5 C23.5 52.5 22.5 58.8 25.8 61.6 C28 63.4 34 63.4 36.2 61.6 C39.5 58.8 38.5 52.5 31 49.5 Z" fill="var(--brand-earth)" />
      <path d="M28 62.6 Q26.2 65.6 23.6 66.8 M31 63.2 V68 M34 62.6 Q35.8 65.6 38.4 66.8" stroke="var(--brand-bud)" stroke-width="1.4" fill="none" stroke-linecap="round" />
      <g stroke="var(--brand-stroke)" stroke-width="7.5" fill="none" stroke-linecap="round" stroke-linejoin="round">
      <path d="M64 11 V60 M64 36 H77.5" />
      <path d="M22 74 V85 H81" />
      </g>
    </svg>
  );
}
