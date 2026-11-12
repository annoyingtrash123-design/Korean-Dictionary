import { downloadAll, expect, test } from './fixtures';

/** Settings: each dictionary shows when it was updated (not a fingerprint), Data says whether
 *  anything needs updating, and unexpected errors land in an on-device log that can be copied/cleared. */
test('settings: update dates, update status and the error log (phone + iPad)', async ({ page }) => {
  test.setTimeout(300_000);
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/');
  await downloadAll(page);
  await page.evaluate(() => localStorage.setItem('kd.settings.open', '["dictionaries","data"]'));
  await page.goto('/#/settings');

  const today = new Date();
  const p = (n: number) => String(n).padStart(2, '0');
  const ymd = `${today.getFullYear()}-${p(today.getMonth() + 1)}-${p(today.getDate())}`;
  await expect(page.locator('.pack-line').first()).toContainText(`updated ${ymd}`);
  await expect(page.locator('.pack-line').first()).not.toContainText(/v[0-9a-f]{16}/);
  await expect(page.locator('dl.facts')).toContainText('Up to date');

  // an unexpected error is recorded on the device
  await page.evaluate(() => { setTimeout(() => { void Promise.reject(new Error('e2e: test error')); }, 0); });
  await page.getByText('Technical details').click();
  const log = page.locator('details.errlog');
  await log.locator('summary').click();
  await expect(log.locator('summary')).toHaveText(/Recent errors \([1-9]\d*\)/);
  await expect(log).toContainText('e2e: test error');
  await log.scrollIntoViewIfNeeded();
  await page.waitForTimeout(300);
  await page.screenshot({ path: '../docs/screenshots/settings-diag-phone.png' });
  await page.setViewportSize({ width: 1180, height: 820 });
  await page.waitForTimeout(400);
  await log.scrollIntoViewIfNeeded();
  await page.screenshot({ path: '../docs/screenshots/settings-diag-ipad.png' });

  await log.getByRole('button', { name: 'Clear' }).click();
  await expect(log.locator('summary')).toHaveText('Recent errors (0)');
  await expect(log).toContainText('No errors recorded.');
});
