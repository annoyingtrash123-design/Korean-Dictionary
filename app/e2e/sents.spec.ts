import { downloadAll, expect, test, type Page } from './fixtures';
import type { Locator } from '@playwright/test';

/** Tap the centre of `word` inside element `el` (viewport coordinates via a DOM Range). */
async function tapWord(page: Page, el: Locator, word: string) {
  await el.scrollIntoViewIfNeeded();
  const p = await el.evaluate((root, w) => {
    const t = root.firstChild as Text; const i = t.data.indexOf(w);
    const r = document.createRange(); r.setStart(t, i); r.setEnd(t, i + w.length);
    const b = r.getBoundingClientRect(); return { x: b.left + b.width / 2, y: b.top + b.height / 2 };
  }, word);
  await page.mouse.click(p.x, p.y);
}
const hlText = (page: Page) => page.evaluate(() => {
  const h = (CSS as unknown as { highlights: Map<string, Set<Range>> }).highlights.get('tap-word');
  return h ? [...h][0].toString() : null;
});

/** A kengdic-only word (no translated sentence) gets real krdict example sentences in Sents. */
test('Sents tab: real Korean examples for a word without translated ones', async ({ page }) => {
  test.setTimeout(300_000);
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/');
  await downloadAll(page);
  await page.getByRole('searchbox', { name: 'Search' }).fill('희미해지다');
  await page.locator('.row', { hasText: '희미해지다' }).first().click();
  await expect(page.locator('h1')).toContainText('희미해지다');
  await page.getByRole('tab', { name: 'Sents', exact: true }).click();
  const ex = page.locator('.sents .ex');
  await expect(ex.first()).toBeVisible({ timeout: 20_000 });
  expect(await ex.count()).toBeGreaterThanOrEqual(2);
  // every row contains the word and is a sentence, not the bare headword
  for (const t of await page.locator('.sents .ex-ko').allTextContents()) {
    expect(t).toContain('희미해지');
    expect(t.trim().split(/\s+/).length).toBeGreaterThanOrEqual(2);
  }
  await page.waitForTimeout(300);
  await page.screenshot({ path: '../docs/screenshots/sents-krdict-phone.png' });

  // tap a word in a sentence: the Reader's lookup card, the word highlighted
  const sent = page.locator('.sents .ex-ko', { hasText: '정신을' });
  await tapWord(page, sent, '정신');
  const card = page.getByRole('dialog', { name: 'Dictionary lookup' });
  await expect(card).toBeVisible();
  await expect(card.locator('.lk-hw')).toHaveText('정신');
  await expect(card.locator('.lk-gloss li').first()).toBeVisible();
  expect(await hlText(page)).toContain('정신');   // the eojeol as written (정신을), as in the Reader
  await page.waitForTimeout(300);
  await page.screenshot({ path: '../docs/screenshots/tap-lookup-phone.png' });
  // ▶ moves to the next word
  await card.getByRole('button', { name: 'Next word' }).click();
  await expect(card.locator('.lk-hw')).toHaveText('겨우');
  expect(await hlText(page)).toContain('겨우');
  // tapping elsewhere closes it
  await page.locator('h1').click();
  await expect(card).toHaveCount(0);
  expect(await hlText(page)).toBeNull();

  await page.setViewportSize({ width: 1180, height: 820 });
  await page.waitForTimeout(500);
  await page.screenshot({ path: '../docs/screenshots/sents-krdict-ipad.png' });
  await tapWord(page, page.locator('.sents .ex-ko', { hasText: '정신을' }), '정신');
  await expect(card.locator('.lk-hw')).toHaveText('정신');
  // the card sits in the entry pane, not over the results list
  const cb = (await card.boundingBox())!, pb = (await page.locator('main#main').boundingBox())!;
  expect(cb.x).toBeGreaterThanOrEqual(pb.x - 1);
  await page.waitForTimeout(300);
  await page.screenshot({ path: '../docs/screenshots/tap-lookup-ipad.png' });
  // Open full entry navigates and closes the card
  await card.getByRole('link', { name: 'Open full entry' }).click();
  await expect(page.locator('h1')).toContainText('정신');
  await expect(card).toHaveCount(0);
});

