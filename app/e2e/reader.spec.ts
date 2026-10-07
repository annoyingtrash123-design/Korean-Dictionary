import { expect, test, type Page } from '@playwright/test';
import { mkdirSync } from 'node:fs';
import { join } from 'node:path';

const SHOTS = join(process.cwd(), '..', 'docs', 'screenshots');
mkdirSync(SHOTS, { recursive: true });
const shot = (page: Page, name: string) => page.screenshot({ path: join(SHOTS, `reader-${name}.png`) });

/** Centre of characters [s, e) of paragraph n (viewport coordinates). */
const wordPoint = (page: Page, n: number, s: number, e: number) => page.evaluate(([n, s, e]) => {
  const root = document.querySelector(`[data-ko="${n}"]`)!;
  const w = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
  let pos = 0; const r = document.createRange(); let started = false;
  for (let t = w.nextNode() as Text | null; t; t = w.nextNode() as Text | null) {
    if (!started && s < pos + t.data.length) { r.setStart(t, s - pos); started = true; }
    if (started && e <= pos + t.data.length) { r.setEnd(t, e - pos); break; }
    pos += t.data.length;
  }
  const b = r.getBoundingClientRect();
  return { x: b.left + b.width / 2, y: b.top + b.height / 2 };
}, [n, s, e] as [number, number, number]);
const tap = async (page: Page, n: number, s: number, e: number) => { const p = await wordPoint(page, n, s, e); await page.mouse.click(p.x, p.y); };
const mark = (page: Page) => page.locator('mark.hl');

async function readerFlow(page: Page) {
  const root = page.locator('#main');
  await page.getByRole('link', { name: 'Reader' }).click();
  await expect(page.getByRole('heading', { name: /Reader/ })).toBeVisible();
  await expect(page.locator('.rd-row').first()).toBeVisible();
  await page.waitForTimeout(400);
  await shot(page, 'library');

  {
    await page.getByRole('tab', { name: 'Timeline' }).click();
    await expect(page.locator('.tl-band').first()).toBeVisible();
    await page.waitForTimeout(300);
    await shot(page, 'timeline');
    await page.getByRole('button', { name: /Filters/ }).click();
    await page.getByLabel('Script', { exact: true }).selectOption('hanmun');
    await expect(page.locator('.rd-row')).toHaveCount(1);
    await page.getByLabel('Script', { exact: true }).selectOption('');
    await page.getByRole('button', { name: /Filters/ }).click();
    await page.getByRole('tab', { name: 'Shelves' }).click();
  }

  // open a graded reader
  await page.locator('.rd-row', { hasText: '단군 이야기' }).click();
  await expect(root.locator('.rd-title')).toHaveText('단군 이야기');
  await expect(root.getByText('AI-generated translation — may contain errors')).toBeVisible();
  await expect(page.locator('.rd-en').first()).toBeVisible();
  await page.waitForTimeout(400);
  await shot(page, 'text-light');

  // tap a word: highlighted span + dictionary card
  await tap(page, 0, 0, 2);                      // 옛날
  await expect(page.locator('.lk')).toBeVisible();
  await expect(mark(page)).toHaveText('옛날');
  await expect(page.locator('.lk-hw')).toHaveText('옛날');
  await expect(page.locator('.lk-gloss li').first()).toBeVisible();
  await expect(page.locator('.lk[data-loading="0"]')).toBeVisible();
  // highlight is accent-tinted and the paragraph is not split into per-character elements
  expect(await page.locator('.rd-ko').first().evaluate((el) => el.children.length)).toBe(1);
  await page.waitForTimeout(300);
  await shot(page, 'popup-phone');

  // ▶ moves to the next word, ⇥ extends by a character, ⇤ shrinks again
  const first = await mark(page).innerText();
  await page.getByRole('button', { name: 'Next word' }).click();
  await expect(mark(page)).not.toHaveText(first);
  await expect(page.locator('.lk[data-loading="0"]')).toBeVisible();
  const second = await mark(page).innerText();
  await page.getByRole('button', { name: 'Previous word' }).click();
  await expect(mark(page)).toHaveText(first);
  await page.getByRole('button', { name: 'Next word' }).click();
  await expect(mark(page)).toHaveText(second);
  await page.getByRole('button', { name: /Extend selection/ }).click();
  await expect(mark(page)).toHaveText(await mark(page).innerText());
  await expect.poll(async () => (await mark(page).innerText()).length).toBeGreaterThan(second.length);
  await page.getByRole('button', { name: /Shrink selection/ }).click();
  await expect.poll(async () => (await mark(page).innerText()).length).toBe(second.length);

  // dismiss with Escape, then by tapping outside the words
  await page.keyboard.press('Escape');
  await expect(page.locator('.lk')).toHaveCount(0);
  await expect(mark(page)).toHaveCount(0);
  await tap(page, 1, 0, 1);
  await expect(page.locator('.lk')).toBeVisible();
  await page.locator('.rd-card .rd-title').click();
  await expect(page.locator('.lk')).toHaveCount(0);

  // English toggle
  await page.getByRole('button', { name: 'English', exact: true }).click();
  await expect(page.locator('.rd-en')).toHaveCount(0);
  await page.getByRole('button', { name: 'English', exact: true }).click();
  await expect(page.locator('.rd-en').first()).toBeVisible();

  // vocabulary + questions (answers revealed on tap)
  await expect(page.getByRole('heading', { name: /Vocabulary/ })).toBeVisible();
  await page.getByRole('button', { name: 'Show answer' }).first().click();
  await expect(page.locator('.rd-ans').first()).toBeVisible();

  // notes are rendered as elements (no raw markdown markers)
  await page.locator('.rd-notes > summary').click();
  await expect(page.locator('.rd-notes strong').first()).toBeVisible();
  expect(await page.locator('.rd-notes').innerText()).not.toContain('**');
  await page.waitForTimeout(300);
  await shot(page, 'notes');
  await page.locator('.rd-notes > summary').click();

  // reading position is remembered across a reload
  const max = await page.locator('.rd-scroll').evaluate((el) => el.scrollHeight - el.clientHeight);
  expect(max).toBeGreaterThan(300);
  const target = Math.min(500, max - 50);
  await page.locator('.rd-scroll').evaluate((el, y) => { el.scrollTop = y; }, target);
  await page.waitForTimeout(600);
  await page.reload();
  await expect(page.locator('.rd-title')).toHaveText('단군 이야기');
  await expect.poll(() => page.locator('.rd-scroll').evaluate((el) => el.scrollTop), { timeout: 10_000 }).toBeGreaterThan(target - 40);
  expect(await page.locator('.rd-scroll').evaluate((el) => el.scrollTop)).toBeLessThan(target + 40);
}

