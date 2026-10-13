import { downloadAll, expect, test } from './fixtures';

// Phone: Back while typing in the search box only closes the keyboard; the next Back navigates.
test.use({ viewport: { width: 390, height: 844 }, isMobile: true, hasTouch: true });

test('back closes the keyboard first, then goes back', async ({ page }) => {
  await page.goto('/');
  await downloadAll(page);
  const box = page.getByRole('searchbox', { name: 'Search' });
  await box.focus();
  await box.pressSequentially('school', { delay: 30 });
  await page.waitForURL((u) => u.hash.includes('q=school'));
  await expect(page.locator('a.row').first()).toBeVisible();
  await expect(box).toBeFocused();

  await page.evaluate(() => history.back());
  await expect(box).not.toBeFocused();
  await page.waitForTimeout(400);
  expect(page.url()).toContain('q=school');           // still on the results
  await expect(box).toHaveValue('school');
  await expect(page.locator('a.row').first()).toBeVisible();

  await page.evaluate(() => history.back());           // keyboard closed: now Back navigates
  await page.waitForURL((u) => !u.hash.includes('q='));
  await expect(box).toHaveValue('');
});
