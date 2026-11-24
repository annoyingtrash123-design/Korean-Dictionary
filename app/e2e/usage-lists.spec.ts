import { downloadAll, expect, test, type Page } from './fixtures';
import { mkdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

const SHOTS = join(process.cwd(), '..', 'docs', 'screenshots');
mkdirSync(SHOTS, { recursive: true });
const shot = async (page: Page, name: string) => { console.log('SHOT', name); await page.screenshot({ path: join(SHOTS, `${name}.png`) }); };

/** The core pack has the "Writing & usage" tables. Every row of the data files is a draft and the pipeline ships only approved
 *  ones, so the local e2e pack gets sample rows from scripts/e2e-usage.py (counts.usage_sample); a pack with real approved rows
 *  is checked more loosely, and a pack without the tables must not show the group. */
const COUNTS = (() => {
  try {
    const m = JSON.parse(readFileSync(join(process.cwd(), 'public', 'data', 'manifest.json'), 'utf8'));
    return (m.packs?.find((p: { id: string }) => p.id === 'core')?.counts ?? {}) as Record<string, number>;
  } catch { return {} as Record<string, number>; }
})();
const SAMPLE = !!COUNTS.usage_sample;
const HAS = SAMPLE || (COUNTS.writing_expr ?? 0) > 0 || (COUNTS.confusable ?? 0) > 0;

const study = (page: Page) => page.locator('section[data-source=study]');
const group = (page: Page) => study(page).getByRole('group', { name: 'Writing & usage' });
const fold = (page: Page) => study(page).getByRole('button', { name: /^Writing & usage/ });
const link = (page: Page, name: string) => group(page).locator('.list-row', { has: page.locator('.list-name', { hasText: new RegExp(`^${name.replace(/[()]/g, '\\$&')}$`) }) });
const WRITING = '쓰기 expressions (TOPIK writing)';
const CONFUSABLE = 'Confusable words';

async function overview(page: Page) {
  await page.goto('/#/lists');
  await expect(study(page).getByRole('heading', { name: 'Study lists' })).toBeVisible();
  if ((await fold(page).getAttribute('aria-expanded')) !== 'true') await fold(page).click();
  await expect(fold(page)).toHaveAttribute('aria-expanded', 'true');
}

for (const vp of [{ n: 'phone', w: 390, h: 844 }, { n: 'ipad', w: 1180, h: 820 }]) {
  test(`Writing & usage lists: 쓰기 expressions and confusable words (${vp.n})`, async ({ page }) => {
    test.setTimeout(900_000);
    page.on('pageerror', (e) => console.log('PAGEERROR', e.message));
    await page.setViewportSize({ width: vp.w, height: vp.h });
    await page.goto('/');
    await downloadAll(page);
    await page.getByRole('link', { name: 'Settings' }).click();
    await page.getByRole('radio', { name: 'Light' }).click();
    await page.getByRole('link', { name: 'Lists', exact: true }).click();
    await expect(page.getByRole('heading', { name: 'My lists' })).toBeVisible();

    if (!HAS) {
      // an older pack (or nothing approved yet): the group is simply not there
      await expect(group(page)).toHaveCount(0);
      return;
    }

    // ---- overview: a collapsible group like the others, two lists with counts ----
    await expect(group(page)).toBeVisible();
    await expect(fold(page)).toHaveAttribute('aria-expanded', 'false');
    await expect(fold(page).locator('.group-sum')).toHaveText('2 lists');
    await fold(page).click();
    await expect(group(page).locator('.list-row:visible')).toHaveCount(2);
    await expect(link(page, WRITING)).toContainText(SAMPLE ? '12 expressions' : 'expressions');
    await expect(link(page, CONFUSABLE)).toContainText(SAMPLE ? '6 groups' : 'groups');
    await group(page).scrollIntoViewIfNeeded();
    await page.waitForTimeout(300);
    await shot(page, `usage-overview-${vp.n}`);

    // ---- 쓰기 expressions: AI label, groups by function, expandable rows ----
    await link(page, WRITING).click();
    await expect(page.getByRole('heading', { level: 1 })).toHaveText(WRITING);
    await expect(page.getByTestId('usage-ai')).toContainText('AI-written, reviewed');
    await expect(page.getByTestId('usage-ai').locator('.chip-ai')).toHaveText('AI');
    const groups = page.getByTestId('writing-group');
    if (SAMPLE) {
      await expect(groups).toHaveCount(4);
      await expect(page.getByTestId('usage-count')).toHaveText('12 expressions · 4 groups');
      // the first group is open until the reader chooses; the others are folded
      await expect(groups.nth(0).locator('.group-fold')).toHaveAttribute('aria-expanded', 'true');
      await expect(groups.nth(1).locator('.group-fold')).toHaveAttribute('aria-expanded', 'false');
      await expect(groups.nth(0).locator('.group-title')).toHaveText('Introducing the topic');
      await expect(groups.nth(0).getByTestId('writing-item')).toHaveCount(3);
    }
    const first = groups.nth(0).getByTestId('writing-item').first();
    await first.locator('summary').click();
    await expect(first).toHaveJSProperty('open', true);
    await expect(first.locator('.uw-register')).toContainText('Written');
    await expect(first.locator('.uw-register')).toContainText('Spoken');
    await expect(first.locator('.ex')).toHaveCount(2);
    await expect(first.locator('.uw-ai-inline')).toContainText('AI-written, reviewed');
    if (SAMPLE) {
      await expect(first.locator('.uw-expr')).toHaveText('-에 대해 살펴보고자 한다');
      await expect(first.getByTestId('uw-entry')).toHaveCount(1);   // linked to the grammar entry -고자
    }
    await first.scrollIntoViewIfNeeded();
    await page.waitForTimeout(300);
    await shot(page, `usage-writing-${vp.n}`);

    // filter: matches across groups, groups open themselves, a miss says so
    if (SAMPLE) {
      await page.getByRole('searchbox', { name: 'Filter expressions' }).fill('차지');
      await expect(groups).toHaveCount(1);
      await expect(page.getByTestId('writing-item')).toHaveCount(1);
      await expect(page.getByTestId('writing-item').locator('.uw-expr')).toHaveText('-을/를 차지하다');
      await expect(page.getByTestId('usage-count')).toContainText('1 of 12 expressions');
      await page.getByRole('searchbox', { name: 'Filter expressions' }).fill('zzzz');
      await expect(page.getByText('No expression matches.')).toBeVisible();
      await page.getByRole('searchbox', { name: 'Filter expressions' }).fill('');
      // unfold another group, remembered on this device
      await groups.nth(1).locator('.group-fold').click();
      await expect(groups.nth(1).locator('.group-fold')).toHaveAttribute('aria-expanded', 'true');
      await page.reload();
      await expect(groups.nth(1).locator('.group-fold')).toHaveAttribute('aria-expanded', 'true');
    }
    // the dictionary entry link opens an entry
    const entryLink = page.getByTestId('uw-entry').first();
    if (await entryLink.count()) {
      if (!(await entryLink.isVisible())) await page.getByTestId('writing-item').first().locator('summary').click();
      await entryLink.click();
      await expect(page).toHaveURL(/#\/entry\//);
      await page.goBack();
    }

    // ---- confusable words: index with kind filter, then a comparison view ----
    await overview(page);
    await link(page, CONFUSABLE).click();
    await expect(page.getByRole('heading', { level: 1 })).toHaveText(CONFUSABLE);
    await expect(page.getByTestId('usage-ai')).toContainText('AI-written, reviewed');
    const rows = page.getByTestId('confusable-row');
    if (SAMPLE) {
      await expect(rows).toHaveCount(6);
      await expect(rows.first().locator('.hw')).toHaveText('바라다 · 원하다');
      await expect(rows.first().locator('.kind-tag')).toHaveText('Meaning');
      await page.getByRole('radio', { name: /^Spelling/ }).click();
      await expect(rows).toHaveCount(3);
      await page.getByRole('radio', { name: /^All/ }).click();
      await page.getByRole('searchbox', { name: 'Filter groups' }).fill('바램');
      await expect(rows).toHaveCount(2);   // the 바람 group and the 바라다 group that warns about 바램
      await page.getByRole('searchbox', { name: 'Filter groups' }).fill('');
      await expect(rows).toHaveCount(6);
    }
    await page.waitForTimeout(300);
    await shot(page, `usage-confusable-list-${vp.n}`);

    await rows.first().click();
    const detail = page.getByTestId('confusable-detail');
    await expect(detail).toBeVisible();
    await expect(detail.locator('.chip-ai').first()).toHaveText('AI');
    await expect(detail.getByTestId('confusable-mistake')).toContainText('Common mistake');
    await expect(detail.getByTestId('word-card').first().locator('.ex-ko')).not.toBeEmpty();
    await expect(detail.getByTestId('word-card').first().locator('.ex-en')).not.toBeEmpty();
    if (SAMPLE) {
      await expect(detail.locator('h1')).toHaveText('바라다 · 원하다');
      await expect(detail.getByTestId('word-card')).toHaveCount(2);
      await expect(detail.getByTestId('word-card').first().locator('.uw-w')).toHaveText('바라다');
    }
    await page.waitForTimeout(300);
    await shot(page, `usage-confusable-${vp.n}`);

    if (SAMPLE) {
      // next group, then the group with a wrong spelling (struck through, no sentence)
      await detail.getByRole('link', { name: /다르다 · 틀리다/ }).click();
      await expect(page.getByTestId('confusable-detail').locator('h1')).toHaveText('다르다 · 틀리다');
      await page.goBack();
      await page.goBack();   // 바라다 group, then the index
      await expect(page.getByTestId('confusable-list')).toBeVisible();
      await page.getByRole('searchbox', { name: 'Filter groups' }).fill('바램');
      await page.getByTestId('confusable-row').filter({ has: page.locator('.hw', { hasText: /^바람$/ }) }).click();
      const wrong = page.getByTestId('confusable-detail').getByTestId('word-card-wrong');
      await expect(wrong).toHaveCount(1);
      await expect(wrong.locator('s')).toHaveText('바램');
      await expect(page.getByTestId('confusable-detail').getByTestId('word-card')).toHaveCount(1);
      await page.waitForTimeout(300);
      await shot(page, `usage-confusable-wrong-${vp.n}`);
      // back to the index
      await page.getByRole('link', { name: CONFUSABLE }).first().click();
      await expect(page.getByTestId('confusable-list')).toBeVisible();
    }
  });
}
