import { downloadAll, expect, test, type Page } from './fixtures';
import { mkdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

const SHOTS = join(process.cwd(), '..', 'docs', 'screenshots');
mkdirSync(SHOTS, { recursive: true });
const shot = (page: Page, name: string) => page.screenshot({ path: join(SHOTS, name + '.png') });

/** The core pack carries Wiktionary translation tables (`en_ko`, optional source). */
const HAS_EN_KO = (() => {
  try {
    const m = JSON.parse(readFileSync(join(process.cwd(), 'public', 'data', 'manifest.json'), 'utf8'));
    return (m.packs?.find((p: { id: string }) => p.id === 'core')?.counts?.en_ko ?? 0) > 0;
  } catch { return false; }
})();
const search = async (page: Page, q: string) => {
  await page.getByRole('searchbox', { name: 'Search' }).fill(q);
  await page.waitForURL((u) => u.hash.includes('q=') && decodeURIComponent(u.hash).includes(q));
};

/** Dict | Words | Chars | Sents: each tab renders for 학교, ?tab= drives Back/Forward, the choice is remembered. */
async function tabs(page: Page, scope: string, shotPrefix: string, shotFn: (p: Page, n: string) => Promise<unknown>) {
  const root = page.locator(scope);
  const tab = (n: string) => root.getByRole('tab', { name: n, exact: true });
  await expect(root.getByRole('tablist')).toBeVisible();
  await expect(tab('Dict')).toHaveAttribute('aria-selected', 'true');
  await expect(root.locator('details.dict').first()).toBeVisible();
  await tab('Words').click();
  await expect(page).toHaveURL(/tab=words/);
  await expect(tab('Words')).toHaveAttribute('aria-selected', 'true');
  await expect(root.locator('.list .row').first()).toBeVisible({ timeout: 20_000 });
  await expect(root.locator('details.dict')).toHaveCount(0);
  await page.waitForTimeout(300);
  await shotFn(page, `${shotPrefix}-words`);
  await tab('Chars').click();
  await expect(root.locator('.char .hanja-big').first()).toBeVisible({ timeout: 20_000 });
  await page.waitForTimeout(300);
  await shotFn(page, `${shotPrefix}-chars`);
  await tab('Sents').click();
  await expect(root.locator('.sents .ex, .etab-panel .muted').first()).toBeVisible({ timeout: 20_000 });
  await page.waitForTimeout(300);
  await shotFn(page, `${shotPrefix}-sents`);
  // keyboard: arrows move between tabs
  await tab('Sents').focus();
  await page.keyboard.press('ArrowLeft');
  await expect(tab('Chars')).toHaveAttribute('aria-selected', 'true');
  await page.keyboard.press('Home');
  await expect(tab('Dict')).toHaveAttribute('aria-selected', 'true');
  // back/forward follow the tab
  await page.goBack();
  await expect(tab('Chars')).toHaveAttribute('aria-selected', 'true');
  await page.goForward();
  await expect(tab('Dict')).toHaveAttribute('aria-selected', 'true');
  // the last tab is remembered across entries
  await tab('Words').click();
  await expect(tab('Words')).toHaveAttribute('aria-selected', 'true');
  await root.locator('.list .row').first().click();
  await expect(root.getByRole('tab', { name: 'Words', exact: true })).toHaveAttribute('aria-selected', 'true');
  await page.goBack();
  await root.getByRole('tab', { name: 'Dict', exact: true }).click();
  await expect(root.getByRole('tab', { name: 'Dict', exact: true })).toHaveAttribute('aria-selected', 'true');
}

async function tour(page: Page, theme: 'light' | 'dark') {
  await page.goto('/#/');
  await expect(page.getByRole('heading', { name: 'Recent' })).toBeVisible();
  await shot(page, `${theme}-home`);
  await search(page, '먹다');
  await expect(page.locator('.row .hw', { hasText: '먹다' }).first()).toBeVisible();
  await shot(page, `${theme}-results`);
  await search(page, 'eat');
  await expect(page.locator('.row .hw').first()).toBeVisible();
  await expect(page.locator('.pos-h', { hasText: 'Verbs' })).toHaveCount(1);
  await shot(page, `${theme}-results-eat`);
  // English search: Wiktionary translations block on top, phrases (collapsible) below the rows
  await search(page, 'report');
  await expect(page.locator('.row .hw').first()).toBeVisible();
  if (HAS_EN_KO) {
    await expect(page.getByRole('region', { name: 'Wiktionary translations' })).toBeVisible();
    await expect(page.locator('.enko .enko-words a').first()).toBeVisible();
    await page.waitForTimeout(300);
    await shot(page, `${theme}-results-report`);
    const phrases = page.getByRole('region', { name: 'English phrases', exact: true });
    if (await phrases.count()) {
      await phrases.scrollIntoViewIfNeeded();
      await page.waitForTimeout(200);
      await shot(page, `${theme}-results-report-phrases`);
    }
  }
  await search(page, '먹다');
  await page.locator('.row', { hasText: '먹다' }).first().click();
  await expect(page.locator('h1')).toContainText('먹다');
  await page.waitForTimeout(500);
  await shot(page, `${theme}-entry`);
  await page.getByRole('link', { name: 'Settings' }).click();
  await expect(page.getByRole('heading', { name: 'Appearance' })).toBeVisible();
  await shot(page, `${theme}-settings`);
}

test('first run, search, bookmarks, themes, offline persistence', async ({ page, context }) => {
  test.setTimeout(600_000);
  page.on('console', (m) => { if (m.text().startsWith('KDPERF')) console.log(m.text()); });
  page.on('pageerror', (e) => console.log('PAGEERROR', e.message));
  await page.goto('/');
  // ---- first-run download ----
  await expect(page.getByRole('heading', { name: 'Korean Dictionary' })).toBeVisible();
  await expect(page.getByRole('switch', { name: /표준국어대사전/ })).toBeChecked();
  await expect(page.getByRole('switch', { name: /우리말샘/ })).not.toBeChecked(); // large optional pack: default off
  await shot(page, 'light-firstrun');
  await downloadAll(page);

  // ---- searches ----
  await search(page, '학교');
  await expect(page.locator('.row .hw', { hasText: '학교' }).first()).toBeVisible();
  await expect(page.locator('.row').first()).toContainText(/school/i);

  await search(page, '갔어요');
  const t0 = Date.now();
  await expect(page.getByText('Did you mean')).toBeVisible({ timeout: 60_000 });
  console.log('deconj search ms', Date.now() - t0);
  await expect(page.locator('.row .hw', { hasText: '가다' }).first()).toBeVisible();

  await search(page, 'school');
  await expect(page.locator('.row .hw', { hasText: '학교' }).first()).toBeVisible();

  await search(page, '學');
  await expect(page.locator('.hanja-card')).toContainText('學'); // readings/meaning are empty in the current real pack (Unihan missing)
  await expect(page.locator('.row .hw', { hasText: '학교' }).first()).toBeVisible();

  // ---- entry + bookmark ----
  await search(page, '학교');
  await page.locator('.row', { hasText: '학교' }).first().click();
  await expect(page.locator('h1')).toContainText('학교');
  await expect(page.getByText('krdict 한국어기초사전')).toBeVisible();
  await page.getByRole('button', { name: 'Add bookmark' }).click();
  await page.getByRole('button', { name: 'Saved' }).click();
  await expect(page.getByRole('button', { name: 'Edit bookmark' })).toBeVisible();
  await tabs(page, 'main', 'phone-entry-tab', shot);
  await page.locator('.hj-link', { hasText: '學' }).click();
  await expect(page.locator('.hanja-giant')).toHaveText('學');
  await page.getByRole('link', { name: 'Bookmarks' }).click();
  await expect(page.locator('.bm .hw', { hasText: '학교' })).toBeVisible();
  // bookmarks open by headword (stable across data rebuilds), not by entry id
  await page.locator('.bm .row', { hasText: '학교' }).click();
  await expect(page).toHaveURL(/#\/word\//);
  await expect(page.locator('h1')).toContainText('학교');
  await expect(page.getByRole('button', { name: 'Edit bookmark' })).toBeVisible();

  // ---- light screenshots, then dark theme ----
  await page.goto('/#/settings');
  await page.getByRole('radio', { name: 'Light' }).click();
  await tour(page, 'light');
  await page.getByRole('radio', { name: 'Dark' }).click();
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');
  await tour(page, 'dark');
  await page.goto('/#/bookmarks');
  await shot(page, 'dark-bookmarks');

  // ---- reload offline: data, theme and bookmarks persist ----
  await page.goto('/');
  await page.evaluate(() => navigator.serviceWorker?.ready);
  await context.setOffline(true);
  await page.reload();
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');
  await expect(page.getByRole('searchbox', { name: 'Search' })).toBeVisible({ timeout: 30_000 });
  await search(page, '먹다');
  await expect(page.locator('.row .hw', { hasText: '먹다' }).first()).toBeVisible();
  await page.goto('/#/bookmarks');
  await expect(page.locator('.bm .hw', { hasText: '학교' })).toBeVisible();
  await context.setOffline(false);
});

test('interrupted download resumes per chunk', async ({ page }) => {
  test.setTimeout(600_000);
  let n = 0;
  await page.route('**/data/core.sqlite.gz.001?v=*', (route) => (n++ === 0 ? route.abort() : route.continue()));
  await page.goto('/');
  await page.getByRole('button', { name: 'Download' }).click();
  await expect(page.getByRole('alert')).toBeVisible({ timeout: 120_000 });
  await page.getByRole('button', { name: 'Resume download' }).click();
  await expect(page.getByRole('searchbox', { name: 'Search' })).toBeVisible({ timeout: 240_000 });
  expect(n).toBeGreaterThan(1);
});

/** Time from an input event to the frame after matching results are in the DOM. */
async function typeAndMeasure(page: Page, q: string): Promise<number> {
  return page.evaluate((q) => new Promise<number>((resolve) => {
    const input = document.querySelector<HTMLInputElement>('.searchbar input')!;
    const main = document.querySelector('main')!;
    const t0 = performance.now();
    const done = () => requestAnimationFrame(() => requestAnimationFrame(() => resolve(performance.now() - t0)));
    const ok = () => {
      const page = main.querySelector('.page[data-q]');
      if (!page || page.getAttribute('data-q') !== q) return false;
      const h = main.querySelector('.row .hw, .hanja-card .hanja-big, .empty-title');
      return !!h && !!location.hash.includes('q=') &&
        decodeURIComponent(location.hash).includes('q=' + q);
    };
    const t1 = setInterval(() => { if (ok()) { clearInterval(t1); clearTimeout(t2); done(); } }, 1);
    const t2 = setTimeout(() => { clearInterval(t1); resolve(-1); }, 30000);
    input.focus();
    input.value = q;
    input.dispatchEvent(new Event('input', { bubbles: true }));
  }), q);
}
const p95 = (xs: number[]) => [...xs].sort((a, b) => a - b)[Math.min(xs.length - 1, Math.ceil(xs.length * 0.95) - 1)];

test('performance budgets (loose: <100ms headless)', async ({ page }) => {
  page.on('console', (m) => { if (m.text().startsWith('KDPERF')) console.log(m.text()); });
  test.setTimeout(600_000);
  await page.goto('/');
  await downloadAll(page);
  // warm the engine (first queries touch cold pages), then measure typing a word keystroke by keystroke
  await typeAndMeasure(page, '학교');
  await page.waitForTimeout(2500); // let background warm-up start
  const keys: number[] = [];
  for (const q of ['ㅎ', '하', '학', '학교', '가', '갔', '먹', '먹다', '사', '사랑']) keys.push(await typeAndMeasure(page, q));
  for (const q of ['갔어요', 'eat', 'school', 'go', 'e', 'ea', 'love', '學']) keys.push(await typeAndMeasure(page, q));
  console.log('keystroke->results ms', keys.map((k) => Math.round(k)).join(' '), 'p95', Math.round(p95(keys)));

  await typeAndMeasure(page, '학교');
  const tap = await page.evaluate(() => new Promise<number>((resolve) => {
    const t = performance.now();
    const main = document.querySelector('main')!;
    const mo = new MutationObserver(() => { if (main.querySelector('h1')) { mo.disconnect(); requestAnimationFrame(() => requestAnimationFrame(() => resolve(performance.now() - t))); } });
    mo.observe(main, { childList: true, subtree: true });
    document.querySelector<HTMLElement>('.row')!.click();
  }));
  const back = await page.evaluate(() => new Promise<number>((resolve) => {
    const t = performance.now();
    const main = document.querySelector('main')!;
    const mo = new MutationObserver(() => { if (main.querySelector('.row')) { mo.disconnect(); requestAnimationFrame(() => requestAnimationFrame(() => resolve(performance.now() - t))); } });
    mo.observe(main, { childList: true, subtree: true });
    history.back();
  }));
  const tabs: number[] = [];
  for (const name of ['Bookmarks', 'Settings', 'Bookmarks']) {
    tabs.push(await page.evaluate((n) => new Promise<number>((resolve) => {
      const t = performance.now();
      const main = document.querySelector('main')!;
      const before = main.innerHTML;
      const mo = new MutationObserver(() => { if (main.innerHTML !== before && !main.querySelector('.skeleton')) { mo.disconnect(); requestAnimationFrame(() => requestAnimationFrame(() => resolve(performance.now() - t))); } });
      mo.observe(main, { childList: true, subtree: true });
      [...document.querySelectorAll<HTMLElement>('.tab')].find((a) => a.textContent === n)!.click();
    }), name));
  }
  console.log(`tap->entry ${Math.round(tap)}ms, back->results ${Math.round(back)}ms, tabs ${tabs.map(Math.round).join(' ')}ms`);
  expect(Math.min(...keys)).toBeGreaterThan(0); // -1 = timed out
  // Engine time is 2-35 ms per cold query in headless Chromium; the rest of the keystroke->results
  // time is the worker round trip + rendering (a cached 50-row result still takes 40-90 ms to paint).
  expect([...keys].sort((a, b) => a - b)[Math.floor(keys.length / 2)]).toBeLessThan(90);
  expect(p95(keys)).toBeLessThan(200);
  expect(tap).toBeLessThan(100);
  expect(back).toBeLessThan(100);
  expect(Math.max(...tabs)).toBeLessThan(150);

  // cold start with installed packs: time until the engine answers (search box is usable immediately)
  await page.reload();
  await expect(page.getByRole('searchbox', { name: 'Search' })).toBeVisible();
  await page.waitForFunction(() => performance.getEntriesByName('kd-engine-ready').length > 0, null, { timeout: 120_000 });
  const ready = await page.evaluate(() => Math.round(performance.getEntriesByName('kd-engine-ready')[0].startTime));
  console.log('engine ready (ms since navigation start):', ready);
});
