import { downloadAll, expect, test } from './fixtures';

/** Settings → Korean font: five choices; the choice applies to Korean text everywhere and is remembered. */
test('Korean font options', async ({ page }) => {
  test.setTimeout(300_000);
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/');
  await downloadAll(page);
  await page.goto('/#/settings');
  const group = page.getByRole('radiogroup', { name: 'Korean font' });
  await expect(group.getByRole('radio')).toHaveCount(5);
  await expect(group.getByRole('radio', { name: /System/ })).toHaveAttribute('aria-checked', 'true');
  const fam = () => page.evaluate(() => getComputedStyle(document.querySelector('.hangul')!).fontFamily);

  for (const [name, family] of [['Gowun Dodum', 'KD Gowun Dodum'], ['Nanum Myeongjo', 'KD Nanum Myeongjo'], ['Nanum Pen', 'KD Nanum Pen'], ['Gowun Batang', 'KD Gowun Batang']] as const) {
    await group.getByRole('radio', { name: new RegExp(name) }).click();
    await expect(group.getByRole('radio', { name: new RegExp(name) })).toHaveAttribute('aria-checked', 'true');
    await expect.poll(fam).toContain(family);
    // the bundled file really loads (offline-capable: served from the app, not the network)
    await expect.poll(() => page.evaluate((f) => document.fonts.load(`20px '${f}'`, '한글').then((l) => l.length), family)).toBeGreaterThan(0);
  }
  await page.locator('.fonts').scrollIntoViewIfNeeded();
  await page.waitForTimeout(300);
  await page.screenshot({ path: '../docs/screenshots/fonts-settings-phone.png' });

  // applies to entries and survives a reload
  await page.reload();
  await page.getByRole('searchbox', { name: 'Search' }).fill('학교');
  await page.locator('.row', { hasText: '학교' }).first().click();
  await expect(page.locator('h1')).toContainText('학교');
  expect(await page.evaluate(() => getComputedStyle(document.querySelector('.sense .ex-ko, .hangul')!).fontFamily)).toContain('KD Gowun Batang');
  await page.waitForTimeout(300);
  await page.screenshot({ path: '../docs/screenshots/fonts-entry-phone.png' });
  await page.setViewportSize({ width: 1180, height: 820 });
  await page.waitForTimeout(400);
  await page.screenshot({ path: '../docs/screenshots/fonts-entry-ipad.png' });

  // back to the system font
  await page.goto('/#/settings');
  await page.getByRole('radiogroup', { name: 'Korean font' }).getByRole('radio', { name: /System/ }).click();
  await expect.poll(fam).not.toContain('KD Gowun');
});
