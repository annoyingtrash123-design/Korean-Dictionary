import { downloadAll, expect, test, type Page } from './fixtures';
import { join } from 'node:path';

const SHOTS = join(process.cwd(), '..', 'docs', 'screenshots');
const shot = (page: Page, name: string) => page.screenshot({ path: join(SHOTS, name + '.png') });

async function openGames(page: Page) {
  await page.getByRole('link', { name: 'Games' }).click();
  await expect(page.getByRole('heading', { name: /게임/ })).toBeVisible();
}
const card = (page: Page, name: string) => page.locator('.gm-card', { hasText: name });

test('games: tab order, six games, smart review, word chain', async ({ page }) => {
  test.setTimeout(400_000);
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/');
  await downloadAll(page);
  await expect(page.locator('.tabbar .tab span')).toHaveText(['Search', 'Reader', 'Games', 'Bookmarks', 'Settings']);
  await openGames(page);
  await expect(page.locator('.gm-card')).toHaveCount(6);
  await shot(page, 'games-home');

  // Meaning quiz: answer every question (first choice), always reaching the results.
  await card(page, 'Meaning quiz').click();
  await expect(page.locator('.gm-choice')).toHaveCount(4);
  await shot(page, 'games-meaning');
  // one step at a time: answer, then wait for Next (wrong), the next question (right) or the results
  const score = page.locator('.gm-score');
  for (let i = 0; i < 10; i++) {
    const before = (await page.locator('.gm-progress').textContent()) ?? '';
    await page.locator('.gm-choice:not([disabled])').first().click();
    const next = page.getByRole('button', { name: 'Next', exact: true });
    await expect(next.or(score).or(page.locator('.gm-progress', { hasNotText: before }))).toBeVisible();
    if (await next.isVisible()) await next.click();
    if (await score.isVisible()) break;
  }
  await expect(page.locator('.gm-score')).toHaveText(/\d+ \/ 10/);
  await shot(page, 'games-results');
  await page.getByRole('button', { name: 'Back to games', exact: true }).click();
  // the misses are now due for review
  await expect(page.locator('.gm-stats')).toContainText(/🔥 1/);

  // Type the word: a wrong answer reveals the right one.
  await card(page, 'Type the word').click();
  await page.getByLabel('Your answer').fill('없는말');
  await page.getByRole('button', { name: 'Check' }).click();
  await expect(page.locator('.gm-reveal.no')).toBeVisible();
  await page.getByRole('button', { name: 'Quit game' }).click();

  // Match pairs: solve one board by trying meanings until each pair matches.
  await card(page, 'Match pairs').click();
  await expect(page.locator('.gm-tile.ko')).toHaveCount(5);
  await shot(page, 'games-match');
  for (let k = 0; k < 5; k++) {
    const ko = page.locator('.gm-tile.ko:not(.done)').first();
    for (let j = 0; j < 5; j++) {
      const en = page.locator('.gm-tile:not(.ko):not(.done)').nth(j);
      if (!(await en.count())) break;
      await ko.click();
      await en.click();
      await page.waitForTimeout(500);
      if ((await page.locator('.gm-tile.ko.done').count()) > k) break;
    }
  }
  await expect(page.locator('.gm-tile.ko.done').or(page.locator('.gm-progress', { hasText: 'Board 2' })).first()).toBeVisible();
  await page.getByRole('button', { name: 'Quit game' }).click();

  // Fill the gap: a real sentence with a blank and four dictionary forms.
  await card(page, 'Fill the gap').click();
  await expect(page.locator('.gm-blank')).toBeVisible();
  await expect(page.locator('.gm-choice')).toHaveCount(4);
  await shot(page, 'games-gap');
  await page.locator('.gm-choice').first().click();
  await expect(page.locator('.gm-reveal')).toBeVisible();
  await page.getByRole('button', { name: 'Quit game' }).click();

  // 초성 quiz: initials shown; a word with other initials is explained.
  await card(page, 'Initials quiz').click();
  await expect(page.locator('.gm-cho')).toHaveText(/^[ㄱ-ㅎ]{2,}$/);
  await shot(page, 'games-cho');
  await page.getByLabel('Your answer').fill('아아아');
  await page.getByRole('button', { name: 'Check' }).click();
  await expect(page.getByText(/initials are/)).toBeVisible();
  await page.getByRole('button', { name: 'Quit game' }).click();

  // 끝말잇기: the app opens; a word that does not follow is refused; giving up ends the game.
  await card(page, 'Word chain').click();
  await expect(page.locator('.gm-bubble.by-app')).toHaveCount(1, { timeout: 20_000 });
  // `.app` is the pinned app frame: no other element may carry that class (a bubble once did and covered the screen)
  expect(await page.locator('.app').count()).toBe(1);
  await page.getByLabel('Your word').fill('아아');
  await page.getByRole('button', { name: 'Play' }).click();
  await expect(page.getByText(/^Start with /)).toBeVisible();
  await shot(page, 'games-chain');
  await page.getByRole('button', { name: /^Hint/ }).click();
  await expect(page.locator('.gm-hint')).toBeVisible();
  await page.getByRole('button', { name: 'Give up' }).click();
  await page.getByRole('button', { name: 'See result' }).click();
  await expect(page.locator('.gm-score')).toContainText('point');
});

test('games on iPad', async ({ page }) => {
  test.setTimeout(300_000);
  await page.setViewportSize({ width: 1180, height: 820 });
  await page.goto('/');
  await downloadAll(page);
  await openGames(page);
  await shot(page, 'games-home-ipad');
  await card(page, 'Meaning quiz').click();
  await expect(page.locator('.gm-choice')).toHaveCount(4);
  await shot(page, 'games-meaning-ipad');
});
