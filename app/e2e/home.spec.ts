import { downloadAll, expect, test, type Page, texts } from './fixtures';
import { mkdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

const SHOTS = join(process.cwd(), '..', 'docs', 'screenshots');
mkdirSync(SHOTS, { recursive: true });
const shot = (page: Page, name: string) => page.screenshot({ path: join(SHOTS, `${name}.png`) });

/** The core pack has the `curriculum` table (the pipeline's, or scripts/e2e-curriculum.py's sample rows in local data). */
const HAS_CURRICULUM = (() => {
  try {
    const m = JSON.parse(readFileSync(join(process.cwd(), 'public', 'data', 'manifest.json'), 'utf8'));
    return (m.packs?.find((p: { id: string }) => p.id === 'core')?.counts?.curriculum ?? 0) > 0;
  } catch { return false; }
})();

const pager = (page: Page) => page.getByRole('region', { name: /^Today/ });
const slides = (page: Page) => pager(page).getByRole('group', { name: /^\d of \d$/ }).filter({ has: page.locator('.today-card') });
const titles = (page: Page) => texts(page.locator('.today-card .card-hw .hangul'));
const kinds = (page: Page) => page.locator('.today-card').evaluateAll((els) => els.map((e) => e.getAttribute('data-kind')));
const dateRe = /^Today\s*·\s*\d{1,2} [A-Z][a-z]{2}$/;

test('home: one framed card per page, six tabs, search bar only on the Search tab', async ({ page }) => {
  test.setTimeout(600_000);
  await page.goto('/');
  await downloadAll(page);
  await page.getByRole('link', { name: 'Settings' }).click();
  await page.getByRole('radio', { name: 'Light' }).click();
  await page.goto('/#/');

  // ---- the app opens on the Home tab, first of six; no search bar, no Recent block ----
  const nav = page.getByRole('navigation', { name: 'Main' });
  await expect(nav.getByRole('link')).toHaveText(['Home', 'Search', 'Reader', 'Games', 'Lists', 'Settings']);
  await expect(nav.getByRole('link', { name: 'Home' })).toHaveAttribute('aria-current', 'page');
  await expect(page.getByRole('heading', { level: 1 })).toHaveText(dateRe);
  await expect(page.getByRole('searchbox', { name: 'Search' })).toHaveCount(0);
  await expect(page.getByRole('heading', { name: 'Recent' })).toHaveCount(0);
  for (const l of await nav.getByRole('link').all()) {   // all six labels fit (none is cut off)
    const fits = await l.locator('span').evaluate((s) => s.scrollWidth <= s.clientWidth + 1 && s.getBoundingClientRect().right <= (s.closest('a') as HTMLElement).getBoundingClientRect().right + 1);
    expect(fits).toBe(true);
  }
  await nav.screenshot({ path: join(SHOTS, 'tabbar-phone.png') });

  // ---- pages: idiom first, then 3 words, 2 grammar patterns (6 with a curriculum table, else just the idiom) ----
  await expect(pager(page)).toBeVisible();
  const n = HAS_CURRICULUM ? 6 : 1;
  await expect(slides(page)).toHaveCount(n);
  expect(await kinds(page)).toEqual(HAS_CURRICULUM ? ['idiom', 'word', 'word', 'word', 'grammar', 'grammar'] : ['idiom']);
  const idiom = page.locator('.today-card[data-kind=idiom]');
  await expect(idiom.locator('.card-kind')).toHaveText(/^(Idiom 관용구|Proverb 속담)$/);
  await expect(idiom.locator('.cur-lvl')).toHaveCount(0);           // idioms have no curriculum level
  await expect(idiom.locator('.card-gloss')).not.toBeEmpty();
  await expect(idiom.locator('.card-exs .ex-ko').first()).toBeVisible();
  await expect(idiom.getByRole('link', { name: 'Open entry' })).toBeVisible();
  // fixed sections in a fixed order; an empty block is left out, never shown empty
  const order = (card: import('@playwright/test').Locator) => card.locator('.card-label').evaluateAll((els) => els.map((e) => e.textContent));
  const SECTIONS = ['Definition', 'Examples', 'Related'];
  const isOrdered = (got: (string | null)[]) => got.every((x, i) => SECTIONS.indexOf(x!) >= 0 && (i === 0 || SECTIONS.indexOf(x!) > SECTIONS.indexOf(got[i - 1]!)));
  expect(isOrdered(await order(idiom))).toBe(true);
  // the card sits inside the page with margins, and the button belongs to it
  const cb = (await idiom.boundingBox())!, ob = (await idiom.getByRole('link', { name: 'Open entry' }).boundingBox())!;
  expect(cb.x).toBeGreaterThanOrEqual(12);
  expect(ob.y + ob.height).toBeLessThanOrEqual(cb.y + cb.height);
  // the first page fills the space between header and tab bar
  const track = pager(page).locator('.car-track');
  const tb = (await track.boundingBox())!, hb = (await page.locator('.topbar').first().boundingBox())!, nb = (await nav.boundingBox())!;
  expect(tb.y).toBeLessThanOrEqual(hb.y + hb.height + 1);
  expect(tb.y + tb.height).toBeLessThanOrEqual(nb.y + 1);
  expect(tb.height).toBeGreaterThan(400);
  await page.waitForTimeout(300);
  await shot(page, 'home-phone-idiom');

  if (!HAS_CURRICULUM) {   // current packs: no placeholder pages, no pager controls
    await expect(pager(page).getByRole('button', { name: /^\d of \d$/ })).toHaveCount(0);
    await shot(page, 'home-phone');
  } else {
    for (let i = 0; i < 6; i++) await expect(pager(page).getByRole('group', { name: `${i + 1} of 6` }).first()).toBeAttached();
    // level labels name the curriculum, never plain "TOPIK n"
    const lv = page.locator('.today-card .cur-lvl');
    await expect(lv).toHaveCount(5);
    for (const t of await texts(lv)) expect(t).toMatch(/^[56]급 · 표준 교육과정$/);
    await expect(lv.first()).toHaveAttribute('title', /^National standard Korean curriculum level [56] \(≈ TOPIK [56]\)$/);
    expect(await pager(page).innerText()).not.toMatch(/TOPIK/);

    const first = await titles(page);
    const prev = pager(page).getByRole('button', { name: 'Previous page' });
    const next = pager(page).getByRole('button', { name: 'Next page' });
    const dots = pager(page).getByRole('button', { name: /^\d of 6$/ });
    await expect(dots).toHaveCount(6);
    await expect(prev).toBeDisabled();
    await expect(dots.nth(0)).toHaveAttribute('aria-current', 'true');
    await next.click();
    await expect(dots.nth(1)).toHaveAttribute('aria-current', 'true');
    await expect(prev).toBeEnabled();
    // a word page: badge, headword, gloss, Korean definition, examples, Open entry
    const w = page.locator('.today-card[data-kind=word]').first();
    await expect(w.locator('.card-gloss')).not.toBeEmpty();
    await expect(w.locator('.card-kind')).toHaveText('Word');
    await expect(w.locator('.card-exs .ex-ko').first()).toBeVisible();
    expect(await w.locator('.card-exs .ex').count()).toBeLessThanOrEqual(3);
    for (const c of await page.locator('.today-card').all()) expect(isOrdered(await order(c))).toBe(true);
    // every card's header strip is one line: kind, level, star
    for (const c of await page.locator('.today-card .card-head').all()) expect((await c.boundingBox())!.height).toBeLessThan(60);
    await expect(w.getByRole('link', { name: 'Open entry' })).toBeVisible();
    await page.waitForTimeout(400);
    await shot(page, 'home-phone');
    await dots.nth(4).click();
    await expect(dots.nth(4)).toHaveAttribute('aria-current', 'true');
    await expect(page.locator('.today-card[data-kind=grammar]').first()).toBeInViewport({ ratio: 0.9 });
    await expect(page.locator('.today-card[data-kind=grammar]').first().locator('.card-gloss')).not.toBeEmpty();
    await page.waitForTimeout(400);
    await shot(page, 'home-phone-grammar');
    await dots.nth(5).click();
    await expect(next).toBeDisabled();   // the last page simply ends
    await track.focus();
    await page.keyboard.press('ArrowLeft');
    await expect(dots.nth(4)).toHaveAttribute('aria-current', 'true');
    await page.keyboard.press('ArrowRight');
    await expect(dots.nth(5)).toHaveAttribute('aria-current', 'true');
    // mouse drag (touch swipes natively; a desktop pointer drags the track)
    await dots.nth(0).click();
    await expect(dots.nth(0)).toHaveAttribute('aria-current', 'true');
    const box = (await track.boundingBox())!;
    await page.mouse.move(box.x + box.width - 30, box.y + 200);
    await page.mouse.down();
    await page.mouse.move(box.x + box.width - 120, box.y + 200, { steps: 6 });
    await page.mouse.move(box.x + 20, box.y + 200, { steps: 6 });
    await page.mouse.up();
    await expect(dots.nth(1)).toHaveAttribute('aria-current', 'true');
    expect(page.url()).not.toMatch(/#\/(entry|word)\//);   // a drag is not a tap

    // ---- the save star ----
    const star = w.getByRole('button', { name: /^Save to / });
    await star.click();
    await expect(w.getByRole('button', { name: /^Remove from / })).toBeVisible();
    await w.getByRole('button', { name: /^Remove from / }).click();

    // ---- the headword and "Open entry" open the entry; Back returns to Home, again on page 1 ----
    await w.locator('.card-link').click();
    await expect(page).toHaveURL(/#\/entry\//);
    await expect(page.locator('h1')).toContainText(first[1]);
    await expect(nav.getByRole('link', { name: 'Home' })).toHaveAttribute('aria-current', 'page');
    await page.goBack();
    await expect(pager(page)).toBeVisible();
    await expect(page).toHaveURL(/#\/$/);
    await expect(dots.nth(0)).toHaveAttribute('aria-current', 'true');
    await dots.nth(2).click();
    await page.locator('.today-card[data-kind=word]').nth(1).getByRole('link', { name: 'Open entry' }).click();
    await expect(page).toHaveURL(/#\/entry\//);
    await page.goBack();

    // ---- the same pages for the same date, others on other dates ----
    await page.clock.setFixedTime(new Date(2026, 2, 5, 12, 0, 0));
    await page.reload();
    await expect(slides(page)).toHaveCount(6);
    const mar5 = await titles(page);
    await page.clock.setFixedTime(new Date(2026, 2, 5, 23, 59, 0));
    await page.reload();
    await expect(slides(page)).toHaveCount(6);
    expect(await titles(page)).toEqual(mar5);
    await expect(page.getByRole('heading', { level: 1 })).toHaveText('Today · 5 Mar');
    await page.clock.setFixedTime(new Date(2026, 2, 6, 0, 1, 0));
    await page.reload();
    await expect(slides(page)).toHaveCount(6);
    expect(await titles(page)).not.toEqual(mar5);
    await page.clock.resume();
  }

  // ---- the Search tab: Recent only, search bar low on the screen with a gap above the tab bar ----
  await nav.getByRole('link', { name: 'Search' }).click();
  await expect(page.getByRole('heading', { name: 'Recent' })).toBeVisible();
  await expect(pager(page)).toHaveCount(0);
  const bar = page.getByRole('searchbox', { name: 'Search' });
  const vh = page.viewportSize()!.height;
  const barGap = async () => {
    const bb = (await page.locator('form.searchbar').boundingBox())!, nb2 = (await nav.boundingBox())!;
    expect(bb.y).toBeGreaterThan(vh * 0.7);              // lower part of the screen
    const gap = nb2.y - (bb.y + bb.height);              // above the tab bar, not under it or flush on it
    expect(gap).toBeGreaterThanOrEqual(12);
    expect(gap).toBeLessThanOrEqual(20);
  };
  await barGap();
  await page.waitForTimeout(300);
  await shot(page, 'search-phone');
  await bar.fill('학교');
  await expect(page.locator('.row .hw', { hasText: '학교' }).first()).toBeVisible();
  await barGap();                                        // the results list keeps it
  await page.waitForTimeout(300);
  await shot(page, 'search-phone-results');
  // a keyboard shrinks the viewport: the bar stays whole, right above it, and the tab bar gets out of the way
  await bar.focus();
  await page.setViewportSize({ width: 390, height: 500 });
  await page.waitForTimeout(300);
  const kb = (await bar.boundingBox())!;
  expect(kb.y).toBeGreaterThanOrEqual(0);
  expect(kb.y + kb.height).toBeLessThanOrEqual(500);
  expect(kb.y + kb.height).toBeGreaterThan(500 - 80);  // sits at the bottom edge of what is left
  await expect(nav).toBeHidden();
  await shot(page, 'search-phone-keyboard');
  await page.setViewportSize({ width: 390, height: 844 });
  await expect(nav).toBeVisible();
  // ---- no search bar anywhere else: an entry (even one opened from the results), Lists, Settings, Home ----
  await bar.blur();
  await page.locator('a.row', { has: page.locator('.hw', { hasText: '학교' }) }).first().click();
  await expect(page).toHaveURL(/#\/(entry|word)\//);
  await expect(page.locator('h1')).toContainText('학교');
  await expect(bar).toHaveCount(0);
  await expect(nav.getByRole('link', { name: 'Search' })).toHaveAttribute('aria-current', 'page');
  await nav.getByRole('link', { name: 'Search' }).click();     // the Search tab brings the results (and the bar) back
  await expect(page).toHaveURL(/q=/);
  await expect(bar).toHaveValue('학교');
  await bar.fill('');
  await expect(page.getByRole('heading', { name: 'Recent' })).toBeVisible();
  for (const tab of ['Lists', 'Settings', 'Reader', 'Games', 'Home']) {
    await nav.getByRole('link', { name: tab }).click();
    await expect(nav.getByRole('link', { name: tab })).toHaveAttribute('aria-current', 'page');
    await expect(bar).toHaveCount(0);
  }
  await expect(pager(page)).toBeVisible();
});

test('home: iPad two-pane and dark mode screenshots', async ({ page }) => {
  test.setTimeout(600_000);
  await page.setViewportSize({ width: 1180, height: 820 });
  await page.goto('/');
  await downloadAll(page);
  await page.getByRole('link', { name: 'Settings' }).click();
  await page.getByRole('radio', { name: 'Light' }).click();
  await page.getByRole('link', { name: 'Home' }).click();
  await expect(page.locator('.pane-left').getByRole('heading', { level: 1 })).toHaveText(dateRe);
  await expect(page.locator('.pane-left .today-card[data-kind=idiom]')).toBeVisible();
  await expect(page.locator('.pane-left').getByRole('button', { name: 'Next page' })).toHaveCount(HAS_CURRICULUM ? 1 : 0);
  await page.waitForTimeout(500);
  if (HAS_CURRICULUM) await page.locator('.pane-left').getByRole('button', { name: 'Next page' }).click();
  await page.waitForTimeout(500);
  await shot(page, 'home-ipad');
  // a card opens its entry in the right pane; the left pane stays on Home
  await page.locator('.pane-left .today-card .card-link').first().click();
  await expect(page.locator('.pane-right h1')).toBeVisible();
  await expect(page.locator('.pane-left .today-card').first()).toBeVisible();
  await expect(page.getByRole('link', { name: 'Home' })).toHaveAttribute('aria-current', 'page');
  // Search tab: Recent only, search bar stays at the top of the left pane
  await page.getByRole('link', { name: 'Search' }).click();
  await expect(page.locator('.pane-left').getByRole('heading', { name: 'Recent' })).toBeVisible();
  await expect(page.locator('.pane-left .today-card')).toHaveCount(0);
  const sb = (await page.locator('.pane-left').getByRole('searchbox', { name: 'Search' }).boundingBox())!;
  expect(sb.y).toBeLessThan(120);
  await page.waitForTimeout(300);
  await shot(page, 'search-ipad');

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/#/settings');
  await page.getByRole('radio', { name: 'Dark' }).click();
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');
  await page.goto('/#/');
  await expect(page.locator('.today-card[data-kind=idiom]')).toBeVisible();
  if (HAS_CURRICULUM) {
    await page.getByRole('button', { name: 'Next page' }).click();
    await expect(page.getByRole('button', { name: '2 of 6' })).toHaveAttribute('aria-current', 'true');
  }
  await page.waitForTimeout(500);
  await shot(page, 'home-dark');
});
