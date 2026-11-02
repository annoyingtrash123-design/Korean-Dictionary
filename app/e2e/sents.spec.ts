import { downloadAll, expect, test } from './fixtures';

/** A kengdic-only word (no translated sentence) gets real krdict example sentences in Sents. */
test('Sents tab: real Korean examples for a word without translated ones', async ({ page }) => {
  test.setTimeout(300_000);
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/');
  await downloadAll(page);
  await page.getByRole('searchbox', { name: 'Search' }).fill('희미해지다');
  await page.locator('.row', { hasText: '희미해지다' }).first().click();
  await expect(page.locator('h1')).toContainText('희미해지다');
  await page.getByRole('tab', { name: 'Sents', exact: true }).click();
  const ex = page.locator('.sents .ex');
  await expect(ex.first()).toBeVisible({ timeout: 20_000 });
  expect(await ex.count()).toBeGreaterThanOrEqual(2);
  // every row contains the word and is a sentence, not the bare headword
  for (const t of await page.locator('.sents .ex-ko').allTextContents()) {
    expect(t).toContain('희미해지');
    expect(t.trim().split(/\s+/).length).toBeGreaterThanOrEqual(2);
  }
  await page.waitForTimeout(300);
  await page.screenshot({ path: '../docs/screenshots/sents-krdict-phone.png' });
  await page.setViewportSize({ width: 1180, height: 820 });
  await page.waitForTimeout(500);
  await page.screenshot({ path: '../docs/screenshots/sents-krdict-ipad.png' });
});
