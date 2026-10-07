import { expect, test, type Page } from '@playwright/test';
import { mkdirSync } from 'node:fs';
import { join } from 'node:path';

const SHOTS = join(process.cwd(), '..', 'docs', 'screenshots');
mkdirSync(SHOTS, { recursive: true });
const shot = (page: Page, name: string) => page.screenshot({ path: join(SHOTS, `tablet-${name}.png`) });
const search = async (page: Page, q: string) => {
  await page.getByRole('searchbox', { name: 'Search' }).fill(q);
  await expect(page.locator(`.pane-left .page[data-q="${q}"][data-busy="0"]`)).toBeVisible({ timeout: 30_000 });
};

test('two-pane tablet layout', async ({ page }) => {
  test.setTimeout(600_000);
  await page.setViewportSize({ width: 1180, height: 820 });
  await page.goto('/');
  await page.getByRole('button', { name: 'Download' }).click();
  await expect(page.getByRole('searchbox', { name: 'Search' })).toBeVisible({ timeout: 240_000 });
  await expect(page.locator('.app.wide')).toBeVisible();
  await page.getByRole('link', { name: 'Settings' }).click();
  await page.getByRole('radio', { name: 'Light' }).click();
  await page.getByRole('link', { name: 'Search' }).click();

  // empty state
  await expect(page.getByText('Search or pick a word')).toBeVisible();
  await shot(page, 'light-empty');

  // results on the left
  await search(page, '학교');
  const rows = page.locator('.pane-left .list .row');
  await expect(rows.first()).toBeVisible();
  await page.waitForTimeout(600); // progressive row paint settles
  const before = await rows.count();
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
