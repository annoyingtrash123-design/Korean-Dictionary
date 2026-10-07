// Library model for the Reader: shelves, periods, themes, filters and timeline grouping.
import type { TextSummary } from '../db/types';

export const SHELVES: { id: string; label: string; ko: string }[] = [
  { id: 'classical-prose', label: 'Classical prose', ko: '고전 산문' },
  { id: 'verse', label: 'Verse', ko: '시가' },
  { id: 'hanmun', label: 'Hanmun', ko: '한문' },
  { id: 'sino-korean', label: 'Sino-Korean relations', ko: '한중 관계' },
  { id: 'documents', label: 'Historical documents', ko: '역사 문서' },
  { id: 'constitution', label: 'Constitution & key documents', ko: '헌법과 주요 문서' },
  { id: 'modern-poetry', label: 'Modern poetry', ko: '현대시' },
  { id: 'modern-fiction', label: 'Modern fiction', ko: '현대 소설' },
  { id: 'essays-children', label: "Essays & children's", ko: '수필·동화' },
  { id: 'news', label: 'News', ko: '뉴스' },
  { id: 'graded', label: 'Graded readers', ko: '단계별 읽기' },
];
export const shelfLabel = (id: string) => SHELVES.find((s) => s.id === id)?.label ?? id;

export interface Period { id: string; label: string; ko: string; from: number; to: number; span: string }
/** Period bands, Gojoseon -> today. `from` inclusive, `to` exclusive. */
export const PERIODS: Period[] = [
  { id: 'ancient', label: 'Ancient', ko: '고조선', from: -Infinity, to: -57, span: 'to 57 BCE' },
  { id: 'three-kingdoms', label: 'Three Kingdoms', ko: '삼국', from: -57, to: 668, span: '57 BCE – 668' },
  { id: 'unified-silla', label: 'Unified Silla', ko: '통일신라', from: 668, to: 918, span: '668 – 935' },
  { id: 'goryeo', label: 'Goryeo', ko: '고려', from: 918, to: 1392, span: '918 – 1392' },
  { id: 'joseon-early', label: 'Early Joseon', ko: '조선 전기', from: 1392, to: 1592, span: '1392 – 1592' },
  { id: 'joseon-late', label: 'Late Joseon', ko: '조선 후기', from: 1592, to: 1897, span: '1592 – 1897' },
  { id: 'korean-empire', label: 'Korean Empire', ko: '대한제국', from: 1897, to: 1910, span: '1897 – 1910' },
  { id: 'colonial', label: 'Colonial era', ko: '일제강점기', from: 1910, to: 1945, span: '1910 – 1945' },
  { id: 'modern', label: 'Modern', ko: '현대', from: 1945, to: Infinity, span: '1945 –' },
];
export const periodOfYear = (y: number): string => PERIODS.find((p) => y >= p.from && y < p.to)!.id;
const PERIOD_ALIASES: Record<string, string> = {
  ancient: 'ancient', gojoseon: 'ancient', 'three kingdoms': 'three-kingdoms', 'three-kingdoms': 'three-kingdoms', 'unified silla': 'unified-silla', 'unified-silla': 'unified-silla',
  goryeo: 'goryeo', 'early joseon': 'joseon-early', 'joseon-early': 'joseon-early', 'late joseon': 'joseon-late', 'joseon-late': 'joseon-late',
  'korean empire': 'korean-empire', 'korean-empire': 'korean-empire', colonial: 'colonial', 'colonial era': 'colonial', modern: 'modern',
};
/** Period id for a text: the pack's `period` (slug or display name; plain "Joseon" is split by year), else derived from `year`. */
export function periodId(t: Pick<TextSummary, 'period' | 'year'>): string | null {
  const k = (t.period ?? '').trim().toLowerCase();
  if (PERIOD_ALIASES[k]) return PERIOD_ALIASES[k];
  if (k === 'joseon') return t.year != null ? periodOfYear(t.year) === 'joseon-early' ? 'joseon-early' : 'joseon-late' : 'joseon-late';
  return t.year != null ? periodOfYear(t.year) : null;
}
export const periodLabel = (id: string | null) => PERIODS.find((p) => p.id === id)?.label ?? '';

