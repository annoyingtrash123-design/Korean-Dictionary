import { downloadAll, expect, test } from './fixtures';

// Chrome, short of space, clears a site's IndexedDB and file storage but keeps localStorage.
// Simulate that: carry only localStorage into a fresh browser profile.
test('after the browser clears storage: explain it, keep bookmarks and history', async ({ page, browser }) => {
  test.setTimeout(400_000);
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/');
  await downloadAll(page);
  const box = page.getByRole('searchbox', { name: 'Search' });
  await box.fill('학교');
  await page.locator('.row', { hasText: '학교' }).first().click();
  await expect(page.locator('h1')).toContainText('학교');
  await page.getByRole('button', { name: 'Add bookmark' }).click();
  await page.getByRole('button', { name: 'Saved' }).click();
  await expect(page.getByRole('button', { name: 'Edit bookmark' })).toBeVisible();
  await page.waitForTimeout(500);
  const ls = await page.evaluate(() => Object.fromEntries(Object.keys(localStorage).map((k) => [k, localStorage.getItem(k)!])));
  expect(Object.keys(ls)).toEqual(expect.arrayContaining(['kd.packs', 'kd.backup.bookmarks', 'kd.backup.history']));

  // Settings → Data → Technical details: always there, shows the packs and the engine's startup log
  await page.goto('/#/settings');
  await page.getByText('Technical details').click();
  const pre = page.locator('.diag pre');
  await expect(pre).toContainText(/storage in use: \d+ MB/);
  await expect(pre).toContainText(/installed:\n\s+core /);
  await expect(pre).toContainText(/(installed|opened) core/);
  await pre.scrollIntoViewIfNeeded();
  await page.screenshot({ path: '../docs/screenshots/settings-tech-details.png' });

  // a fresh profile with only the old localStorage = what Chrome leaves after its clean-up
  const ctx = await browser.newContext({ viewport: { width: 390, height: 844 }, serviceWorkers: 'block' });
  await ctx.addInitScript((data: Record<string, string>) => {
    if (sessionStorage.getItem('seeded')) return;
    sessionStorage.setItem('seeded', '1');
    for (const [k, v] of Object.entries(data)) localStorage.setItem(k, v);
  }, ls);
  const p2 = await ctx.newPage();
  await p2.goto('/');
  await expect(p2.getByText(/dictionaries are no longer on this phone/)).toBeVisible({ timeout: 60_000 });
  await p2.getByText(/Technical details/).click();
  await expect(p2.locator('.diag pre')).toContainText(/storage in use: \d+ MB/);
  await expect(p2.locator('.diag pre')).toContainText(/last working: \d{4}-/);
  await expect(p2.getByText(/bookmarks, history and game progress were kept/)).toBeVisible();
  await p2.screenshot({ path: '../docs/screenshots/storage-cleared.png' });
  await downloadAll(p2);
  await p2.getByRole('link', { name: 'Bookmarks' }).click();
  await expect(p2.locator('.hw', { hasText: '학교' }).first()).toBeVisible();
  await p2.getByRole('link', { name: 'Search' }).click();
  await expect(p2.locator('.hw', { hasText: '학교' }).first()).toBeVisible();   // recent lookups
  await ctx.close();
});
