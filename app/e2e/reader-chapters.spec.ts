import { downloadAll, expect, test } from './fixtures';

/** Long texts with headings fold into chapters: one open at a time to start, remembered, expand/collapse all. */
test('reader: collapsible chapters (phone + iPad)', async ({ page }) => {
  test.setTimeout(600_000);
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/');
  await downloadAll(page);
  await page.getByRole('link', { name: 'Reader' }).click();
  await page.getByRole('button', { name: /Download Reader library/ }).click();
  await expect(page.locator('.rd-shelf-h .rd-fold').first()).toBeVisible({ timeout: 120_000 });

  await page.goto('/#/reader/constitution_1948');
  const heads = page.locator('.rd-sec-btn');
  await expect(heads.first()).toBeVisible();
  expect(await heads.count()).toBe(16);
  // first visit: only the first section is open
  await expect(heads.first()).toHaveAttribute('aria-expanded', 'true');
  await expect(page.locator('.rd-sec-btn[aria-expanded="true"]')).toHaveCount(1);
  const ch2 = page.locator('.rd-sec-btn', { hasText: '제2장' });
  await expect(ch2).toHaveAttribute('aria-expanded', 'false');
  await expect(ch2.locator('.rd-sec-n')).toHaveText(/\d+ ¶/);
  const before = await page.locator('.rd-ko').count();
  await ch2.click();
  await expect(ch2).toHaveAttribute('aria-expanded', 'true');
  expect(await page.locator('.rd-ko').count()).toBeGreaterThan(before);
  await page.waitForTimeout(300);
  await page.screenshot({ path: '../docs/screenshots/reader-chapters-phone.png' });

  // remembered across visits
  await page.goto('/#/reader');
  await page.goto('/#/reader/constitution_1948');
  await expect(page.locator('.rd-sec-btn', { hasText: '제2장' })).toHaveAttribute('aria-expanded', 'true');

  // expand all / collapse all
  await page.getByRole('button', { name: 'Expand all' }).click();
  await expect(page.locator('.rd-sec-btn[aria-expanded="false"]')).toHaveCount(0);
  await page.getByRole('button', { name: 'Collapse all' }).click();
  await expect(page.locator('.rd-sec-btn[aria-expanded="true"]')).toHaveCount(0);
  expect(await page.locator('.rd-ko').count()).toBeLessThan(before);

  // a short poem's version headings do not fold
  await page.goto('/#/reader/alsuopseoyo');
  await expect(page.locator('.rd-title')).toBeVisible();
  await expect(page.locator('.rd-sec-btn')).toHaveCount(0);

  await page.setViewportSize({ width: 1180, height: 820 });
  await page.goto('/#/reader/gwontae');
  await expect(page.locator('.rd-sec-btn').first()).toBeVisible();
  await page.waitForTimeout(400);
  await page.screenshot({ path: '../docs/screenshots/reader-chapters-ipad.png' });
});
