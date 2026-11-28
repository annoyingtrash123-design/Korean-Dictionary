import { downloadAll, expect, test, type Page, texts } from './fixtures';
import { mkdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

const SHOTS = join(process.cwd(), '..', 'docs', 'screenshots');
mkdirSync(SHOTS, { recursive: true });
const shot = async (page: Page, name: string) => { console.log('SHOT', name); await page.screenshot({ path: join(SHOTS, `${name}.png`) }); };

/** The core pack has the study lists' tables (`curriculum` and `topic`: the pipeline's build, not an older pack). */
const HAS_STUDY = (() => {
  try {
    const m = JSON.parse(readFileSync(join(process.cwd(), 'public', 'data', 'manifest.json'), 'utf8'));
    const c = m.packs?.find((p: { id: string }) => p.id === 'core')?.counts ?? {};
    return (c.curriculum ?? 0) > 0 && (c.topic ?? 0) > 0;
  } catch { return false; }
})();

const study = (page: Page) => page.locator('section[data-source=study]');
const listLink = (page: Page, group: string, name: string) =>
  study(page).getByRole('group', { name: group }).locator('.list-row', { has: page.locator('.list-name', { hasText: new RegExp(`^${name}$`) }) }).first();
const fold = (page: Page, group: string) => study(page).getByRole('button', { name: new RegExp(`^${group}`) });
/** Expand a group of the overview (idempotent: the choice is remembered on the device). */
async function expand(page: Page, group: string) {
  if ((await fold(page, group).getAttribute('aria-expanded')) !== 'true') await fold(page, group).click();
  await expect(fold(page, group)).toHaveAttribute('aria-expanded', 'true');
}
const rows = (page: Page) => page.locator('.vl-row:not(.vl-skel)');
const rowAt = (page: Page, n: number) => page.locator(`.vl-row[aria-posinset="${n}"]`);
const scroller = (page: Page) => page.locator('.vl-scroll');
const toast = (page: Page) => page.locator('.app-toast');
const titles = (page: Page) => texts(rows(page).locator('.hw'));
const countOf = async (page: Page) => Number(((await page.getByTestId('study-count').innerText()).match(/[\d,]+/) ?? ['0'])[0].replace(/,/g, ''));

async function openLists(page: Page) {
  await page.getByRole('link', { name: 'Lists', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'My lists' })).toBeVisible();
}
/** The overview, whichever list is open (the Lists tab keeps the open list on a tablet). */
async function overview(page: Page) {
  await page.goto('/#/lists');
  await expect(study(page).getByRole('heading', { name: 'Study lists' })).toBeVisible();
}

