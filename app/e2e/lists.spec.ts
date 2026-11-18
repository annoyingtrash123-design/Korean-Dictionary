import { downloadAll, expect, test, type Page } from './fixtures';
import { mkdirSync } from 'node:fs';
import { join } from 'node:path';

const SHOTS = join(process.cwd(), '..', 'docs', 'screenshots');
mkdirSync(SHOTS, { recursive: true });
const shot = (page: Page, name: string) => page.screenshot({ path: join(SHOTS, name + '.png') });

/** Bookmarks as the version before Lists stored them (IndexedDB kd-user/kv, key `bookmarks`). */
const OLD_BOOKMARKS = {
  version: 1,
  folders: [{ id: 'saved', name: 'Saved' }, { id: 'fverbs', name: 'Verbs' }],
  items: [
    { key: 'krdict|학교||noun', source: 'krdict', headword: '학교', homonym: null, pos: 'noun', hanja: '學校', gloss: 'school', folder: 'saved', added: 1_700_000_000_000 },
    { key: 'krdict|가다||verb', source: 'krdict', headword: '가다', homonym: null, pos: 'verb', hanja: null, gloss: 'go', folder: 'fverbs', added: 1_700_000_000_001 },
  ],
};

async function seedOldBookmarks(page: Page) {
  await page.evaluate((data) => new Promise<void>((ok, fail) => {
    localStorage.removeItem('kd.backup.lists');
    const r = indexedDB.open('kd-user');
    r.onupgradeneeded = () => r.result.createObjectStore('kv');
    r.onerror = () => fail(r.error);
    r.onsuccess = () => {
      const tx = r.result.transaction('kv', 'readwrite');
      tx.objectStore('kv').put(data, 'bookmarks');
      tx.objectStore('kv').delete('lists');
      tx.oncomplete = () => { r.result.close(); ok(); };
      tx.onerror = () => fail(tx.error);
    };
  }), OLD_BOOKMARKS);
}

const search = async (page: Page, q: string) => {
  // the search box lives on the Search tab only: from an entry or the Lists tab, go there first (as a user would)
  const box = page.getByRole('searchbox', { name: 'Search' });
  if (!(await box.count())) await page.getByRole('navigation', { name: 'Main' }).getByRole('link', { name: 'Search' }).click();
  await box.fill(q);
  await page.waitForURL((u) => u.hash.includes('q=') && decodeURIComponent(u.hash).includes(q));
};
const openEntry = async (page: Page, q: string) => {
  await search(page, q);
  await page.locator('.row', { has: page.locator('.hw', { hasText: new RegExp(`^${q.replace(/[-()]/g, '\\$&')}$`) }) }).first().click();
  await expect(page.locator('h1')).toContainText(q);
};
const star = (page: Page) => page.locator('.entry-head .save-btn');
const toast = (page: Page) => page.locator('.app-toast');
const picker = (page: Page) => page.getByRole('dialog', { name: 'Save to list' });
const listRow = (page: Page, name: string) => page.locator('.list-row', { has: page.locator('.list-name', { hasText: new RegExp(`^${name}$`) }) });

async function longPress(page: Page, el: ReturnType<Page['locator']>) {
  const b = (await el.boundingBox())!;
  await page.mouse.move(b.x + b.width / 2, b.y + b.height / 2);
  await page.mouse.down();
  await page.waitForTimeout(800);
  await page.mouse.up();
}

