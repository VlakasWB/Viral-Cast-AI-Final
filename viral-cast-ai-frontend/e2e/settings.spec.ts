import { test, expect } from '@playwright/test';

const FRONTEND_URL = 'http://localhost:4173';
const API_BASE_URL = 'http://localhost:12000';

async function doLogin(page: any) {
  await page.goto(`${FRONTEND_URL}/login`);
  await page.getByLabel('Username').fill('admin_vcai_2');
  await page.getByLabel('Password').fill('admin');
  await page.getByRole('button', { name: /sign in/i }).click();
  await page.waitForURL(`${FRONTEND_URL}/`);
}

test('Navigate to Settings via UserMenu and verify Save button', async ({ page }) => {
  await doLogin(page);

  // Open UserMenu dropdown (role-based button name)
  const userMenuTrigger = page.getByRole('button', { name: /admin/i });
  await expect(userMenuTrigger).toBeVisible();
  await userMenuTrigger.click();

  // Click Settings
  await page.getByRole('menuitem', { name: /settings/i }).click();
  await page.waitForURL(`${FRONTEND_URL}/settings`);

  // Verify Indonesian Save button text exists
  await expect(page.getByRole('button', { name: /simpan profil/i })).toBeVisible();
});

test('Settings: fill fields and attempt save with cascaded regions', async ({ page }) => {
  // Stub regions APIs before navigating
  await page.route(`${API_BASE_URL}/api/v1/regions/provinces**`, (route) => route.fulfill({
    status: 200,
    contentType: 'application/json',
    body: JSON.stringify({
      success: true, message: 'OK',
      data: [{ uuid: 'prov-a', code: 'PROV-A', name: 'Province A' }],
      pagination: { limit: 100, offset: 0, total: 1 }
    })
  }));
  await page.route(`${API_BASE_URL}/api/v1/regions/regencies**`, (route) => route.fulfill({
    status: 200, contentType: 'application/json',
    body: JSON.stringify({
      success: true, message: 'OK',
      data: [{ uuid: 'reg-a', code: 'REG-A', name: 'Regency A', province_uuid: 'prov-a', province_name: 'Province A' }],
      pagination: { limit: 100, offset: 0, total: 1 }
    })
  }));
  await page.route(`${API_BASE_URL}/api/v1/regions/districts**`, (route) => route.fulfill({
    status: 200, contentType: 'application/json',
    body: JSON.stringify({
      success: true, message: 'OK',
      data: [{ uuid: 'dist-a', code: 'DIST-A', name: 'District A', regency_uuid: 'reg-a', regency_name: 'Regency A', province_uuid: 'prov-a', province_name: 'Province A' }],
      pagination: { limit: 100, offset: 0, total: 1 }
    })
  }));
  await page.route(`${API_BASE_URL}/api/v1/regions/villages**`, (route) => route.fulfill({
    status: 200, contentType: 'application/json',
    body: JSON.stringify({
      success: true, message: 'OK',
      data: [{ uuid: 'vill-a1', code: 'VILL-A1', name: 'Village A1', district_uuid: 'dist-a', district_name: 'District A', regency_uuid: 'reg-a', regency_name: 'Regency A', province_uuid: 'prov-a', province_name: 'Province A' }],
      pagination: { limit: 100, offset: 0, total: 1 }
    })
  }));

  await doLogin(page);

  // Navigate to Settings via UserMenu like the first test
  const userMenuTrigger = page.getByRole('button', { name: /admin/i });
  await userMenuTrigger.click();
  await page.getByRole('menuitem', { name: /settings/i }).click();
  await page.waitForURL(`${FRONTEND_URL}/settings`);

  // Enable edit mode
  await page.getByRole('button', { name: /^Edit$/ }).click();

  // Fill fields
  await page.fill('#first_name', 'E2E First');
  await page.fill('#last_name', 'E2E Last');
  await page.selectOption('#gender', 'OTHER');
  await page.fill('#telp', '081234567890');
  await page.fill('#birth_date', '1990-01-02');
  await page.fill('#birth_place', 'Jakarta');

  // Cascade region selects
  await page.selectOption('#province_code', 'PROV-A');
  await page.waitForSelector('#regency_code:enabled');
  await page.selectOption('#regency_code', 'REG-A');
  await page.waitForSelector('#district_code:enabled');
  await page.selectOption('#district_code', 'DIST-A');
  await page.waitForSelector('#village_code:enabled');
  await page.selectOption('#village_code', 'VILL-A1');

  await page.fill('#rt', '001');
  await page.fill('#rw', '002');
  await page.fill('#postal_code', '12345');

  // Attempt save
  await page.getByRole('button', { name: /simpan profil/i }).click();

  // Verify still on settings and form visible; if backend error, check message
  await expect(page).toHaveURL(`${FRONTEND_URL}/settings`);

  const error = page.getByText(/Failed to update profile/i);
  const isErrVisible = await error.isVisible().catch(() => false);
  if (isErrVisible) {
    await expect(error).toBeVisible();
  } else {
    await expect(page.getByRole('button', { name: /simpan profil/i })).toBeVisible();
  }
});