test('Dict tab: example sentences and Korean definitions are tappable', async ({ page }) => {
  test.setTimeout(300_000);
  // record what the listen button would say (the test browser has no Korean voice)
  await page.addInitScript(() => {
    const w = window as unknown as { __said: string[] };
    w.__said = [];
    window.speechSynthesis.speak = (u: SpeechSynthesisUtterance) => { w.__said.push(u.text); };
  });
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/');
  await downloadAll(page);
  await page.getByRole('searchbox', { name: 'Search' }).fill('학교');
  await page.locator('.row', { hasText: '학교' }).first().click();
  await expect(page.locator('h1')).toContainText('학교');
  const ex = page.locator('.sense .ex-ko[data-tap]').first();
  await expect(ex).toBeVisible();
  const first = (await ex.textContent())!.trim().split(/\s+/)[0];
  await tapWord(page, ex, first);
  const card = page.getByRole('dialog', { name: 'Dictionary lookup' });
  await expect(card).toBeVisible();
  await expect(card).toHaveAttribute('data-loading', '0');
  expect(first).toContain((await card.locator('.lk-hw').textContent())!.slice(0, 1));
  await expect(page.locator('.def[data-tap]').first()).toBeAttached();
  await page.getByRole('button', { name: 'Close lookup' }).click();

  // examples are numbered, dialogues are speaker turns, and 🔊 reads the sentence aloud
  const sense1 = page.locator('.sense').first();
  const nums = await sense1.locator('.exs > .ex > .ex-n').allTextContents();
  expect(nums.slice(0, 3)).toEqual(['1', '2', '3']);
  await sense1.getByRole('button', { name: /Show \d+ more examples/ }).click();
  const turn = page.locator('.ex-turn').first();
  await expect(turn).toBeVisible();
  await expect(turn.locator('.ex-who')).toHaveText(/^[가-힣A-Z]{1,3}$/);
  await page.locator('.ex', { has: turn }).getByRole('button', { name: 'Listen' }).click();
  const said = await page.evaluate(() => (window as unknown as { __said: string[] }).__said);
  expect(said.length).toBe(1);
  expect(said[0]).not.toMatch(/(^|\s)[가나]\s*:/);   // speaker labels are not read out
  await turn.scrollIntoViewIfNeeded();
  await page.waitForTimeout(300);
  await page.screenshot({ path: '../docs/screenshots/examples-phone.png' });
  await page.setViewportSize({ width: 1180, height: 820 });
  await page.waitForTimeout(400);
  await page.screenshot({ path: '../docs/screenshots/examples-ipad.png' });
});

/** One reviewed AI translation per word: its English shows under the sentence with an AI tag. */
test('AI-translated example English is shown and labelled', async ({ page }) => {
  test.setTimeout(300_000);
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/');
  await downloadAll(page);
  await page.getByRole('searchbox', { name: 'Search' }).fill('맥주');
  await page.locator('.row', { hasText: '맥주' }).first().click();
  await expect(page.locator('h1')).toContainText('맥주');
  const ex = page.locator('.ex', { hasText: '김이 빠져서' });
  await expect(ex.locator('.ex-ai')).toHaveText('AI');
  await expect(ex.locator('.ex-en')).toContainText(/beer/i);
  await ex.scrollIntoViewIfNeeded();
  await page.waitForTimeout(300);
  await page.screenshot({ path: '../docs/screenshots/examples-ai-phone.png' });
  await page.setViewportSize({ width: 1180, height: 820 });
  await page.waitForTimeout(400);
  await ex.scrollIntoViewIfNeeded();
  await page.screenshot({ path: '../docs/screenshots/examples-ai-ipad.png' });
});

