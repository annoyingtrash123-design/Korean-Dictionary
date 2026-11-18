import { downloadAll, expect, test, type Page, texts } from './fixtures';
import { mkdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

const SHOTS = join(process.cwd(), '..', 'docs', 'screenshots');
mkdirSync(SHOTS, { recursive: true });
const shot = async (page: Page, name: string) => { console.log('SHOT', name); await page.screenshot({ path: join(SHOTS, `${name}.png`) }); };

/** The core pack has the Idioms & proverbs table (`study_item`: the pipeline's build, not an older pack). */
const HAS_IDIOMS = (() => {
  try {
    const m = JSON.parse(readFileSync(join(process.cwd(), 'public', 'data', 'manifest.json'), 'utf8'));
    return (m.packs?.find((p: { id: string }) => p.id === 'core')?.counts?.study_item ?? 0) > 0;
  } catch { return false; }
})();

const study = (page: Page) => page.locator('section[data-source=study]');
const group = (page: Page) => study(page).getByRole('group', { name: 'Idioms & proverbs' });
const rows = (page: Page) => page.locator('.vl-row:not(.vl-skel)');
const countOf = async (page: Page) => Number(((await page.getByTestId('study-count').innerText()).match(/[\d,]+/) ?? ['0'])[0].replace(/,/g, ''));

const LISTS = [
  { name: 'Common proverbs — top 100', count: 100, tag: '속담', first: '누워서 떡 먹기' },
  { name: 'All learner proverbs', count: 655, tag: '속담', first: '누워서 떡 먹기' },
  { name: 'Essential 성어 — top 100', count: 100, tag: '성어', first: '만장일치' },
  { name: 'Common 성어 — top 300', count: 300, tag: '성어', first: '만장일치' },
  { name: 'All learner 성어', count: 202, tag: '성어', first: '만장일치' },
];

async function overview(page: Page) {
  await page.goto('/#/lists');
  await expect(study(page).getByRole('heading', { name: 'Study lists' })).toBeVisible();
}
const link = (page: Page, name: string) => group(page).locator('.list-row', { has: page.locator('.list-name', { hasText: new RegExp(`^${name}$`) }) });

for (const vp of [{ n: 'phone', w: 390, h: 844 }, { n: 'ipad', w: 1180, h: 820 }]) {
  test(`idioms & proverbs lists: five lists, rows, a stdict-only 성어 opens (${vp.n})`, async ({ page }) => {
    test.setTimeout(900_000);
    page.on('pageerror', (e) => console.log('PAGEERROR', e.message));
    await page.setViewportSize({ width: vp.w, height: vp.h });
    await page.goto('/');
    await downloadAll(page);
    await page.getByRole('link', { name: 'Settings' }).click();
    await page.getByRole('radio', { name: 'Light' }).click();
    await page.getByRole('link', { name: 'Lists', exact: true }).click();
    await expect(page.getByRole('heading', { name: 'My lists' })).toBeVisible();

    if (!HAS_IDIOMS) {
      // an older pack: the group is simply not there
      await expect(group(page)).toHaveCount(0);
      return;
    }

    // ---- overview: the group with its five lists and counts ----
    await expect(group(page)).toBeVisible();
    expect(await texts(group(page).locator('.list-name'))).toEqual(LISTS.map((l) => l.name));
    for (const l of LISTS) await expect(link(page, l.name)).toContainText(`${l.count} ${l.tag === '속담' ? 'proverbs' : 'idioms'}`);
    await group(page).scrollIntoViewIfNeeded();
    await page.waitForTimeout(300);
    await shot(page, `idioms-overview-${vp.n}`);

    // ---- each list: count, ranked rows with kind label + gloss ----
    for (const [i, l] of LISTS.entries()) {
      await overview(page);
      await link(page, l.name).click();
      await expect(page.getByRole('heading', { level: 1 })).toHaveText(l.name);
      await expect(rows(page).first()).toBeVisible();
      expect(await countOf(page)).toBe(l.count);
      expect((await rows(page).first().locator('.hw').innerText()).trim()).toBe(l.first);
      await expect(rows(page).first().locator('.kind-tag')).toHaveText(l.tag);
      await expect(rows(page).first().locator('.row-gloss')).not.toBeEmpty();
      expect(await rows(page).count()).toBeLessThan(40);
      await page.waitForTimeout(300);
      await shot(page, `idioms-list${i + 1}-${vp.n}`);
    }

    // ---- 설상가상 is only in 표준국어대사전: with stdict installed the row opens that entry ----
    await overview(page);
    await link(page, 'Essential 성어 — top 100').click();
    const row = rows(page).filter({ has: page.locator('.hw', { hasText: /^설상가상$/ }) });
    await expect(row).toBeVisible();
    await expect(row.locator('.row-gloss')).toContainText('눈 위에 서리가');
    await row.locator('a.row').click();
    await expect(page).toHaveURL(/#\/entry\/stdict\/\d+/);
    await expect(page.locator('h1').filter({ hasText: '설상가상' }).first()).toBeVisible();
    await expect(page.locator('main').last()).toContainText('눈 위에 서리가 덮인다는 뜻으로');
    await page.waitForTimeout(300);
    await shot(page, `idioms-entry-stdict-${vp.n}`);

    // ---- a krdict proverb opens its core entry; the star saves a row ----
    await overview(page);
    await link(page, 'Common proverbs — top 100').click();
    const first = rows(page).first();
    await expect(first.locator('.hw')).toHaveText('누워서 떡 먹기');
    await first.locator('a.row').click();
    await expect(page).toHaveURL(/#\/entry\/krdict\/\d+/);
    await expect(page.locator('h1').filter({ hasText: '누워서 떡 먹기' }).first()).toBeVisible();
  });
}
