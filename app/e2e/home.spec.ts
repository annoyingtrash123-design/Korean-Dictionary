import { downloadAll, expect, test, type Page } from './fixtures';
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

const carousel = (page: Page) => page.getByRole('region', { name: /^Today/ });
const slides = (page: Page) => carousel(page).getByRole('group', { name: /^\d of \d$/ }).filter({ has: page.locator('.today-card') });
const heads = (page: Page) => page.locator('.today-card .card-hw .hangul').allInnerTexts();

test('home: today carousel, idiom of the day, recent', async ({ page }) => {
  test.setTimeout(600_000);
  await page.goto('/');
  await downloadAll(page);
  await page.getByRole('link', { name: 'Settings' }).click();
  await page.getByRole('radio', { name: 'Light' }).click();
  await page.goto('/#/');

  // ---- idiom of the day (needs no curriculum table) ----
  const idiom = page.getByRole('region', { name: 'Idiom of the day' });
  await expect(idiom).toBeVisible();
  await expect(idiom.locator('.card-tag')).toHaveText(/관용구|속담/);
  await expect(idiom.locator('.card-gloss')).not.toBeEmpty();
  await expect(idiom.locator('.card-ex .ex-ko')).toHaveCount(1);
  await expect(page.getByRole('heading', { name: 'Recent' })).toBeVisible();

  if (!HAS_CURRICULUM) {
    // current packs: no placeholder at all
    await expect(carousel(page)).toHaveCount(0);
    await expect(page.locator('.car-track')).toHaveCount(0);
    await shot(page, 'home-phone');
    return;
  }

  // ---- carousel: 3 words + 2 grammar patterns ----
  await expect(carousel(page)).toBeVisible();
  await expect(slides(page)).toHaveCount(5);
  for (let i = 0; i < 5; i++) await expect(carousel(page).getByRole('group', { name: `${i + 1} of 5` }).first()).toBeAttached();
  await expect(page.locator('.today-card[data-kind=word]')).toHaveCount(3);
  await expect(page.locator('.today-card[data-kind=grammar]')).toHaveCount(2);
  // level labels name the curriculum, never plain "TOPIK n"
  const lv = page.locator('.today-card .cur-lvl');
  await expect(lv).toHaveCount(5);
  for (const t of await lv.allInnerTexts()) expect(t).toMatch(/^[56]급 · 표준 교육과정$/);
  await expect(lv.first()).toHaveAttribute('title', /^National standard Korean curriculum level [56] \(≈ TOPIK [56]\)$/);
  expect(await carousel(page).innerText()).not.toMatch(/TOPIK/);
  // each word card has hanja-or-not, POS, a gloss and examples; examples may carry the AI chip
  const w0 = page.locator('.today-card[data-kind=word]').first();
  await expect(w0.locator('.card-gloss')).not.toBeEmpty();
  await expect(w0.locator('.card-ex .ex-ko').first()).toBeVisible();
  expect(await w0.locator('.card-ex').count()).toBeLessThanOrEqual(2);
  await expect(page.locator('.today-card[data-kind=grammar] .card-gloss').first()).not.toBeEmpty();
  await shot(page, 'home-phone');

  // ---- navigating: arrows, dots, keyboard, swipe ----
  const first = await heads(page);
  const prev = carousel(page).getByRole('button', { name: 'Previous card' });
  const next = carousel(page).getByRole('button', { name: 'Next card' });
  const dots = carousel(page).getByRole('button', { name: /^\d of 5$/ });
  await expect(dots).toHaveCount(5);
  await expect(prev).toBeDisabled();
  await expect(dots.nth(0)).toHaveAttribute('aria-current', 'true');
  await next.click();
  await expect(dots.nth(1)).toHaveAttribute('aria-current', 'true');
  await expect(prev).toBeEnabled();
  await dots.nth(4).click();
  await expect(dots.nth(4)).toHaveAttribute('aria-current', 'true');
  await expect(next).toBeDisabled();
  await expect(page.locator('.today-card[data-kind=grammar]').last()).toBeInViewport({ ratio: 0.9 });
  await carousel(page).locator('.car-track').focus();
  await page.keyboard.press('ArrowLeft');
  await expect(dots.nth(3)).toHaveAttribute('aria-current', 'true');
  await page.keyboard.press('ArrowRight');
  await expect(dots.nth(4)).toHaveAttribute('aria-current', 'true');
  // mouse drag (touch swipes natively; a desktop pointer drags the track)
  await dots.nth(0).click();
  await expect(dots.nth(0)).toHaveAttribute('aria-current', 'true');
  const box = (await carousel(page).locator('.car-track').boundingBox())!;
  await page.mouse.move(box.x + box.width - 40, box.y + 130);
  await page.mouse.down();
  await page.mouse.move(box.x + box.width - 140, box.y + 130, { steps: 6 });
  await page.mouse.move(box.x + 20, box.y + 130, { steps: 6 });
  await page.mouse.up();
  await expect(dots.nth(1)).toHaveAttribute('aria-current', 'true');
  expect(page.url()).not.toMatch(/#\/(entry|word)\//);   // a drag is not a tap on the card

  // ---- the save star on a card ----
  await dots.nth(0).click();
  const star = page.locator('.today-card[data-kind=word]').first().getByRole('button', { name: /^Save to / });
  await star.click();
  await expect(page.locator('.today-card[data-kind=word]').first().getByRole('button', { name: /^Remove from / })).toBeVisible();
  await page.locator('.today-card[data-kind=word]').first().getByRole('button', { name: /^Remove from / }).click();

  // ---- a tap opens the entry ----
  await page.locator('.today-card[data-kind=word]').first().locator('.card-link').click();
  await expect(page).toHaveURL(/#\/entry\//);
  await expect(page.locator('h1')).toContainText(first[0]);
  await page.goBack();
  await expect(carousel(page)).toBeVisible();

  // ---- typing switches to results, clearing brings the home page back ----
  const box2 = page.getByRole('searchbox', { name: 'Search' });
  await box2.fill('학교');
  await expect(page.locator('.row .hw', { hasText: '학교' }).first()).toBeVisible();
  await expect(carousel(page)).toHaveCount(0);
  await box2.fill('');
  await expect(carousel(page)).toBeVisible();

  // ---- the same cards for the same date, other cards on other dates ----
  await page.clock.setFixedTime(new Date(2026, 2, 5, 12, 0, 0));
  await page.reload();
  await expect(slides(page)).toHaveCount(5);
  const mar5 = await heads(page);
  const idiomMar5 = await idiom.locator('.card-hw .hangul').innerText();
  await page.clock.setFixedTime(new Date(2026, 2, 5, 23, 59, 0));
  await page.reload();
  await expect(slides(page)).toHaveCount(5);
  expect(await heads(page)).toEqual(mar5);
  expect(await idiom.locator('.card-hw .hangul').innerText()).toBe(idiomMar5);
  await page.clock.setFixedTime(new Date(2026, 2, 6, 0, 1, 0));
  await page.reload();
  await expect(slides(page)).toHaveCount(5);
  expect(await heads(page)).not.toEqual(mar5);
});

test('home: iPad two-pane and dark mode screenshots', async ({ page }) => {
  test.setTimeout(600_000);
  await page.setViewportSize({ width: 1180, height: 820 });
  await page.goto('/');
  await downloadAll(page);
  await page.getByRole('link', { name: 'Settings' }).click();
  await page.getByRole('radio', { name: 'Light' }).click();
  await page.getByRole('link', { name: 'Search' }).click();
  await expect(page.getByRole('region', { name: 'Idiom of the day' })).toBeVisible();
  if (HAS_CURRICULUM) {
    await expect(slides(page)).toHaveCount(5);
    await expect(page.locator('.pane-left .today-card[data-kind=word]').first()).toBeInViewport();
  }
  await page.waitForTimeout(400);
  await shot(page, 'home-ipad');

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/#/settings');
  await page.getByRole('radio', { name: 'Dark' }).click();
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');
  await page.goto('/#/');
  await expect(page.getByRole('region', { name: 'Idiom of the day' })).toBeVisible();
  if (HAS_CURRICULUM) await expect(slides(page)).toHaveCount(5);
  await page.waitForTimeout(400);
  await shot(page, 'home-dark');
});
