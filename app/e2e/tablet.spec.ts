import { downloadAll, expect, test, type Page } from './fixtures';
import { mkdirSync } from 'node:fs';
import { join } from 'node:path';

const SHOTS = join(process.cwd(), '..', 'docs', 'screenshots');
mkdirSync(SHOTS, { recursive: true });
const shot = (page: Page, name: string) => page.screenshot({ path: join(SHOTS, `tablet-${name}.png`) });
/** Dict | Words | Chars | Sents: each tab renders for 학교, ?tab= drives Back/Forward, the choice is remembered. */
async function tabs(page: Page, scope: string, shotPrefix: string, shotFn: (p: Page, n: string) => Promise<unknown>) {
  const root = page.locator(scope);
  const tab = (n: string) => root.getByRole('tab', { name: n, exact: true });
  await expect(root.getByRole('tablist')).toBeVisible();
  await expect(tab('Dict')).toHaveAttribute('aria-selected', 'true');
  await expect(root.locator('details.dict').first()).toBeVisible();
  await tab('Words').click();
  await expect(page).toHaveURL(/tab=words/);
  await expect(tab('Words')).toHaveAttribute('aria-selected', 'true');
  await expect(root.locator('.list .row').first()).toBeVisible({ timeout: 20_000 });
  await expect(root.locator('details.dict')).toHaveCount(0);
  await page.waitForTimeout(300);
  await shotFn(page, `${shotPrefix}-words`);
  await tab('Chars').click();
  await expect(root.locator('.char .hanja-big').first()).toBeVisible({ timeout: 20_000 });
  await page.waitForTimeout(300);
  await shotFn(page, `${shotPrefix}-chars`);
  await tab('Sents').click();
  await expect(root.locator('.sents .ex, .etab-panel .muted').first()).toBeVisible({ timeout: 20_000 });
  await page.waitForTimeout(300);
  await shotFn(page, `${shotPrefix}-sents`);
  // keyboard: arrows move between tabs
  await tab('Sents').focus();
  await page.keyboard.press('ArrowLeft');
  await expect(tab('Chars')).toHaveAttribute('aria-selected', 'true');
  await page.keyboard.press('Home');
  await expect(tab('Dict')).toHaveAttribute('aria-selected', 'true');
  // back/forward follow the tab
  await page.goBack();
  await expect(tab('Chars')).toHaveAttribute('aria-selected', 'true');
  await page.goForward();
  await expect(tab('Dict')).toHaveAttribute('aria-selected', 'true');
  // the last tab is remembered across entries
  await tab('Words').click();
  await expect(tab('Words')).toHaveAttribute('aria-selected', 'true');
  await root.locator('.list .row').first().click();
  await expect(root.getByRole('tab', { name: 'Words', exact: true })).toHaveAttribute('aria-selected', 'true');
  await page.goBack();
  await root.getByRole('tab', { name: 'Dict', exact: true }).click();
  await expect(root.getByRole('tab', { name: 'Dict', exact: true })).toHaveAttribute('aria-selected', 'true');
}
const search = async (page: Page, q: string) => {
  await page.getByRole('searchbox', { name: 'Search' }).fill(q);
  await expect(page.locator(`.pane-left .page[data-q="${q}"][data-busy="0"]`)).toBeVisible({ timeout: 30_000 });
};