test('lists: old bookmarks migrate, tap to save, long-press picker, several lists, Lists tab', async ({ page }) => {
  test.setTimeout(600_000);
  page.on('pageerror', (e) => console.log('PAGEERROR', e.message));
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/');
  await downloadAll(page);

  // ---- old bookmarks become "Saved words" (and a list per folder), once ----
  await seedOldBookmarks(page);
  await page.reload();
  await page.getByRole('link', { name: 'Lists', exact: true }).click();
  await expect(listRow(page, 'Saved words')).toContainText('1 item');
  await expect(listRow(page, 'Verbs')).toContainText('1 item');
  await page.reload();   // a second start changes nothing
  await expect(page.locator('.list-row')).toHaveCount(2);
  await expect(listRow(page, 'Saved words')).toContainText('1 item');
  // the old Bookmarks route lands on Lists
  await page.goto('/#/bookmarks');
  await expect(page).toHaveURL(/#\/lists$/);
  await expect(page.getByRole('link', { name: 'Lists', exact: true })).toHaveAttribute('aria-current', 'page');

  // ---- tap: save to the last-used list, toast with Change ----
  await openEntry(page, '먹다');
  await expect(star(page)).toHaveAccessibleName('Save to Saved words');
  await star(page).click();
  await expect(toast(page)).toContainText('Saved to Saved words');
  await expect(star(page)).toHaveAccessibleName('Remove from Saved words');
  await expect(star(page)).toHaveClass(/\bon\b/);
  await page.waitForTimeout(400);   // toast fade-in
  await shot(page, 'lists-toast');
  await toast(page).getByRole('button', { name: 'Change' }).click();
  await expect(picker(page)).toBeVisible();
  await expect(picker(page).getByRole('checkbox', { name: /Saved words/ })).toBeChecked();
  await expect(picker(page).getByRole('checkbox', { name: /Saved words/ })).toBeFocused();
  await page.keyboard.press('Escape');
  await expect(picker(page)).toHaveCount(0);
  await expect(star(page)).toBeFocused();   // focus goes back to the star

  // ---- long-press: picker, new list, item in two lists ----
  await longPress(page, star(page));
  await expect(picker(page)).toBeVisible();
  await expect(toast(page)).toHaveCount(0);   // the long-press did not also tap
  await picker(page).getByRole('button', { name: 'New list' }).click();
  const name = picker(page).getByRole('textbox', { name: 'New list name' });
  await expect(name).toBeFocused();
  await name.fill('TOPIK 6급');
  await page.keyboard.press('Enter');
  await expect(picker(page).getByRole('checkbox', { name: /TOPIK 6급/ })).toBeChecked();
  await expect(picker(page).getByRole('checkbox', { name: /Saved words/ })).toBeChecked();
  await expect(picker(page).locator('.pick', { hasText: 'TOPIK 6급' })).toContainText('tap saves here');
  await page.waitForTimeout(300);
  await shot(page, 'lists-picker');
  await picker(page).getByRole('button', { name: 'Done' }).click();
  await expect(star(page)).toHaveAccessibleName('Remove from TOPIK 6급');   // the picker set the last-used list

  // ---- tap on a saved item removes it from that list; Undo puts it back ----
  await star(page).click();
  await expect(toast(page)).toContainText('Removed from TOPIK 6급');
  await expect(star(page)).toHaveAccessibleName('Save to TOPIK 6급');
  await expect(star(page)).toHaveClass(/\bon\b/);   // still in Saved words
  await toast(page).getByRole('button', { name: 'Undo' }).click();
  await expect(star(page)).toHaveAccessibleName('Remove from TOPIK 6급');

  // ---- right-click and the keyboard (Shift+F10) open the picker too ----
  await star(page).click({ button: 'right' });
  await expect(picker(page)).toBeVisible();
  await picker(page).getByRole('button', { name: 'Close' }).click();
  await star(page).focus();
  await page.keyboard.press('Shift+F10');
  await expect(picker(page)).toBeVisible();
  // Tab stays inside the sheet
  for (let i = 0; i < 8; i++) await page.keyboard.press('Tab');
  await expect(picker(page).locator(':focus')).toHaveCount(1);
  await page.keyboard.press('Escape');

  // ---- a grammar pattern saves as the pattern (survives data rebuilds) ----
  await openEntry(page, '-아서');
  await expect(star(page)).toHaveAttribute('data-kind', 'grammar');
  await star(page).click();
  await expect(toast(page)).toContainText('Saved to TOPIK 6급');

  // ---- Lists tab ----
  await page.getByRole('link', { name: 'Lists', exact: true }).click();
  await expect(listRow(page, 'Saved words')).toContainText('2 items');
  await expect(listRow(page, 'TOPIK 6급')).toContainText('2 items');
  await expect(listRow(page, 'TOPIK 6급')).toContainText('updated today');
  await shot(page, 'lists-overview');
  await listRow(page, 'TOPIK 6급').click();
  await expect(page.getByRole('heading', { name: 'TOPIK 6급' })).toBeVisible();
  const rows = page.locator('.list-detail .bm .hw');
  await expect(rows).toHaveText(['-아서', '먹다']);   // newest first
  await expect(page.locator('.list-detail .bm', { hasText: '-아서' })).toContainText('grammar');
  await page.getByRole('radio', { name: 'Alphabetical' }).click();
  await expect(rows).toHaveText(['먹다', '-아서']);   // the pattern sorts by its first syllable (아)
  await page.waitForTimeout(200);
  await shot(page, 'lists-detail');

  // open an item, come back
  await page.locator('.list-detail .bm .row', { hasText: '-아서' }).click();
  await expect(page).toHaveURL(/#\/word\//);
  await expect(page.locator('h1')).toContainText('-아서');
  await page.goBack();

  // remove (Undo), rename
  await page.getByRole('button', { name: 'Edit' }).click();
  await page.getByRole('button', { name: 'Remove 먹다' }).click();
  await expect(rows).toHaveText(['-아서']);
  await expect(toast(page)).toContainText('Removed 먹다 from TOPIK 6급');
  await toast(page).getByRole('button', { name: 'Undo' }).click();
  await expect(rows).toHaveCount(2);
  await page.getByRole('button', { name: 'Remove 먹다' }).click();
  await expect(rows).toHaveText(['-아서']);
  await page.getByRole('button', { name: 'Rename' }).click();
  await page.getByRole('textbox', { name: 'List name' }).fill('TOPIK 문법');
  await page.keyboard.press('Enter');
  await expect(page.getByRole('heading', { name: 'TOPIK 문법' })).toBeVisible();
  // 먹다 is still in Saved words
  await page.getByRole('link', { name: 'All lists' }).click();
  await expect(listRow(page, 'Saved words')).toContainText('2 items');

  // delete a list (confirm)
  await listRow(page, 'Verbs').click();
  await page.getByRole('button', { name: 'Edit' }).click();
  page.once('dialog', (d) => d.accept());
  await page.getByRole('button', { name: 'Delete list' }).click();
  await expect(page).toHaveURL(/#\/lists$/);
  await expect(listRow(page, 'Verbs')).toHaveCount(0);

  // new list from the Lists tab
  await page.getByRole('button', { name: 'New list' }).click();
  await page.getByRole('textbox', { name: 'New list name' }).fill('Idioms');
  await page.keyboard.press('Enter');
  await expect(page.getByRole('heading', { name: 'Idioms' })).toBeVisible();
  await expect(page.getByText('Nothing here yet')).toBeVisible();

  // ---- everything survives a restart ----
  await page.reload();
  await page.goto('/#/lists');
  await expect(page.locator('.list-row .list-name')).toHaveText(['Saved words', 'TOPIK 문법', 'Idioms']);

  // ---- iPad: list on the left, entry on the right; picker as a centred sheet ----
  await page.setViewportSize({ width: 1180, height: 820 });
  await listRow(page, 'Saved words').click();
  await expect(page.getByText('Pick a saved word')).toBeVisible();
  await page.locator('.pane-left .bm .row', { hasText: '먹다' }).click();
  await expect(page.locator('.pane-right h1')).toContainText('먹다');
  await expect(page.locator('.pane-left .bm .row[aria-current="true"]')).toHaveCount(1);
  await expect(page.getByRole('link', { name: 'Lists', exact: true })).toHaveAttribute('aria-current', 'page');
  await page.waitForTimeout(300);
  await shot(page, 'tablet-lists-detail');
  await longPress(page, star(page));
  await expect(picker(page)).toBeVisible();
  await page.waitForTimeout(300);
  await shot(page, 'tablet-lists-picker');
  await page.keyboard.press('Escape');
});
