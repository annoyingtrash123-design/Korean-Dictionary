import { downloadAll, expect, test } from './fixtures';

/** Grammar patterns of the NIKL curriculum list that krdict lacks come from the reviewed overlay
 *  (pipeline/data/grammar_extra.tsv, source hanppuri). Everything AI-written in them is labelled with the
 *  AI chip (CLAUDE.md): the English summary, an AI Korean definition, and AI Korean example sentences. */
test('overlay grammar entry is found by search and its AI text is labelled (phone + iPad)', async ({ page }) => {
  test.setTimeout(300_000);
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/');
  await downloadAll(page);
  await page.getByRole('searchbox', { name: 'Search' }).fill('기에 따라');
  await page.locator('.row', { hasText: '-기에 따라' }).first().click();
  await expect(page.locator('h1')).toContainText('-기에 따라');

  // a sensible source heading
  await expect(page.locator('details.dict > summary').first()).toContainText('한뿌리 grammar notes (AI-assisted)');

  const sense = page.locator('.sense').first();
  // AI English summary, AI Korean definition
  await expect(sense.locator('.def:not(.ko) .chip-ai')).toHaveText('AI');
  await expect(sense.locator('.def.ko .chip-ai')).toHaveText('AI');
  await expect(sense.locator('.def.ko .chip-ai')).toHaveAttribute('aria-label', 'Korean definition written by AI');
  // three AI Korean example sentences, each with its AI English translation
  const exs = sense.locator('.ex');
  await expect(exs).toHaveCount(3);
  await expect(exs.first().locator('.ex-ko .chip-ai')).toHaveAttribute('aria-label', 'Korean sentence written by AI');
  await expect(exs.first().locator('.ex-en .chip-ai')).toHaveText('AI');
  for (let i = 0; i < 3; i++) await expect(exs.nth(i).locator('.ex-ko .chip-ai')).toHaveCount(1);
  await sense.scrollIntoViewIfNeeded();
  await page.waitForTimeout(300);
  await page.screenshot({ path: '../docs/screenshots/grammar-extra-phone.png' });
  await page.setViewportSize({ width: 1180, height: 820 });
  await page.waitForTimeout(400);
  await sense.scrollIntoViewIfNeeded();
  await page.screenshot({ path: '../docs/screenshots/grammar-extra-ipad.png' });
});

/** -노라면 has a real 표준국어대사전 definition and examples: only the English is AI there. */
test('overlay entry with a dictionary definition names its source and labels only the English', async ({ page }) => {
  test.setTimeout(300_000);
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/');
  await downloadAll(page);
  await page.getByRole('searchbox', { name: 'Search' }).fill('노라면');
  await page.locator('.row', { hasText: '-노라면' }).first().click();
  const sense = page.locator('details.dict', { hasText: '한뿌리' }).locator('.sense').first();
  await expect(sense.locator('.def.ko')).toContainText('하다가 보면');
  await expect(sense.locator('.def.ko')).toContainText('표준국어대사전');
  await expect(sense.locator('.def.ko .chip-ai')).toHaveCount(0);
  await expect(sense.locator('.ex-ko .chip-ai')).toHaveCount(0);
  await expect(sense.locator('.ex-en .chip-ai').first()).toHaveText('AI');
});