test('study lists: levels, deep scroll, entry, save, grammar, topics (phone and iPad)', async ({ page }) => {
  test.setTimeout(900_000);
  page.on('pageerror', (e) => console.log('PAGEERROR', e.message));
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/');
  await downloadAll(page);
  await page.getByRole('link', { name: 'Settings' }).click();
  await page.getByRole('radio', { name: 'Light' }).click();
  await openLists(page);

  if (!HAS_STUDY) {
    // an older pack: the section is simply not there
    await expect(page.getByRole('heading', { name: 'My lists' })).toBeVisible();
    await expect(page.locator('section[data-source=study]')).toHaveCount(0);
    return;
  }

  // ---- overview: "Study lists" below "My lists" ----
  await expect(study(page).getByRole('heading', { name: 'Study lists' })).toBeVisible();
  const sections = await page.locator('.lists > section h2').allTextContents();
  expect(sections).toEqual(['My lists', 'Study lists']);
  const vocab = study(page).getByRole('group', { name: 'Vocabulary by level' });
  // collapsed by default, each heading summarising what is inside
  const GROUPS = { 'Vocabulary by level': '6 levels', 'Grammar by level': '6 levels', 'Topics': /^\d+ topics$/ };
  for (const [g, sum] of Object.entries(GROUPS)) {
    await expect(fold(page, g)).toHaveAttribute('aria-expanded', 'false');
    await expect(fold(page, g)).toHaveAttribute('aria-controls', /.+/);
    await expect(fold(page, g).locator('.group-sum')).toHaveText(sum);
    expect((await fold(page, g).boundingBox())!.height).toBeGreaterThanOrEqual(44);
  }
  await expect(study(page).locator('.list-row:visible')).toHaveCount(0);
  await page.waitForTimeout(300);
  await shot(page, 'study-lists-overview-phone');
  // keyboard: Enter on the focused heading toggles, the summary goes away when open
  await fold(page, 'Vocabulary by level').focus();
  await page.keyboard.press('Enter');
  await expect(fold(page, 'Vocabulary by level')).toHaveAttribute('aria-expanded', 'true');
  await expect(fold(page, 'Vocabulary by level').locator('.group-sum')).toHaveCount(0);
  await page.keyboard.press('Enter');
  await expect(fold(page, 'Vocabulary by level')).toHaveAttribute('aria-expanded', 'false');
  // open state survives a reload; the others stay closed
  await expand(page, 'Vocabulary by level');
  await expand(page, 'Grammar by level');
  await expand(page, 'Topics');
  await page.reload();
  await expect(study(page).getByRole('heading', { name: 'Study lists' })).toBeVisible();
  for (const g of Object.keys(GROUPS)) await expect(fold(page, g)).toHaveAttribute('aria-expanded', 'true');
  await fold(page, 'Grammar by level').click();
  await page.reload();
  await expect(fold(page, 'Grammar by level')).toHaveAttribute('aria-expanded', 'false');
  await expect(fold(page, 'Vocabulary by level')).toHaveAttribute('aria-expanded', 'true');
  await expand(page, 'Grammar by level');
  await expect(vocab.locator('.list-row')).toHaveCount(6);
  expect(await texts(vocab.locator('.list-name'))).toEqual([1, 2, 3, 4, 5, 6].map((n) => `${n}급 · 표준 교육과정`));
  const grammar = study(page).getByRole('group', { name: 'Grammar by level' });
  await expect(grammar.locator('.list-row')).toHaveCount(6);
  // the curriculum level is never plain "TOPIK N" (only the hint says how it relates)
  await expect(vocab.locator('.list-row').nth(5)).toHaveAttribute('title', 'National standard Korean curriculum level 6 (≈ TOPIK 6)');
  await expect(vocab.locator('.list-row').nth(5)).toHaveAccessibleName(/National standard Korean curriculum level 6 \(≈ TOPIK 6\)/);
  expect(await texts(study(page).locator('.list-name, .group-head'))).not.toContain('TOPIK 6');
  // (the Writing & usage list "쓰기 expressions (TOPIK writing)" names the exam on purpose: it is not a curriculum level)
  for (const t of await texts(study(page).locator('.list-name'))) if (!/writing/.test(t)) expect(t).not.toMatch(/TOPIK/);
  await expect(study(page).getByRole('group', { name: 'Topics' }).locator('.list-row').first()).toBeVisible();
  await expect(study(page).locator('.resume')).toHaveCount(0);   // nothing opened yet
  await page.waitForTimeout(300);
  await shot(page, 'study-lists-expanded-phone');

  // ---- 6급 vocabulary: thousands of rows, only the visible ones in the page ----
  const t0 = Date.now();
  await listLink(page, 'Vocabulary by level', '6급 · 표준 교육과정').click();
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('6급 · 표준 교육과정');
  await expect(rows(page).first()).toBeVisible();
  const firstPaint = Date.now() - t0;
  const total = await countOf(page);
  expect(total).toBeGreaterThan(2000);
  expect(await rows(page).count()).toBeLessThan(40);
  await expect(page.locator('.list-hint')).toHaveText('National standard Korean curriculum level 6 (≈ TOPIK 6)');
  await expect(rowAt(page, 1)).toContainText(/\S/);
  const firstTitle = (await rowAt(page, 1).locator('.hw').innerText()).trim();
  await expect(rowAt(page, 1).locator('.row-gloss')).not.toBeEmpty();
  await page.waitForTimeout(300);
  await shot(page, 'study-lists-vocab-phone');

  // scroll deep: a flick to the middle, then the fast-scroll thumb to the end
  const t1 = Date.now();
  await scroller(page).evaluate((el, n) => { el.scrollTop = n * 60; }, 1200);
  await expect(rowAt(page, 1201)).toContainText(/\S/, { timeout: 15_000 }).catch(async (e) => {
    console.log('DEEP-DIAG', JSON.stringify(await page.evaluate(() => {
      const el = document.querySelector('.vl-scroll') as HTMLElement;
      const li = [...document.querySelectorAll('.vl-row')] as HTMLElement[];
      return { top: el.scrollTop, ch: el.clientHeight, sh: el.scrollHeight, n: li.length, first: li[0]?.outerHTML.slice(0, 200), skel: li.filter((x) => x.classList.contains('vl-skel')).length, err: document.querySelector('.vl-error')?.textContent };
    })));
    throw e;
  });
  const deepLoad = Date.now() - t1;
  await expect(rowAt(page, 1201).locator('.hw')).not.toHaveText(firstTitle);
  const thumb = page.locator('.fs-thumb');
  await expect(thumb).toBeVisible();
  const tb = (await thumb.boundingBox())!;
  const track = (await page.locator('.fs-track').boundingBox())!;
  await page.mouse.move(tb.x + tb.width / 2 + 2, tb.y + tb.height / 2);
  await page.mouse.down();
  await page.mouse.move(tb.x + tb.width / 2 + 2, track.y + track.height * 0.55, { steps: 6 });
  await expect(page.locator('.fs-bubble')).toContainText(`/ ${total.toLocaleString('en-US')}`);
  await page.waitForTimeout(400);
  await shot(page, 'study-lists-vocab-scroll-phone');
  await page.mouse.move(tb.x + tb.width / 2 + 2, track.y + track.height + 40, { steps: 6 });
  await page.mouse.up();
  await expect(rowAt(page, total)).toContainText(/\S/);   // the very last word
  const lastTitle = (await rowAt(page, total).locator('.hw').innerText()).trim();
  expect(lastTitle).not.toBe('');
  console.log(`STUDY-TIMING 6급 (${total} words): first rows ${firstPaint} ms after tap, row 1201 after a jump ${deepLoad} ms`);

  // ---- open an entry from deep in the list, come back to the same place ----
  await scroller(page).evaluate((el, n) => { el.scrollTop = n * 60; }, 1200);
  await expect(rowAt(page, 1201)).toContainText(/\S/);
  await page.waitForTimeout(600);   // the position is remembered a moment after scrolling stops
  const deepTitle = (await rowAt(page, 1203).locator('.hw').innerText()).trim();
  await rowAt(page, 1203).locator('a.row').click();
  await expect(page).toHaveURL(/#\/entry\/krdict\/\d+/);
  await expect(page.locator('h1').first()).toContainText(deepTitle);
  await page.goBack();
  await expect(rowAt(page, 1203)).toContainText(deepTitle);
  await expect(rowAt(page, 1)).toHaveCount(0);   // not rewound to the top
  await expect(page.locator('.save-btn').first()).toBeVisible();

  // ---- the star copies a word into the user's own list ----
  const star = rowAt(page, 1203).locator('.save-btn');
  await expect(star).toHaveAccessibleName('Save to Saved words');
  await star.click();
  await expect(toast(page)).toContainText('Saved to Saved words');
  await expect(star).toHaveAccessibleName('Remove from Saved words');
  await expect(star).toHaveClass(/\bon\b/);
  await page.waitForTimeout(300);
  await shot(page, 'study-lists-saved-phone');
  // the saved word is in "Saved words" (keyed like any entry), and the study list is unchanged
  await overview(page);
  await expect(page.locator('.list-row', { has: page.locator('.list-name', { hasText: /^Saved words$/ }) })).toContainText('1 item');
  // no "Continue: …" shortcut row above the groups (the owner removed it)
  await expect(page.getByText(/^Continue:/)).toHaveCount(0);

  // ---- grammar by level: the official forms, summaries; a tap opens the pattern's entry ----
  await expand(page, 'Grammar by level');
  await listLink(page, 'Grammar by level', '1급 · 표준 교육과정').click();
  await expect(page.locator('.eyebrow')).toHaveText('Grammar');
  await expect(rows(page).first()).toBeVisible();
  const gCount = await countOf(page);
  expect(gCount).toBeGreaterThan(30);
  await expect(rowAt(page, 1)).toHaveAttribute('aria-setsize', String(gCount));
  const gTitle = (await rowAt(page, 1).locator('.hw').innerText()).trim();
  expect(gTitle).toMatch(/\S/);
  await expect(rowAt(page, 1).locator('.row-gloss')).not.toBeEmpty();
  await page.waitForTimeout(300);
  await shot(page, 'study-lists-grammar-phone');
  // "Add all to a list…": a new list with every pattern, once
  await page.getByRole('button', { name: 'Add all to a list…' }).click();
  const addAll = page.getByRole('dialog', { name: 'Add all to list' });
  await expect(addAll).toBeVisible();
  await addAll.getByRole('button', { name: 'New list' }).click();
  await addAll.getByRole('textbox', { name: 'New list name' }).fill('Grammar 1급');
  await page.keyboard.press('Enter');
  await expect(toast(page)).toContainText(/Added [\d,]+ patterns to Grammar 1급/);   // two spellings of one pattern are one item
  await expect(addAll).toHaveCount(0);
  await rowAt(page, 1).locator('a.row').click();
  await expect(page).toHaveURL(/#\/(entry|word)\//);
  await expect(page.locator('h1').first()).toBeVisible();
  await page.goBack();

  // ---- topics: top level, then sub-categories; sorted by level then 가나다, or 가나다 only ----
  await overview(page);
  await expand(page, 'Topics');
  const topics = study(page).getByRole('group', { name: 'Topics' });
  const top = topics.locator('.list-row-wrap').first();
  await expect(top.locator('.list-row')).toContainText(/\d/);
  await top.getByRole('button', { name: /sub-categories/ }).click();
  await expect(topics.locator('.subs .list-row').first()).toBeVisible();
  expect(await topics.locator('.subs .list-row').count()).toBeGreaterThan(2);
  await expect(toast(page)).toHaveCount(0, { timeout: 15_000 });   // keep the toast out of the picture
  await page.waitForTimeout(300);
  await shot(page, 'study-lists-topics-phone');
  const subName = (await topics.locator('.subs .list-name').first().innerText()).trim();
  await topics.locator('.subs .list-row').first().click();
  await expect(rows(page).first()).toBeVisible();
  await expect(page.locator('.eyebrow')).toContainText('Topic');
  const tCount = await countOf(page);
  expect(tCount).toBeGreaterThan(5);
  expect(await page.getByRole('heading', { level: 1 }).innerText()).toContain(subName);
  // by level: the 초/중/고 badges never go down
  const badges = async () => (await texts(rows(page).locator('.lvl'))).map((b) => '초중고'.indexOf(b));
  const lv = await badges();
  expect(lv.length).toBeGreaterThan(0);
  expect(lv).toEqual([...lv].sort((a, b) => a - b));
  await page.waitForTimeout(300);
  await shot(page, 'study-lists-topic-phone');
  await page.getByRole('radio', { name: 'Alphabetical (가나다)' }).click();
  await expect(page.getByRole('radio', { name: 'Alphabetical (가나다)' })).toHaveAttribute('aria-checked', 'true');
  await expect(rowAt(page, 1)).toContainText(/\S/);
  const az = (await titles(page)).map((t) => t.trim());
  expect(az.length).toBeGreaterThan(3);
  expect(az).toEqual([...az].sort());
  // the choice is remembered for the next topic
  await page.reload();
  await expect(page.getByRole('radio', { name: 'Alphabetical (가나다)' })).toHaveAttribute('aria-checked', 'true');

});

test('study lists on an iPad: the list on the left, the entry on the right', async ({ page }) => {
  test.setTimeout(900_000);
  page.on('pageerror', (e) => console.log('PAGEERROR', e.message));
  await page.setViewportSize({ width: 1180, height: 820 });
  await page.goto('/');
  await downloadAll(page);
  await page.getByRole('link', { name: 'Settings' }).click();
  await page.getByRole('radio', { name: 'Light' }).click();
  test.skip(!HAS_STUDY, 'pack without study lists');
  await page.goto('/#/lists');
  await expect(study(page).getByRole('heading', { name: 'Study lists' })).toBeVisible();
  await page.waitForTimeout(300);
  await shot(page, 'tablet-study-lists-overview');
  await expand(page, 'Vocabulary by level');
  await expand(page, 'Grammar by level');
  await listLink(page, 'Vocabulary by level', '5급 · 표준 교육과정').click();
  await expect(rows(page).first()).toBeVisible();
  await rowAt(page, 3).locator('a.row').click();
  await expect(page.locator('.pane-right h1').first()).toBeVisible();
  await expect(page.locator('.pane-left .vl-scroll')).toBeVisible();   // the list stays next to the entry
  await expect(rowAt(page, 3).locator('a.row')).toHaveAttribute('aria-current', 'true');
  await page.waitForTimeout(500);
  await shot(page, 'tablet-study-lists-vocab');
  await page.getByRole('link', { name: 'All lists' }).click();
  await expand(page, 'Grammar by level');
  await listLink(page, 'Grammar by level', '3급 · 표준 교육과정').click();
  await expect(rows(page).first()).toBeVisible();
  await page.waitForTimeout(300);
  await shot(page, 'tablet-study-lists-grammar');
});
