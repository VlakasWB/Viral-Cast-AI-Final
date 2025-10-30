import { test, expect, Page, Route } from '@playwright/test';

const FRONTEND_URL = 'http://localhost:4173';
const API_BASE_URL = 'http://localhost:12000';

async function doLogin(page: Page) {
  await page.goto(`${FRONTEND_URL}/login`);
  await page.getByLabel('Username').fill('admin_vcai_2');
  await page.getByLabel('Password').fill('admin');
  await page.getByRole('button', { name: /sign in/i }).click();
  await page.waitForURL(`${FRONTEND_URL}/`);
}

function fulfillJson(route: Route, json: any, status = 200) {
  return route.fulfill({
    status,
    contentType: 'application/json',
    body: JSON.stringify(json)
  });
}

test('Weather: stub prediction and verify forecasts and ops badges', async ({ page }) => {
  // Stub regions cascade APIs used in the weather page
  await page.route(`${API_BASE_URL}/api/v1/regions/provinces**`, (route) => {
    return fulfillJson(route, {
      success: true,
      message: 'OK',
      data: [
        { uuid: 'prov-a', code: 'PROV-A', name: 'Province A' }
      ],
      pagination: { limit: 100, offset: 0, total: 1 }
    });
  });

  await page.route(`${API_BASE_URL}/api/v1/regions/regencies**`, (route) => {
    return fulfillJson(route, {
      success: true,
      message: 'OK',
      data: [
        { uuid: 'reg-a', code: 'REG-A', name: 'Regency A', province_uuid: 'prov-a', province_name: 'Province A' }
      ],
      pagination: { limit: 100, offset: 0, total: 1 }
    });
  });

  await page.route(`${API_BASE_URL}/api/v1/regions/districts**`, (route) => {
    return fulfillJson(route, {
      success: true,
      message: 'OK',
      data: [
        { uuid: 'dist-a', code: 'DIST-A', name: 'District A', regency_uuid: 'reg-a', regency_name: 'Regency A', province_uuid: 'prov-a', province_name: 'Province A' }
      ],
      pagination: { limit: 100, offset: 0, total: 1 }
    });
  });

  await page.route(`${API_BASE_URL}/api/v1/regions/villages**`, (route) => {
    return fulfillJson(route, {
      success: true,
      message: 'OK',
      data: [
        { uuid: 'vill-a1', code: 'VILL-A1', name: 'Village A1', district_uuid: 'dist-a', district_name: 'District A', regency_uuid: 'reg-a', regency_name: 'Regency A', province_uuid: 'prov-a', province_name: 'Province A' }
      ],
      pagination: { limit: 100, offset: 0, total: 1 }
    });
  });

  // Stub weather prediction API with realistic fields based on WeatherPredictionResponse
  await page.route(`${API_BASE_URL}/api/v1/weather_bmkg/prediction**`, (route) => {
    const url = new URL(route.request().url());
    const regionCode = url.searchParams.get('region_code') || 'VILL-A1';
    const response = {
      success: true,
      message: 'OK',
      data: {
        region_code: regionCode,
        lokasi: {
          province_code: 'PROV-A',
          province_name: 'Province A',
          regency_code: 'REG-A',
          regency_name: 'Regency A',
          district_code: 'DIST-A',
          district_name: 'District A',
          village_code: 'VILL-A1',
          village_name: 'Village A1'
        },
        last_updated: new Date().toISOString(),
        prakiraan_cuaca: [
          {
            local_datetime: '2025-10-19T09:00:00',
            weather_code: '60', // rainy code
            t: 32,
            hu: 88,
            tp: 3.0,
            wd: 'SW',
            ws: 9,
            tcc: 70
          },
          {
            local_datetime: '2025-10-19T12:00:00',
            weather_code: '0', // clear code
            t: 24,
            hu: 50,
            tp: 0,
            wd: 'NE',
            ws: 2,
            tcc: 20
          }
        ]
      }
    };
    return fulfillJson(route, response);
  });

  await doLogin(page);

  // Navigate to weather page and select a region cascade
  await page.goto(`${FRONTEND_URL}/weather`);

  // These selectors are from weather page's dropdowns
  await page.selectOption('#province', 'prov-a');
  await page.selectOption('#regency', 'reg-a');
  await page.selectOption('#district', 'dist-a');
  await page.selectOption('#village', 'vill-a1');

  // Trigger prediction fetch
  await page.getByRole('button', { name: /view prediction|lihat prakiraan/i }).click();

  // Assertions: region info is visible
  await expect(page.getByText(/Province A|Provinsi A/i)).toBeVisible();
  await expect(page.getByText(/Regency A|Kabupaten A|Kota A/i)).toBeVisible();
  await expect(page.getByText(/District A|Kecamatan A/i)).toBeVisible();
  await expect(page.getByText(/Village A1|Desa A1|Kelurahan A1/i)).toBeVisible();

  // Assertions: a rainy forecast card is present
  await expect(page.getByText(/Rain|Hujan/i)).toBeVisible();

  // Assertions: operational recommendation badges from computeOpsBadges are visible
  await expect(page.getByText(/Provide umbrellas\/canopy|Siapkan payung\/kanopi/i)).toBeVisible();
  await expect(page.getByText(/Promote delivery & takeaway|Dorong promo delivery/i)).toBeVisible();
  await expect(page.getByText(/Add ice stock|Tambahkan stok es/i)).toBeVisible();
  await expect(page.getByText(/Strengthen cooling|Perkuat pendinginan/i)).toBeVisible();
  await expect(page.getByText(/Protect dry goods|Lindungi bahan kering/i)).toBeVisible();
  await expect(page.getByText(/Secure outdoor signage|Amankan signage/i)).toBeVisible();
});