import { test, expect } from '@playwright/test';

const FRONTEND_URL = 'http://localhost:4173';

async function doLogin(page: any) {
  await page.goto(`${FRONTEND_URL}/login`);
  await page.getByLabel('Username').fill('admin_vcai_2');
  await page.getByLabel('Password').fill('admin');
  await page.getByRole('button', { name: /sign in/i }).click();
  await page.waitForURL(`${FRONTEND_URL}/`);
}

test('Login redirects to dashboard and shows UserMenu', async ({ page }) => {
  await doLogin(page);

  // Verify html lang default
  const langAttr = await page.evaluate(() => document.documentElement.lang);
  expect(langAttr).toBe('en');

  // Verify UserMenu trigger with name Admin is visible
  await expect(page.getByRole('button', { name: /admin/i })).toBeVisible();
});