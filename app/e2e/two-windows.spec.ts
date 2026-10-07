import { expect, test, type Page } from '@playwright/test';

async function search(page: Page, q: string) {
  await page.getByRole('searchbox', { name: 'Search' }).fill(q);
  await expect(page.locator(`.page[data-q="${q}"][data-busy="0"]`)).toBeVisible({ timeout: 30_000 });
}

// An installed app and a browser tab of the same site share the dictionary files; whichever one
// the user is in must get them, without closing the other.
test('two windows of the app hand the engine over', async ({ context }) => {
  test.setTimeout(600_000);
  const a = await context.newPage();
  await a.goto('/');
  await a.getByRole('button', { name: 'Download' }).click();
  await expect(a.getByRole('searchbox', { name: 'Search' })).toBeVisible({ timeout: 240_000 });
  await search(a, '학교');
  await expect(a.locator('.row .hw').first()).toHaveText('학교');

  const b = await context.newPage();
  await b.goto('/');
  await b.bringToFront();
  await search(b, '천지');
  await expect(b.locator('.row .hw').first()).toHaveText('천지');

  await a.bringToFront();
  await search(a, '사랑');
  await expect(a.locator('.row .hw').first()).toHaveText('사랑');
  await b.bringToFront();
  await search(b, '먹다');
  await expect(b.locator('.row .hw').first()).toHaveText('먹다');
});
