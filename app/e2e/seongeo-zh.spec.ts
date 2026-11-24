import { downloadAll, expect, test, type Page } from './fixtures';
import { mkdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

const SHOTS = join(process.cwd(), '..', 'docs', 'screenshots');
mkdirSync(SHOTS, { recursive: true });
const shot = async (page: Page, name: string) => { console.log('SHOT', name); await page.screenshot({ path: join(SHOTS, `${name}.png`) }); };

/** The core pack has `seongeo_zh` rows. CC-CEDICT is fetched only in CI and no English equivalent is approved yet, so the
 *  local e2e pack gets three sample rows from scripts/e2e-seongeo-zh.py (counts.seongeo_zh_sample); a CI pack with real
 *  CEDICT rows is checked more loosely. */
const COUNTS = (() => {
  try {
    const m = JSON.parse(readFileSync(join(process.cwd(), 'public', 'data', 'manifest.json'), 'utf8'));
    return (m.packs?.find((p: { id: string }) => p.id === 'core')?.counts ?? {}) as Record<string, number>;
  } catch { return {} as Record<string, number>; }
})();
const SAMPLE = !!COUNTS.seongeo_zh_sample;
const block = (page: Page) => page.locator('[data-testid=seongeo-zh]');

async function openEntry(page: Page, word: string) {
  await page.getByRole('searchbox', { name: 'Search' }).fill(word);
  await page.locator('.row', { has: page.locator('.hangul', { hasText: new RegExp(`^${word}$`) }) }).first().click();
  await expect(page.locator('h1').first()).toContainText(word);
}

test('성어 page: "Chinese & English" block (phone and iPad)', async ({ page }) => {
  test.setTimeout(600_000);
  page.on('pageerror', (e) => console.log('PAGEERROR', e.message));
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/');
  await downloadAll(page);
  await page.getByRole('link', { name: 'Settings' }).click();
  await page.getByRole('radio', { name: 'Light' }).click();
  await page.getByRole('link', { name: 'Search', exact: true }).click().catch(() => undefined);

  if (!SAMPLE && !(COUNTS.seongeo_zh > 0)) {
    // no rows in this pack (no CC-CEDICT, nothing approved): a 성어 page shows no such block
    await openEntry(page, '과유불급');
    await expect(block(page)).toHaveCount(0);
    return;
  }

  // ---- 과유불급: chengyu (simplified + traditional + pinyin + CEDICT English + source) ----
  await openEntry(page, '과유불급');
  await expect(block(page)).toHaveCount(1);
  await expect(block(page)).toHaveJSProperty('open', true);
  await expect(block(page).locator('summary')).toContainText('Chinese & English');
  const zh = block(page).locator('[data-testid=zh-chengyu]');
  await expect(zh.locator('.zh-han')).toHaveText('过犹不及');
  await expect(zh.locator('.zh-trad')).toHaveText('過猶不及');
  await expect(zh.locator('.zh-pin')).toHaveText('guò yóu bù jí');
  await expect(zh.locator('.zh-def').first()).toContainText('too far');
  await expect(zh.locator('.zh-src')).toContainText('CC-CEDICT');
  // English equivalent: always labelled AI
  const en = block(page).locator('[data-testid=en-equiv]');
  if (SAMPLE) {
    await expect(en.locator('.gloss')).toHaveText('too much of a good thing');
    await expect(en.locator('.chip-ai')).toHaveText('AI');
    await expect(en.locator('.chip-ai')).toHaveAttribute('aria-label', 'English equivalent written by AI');
  } else if (await en.count()) {
    await expect(en.locator('.chip-ai')).toHaveText('AI');
  }
  await block(page).scrollIntoViewIfNeeded();
  await page.waitForTimeout(300);
  await shot(page, 'seongeo-zh-phone');

  if (SAMPLE) {
    // ---- 타산지석: Chinese only (simplified = traditional: one form), two CEDICT definitions, no English block ----
    await page.goBack();
    await openEntry(page, '타산지석');
    await expect(block(page).locator('.zh-han')).toHaveText('他山之石');
    await expect(block(page).locator('.zh-trad')).toHaveCount(0);
    await expect(block(page).locator('.zh-def')).toHaveCount(2);
    await expect(block(page).locator('[data-testid=en-equiv]')).toHaveCount(0);
    await shot(page, 'seongeo-zh-chinese-only-phone');

    // ---- 새옹지마: English equivalent only, with the nuance note ----
    await page.goBack();
    await openEntry(page, '새옹지마');
    await expect(block(page).locator('[data-testid=zh-chengyu]')).toHaveCount(0);
    await expect(block(page).locator('.gloss')).toHaveText('a blessing in disguise');
    await expect(block(page).locator('.zh-note')).toContainText('luck and misfortune');
    await shot(page, 'seongeo-zh-english-only-phone');

    // ---- an ordinary word has no block ----
    await page.goBack();
    await openEntry(page, '사과');
    await expect(block(page)).toHaveCount(0);
    await page.goBack();
    await openEntry(page, '과유불급');
  }

  // ---- iPad ----
  await page.setViewportSize({ width: 1180, height: 820 });
  await page.waitForTimeout(500);
  await expect(block(page)).toHaveCount(1);
  await block(page).scrollIntoViewIfNeeded();
  await page.waitForTimeout(300);
  await shot(page, 'seongeo-zh-ipad');
});
