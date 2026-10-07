import { downloadAll, expect, test, type Page } from './fixtures';
test.use({ serviceWorkers: 'block' });

test('reload during an update import keeps the old pack', async ({ page }) => {
  test.setTimeout(600_000);
  await page.goto('/');
  await downloadAll(page);
  await page.route('**/data/manifest.json*', async (route) => {
    const res = await route.fetch(); const m = await res.json(); m.version += '-y';
    await route.fulfill({ response: res, json: m });
  });
  await page.goto('/#/settings'); await page.reload();
  await expect(page.locator('dd', { hasText: /\d{8}-\d{4}$/ }).first()).toBeVisible();
  await page.getByRole('button', { name: 'Re-download' }).click();
  await page.locator('.installer .btn.primary').click();
  await expect(page.getByText(/Installing [1-9]/).first()).toBeVisible({ timeout: 120_000 });
  await page.reload(); // app killed mid-import
  await page.waitForTimeout(3000);
  await page.goto('/#/search?q=%ED%95%99%EA%B5%90');
  await expect(page.locator('.row .hw').first()).toHaveText('학교', { timeout: 30_000 });
});