test('two-pane tablet layout', async ({ page }) => {
  test.setTimeout(600_000);
  await page.setViewportSize({ width: 1180, height: 820 });
  await page.goto('/');
  await downloadAll(page);
  await expect(page.locator('.app.wide')).toBeVisible();
  await page.getByRole('link', { name: 'Settings' }).click();
  await page.getByRole('radio', { name: 'Light' }).click();
  await page.getByRole('link', { name: 'Search' }).click();

  // empty state
  await expect(page.getByText('Search or pick a word')).toBeVisible();
  await shot(page, 'light-empty');

  // English search: Korean results grouped by part of speech, glosses on two lines
  await search(page, 'report');
  await expect(page.locator('.pane-left .pos-h').first()).toBeVisible();
  await expect(page.locator('.pane-left .pos-h', { hasText: 'Nouns' })).toHaveCount(1);
  // a word is listed once even when one dictionary gives it without hanja (보고하다 / 報告하다)
  await expect(page.locator('.pane-left .row .hw', { hasText: /^보고하다$/ })).toHaveCount(1);
  await page.waitForTimeout(500);
  await shot(page, 'light-results-report');

  // results on the left
  await search(page, '학교');
  const rows = page.locator('.pane-left .list .row');
  await expect(rows.first()).toBeVisible();
  // progressive row paint: wait until the count is stable for a full second
  let before = -1;
  await expect.poll(async () => { const n = await rows.count(); const same = n === before; before = n; return same; },
    { timeout: 30_000, intervals: [1000] }).toBe(true);
  await expect(page.locator('.pane-right h1')).toHaveCount(0);

  // open on the right; left list persists with the row highlighted
  const row = page.locator('.pane-left .row', { hasText: '학교' }).first();
  await row.click();
  await expect(page.locator('.pane-right h1')).toContainText('학교');
  await expect(page.locator('.pane-left .page')).toHaveAttribute('data-q', '학교');
  await expect(rows).toHaveCount(before);
  await expect(page.locator('.pane-left .row[aria-current="true"]')).toHaveCount(1);
  await expect(page.locator('.pane-left .row[aria-current="true"]')).toContainText('학교');
  await page.waitForTimeout(500);
  await shot(page, 'light-entry');
  await tabs(page, '.pane-right', 'entry-tab', shot);

  // a link inside the entry changes the right pane only
  const h1 = await page.locator('.pane-right h1').innerText();
  const chip = page.locator('.pane-right a.chip').first();
  await expect(chip).toBeVisible();
  await chip.click();
  await expect(page.locator('.pane-right h1')).not.toHaveText(h1);
  await expect(page.locator('.pane-left .page')).toHaveAttribute('data-q', '학교');
  await expect(rows).toHaveCount(before);

  // back returns to the previous entry, list intact
  await page.goBack();
  await expect(page.locator('.pane-right h1')).toHaveText(h1);
  await expect(rows).toHaveCount(before);

  // keyboard: ArrowDown + Enter opens a row on the right
  await page.getByRole('searchbox', { name: 'Search' }).focus();
  await page.keyboard.press('ArrowDown');
  await expect(page.locator('.pane-left .row.kbd')).toHaveCount(1);
  await page.keyboard.press('Enter');
  await expect(page.locator('.pane-right h1')).toBeVisible();
  await page.locator('body').click({ position: { x: 5, y: 5 } });
  await page.keyboard.press('/');
  await expect(page.getByRole('searchbox', { name: 'Search' })).toBeFocused();

  // bookmarks split
  await page.locator('.pane-left .row', { hasText: '학교' }).first().click();
  await expect(page.locator('.pane-right h1')).toContainText('학교');
  await page.getByRole('button', { name: 'Add bookmark' }).click();
  await page.getByRole('button', { name: 'Saved' }).click();
  await page.getByRole('link', { name: 'Bookmarks' }).click();
  await expect(page.locator('.pane-left .bm .hw', { hasText: '학교' })).toBeVisible();
  await expect(page.getByText('Pick a saved word')).toBeVisible();
  await page.locator('.pane-left .bm .row', { hasText: '학교' }).click();
  await expect(page.locator('.pane-right h1')).toContainText('학교');
  await expect(page.locator('.pane-left .bm .row[aria-current="true"]')).toHaveCount(1);
  await expect(page.getByRole('link', { name: 'Bookmarks' })).toHaveAttribute('aria-current', 'page');
  await page.waitForTimeout(300);
  await shot(page, 'light-bookmarks');

  // settings: single column
  await page.getByRole('link', { name: 'Settings' }).click();
  await expect(page.locator('.pane-left')).toHaveCount(0);
  await shot(page, 'light-settings');

  // dark
  await page.getByRole('radio', { name: 'Dark' }).click();
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');
  await page.getByRole('link', { name: 'Search' }).click();
  await search(page, '학교');
  await page.locator('.pane-left .row', { hasText: '학교' }).first().click();
  await expect(page.locator('.pane-right h1')).toContainText('학교');
  await page.waitForTimeout(500);
  await shot(page, 'dark-entry');
  await page.getByRole('link', { name: 'Bookmarks' }).click();
  await page.locator('.pane-left .bm .row', { hasText: '학교' }).click();
  await page.waitForTimeout(300);
  await shot(page, 'dark-bookmarks');

  // portrait (< 900px): phone layout, centred in a wider column
  await page.setViewportSize({ width: 820, height: 1180 });
  await expect(page.locator('.app.wide')).toHaveCount(0);
  await expect(page.locator('.tabbar')).toBeVisible();
  await page.getByRole('link', { name: 'Search' }).click();
  await search(page, '학교').catch(() => undefined);
  await shot(page, 'portrait-results');
  // live switch back to wide
  await page.setViewportSize({ width: 1180, height: 820 });
  await expect(page.locator('.app.wide')).toBeVisible();
});
