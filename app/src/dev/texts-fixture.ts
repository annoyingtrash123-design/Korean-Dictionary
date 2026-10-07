// DEV-ONLY stand-in for the `texts` pack until the engine's listTexts()/getText() ship.
// Enabled with localStorage['kd.fixture.texts']='1' (or opening the app once with ?fixture=texts).
// Built from three real graded readers plus two tiny hand-written PD texts (verse + hanmun) so every layout is exercised.
import dangun from './fixtures/graded-dangun.json';
import sejong from './fixtures/graded-sejong-hangul.json';
import yeonhaengsa from './fixtures/graded-yeonhaengsa.json';
import type { TextDoc, TextSummary } from '../db/types';

const FLAG = 'kd.fixture.texts';
export function fixtureOn(): boolean {
  try {
    if (typeof location !== 'undefined' && /[?&]fixture=texts\b/.test(location.search)) localStorage.setItem(FLAG, '1');
    return localStorage.getItem(FLAG) === '1';
  } catch { return false; }
}

const prov = (d: any) => ({ source: 'original', edition: d.card.edition_en, licence: d.meta.pd_basis });
const verse: TextDoc = {
  id: 'fixture-jindallae', meta: { title_ko: '진달래꽃', title_en: 'Azaleas', author_ko: '김소월', author_en: 'Kim Sowol', author_dates: '1902–1934', date: '1922', year: 1922, period: 'colonial', themes: ['colonial-independence'], shelf: 'modern-poetry', script: 'hangul', excerpt: true },
  card: { summary_ko: '이별의 슬픔을 절제한 목소리로 노래한 시.', summary_en: 'A restrained poem of parting.', level: 'advanced', edition_ko: '위키문헌 (개벽, 1922)', edition_en: 'Wikisource (Gaebyeok, 1922)' },
  notes: { ko: '**배경** 김소월의 대표작입니다.\n\n- 7·5조의 리듬\n- *이별*과 체념', en: '**Background** Kim Sowol\'s best-known poem.\n\n- 7-5 metre\n- *Parting* and resignation' },
  provenance: { source: 'wikisource-ko', url: 'https://ko.wikisource.org/wiki/진달래꽃', licence: 'Public domain (author d. 1934)', edition: 'Wikisource', revision_id: 123456 },
  labels: { notes: 'ai', translation: 'ai', modern: null, text: 'original' }, review: { status: 'approved' }, vocab: null, questions: null,
  paragraphs: [
    { n: 0, orig: '나 보기가 역겨워\n가실 때에는\n말없이 고이 보내 드리오리다', modern: null, reading: null, en: 'If you grow weary of seeing me\nand must leave,\nI will let you go quietly, without a word.' },
    { n: 1, orig: '영변에 약산\n진달래꽃\n아름 따다 가실 길에 뿌리오리다', modern: null, reading: null, en: 'From Yaksan hill in Yeongbyeon\nI will gather armfuls of azaleas\nand strew them along your path.' },
    { n: 2, orig: '가시는 걸음걸음\n놓인 그 꽃을\n사뿐히 즈려밟고 가시옵소서', modern: null, reading: null, en: 'Step by step as you go,\ntread lightly on the flowers laid there.' },
  ],
};
const hanmun: TextDoc = {
  id: 'fixture-chuya', meta: { title_ko: '추야우중', title_en: 'On an Autumn Night in the Rain', author_ko: '최치원', author_en: 'Choe Chiwon', author_dates: '857–?', date: 'c. 880s', year: 885, period: 'unified-silla', themes: ['ancient-goryeo', 'sino-korean'], shelf: 'hanmun', script: 'hanmun', excerpt: false },
  card: { summary_ko: '당나라에서 지은 오언절구.', summary_en: 'A five-character quatrain written in Tang China.', level: 'classical', edition_ko: '위키문헌', edition_en: 'Wikisource' },
  notes: { ko: '**배경** 최치원이 당나라 유학 중에 지은 시입니다.', en: '**Background** Written while Choe Chiwon studied in Tang China.' },
  provenance: { source: 'wikisource-zh', url: 'https://zh.wikisource.org/', licence: 'Public domain' },
  labels: { notes: 'ai', translation: 'ai', modern: null, text: 'original' }, review: { status: 'approved' }, vocab: null, questions: null,
  paragraphs: [
    { n: 0, orig: '秋風唯苦吟\n世路少知音\n窓外三更雨\n燈前萬里心', modern: null, reading: '추풍유고음\n세로소지음\n창외삼경우\n등전만리심', en: 'In the autumn wind I only chant in pain;\nfew in this world know my voice.\nOutside the window, rain at the third watch;\nbefore the lamp, my heart ten thousand miles away.' },
  ],
};

const docs: TextDoc[] = [dangun, sejong, yeonhaengsa].map((d: any): TextDoc => ({
  id: d.id, meta: d.meta, card: d.card, notes: d.notes, labels: d.labels, review: d.review, vocab: d.vocab, questions: d.questions, provenance: prov(d),
  paragraphs: d.paragraphs.map((p: any, n: number) => ({ n, ...p })),
})).concat([verse, hanmun]);

const summary = (d: TextDoc, sort: number): TextSummary & { sort: number } => ({
  id: d.id, sort, shelf: d.meta.shelf ?? 'graded', period: d.meta.period, year: d.meta.year, script: d.meta.script ?? 'hangul',
  level: d.card.level, chars: d.paragraphs.reduce((n, p) => n + p.orig.length, 0), title_ko: d.meta.title_ko, title_en: d.meta.title_en,
  author_ko: d.meta.author_ko, author_en: d.meta.author_en, date: d.meta.date, themes: d.meta.themes ?? [], excerpt: d.meta.excerpt, labels: d.labels,
});
export const fixtureList = async (): Promise<TextSummary[]> => docs.map(summary);
export const fixtureText = async (id: string): Promise<TextDoc | null> => docs.find((d) => d.id === id) ?? null;
