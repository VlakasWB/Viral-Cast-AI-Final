import { test, expect } from '@playwright/test';

const FRONTEND_URL = 'http://localhost:4173';

async function doLogin(page: any) {
  await page.goto(`${FRONTEND_URL}/login`);
  await page.getByLabel('Username').fill('admin_vcai_2');
  await page.getByLabel('Password').fill('admin');
  await page.getByRole('button', { name: /sign in/i }).click();
  await page.waitForURL(`${FRONTEND_URL}/`);
}

test('Language toggle updates html lang', async ({ page }) => {
  await doLogin(page);

  // Click ID and wait for html lang to reflect
  await page.getByRole('button', { name: /^ID$/ }).click();
  await page.waitForFunction(() => document.documentElement.lang === 'id');
  const langAttr = await page.evaluate(() => document.documentElement.lang);
  expect(langAttr).toBe('id');

  // Switch back to EN and wait
  await page.getByRole('button', { name: /^EN$/ }).click();
  await page.waitForFunction(() => document.documentElement.lang === 'en');
  const langAttr2 = await page.evaluate(() => document.documentElement.lang);
  expect(langAttr2).toBe('en');
});