test('reader: library, timeline, tap lookup, resume (phone + iPad)', async ({ page }) => {
  test.setTimeout(900_000);
  await page.addInitScript(() => { try { localStorage.setItem('kd.fixture.texts', '1'); } catch { /* */ } });
  await page.goto('/');
  await page.getByRole('button', { name: 'Download' }).click();
  await expect(page.getByRole('searchbox', { name: 'Search' })).toBeVisible({ timeout: 240_000 });
  await page.getByRole('link', { name: 'Settings' }).click();
  await page.getByRole('radio', { name: 'Light' }).click();
  await expect(page.getByText('Reader library')).toBeVisible();

  await readerFlow(page);

  // verse + hanmun layouts, dark theme
  await page.goto('/#/reader/fixture-jindallae');
  await expect(page.locator('.rd-para.verse').first()).toBeVisible();
  expect(await page.locator('.rd-ko').first().evaluate((el) => getComputedStyle(el).whiteSpace)).toBe('pre-wrap');
  await page.goto('/#/reader/fixture-chuya');
  await expect(page.locator('.rd-reading').first()).toBeVisible();
  await tap(page, 0, 0, 1);                       // 秋: hanja lookup
  await expect(page.locator('.lk-chars')).toBeVisible();
  await page.waitForTimeout(500);
  await shot(page, 'popup-hanja');
  await page.keyboard.press('Escape');
  await page.getByRole('button', { name: 'Reading settings' }).click();
  await expect(page.getByRole('dialog', { name: 'Reading settings' })).toBeVisible();
  await page.waitForTimeout(200);
  await shot(page, 'settings');
  await page.getByRole('button', { name: 'Close' }).click();
  await page.getByRole('link', { name: 'Settings' }).click();
  await page.getByRole('radio', { name: 'Dark' }).click();
  await page.goto('/#/reader/graded-dangun');
  await expect(page.locator('.rd-title')).toBeVisible();
  await page.locator('.rd-scroll').evaluate((el) => { el.scrollTop = 0; });
  await page.waitForTimeout(400);
  await shot(page, 'text-dark');
  await page.goto('/#/reader');
  await page.waitForTimeout(300);
  await shot(page, 'library-dark');

  // iPad two-pane
  await page.setViewportSize({ width: 1180, height: 820 });
  await page.goto('/#/settings');
  await page.getByRole('radio', { name: 'Light' }).click();
  await expect(page.locator('.app.wide')).toBeVisible();
  await page.goto('/#/reader');
  await expect(page.getByText('Choose a text')).toBeVisible();
  await expect(page.locator('.pane-left .rd-row').first()).toBeVisible();
  await readerFlow2(page);
});

async function readerFlow2(page: Page) {
  await page.waitForTimeout(300);
  await shot(page, 'library-ipad');
  await page.locator('.pane-left .rd-row', { hasText: '단군 이야기' }).click();
  await expect(page.locator('.pane-right .rd-title')).toHaveText('단군 이야기');
  await expect(page.locator('.pane-left .rd-row[aria-current="true"]')).toHaveCount(1);
  await tap(page, 0, 0, 2);
  await expect(page.locator('.lk')).toBeVisible();
  await expect(mark(page)).toHaveText('옛날');
  await expect(page.locator('.lk-gloss li').first()).toBeVisible();
  // the card floats inside the text pane and the library on the left stays put
  const card = await page.locator('.lk').boundingBox();
  const pane = await page.locator('.pane-right').boundingBox();
  expect(card!.x).toBeGreaterThanOrEqual(pane!.x - 1);
  expect(card!.x + card!.width).toBeLessThanOrEqual(pane!.x + pane!.width + 1);
  await page.getByRole('button', { name: 'Next word' }).click();
  await expect(page.locator('.lk[data-loading="0"]')).toBeVisible();
  await page.waitForTimeout(300);
  await shot(page, 'popup-ipad');
  await page.getByRole('link', { name: 'Open full entry' }).click();
  await expect(page.locator('.pane-right h1')).toBeVisible();
  await expect(page.locator('.pane-left .rd-row').first()).toBeVisible();
  await page.goto('/#/reader');
}
