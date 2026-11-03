import { downloadAll, expect, test } from './fixtures';

/** Settings sections are collapsible: all closed on a first visit, remembered, contents hidden when closed. */
test('settings: collapsible sections (phone + iPad)', async ({ page }) => {
  test.setTimeout(300_000);
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/');
  await downloadAll(page);
  await page.evaluate(() => localStorage.setItem('kd.settings.open', '[]'));   // a first visit
  await page.goto('/#/settings');
  const heads = page.locator('.set-fold');
  await expect(heads).toHaveText(['Appearance', 'Dictionaries', 'Data', 'Backup', 'About & licences']);
  for (const h of await heads.all()) await expect(h).toHaveAttribute('aria-expanded', 'false');
  await expect(page.getByRole('radiogroup', { name: 'Theme' })).toBeHidden();
  await page.waitForTimeout(300);
  await page.screenshot({ path: '../docs/screenshots/settings-collapsed-phone.png' });

  await page.getByRole('button', { name: 'Data' }).click();
  await expect(page.getByRole('button', { name: 'Data' })).toHaveAttribute('aria-expanded', 'true');
  await expect(page.getByText('Technical details')).toBeVisible();
  await page.getByRole('button', { name: 'Appearance' }).click();
  await expect(page.getByRole('radiogroup', { name: 'Korean font' })).toBeVisible();
  // remembered
  await page.reload();
  await expect(page.getByRole('button', { name: 'Data' })).toHaveAttribute('aria-expanded', 'true');
  await expect(page.getByRole('button', { name: 'Backup' })).toHaveAttribute('aria-expanded', 'false');
  await page.getByRole('button', { name: 'Appearance' }).click();
  await expect(page.getByRole('radiogroup', { name: 'Theme' })).toBeHidden();

  await page.setViewportSize({ width: 1180, height: 820 });
  await page.waitForTimeout(400);
  await page.screenshot({ path: '../docs/screenshots/settings-collapsed-ipad.png' });
});
