import { downloadAll, expect, test, type Page } from './fixtures';

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
/** A new release: each pack's content version changes (as when its data changed), and the build stamp. */
const bump = (m: any, tag: string) => { m.version += tag; for (const p of m.packs) p.version = (p.version ?? m.version) + tag; };
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
  await downloadAll(page);
  await search(page, '학교');
  await expect(firstRow(page)).toHaveText('학교');

  // Corrupt download (wrong checksum): rejected, and the installed pack keeps working.
  await serveManifest(page, (m) => { bump(m, '-x'); for (const p of m.packs) p.sha256 = '0'.repeat(64); });
  await redownload(page);
  await expect(page.locator(':not(details.errlog) > :is(p,div,span)', { hasText: /checksum mismatch/i }).first()).toBeVisible({ timeout: 240_000 });
  await search(page, '학교');
  await expect(firstRow(page)).toHaveText('학교');
  await page.unroute('**/data/manifest.json*');

  // Good update: search keeps working while it imports, and afterwards on the new version.
  await serveManifest(page, (m) => bump(m, '-y'));
  await redownload(page);
  await page.waitForTimeout(300); // search while the import runs
  await search(page, '먹다');
  await expect(firstRow(page)).toHaveText('먹다');
  await page.goto('/#/settings');
  await page.getByText('Technical details').click();
  await expect(page.locator('details.diag > pre')).toContainText(/core \S+-y /, { timeout: 240_000 });
  await page.reload();
  await search(page, '사랑');
  await expect(firstRow(page)).toHaveText('사랑');

  // Updates are manual: opening the app neither checks for new data nor nags with a banner.
  let checks = 0;
  page.on('request', (r) => { if (r.url().includes('/data/manifest.json')) checks++; });
  await page.unroute('**/data/manifest.json*');
  await serveManifest(page, (m) => bump(m, '-z'));
  await page.goto('/');
  await page.reload();
  await expect(page.getByRole('searchbox', { name: 'Search' })).toBeVisible();
  await page.waitForTimeout(1500);
  expect(checks, 'manifest fetched on startup').toBe(0);
  await expect(page.getByText('Dictionary update available')).toHaveCount(0);
  // …but Settings → Check for update finds it.
  await page.goto('/#/settings');
  await page.getByRole('button', { name: 'Check for update' }).click();
  await expect(page.getByRole('button', { name: 'Update now' })).toBeVisible();
});
