import { downloadAll, expect, test, type Page } from './fixtures';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

/** Real touch on the Home pager. The pager is a native scroll-snap track; each card's body scrolls vertically
 *  inside it. A finger swipe goes through the browser's own gesture handling (compositor scrolling, scroll
 *  chaining, overscroll-behavior, touch-action), which a mouse drag never touches: the mouse-drag test in
 *  home.spec.ts passed while a sideways swipe on a card did nothing on the phone (the card body's
 *  `overscroll-behavior: contain` swallowed the horizontal part). So these gestures are genuine touch
 *  events sent through CDP, as a finger on Android sends them. */
test.use({ viewport: { width: 390, height: 844 }, hasTouch: true, isMobile: true });

const HAS_CURRICULUM = (() => {
  try {
    const m = JSON.parse(readFileSync(join(process.cwd(), 'public', 'data', 'manifest.json'), 'utf8'));
    return (m.packs?.find((p: { id: string }) => p.id === 'core')?.counts?.curriculum ?? 0) > 0;
  } catch { return false; }
})();

/** A finger from (x0,y0) to (x1,y1) in `steps` moves ~16 ms apart, then lifted. */
async function swipe(page: Page, x0: number, y0: number, x1: number, y1: number, steps = 12) {
  const cdp = await page.context().newCDPSession(page);
  await cdp.send('Input.dispatchTouchEvent', { type: 'touchStart', touchPoints: [{ x: x0, y: y0 }] });
  for (let i = 1; i <= steps; i++) {
    const t = i / steps;
    await cdp.send('Input.dispatchTouchEvent', { type: 'touchMove', touchPoints: [{ x: x0 + (x1 - x0) * t, y: y0 + (y1 - y0) * t }] });
    await page.waitForTimeout(16);
  }
  await cdp.send('Input.dispatchTouchEvent', { type: 'touchEnd', touchPoints: [] });
  await cdp.detach();
}

test('home: a finger swipe on a card changes page; a vertical one scrolls the card', async ({ page }) => {
  test.skip(!HAS_CURRICULUM, 'needs several pages (curriculum table)');
  test.setTimeout(600_000);
  await page.goto('/');
  await downloadAll(page);
  await page.goto('/#/');
  const pager = page.getByRole('region', { name: /^Today/ });
  const dots = pager.getByRole('button', { name: /^\d of 7$/ });
  const track = pager.locator('.car-track');
  await expect(dots).toHaveCount(7);
  await expect(dots.nth(0)).toHaveAttribute('aria-current', 'true');
  const card = (i: number) => page.locator('.today-card').nth(i);
  await expect(card(0).locator('.entry-inline .sense').first()).toBeVisible();
  await page.waitForTimeout(300);

  /** a point on plain text in the middle of the card body on screen (the gloss or a sense, never a link or button) */
  const onText = async (i: number) => {
    const body = card(i).locator('.card-body');
    const bb = (await body.boundingBox())!;
    const p = await body.evaluate((b) => {
      const r = b.getBoundingClientRect(), midY = r.top + r.height / 2;
      const cands = [...b.querySelectorAll<HTMLElement>('.card-gloss, .sense p, .sense, .def, .ex-en, .ex-ko')]
        .filter((e) => !e.closest('a, button, summary, .chip'));
      let best: { x: number; y: number; d: number } | null = null;
      for (const e of cands) {
        const er = e.getBoundingClientRect();
        if (er.bottom < r.top + 8 || er.top > r.bottom - 8 || er.height < 8) continue;
        const y = Math.max(r.top + 8, Math.min(r.bottom - 8, er.top + Math.min(er.height, 20) / 2));
        const d = Math.abs(y - midY);
        if (!best || d < best.d) best = { x: r.left + r.width / 2, y, d };
      }
      return best;
    });
    expect(p, 'a text point inside the card body').not.toBeNull();
    // the gesture really starts inside the card body, on something that is not interactive
    const hit = await page.evaluate(({ x, y }) => {
      const e = document.elementFromPoint(x, y) as HTMLElement | null;
      return { inBody: !!e?.closest('.card-body'), interactive: !!e?.closest('a, button, summary') };
    }, p!);
    expect(hit).toEqual({ inBody: true, interactive: false });
    expect(p!.y).toBeGreaterThan(bb.y);
    return p!;
  };
  const settled = async () => {
    // wait until the track stops moving and sits exactly on a page
    await expect.poll(async () => {
      const a = await track.evaluate((t) => t.scrollLeft);
      await page.waitForTimeout(120);
      const b = await track.evaluate((t) => t.scrollLeft);
      const w = await track.evaluate((t) => t.clientWidth);
      return a === b && Math.abs(b / w - Math.round(b / w)) < 0.02;
    }, { timeout: 5_000 }).toBe(true);
  };

  // ---- a left swipe in the middle of the first card's body goes to page 2 ----
  let p = await onText(0);
  await swipe(page, p.x + 120, p.y, p.x - 140, p.y + 6);
  await settled();
  await expect(dots.nth(1)).toHaveAttribute('aria-current', 'true');
  expect(page.url()).toMatch(/#\/$/);                     // a swipe is not a tap

  // ---- and another, to page 3 (a word card, whose body is long) ----
  p = await onText(1);
  await swipe(page, p.x + 120, p.y, p.x - 140, p.y - 8);
  await settled();
  await expect(dots.nth(2)).toHaveAttribute('aria-current', 'true');

  // ---- a vertical swipe scrolls the card body and stays on the page ----
  const body = card(2).locator('.card-body');
  await expect(card(2).locator('.entry-inline .sense').first()).toBeVisible();
  expect(await body.evaluate((b) => b.scrollHeight > b.clientHeight + 100)).toBe(true);
  const left0 = await track.evaluate((t) => t.scrollLeft);
  p = await onText(2);
  await swipe(page, p.x, p.y + 150, p.x + 10, p.y - 150);
  await expect.poll(() => body.evaluate((b) => b.scrollTop)).toBeGreaterThan(100);
  await settled();
  await expect(dots.nth(2)).toHaveAttribute('aria-current', 'true');
  expect(await track.evaluate((t) => t.scrollLeft)).toBe(left0);

  // ---- a right swipe goes back, even from a scrolled body ----
  p = await onText(2);
  await swipe(page, p.x - 120, p.y, p.x + 140, p.y + 6);
  await settled();
  await expect(dots.nth(1)).toHaveAttribute('aria-current', 'true');

  // ---- taps still work: the star saves, the arrows page, the headword opens the entry ----
  const star = card(1).getByRole('button', { name: /^Save to / });
  await star.tap();
  await expect(card(1).getByRole('button', { name: /^Remove from / })).toBeVisible();
  await card(1).getByRole('button', { name: /^Remove from / }).tap();
  await expect(card(1).getByRole('button', { name: /^Save to / })).toBeVisible();
  await pager.getByRole('button', { name: 'Next page' }).tap();
  await expect(dots.nth(2)).toHaveAttribute('aria-current', 'true');
  await pager.getByRole('button', { name: 'Previous page' }).tap();
  await expect(dots.nth(1)).toHaveAttribute('aria-current', 'true');
  await settled();
  await card(1).locator('.card-link').tap();
  await expect(page).toHaveURL(/#\/entry\//);
});
