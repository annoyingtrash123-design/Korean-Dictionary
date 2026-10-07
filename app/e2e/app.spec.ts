import { expect, test, type Page } from '@playwright/test';
import { mkdirSync } from 'node:fs';
import { join } from 'node:path';

const SHOTS = join(process.cwd(), '..', 'docs', 'screenshots');
mkdirSync(SHOTS, { recursive: true });
const shot = (page: Page, name: string) => page.screenshot({ path: join(SHOTS, name + '.png') });
const search = async (page: Page, q: string) => {
  await page.getByRole('searchbox', { name: 'Search' }).fill(q);
  await page.waitForURL((u) => u.hash.includes('q=') && decodeURIComponent(u.hash).includes(q));
};

async function tour(page: Page, theme: 'light' | 'dark') {
  await page.goto('/#/');
  await expect(page.getByText('Word of the day')).toBeVisible();
  await shot(page, `${theme}-home`);
  await search(page, '먹다');
  await expect(page.locator('.row .hw', { hasText: '먹다' }).first()).toBeVisible();
  await shot(page, `${theme}-results`);
  await search(page, 'eat');
  await expect(page.locator('.row .hw').first()).toBeVisible();
  await shot(page, `${theme}-results-eat`);
  await search(page, '먹다');
  await page.locator('.row', { hasText: '먹다' }).first().click();
  await expect(page.locator('h1')).toContainText('먹다');
  await page.waitForTimeout(500);
  await shot(page, `${theme}-entry`);
  await page.getByRole('link', { name: 'Grammar' }).click();
  await expect(page.locator('.row .hw').first()).toBeVisible();
  await shot(page, `${theme}-grammar`);
  await page.getByRole('link', { name: 'Settings' }).click();
  await expect(page.getByRole('heading', { name: 'Appearance' })).toBeVisible();
  await shot(page, `${theme}-settings`);
}

test('first run, search, bookmarks, themes, offline persistence', async ({ page, context }) => {
  test.setTimeout(600_000);
  page.on('pageerror', (e) => console.log('PAGEERROR', e.message));
  await page.goto('/');
  // ---- first-run download ----
  await expect(page.getByRole('heading', { name: 'Korean Dictionary' })).toBeVisible();
  await expect(page.getByRole('switch')).toBeChecked();
  await shot(page, 'light-firstrun');
  await page.getByRole('button', { name: 'Download' }).click();
  await expect(page.getByRole('searchbox', { name: 'Search' })).toBeVisible({ timeout: 240_000 });

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
  await page.locator('.hj-link', { hasText: '學' }).click();
  await expect(page.locator('.hanja-giant')).toHaveText('學');
  await page.getByRole('link', { name: 'Bookmarks' }).click();
  await expect(page.locator('.bm .hw', { hasText: '학교' })).toBeVisible();

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
  await page.route('**/data/core.sqlite.gz.001', (route) => (n++ === 0 ? route.abort() : route.continue()));
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
      const h = main.querySelector('.row .hw, .hanja-card .hanja-big');
      return !!h && (h.textContent ?? '').length > 0 && !!location.hash.includes('q=') &&
        decodeURIComponent(location.hash).includes('q=' + q) && (main.querySelector('.page:not(.busy)') !== null);
    };
    const mo = new MutationObserver(() => { if (ok()) { mo.disconnect(); done(); } });
    mo.observe(main, { childList: true, subtree: true, characterData: true });
    input.focus();
    input.value = q;
    input.dispatchEvent(new Event('input', { bubbles: true }));
  }), q);
}
const p95 = (xs: number[]) => [...xs].sort((a, b) => a - b)[Math.min(xs.length - 1, Math.ceil(xs.length * 0.95) - 1)];

test('performance budgets (loose: <100ms headless)', async ({ page }) => {
  test.setTimeout(600_000);
  await page.goto('/');
  await page.getByRole('button', { name: 'Download' }).click();
  await expect(page.getByRole('searchbox', { name: 'Search' })).toBeVisible({ timeout: 240_000 });
  // warm the engine (first queries touch cold pages), then measure typing a word keystroke by keystroke
  await typeAndMeasure(page, '학교');
  const keys: number[] = [];
  for (const q of ['ㅎ', '하', '학', '학교', '가', '갔', '먹', '먹다', '사', '사랑']) keys.push(await typeAndMeasure(page, q));
  keys.push(await typeAndMeasure(page, 'eat'), await typeAndMeasure(page, 'school'));
  console.log('keystroke->results ms', keys.map((k) => Math.round(k)).join(' '), 'p95', Math.round(p95(keys)));

  await typeAndMeasure(page, '학교');
  const t0 = Date.now();
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
  for (const name of ['Grammar', 'Bookmarks', 'Settings', 'Grammar']) {
    tabs.push(await page.evaluate((n) => new Promise<number>((resolve) => {
      const t = performance.now();
      const main = document.querySelector('main')!;
      const before = main.innerHTML;
      const mo = new MutationObserver(() => { if (main.innerHTML !== before && !main.querySelector('.skeleton')) { mo.disconnect(); requestAnimationFrame(() => requestAnimationFrame(() => resolve(performance.now() - t))); } });
      mo.observe(main, { childList: true, subtree: true });
      [...document.querySelectorAll<HTMLElement>('.tab')].find((a) => a.textContent === n)!.click();
    }), name));
  }
  console.log(`tap->entry ${Math.round(tap)}ms, back->results ${Math.round(back)}ms, tabs ${tabs.map(Math.round).join(' ')}ms`, Date.now() - t0 > 0 ? '' : '');
  expect(p95(keys)).toBeLessThan(100);
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
