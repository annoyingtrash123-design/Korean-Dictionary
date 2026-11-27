import { downloadAll, expect, test } from './fixtures';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

/** Reviewed AI English for 표준국어대사전 meanings that had none (pipeline/data/word_en.tsv): English search now finds
 *  them, and the entry labels the English as AI (CLAUDE.md: AI-written content is always labelled). Needs packs built
 *  with the overlay (manifest count `word_en`); older local data skips. */
const WORD_EN = (() => {
  try {
    const m = JSON.parse(readFileSync(join(process.cwd(), 'public', 'data', 'manifest.json'), 'utf8'));
    return (m.packs?.find((p: { id: string }) => p.id === 'core')?.counts?.word_en ?? 0) > 0;
  } catch { return false; }
})();

test('English search finds 선물 先物 "futures" with an AI-labelled gloss (phone + iPad)', async ({ page }) => {
  test.skip(!WORD_EN, 'packs built before the word_en overlay');
  test.setTimeout(300_000);
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/');
  await downloadAll(page);
  await page.getByRole('searchbox', { name: 'Search' }).fill('futures');
  const row = page.locator('.row', { has: page.locator('.hw', { hasText: /^선물$/ }) }).filter({ hasText: 'futures (finance)' }).first();
  await expect(row).toBeVisible();
  await expect(row.locator('.hanja')).toHaveText('先物');
  await page.waitForTimeout(300);
  await page.screenshot({ path: '../docs/screenshots/word-en-search-phone.png' });
  await row.click();
  await expect(page.locator('h1')).toContainText('선물');
  const gloss = page.locator('.gloss', { hasText: 'futures (finance)' }).first();
  await expect(gloss).toBeVisible();
  const chip = gloss.locator('xpath=following-sibling::span[contains(@class,"chip-ai")]').first();
  await expect(chip).toHaveText('AI');
  await expect(chip).toHaveAttribute('aria-label', 'English written by AI (reviewed)');
  // stdict's own Korean definition stays with it
  await expect(page.getByText('장래의 일정한 시기에 현품을 넘겨준다는 조건으로').first()).toBeVisible();
  await page.screenshot({ path: '../docs/screenshots/word-en-entry-phone.png' });
  await page.setViewportSize({ width: 1180, height: 820 });
  await page.waitForTimeout(400);
  await page.screenshot({ path: '../docs/screenshots/word-en-entry-ipad.png' });
});
