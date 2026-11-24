import { downloadAll, expect, test, type Page } from './fixtures';
import { mkdirSync } from 'node:fs';
import { join } from 'node:path';

const SHOTS = join(process.cwd(), '..', 'docs', 'screenshots');
mkdirSync(SHOTS, { recursive: true });
const shot = async (page: Page, name: string) => { console.log('SHOT', name); await page.screenshot({ path: join(SHOTS, `${name}.png`) }); };

async function openEntry(page: Page, word: string) {
  await page.getByRole('link', { name: 'Search', exact: true }).click().catch(() => undefined);
  await page.getByRole('searchbox', { name: 'Search' }).fill(word);
  await page.locator('.row', { has: page.locator('.hangul', { hasText: new RegExp(`^${word}$`) }) }).first().click();
  await expect(page.locator('h1').first()).toContainText(word);
}

/** Hanja that 표준국어대사전 supplied for an entry without any (pipeline/src/hanja_fill.rs): shown with a small "표준" marker
 *  next to the hanja, searchable like any other hanja; entries whose own dictionary has the hanja show no marker. */
test('filled hanja: a small 표준 marker in the entry header (phone and iPad)', async ({ page }) => {
  test.setTimeout(600_000);
  page.on('pageerror', (e) => console.log('PAGEERROR', e.message));
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/');
  await downloadAll(page);
  await page.getByRole('link', { name: 'Settings' }).click();
  await page.getByRole('radio', { name: 'Light' }).click();
  await page.getByRole('link', { name: 'Search', exact: true }).click().catch(() => undefined);

  // 용골 (kengdic-only word, 'keel' also in 우리말샘): hanja 龍骨 from 표준국어대사전
  await openEntry(page, '용골');
  const marker = page.locator('.entry-head .hanja-src');
  await expect(marker).toHaveCount(1);
  await expect(marker).toHaveText('표준');
  await expect(marker).toHaveAttribute('title', /표준국어대사전/);
  await expect(page.locator('.entry-head .hanja-chars')).toHaveText('龍骨');
  // the marker is small and does not wrap the header onto many lines
  const box = await page.locator('.entry-head').boundingBox();
  expect(box!.height).toBeLessThan(150);
  await shot(page, 'hanja-fill-phone');

  // the filled hanja feeds the Chars tab like any other
  await page.getByRole('tab', { name: /Chars/ }).click();
  await expect(page.locator('.etab-panel')).toContainText('龍');

  // an entry whose dictionary gives the hanja itself (krdict 사과 etc.) has no marker; a native word has no hanja at all
  await openEntry(page, '학교');
  await expect(page.locator('.entry-head .hanja-src')).toHaveCount(0);
  await openEntry(page, '하늘');
  await expect(page.locator('.entry-head .hanja-chars')).toHaveCount(0);
  await expect(page.locator('.entry-head .hanja-src')).toHaveCount(0);

  // ---- iPad ----
  await openEntry(page, '용골');
  await page.setViewportSize({ width: 1180, height: 820 });
  await page.waitForTimeout(500);
  await expect(page.locator('.entry-head .hanja-src')).toHaveText('표준');
  await shot(page, 'hanja-fill-ipad');
});
