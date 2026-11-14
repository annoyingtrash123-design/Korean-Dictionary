import { downloadAll, expect, test } from './fixtures';

/** A reviewed AI correction of a krdict sense gloss (pipeline/data/gloss_fixes.tsv) replaces the old
 *  gloss and is labelled with the AI chip (CLAUDE.md: AI-written content is always labelled). */
test('corrected krdict gloss shows the new English with an AI label (phone + iPad)', async ({ page }) => {
  test.setTimeout(300_000);
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/');
  await downloadAll(page);
  await page.getByRole('searchbox', { name: 'Search' }).fill('금고');
  await page.locator('.row', { hasText: 'strongbox' }).first().click();
  await expect(page.locator('h1')).toContainText('금고');
  const sense = page.locator('.gloss', { hasText: 'safe; strongbox' }).first();
  await expect(sense).toBeVisible();
  await expect(page.locator('.gloss', { hasText: 'safety' })).toHaveCount(0);
  const chip = sense.locator('xpath=following-sibling::span[contains(@class,"chip-ai")]').first();
  await expect(chip).toHaveText('AI');
  await expect(chip).toHaveAttribute('aria-label', 'English corrected by AI (reviewed)');
  await sense.scrollIntoViewIfNeeded();
  await page.waitForTimeout(300);
  await page.screenshot({ path: '../docs/screenshots/gloss-fix-phone.png' });
  await page.setViewportSize({ width: 1180, height: 820 });
  await page.waitForTimeout(400);
  await sense.scrollIntoViewIfNeeded();
  await page.screenshot({ path: '../docs/screenshots/gloss-fix-ipad.png' });
});
