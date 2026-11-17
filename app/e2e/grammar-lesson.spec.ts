import { downloadAll, expect, test, type Page } from './fixtures';
import { mkdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

const SHOTS = join(process.cwd(), '..', 'docs', 'screenshots');
mkdirSync(SHOTS, { recursive: true });
const shot = async (page: Page, name: string) => { console.log('SHOT', name); await page.screenshot({ path: join(SHOTS, `${name}.png`) }); };

/** The core pack has grammar lessons. The real lessons stay drafts until reviewed (the pipeline ships only
 *  approved ones), so the local e2e pack gets a few "as if approved" from scripts/e2e-lessons.py. */
const HAS_LESSONS = (() => {
  try {
    const m = JSON.parse(readFileSync(join(process.cwd(), 'public', 'data', 'manifest.json'), 'utf8'));
    return (m.packs?.find((p: { id: string }) => p.id === 'core')?.counts?.grammar_lesson ?? 0) > 0;
  } catch { return false; }
})();

const study = (page: Page) => page.locator('section[data-source=study]');
const rowAt = (page: Page, n: number) => page.locator(`.vl-row[aria-posinset="${n}"]`);
const lessons = (page: Page) => page.locator('[data-testid=lesson]');

/** The lesson from its top, then two screens further down (the page scrolls inside the entry pane). */
async function lessonShots(page: Page, name: string) {
  const top = lessons(page).first();
  for (const [i, dy] of [[1, 0], [2, 1], [3, 2]] as const) {
    await top.evaluate((el, k) => {
      const vh = window.innerHeight * 0.7;
      let sc: HTMLElement | null = el.parentElement;
      while (sc && sc.scrollHeight <= sc.clientHeight + 1) sc = sc.parentElement;
      const target = el.getBoundingClientRect().top + (sc ? sc.scrollTop : window.scrollY) - 60 + k * vh;
      if (sc) sc.scrollTop = target; else window.scrollTo(0, target);
    }, dy);
    await page.waitForTimeout(300);
    await shot(page, `${name}-${i}`);
  }
}

async function openGrammar6(page: Page) {
  await page.getByRole('link', { name: 'Lists', exact: true }).click();
  await expect(study(page).getByRole('heading', { name: 'Study lists' })).toBeVisible();
  await study(page).getByRole('group', { name: 'Grammar by level' })
    .locator('.list-row', { has: page.locator('.list-name', { hasText: /^6급 · 표준 교육과정$/ }) }).first().click();
  await expect(page.locator('.eyebrow')).toHaveText('Grammar');
  await expect(rowAt(page, 1)).toContainText(/\S/);
}

test('grammar lessons: Lesson badge in the grammar list, lesson section on the entry page (phone and iPad)', async ({ page }) => {
  test.setTimeout(600_000);
  page.on('pageerror', (e) => console.log('PAGEERROR', e.message));
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/');
  await downloadAll(page);
  await page.getByRole('link', { name: 'Settings' }).click();
  await page.getByRole('radio', { name: 'Light' }).click();
  if (!HAS_LESSONS) {
    // an older pack, or none approved: nothing shows a lesson
    await page.getByRole('link', { name: 'Lists', exact: true }).click();
    await expect(page.locator('[data-testid=lesson], .chip-lesson')).toHaveCount(0);
    return;
  }

  // ---- the Lesson badge marks the rows that have a lesson ----
  await openGrammar6(page);
  // 6급 order: 마는, 깨나, 이라고, 을랑, 이라면, -거들랑1, -노라면, ...
  await expect(rowAt(page, 7)).toContainText('-노라면');
  await expect(rowAt(page, 7).locator('.chip-lesson')).toHaveText('Lesson');
  await expect(rowAt(page, 6).locator('.chip-lesson')).toHaveCount(1);
  await expect(rowAt(page, 1).locator('.chip-lesson')).toHaveCount(0);
  await expect(rowAt(page, 5).locator('.chip-lesson')).toHaveCount(0);
  await shot(page, 'grammar-lesson-list-phone');

  // ---- a row without a lesson opens a plain entry page ----
  await rowAt(page, 1).locator('a.row').click();
  await expect(page.locator('h1').first()).toContainText('마는');
  await expect(lessons(page)).toHaveCount(0);
  await page.goBack();

  // ---- -노라면: one lesson, labelled as AI, open by default ----
  await rowAt(page, 7).locator('a.row').click();
  await expect(page.locator('h1').first()).toContainText('-노라면');
  await expect(lessons(page)).toHaveCount(1);
  const lesson = lessons(page).first();
  await expect(lesson).toHaveJSProperty('open', true);
  await expect(lesson.locator('summary .lesson-h')).toHaveText('Lesson');
  await expect(lesson.locator('summary .chip-ai')).toHaveText('AI');
  await expect(lesson.locator('summary .chip-ai')).toHaveAttribute('aria-label', 'Written by AI');
  await expect(lesson.locator('summary')).toContainText('AI-written lesson, reviewed');
  // meaning, usage, examples with English and the speaker, compare cards, mistakes, TOPIK tip
  await expect(lesson.locator('.lesson-meaning')).toContainText('literary');
  await expect(lesson.locator('.lesson-usage')).toContainText('verb stems');
  await expect(lesson.locator('.ex')).toHaveCount(5);
  await expect(lesson.locator('.ex').first().locator('.ex-ko')).toContainText('사노라면');
  await expect(lesson.locator('.ex').first().locator('.ex-en')).toContainText('good days');
  await expect(lesson.locator('.ex .ex-say')).toHaveCount(5);
  await expect(lesson.getByRole('heading', { name: 'Compare with' })).toBeVisible();
  const cards = lesson.locator('.cmp');
  await expect(cards).toHaveCount(2);
  await expect(cards.first().locator('.cmp-pattern')).toHaveText('-다 보면');
  await expect(cards.first().locator('.cmp-line')).toHaveCount(2);
  await expect(lesson.getByRole('heading', { name: 'Common mistakes' })).toBeVisible();
  await expect(lesson.locator('.pitfall')).toHaveCount(2);
  await expect(lesson.locator('.pitfall-wrong .hangul').first()).toHaveText('책을 읽었노라면 눈물이 난다.');
  await expect(lesson.locator('.pitfall-right .hangul').first()).toHaveText('책을 읽노라면 눈물이 난다.');
  await expect(lesson.locator('.lesson-tip')).toContainText('TOPIK tip');

  await lessonShots(page, 'grammar-lesson-phone');

  // ---- compare card opens the other pattern when the dictionary has it ----
  const linked = lesson.locator('.cmp[data-linked] a.cmp-link').first();
  if (await linked.count()) {
    const target = (await linked.locator('.cmp-pattern').innerText()).trim();
    await linked.click();
    await expect(page.locator('h1').first()).toContainText(target.replace(/^-/, '').slice(0, 2));
    await page.goBack();
    await expect(lessons(page)).toHaveCount(1);
  }

  // ---- collapsible ----
  await lesson.locator('summary').click();
  await expect(lesson).toHaveJSProperty('open', false);
  await expect(lesson.locator('.lesson-meaning')).toBeHidden();
  await expect(lesson.locator('summary')).toContainText('AI-written lesson, reviewed');   // the label stays when collapsed
  await lesson.locator('summary').click();
  await expect(lesson.locator('.lesson-meaning')).toBeVisible();

  // ---- the same entry on a tablet ----
  await page.setViewportSize({ width: 1180, height: 820 });
  await page.waitForTimeout(500);
  await lessonShots(page, 'grammar-lesson-ipad');
});

test('grammar lessons: two homonym rows of one pattern give two lessons on its page', async ({ page }) => {
  test.setTimeout(600_000);
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/');
  await downloadAll(page);
  if (!HAS_LESSONS) return;
  await openGrammar6(page);
  await expect(rowAt(page, 6)).toContainText('-거들랑');
  await rowAt(page, 6).locator('a.row').click();
  await expect(page.locator('h1').first()).toContainText('-거들랑');
  await expect(lessons(page)).toHaveCount(2);
  // the two rows are told apart by their label, and each is AI-labelled
  await expect(lessons(page).nth(0).locator('.lesson-label')).toHaveText('if (it turns out) ... then');
  await expect(lessons(page).nth(1).locator('.lesson-label')).toHaveText('you see, because ... (giving background)');
  await expect(lessons(page).nth(0).locator('.lesson-form')).toHaveText('-거들랑');
  await expect(page.locator('[data-testid=lesson] summary .chip-ai')).toHaveCount(2);
});
