import { expect, test, type Page } from '@playwright/test';

// The service worker would hide manifest requests from page.route().
test.use({ serviceWorkers: 'block' });

const firstRow = (page: Page) => page.locator('.row .hw').first();
async function search(page: Page, q: string) {
  await page.getByRole('searchbox', { name: 'Search' }).fill(q);
  await expect(page.locator(`.page[data-q="${q}"][data-busy="0"]`)).toBeVisible({ timeout: 30_000 });
}
const serveManifest = (page: Page, edit: (m: any) => void) =>
  page.route('**/data/manifest.json*', async (route) => {
    const res = await route.fetch();
    const m = await res.json();
    edit(m);
    await route.fulfill({ response: res, json: m });
  });
async function redownload(page: Page) {
  await page.goto('/#/settings');
  await page.reload();
  await expect(page.locator('dd', { hasText: /\d{8}-\d{4}/ }).first()).toBeVisible(); // status loaded
  await page.getByRole('button', { name: 'Re-download' }).click();
  await page.locator('.installer .btn.primary').click();
}

test('update keeps the old pack usable; corrupt download is rejected', async ({ page }) => {
  test.setTimeout(600_000);
  await page.goto('/');
  await page.getByRole('button', { name: 'Download' }).click();
  await expect(page.getByRole('searchbox', { name: 'Search' })).toBeVisible({ timeout: 240_000 });
  await search(page, '학교');
  await expect(firstRow(page)).toHaveText('학교');

  // Corrupt download (wrong checksum): rejected, and the installed pack keeps working.
  await serveManifest(page, (m) => { m.version += '-x'; for (const p of m.packs) p.sha256 = '0'.repeat(64); });
  await redownload(page);
  await expect(page.getByText(/checksum mismatch/i)).toBeVisible({ timeout: 240_000 });
  await search(page, '학교');
  await expect(firstRow(page)).toHaveText('학교');
  await page.unroute('**/data/manifest.json*');

  // Good update: search keeps working while it imports, and afterwards on the new version.
  await serveManifest(page, (m) => { m.version += '-y'; });
  await redownload(page);
  await page.waitForTimeout(300); // search while the import runs
  await search(page, '먹다');
  await expect(firstRow(page)).toHaveText('먹다');
  await page.goto('/#/settings');
  await expect(page.locator('dt:has-text("Installed version") + dd')).toHaveText(/-y$/, { timeout: 240_000 });
  await page.reload();
  await search(page, '사랑');
  await expect(firstRow(page)).toHaveText('사랑');
});
