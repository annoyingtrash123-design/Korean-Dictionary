import { expect, test } from './fixtures';

// The first-run screen is taller than a phone screen: a user must be able to scroll to Download.
// (Playwright's click scrolls programmatically even where a finger cannot, so scroll with the wheel.)
test.use({ viewport: { width: 390, height: 640 }, isMobile: true, hasTouch: true });

test('first-run screen scrolls to the Download button', async ({ page }) => {
  await page.goto('/');
  const btn = page.getByRole('button', { name: 'Download' });
  await expect(btn).toBeAttached();
  for (let i = 0; i < 15; i++) {
    const box = await btn.boundingBox();
    if (box && box.y + box.height <= 640) break;
    await page.mouse.move(195, 320);
    await page.mouse.wheel(0, 400);
    await page.waitForTimeout(150);
  }
  const box = await btn.boundingBox();
  expect(box, 'Download button').not.toBeNull();
  expect(box!.y + box!.height).toBeLessThanOrEqual(640);
  expect(box!.y).toBeGreaterThanOrEqual(0);
});
