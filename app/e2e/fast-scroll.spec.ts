import { downloadAll, expect, test, type Page } from './fixtures';

/** Long Reader texts get a draggable scroll bar: the text follows the thumb live, a bubble shows where you are. */
async function dragCheck(page: Page, shot: string) {
  const track = page.locator('.fs-track');
  await expect(track).toBeVisible();
  const thumb = (await page.locator('.fs-thumb').boundingBox())!;
  const tb = (await track.boundingBox())!;
  const sc = page.locator('.rd-scroll');
  const top0 = await sc.evaluate((e) => e.scrollTop);
  await page.mouse.move(thumb.x + thumb.width / 2, thumb.y + thumb.height / 2);
  await page.mouse.down();
  // half-way down the track
  await page.mouse.move(thumb.x + thumb.width / 2, tb.y + tb.height / 2, { steps: 8 });
  await expect(page.locator('.fs-bubble')).toHaveText(/^\d+%/);
  const mid = await sc.evaluate((e) => e.scrollTop / (e.scrollHeight - e.clientHeight));
  expect(mid).toBeGreaterThan(0.35);
  expect(mid).toBeLessThan(0.65);
  expect(await sc.evaluate((e) => e.scrollTop)).toBeGreaterThan(top0);
  await page.screenshot({ path: `../docs/screenshots/${shot}` });
  // to the very end
  await page.mouse.move(thumb.x + thumb.width / 2, tb.y + tb.height + 40, { steps: 4 });
  await page.mouse.up();
  await expect(page.locator('.fs-bubble')).toHaveCount(0);
  expect(await sc.evaluate((e) => Math.abs(e.scrollTop - (e.scrollHeight - e.clientHeight)))).toBeLessThan(4);
}

test('reader: draggable fast scroll (phone + iPad)', async ({ page }) => {
  test.setTimeout(600_000);
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/');
  await downloadAll(page);
  await page.getByRole('link', { name: 'Reader' }).click();
  await page.getByRole('button', { name: /Download Reader library/ }).click();
  await expect(page.locator('.rd-shelf-h .rd-fold').first()).toBeVisible({ timeout: 120_000 });

  await page.goto('/#/reader/mujeong');
  await expect(page.locator('.rd-title')).toBeVisible();
  // an excerpt says so at the top
  await expect(page.locator('.rd-excerpt')).toHaveText(/^Not the complete work · Chapters 1–20 of 126 .*original text, unaltered/);
  await expect(page.locator('.rd-card .eyebrow')).toContainText('part of the work');
  await page.screenshot({ path: '../docs/screenshots/reader-excerpt-phone.png' });
  // sections start collapsed; opened, the novel is long enough for the bar
  await page.getByRole('button', { name: 'Expand all' }).click();
  await dragCheck(page, 'fast-scroll-phone.png');
  // chapters show in the bubble
  await page.goto('/#/reader/constitution_rok');
  await page.getByRole('button', { name: 'Expand all' }).click();
  const tb = (await page.locator('.fs-track').boundingBox())!;
  await page.mouse.move(tb.x + tb.width / 2, tb.y + 30);
  await page.mouse.down();
  await page.mouse.move(tb.x + tb.width / 2, tb.y + tb.height * 0.6, { steps: 6 });
  await expect(page.locator('.fs-bubble')).toHaveText(/^\d+% · 제\d+[장절관]/);
  await page.mouse.up();
  // a short poem has no bar
  await page.goto('/#/reader/hayeoga');   // a four-line sijo: no bar
  await expect(page.locator('.rd-title')).toBeVisible();
  await expect(page.locator('.fs-track')).toHaveCount(0);

  await page.setViewportSize({ width: 1180, height: 820 });
  await page.goto('/#/reader/mujeong');
  await expect(page.locator('.rd-title')).toBeVisible();
  await page.getByRole('button', { name: 'Expand all' }).click();
  await page.locator('.rd-scroll').evaluate((e) => { e.scrollTop = 0; });
  await dragCheck(page, 'fast-scroll-ipad.png');
});