export const THEMES: { id: string; label: string }[] = [
  { id: 'ancient-goryeo', label: 'Ancient & Goryeo' },
  { id: 'joseon', label: 'Joseon' },
  { id: 'colonial-independence', label: 'Colonial era & independence' },
  { id: 'modern-korea', label: 'Modern Korea' },
  { id: 'sino-korean', label: 'Sino-Korean relations' },
];
export function themeId(t: string): string | null {
  const k = t.trim().toLowerCase();
  const hit = THEMES.find((x) => x.id === k || x.label.toLowerCase() === k);
  return hit?.id ?? null;
}
export const themesOf = (t: Pick<TextSummary, 'themes'>): string[] => [...new Set((t.themes ?? []).map(themeId).filter((x): x is string => !!x))];

export const LEVELS: { id: string; label: string }[] = [
  { id: 'topik12', label: 'TOPIK 1–2' }, { id: 'topik34', label: 'TOPIK 3–4' }, { id: 'topik56', label: 'TOPIK 5–6' },
  { id: 'advanced', label: 'Advanced' }, { id: 'classical', label: 'Classical' },
];
export function levelId(level: string | null | undefined): string | null {
  if (!level) return null;
  const l = level.toLowerCase();
  const m = /(\d)/.exec(l);
  if (l.includes('topik') && m) { const n = +m[1]; return n <= 2 ? 'topik12' : n <= 4 ? 'topik34' : 'topik56'; }
  if (l.includes('classic')) return 'classical';
  if (l.includes('advanc')) return 'advanced';
  return null;
}
export const levelLabel = (level: string | null | undefined) => level ?? '';

export const LENGTHS: { id: string; label: string; max: number }[] = [
  { id: 'short', label: 'Short (< 1,000 chars)', max: 1000 },
  { id: 'medium', label: 'Medium', max: 5000 },
  { id: 'long', label: 'Long (> 5,000 chars)', max: Infinity },
];
export const lengthId = (chars: number) => LENGTHS.find((l) => chars < l.max)!.id;
export const fmtChars = (n: number) => (n >= 10000 ? `${Math.round(n / 1000)}k chars` : n >= 1000 ? `${(n / 1000).toFixed(1)}k chars` : `${n} chars`);

export const SCRIPTS: { id: string; label: string }[] = [{ id: 'hangul', label: 'Hangul' }, { id: 'hanmun', label: 'Hanmun (hanja)' }, { id: 'mixed', label: 'Mixed' }];

export interface Filters { shelf: string; period: string; theme: string; level: string; length: string; script: string }
export const NO_FILTERS: Filters = { shelf: '', period: '', theme: '', level: '', length: '', script: '' };
export const activeFilterCount = (f: Filters) => Object.values(f).filter(Boolean).length;

export function matches(t: TextSummary, f: Filters): boolean {
  return (!f.shelf || t.shelf === f.shelf)
    && (!f.period || periodId(t) === f.period)
    && (!f.theme || themesOf(t).includes(f.theme))
    && (!f.level || levelId(t.level) === f.level)
    && (!f.length || lengthId(t.chars) === f.length)
    && (!f.script || t.script === f.script);
}

/** Shelves in canonical order with their (filtered) texts; empty shelves are dropped. `all` counts are unfiltered. */
export function byShelf(list: TextSummary[], f: Filters): { shelf: (typeof SHELVES)[number]; texts: TextSummary[]; total: number }[] {
  return SHELVES.map((shelf) => ({ shelf, texts: list.filter((t) => t.shelf === shelf.id && matches(t, f)), total: list.filter((t) => t.shelf === shelf.id).length }))
    .filter((s) => s.texts.length);
}

export function byPeriod(list: TextSummary[], f: Filters): { period: Period; texts: TextSummary[] }[] {
  const out = PERIODS.map((period) => ({
    period,
    texts: list.filter((t) => periodId(t) === period.id && matches(t, f)).sort((a, b) => (a.year ?? 0) - (b.year ?? 0)),
  }));
  return out.filter((p) => p.texts.length);
}

/** "2333 BCE", "1446", or the free-text date when no year. */
export function fmtYear(y: number | null | undefined): string {
  if (y == null) return '';
  return y < 0 ? `${-y} BCE` : y < 1000 ? `${y} CE` : String(y);
}
/** Author · date line for list rows. */
export function byline(t: Pick<TextSummary, 'author_ko' | 'author_en' | 'date' | 'year'>): string {
  const a = t.author_ko || t.author_en || '';
  const d = t.date || fmtYear(t.year);
  return [a, d].filter(Boolean).join(' · ');
}
