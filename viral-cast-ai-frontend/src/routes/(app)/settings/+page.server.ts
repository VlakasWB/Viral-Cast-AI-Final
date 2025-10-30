import type { Actions, PageServerLoad } from './$types';
import { redirect } from '@sveltejs/kit';
import { getMyProfileApi, patchProfileApi, uploadUserProfilePhotoApi, uploadUserBackgroundApi } from '$lib/services/profile.js';
import { getProvincesApi, getRegenciesApi, getDistrictsApi, getVillagesApi } from '$lib/services/regions.js';
import { getStoreByUuidApi, createStoreApi, patchStoreApi, uploadStoreImageApi } from '$lib/services/store.js';

// API base URL for building absolute image URLs
const API_BASE_URL = (process.env.API_BASE_URL || 'http://localhost:12000');
function resolveImageUrl(u?: string): string | null {
  if (!u || typeof u !== 'string') return null;
  const trimmed = String(u).trim().replace(/^`+|`+$/g, '');
  if (/^https?:\/\//i.test(trimmed)) return trimmed;
  const path = trimmed.startsWith('/') ? trimmed : `/${trimmed}`;
  return `${API_BASE_URL}${path}`;
}

export const load: PageServerLoad = async ({ locals, cookies, fetch }) => {
  if (!locals.user) {
    // hooks.server.ts already redirects unauthenticated, but keep a guard
    return { user: null, profile: null, store: null };
  }

  const token = cookies.get('access_token');
  const cookieStr = token ? `access_token=${token}` : undefined;

  let profile: any = null;
  let regionNames: { province?: string; regency?: string; district?: string; village?: string } = {};
  let store: any = null;
  // Tambah penamaan wilayah untuk toko
  let storeRegionNames: { province?: string; regency?: string; district?: string; village?: string } = {};

  try {
    const res = await getMyProfileApi(cookieStr, fetch);
    // Backend might return { data: Profile } or { data: { profile: Profile } }
    profile = (res?.data?.profile) ?? res?.data ?? null;
  } catch (e) {
    profile = null; // keep page rendering even if API down
  }

  const storeUuid: string | undefined =
    (profile?.store_uuid && typeof profile.store_uuid === 'string')
      ? profile.store_uuid
      : (profile?.store?.uuid && typeof profile.store?.uuid === 'string')
        ? profile.store.uuid
        : undefined;

  if (storeUuid) {
    try {
      const storeRes = await getStoreByUuidApi(storeUuid, cookieStr, fetch);
      store = storeRes?.data ?? null;
    } catch (e) {
      store = null; // store isn't critical for settings page
    }
  }

  // Validate remote images to avoid client-side load errors
  async function headOk(url?: string) {
    if (!url || typeof url !== 'string') return false;
    try {
      const resp = await fetch(url, { method: 'HEAD' });
      return !!resp && resp.ok;
    } catch {
      return false;
    }
  }

  // Sanitize profile image URLs
  if (profile) {
    const ppResolved = resolveImageUrl(profile.photo_profile);
    if (ppResolved && !(await headOk(ppResolved))) {
      profile.photo_profile = null;
    } else {
      profile.photo_profile = ppResolved;
    }
    const bgResolved = resolveImageUrl(profile.background_profile);
    if (bgResolved && !(await headOk(bgResolved))) {
      profile.background_profile = null;
    } else {
      profile.background_profile = bgResolved;
    }
  }

  // Sanitize store brand URL
  if (store) {
    const s = store?.store ?? store;
    const brandResolved = resolveImageUrl(s?.brand_url);
    if (brandResolved && !(await headOk(brandResolved))) {
      if ((store as any)?.store) (store as any).store.brand_url = null;
      else (store as any).brand_url = null;
    } else {
      if ((store as any)?.store) (store as any).store.brand_url = brandResolved;
      else (store as any).brand_url = brandResolved;
    }
  }

  // Resolve region names from saved codes
  async function resolveRegionNamesByCodes() {
    if (!profile) return;
    const token = cookies.get('access_token');
    const cookieStr = token ? `access_token=${token}` : undefined;

    // Province
    if (profile.province_code) {
      try {
        const res = await getProvincesApi({ code: String(profile.province_code), limit: 1 }, cookieStr, fetch);
        const match = (res.data || []).find((p) => p.code === profile.province_code);
        if (match) regionNames.province = match.name;
      } catch {}
    }
    // Regency
    if (profile.regency_code) {
      try {
        const res = await getRegenciesApi({ province_code: String(profile.province_code), code: String(profile.regency_code), limit: 1 }, cookieStr, fetch);
        const match = (res.data || []).find((r) => r.code === profile.regency_code);
        if (match) regionNames.regency = match.name;
      } catch {}
    }
    // District
    if (profile.district_code) {
      try {
        const res = await getDistrictsApi({ regency_code: String(profile.regency_code), code: String(profile.district_code), limit: 1 }, cookieStr, fetch);
        const match = (res.data || []).find((d) => d.code === profile.district_code);
        if (match) regionNames.district = match.name;
      } catch {}
    }
    // Village
    if (profile.village_code) {
      try {
        const res = await getVillagesApi({ district_code: String(profile.district_code), code: String(profile.village_code), limit: 1 }, cookieStr, fetch);
        const match = (res.data || []).find((v) => v.code === profile.village_code);
        if (match) regionNames.village = match.name;
      } catch {}
    }
  }

  // Resolve store region names by store codes
  async function resolveStoreRegionNamesByCodes() {
    const s = store?.store ?? store;
    if (!s) return;

    const token = cookies.get('access_token');
    const cookieStr = token ? `access_token=${token}` : undefined;

    // Province (ikuti pola profil)
    if (s.province_code) {
      try {
        const res = await getProvincesApi({ code: String(s.province_code), limit: 1 }, cookieStr, fetch);
        const match = (res.data || []).find((p) => p.code === s.province_code);
        if (match) storeRegionNames.province = match.name;
      } catch {}
    }
    // Regency (ikuti pola profil)
    if (s.regency_code) {
      try {
        const res = await getRegenciesApi({ province_code: String(s.province_code), code: String(s.regency_code), limit: 1 }, cookieStr, fetch);
        const match = (res.data || []).find((r) => r.code === s.regency_code);
        if (match) storeRegionNames.regency = match.name;
      } catch {}
    }
    // District (ikuti pola profil)
    if (s.district_code) {
      try {
        const res = await getDistrictsApi({ regency_code: String(s.regency_code), code: String(s.district_code), limit: 1 }, cookieStr, fetch);
        const match = (res.data || []).find((d) => d.code === s.district_code);
        if (match) storeRegionNames.district = match.name;
      } catch {}
    }
    // Village (ikuti pola profil)
    if (s.village_code) {
      try {
        const res = await getVillagesApi({ district_code: String(s.district_code), code: String(s.village_code), limit: 1 }, cookieStr, fetch);
        const match = (res.data || []).find((v) => v.code === s.village_code);
        if (match) storeRegionNames.village = match.name;
      } catch {}
    }
  }

  try {
    await resolveRegionNamesByCodes();
  } catch {}

  try {
    await resolveStoreRegionNamesByCodes();
  } catch {}

  return {
    user: locals.user,
    profile,
    store,
    regionNames,
    storeRegionNames
  };
};

export const actions: Actions = {
  // Update full profile aligned to Profiles API
  update_profile: async ({ request, cookies, fetch }) => {
    const form = await request.formData();
    const first_name = (form.get('first_name') as string) || '';
    const last_name = (form.get('last_name') as string) || '';
    const gender = (form.get('gender') as string) || '';
    const telp = (form.get('telp') as string) || '';
    const birth_date = (form.get('birth_date') as string) || '';
    const birth_place = (form.get('birth_place') as string) || '';
    const roles_number_raw = form.get('roles_number');
    const roles_number = typeof roles_number_raw === 'string' && roles_number_raw ? Number(roles_number_raw) : undefined;
    const store_uuid_raw = form.get('store_uuid');
    const store_uuid = typeof store_uuid_raw === 'string' && store_uuid_raw.trim() ? store_uuid_raw : undefined;
    const province_code = (form.get('province_code') as string) || '';
    const regency_code = (form.get('regency_code') as string) || '';
    const district_code = (form.get('district_code') as string) || '';
    const village_code = (form.get('village_code') as string) || '';
    const rt = (form.get('rt') as string) || '';
    const rw = (form.get('rw') as string) || '';
    const postal_code = (form.get('postal_code') as string) || '';

    const name = [first_name, last_name].filter(Boolean).join(' ').trim();

    const payload: any = {
      first_name,
      last_name,
      gender,
      telp,
      birth_date,
      birth_place,
      roles_number,
      province_code,
      regency_code,
      district_code,
      village_code,
      rt,
      rw,
      postal_code
    };
    if (store_uuid) { payload.store_uuid = store_uuid; }
    if (typeof roles_number === 'number' && !Number.isNaN(roles_number)) {
      payload.roles_number = roles_number;
    }

    const token = cookies.get('access_token');
    const cookieStr = token ? `access_token=${token}` : undefined;

    try {
      await patchProfileApi(payload, cookieStr, fetch);
      throw redirect(303, '/settings');
    } catch (e: any) {
      return {
        success: false,
        message: e?.message || 'Failed to update profile'
      };
    }
  },

  // Upload user avatar image and persist to photo_profile
  upload_avatar: async ({ request, cookies, fetch }) => {
    const form = await request.formData();
    const file = form.get('avatar');
    if (!(file && file instanceof File)) {
      return { success: false, message: 'No file uploaded' };
    }

    const token = cookies.get('access_token');
    const cookieStr = token ? `access_token=${token}` : undefined;

    try {
      const res = await uploadUserProfilePhotoApi(file, cookieStr, fetch);
      const imageUrl = res?.data?.image_url || null;
      if (imageUrl) {
        await patchProfileApi({ photo_profile: imageUrl }, cookieStr, fetch);
      }
      throw redirect(303, '/settings');
    } catch (e: any) {
      return {
        success: false,
        message: e?.message || 'Failed to upload avatar'
      };
    }
  },

  // Upload background cover image and persist to background_profile
  upload_background: async ({ request, cookies, fetch }) => {
    const form = await request.formData();
    const file = form.get('background');
    if (!(file && file instanceof File)) {
      return { success: false, message: 'No file uploaded' };
    }

    const token = cookies.get('access_token');
    const cookieStr = token ? `access_token=${token}` : undefined;

    try {
      const res = await uploadUserBackgroundApi(file, cookieStr, fetch);
      const imageUrl = res?.data?.image_url || null;
      if (imageUrl) {
        await patchProfileApi({ background_profile: imageUrl }, cookieStr, fetch);
      }
      throw redirect(303, '/settings');
    } catch (e: any) {
      return {
        success: false,
        message: e?.message || 'Failed to upload cover image'
      };
    }
  },

  // Create a new store
  create_store: async ({ request, cookies, fetch }) => {
    const form = await request.formData();
    const name = (form.get('name') as string) || '';
    const telp = (form.get('telp') as string) || undefined;

    if (!name.trim()) {
      return { success: false, message: 'Nama toko wajib diisi' };
    }

    const token = cookies.get('access_token');
    const cookieStr = token ? `access_token=${token}` : undefined;

    try {
      await createStoreApi({ name, telp }, cookieStr, fetch);
      throw redirect(303, '/settings');
    } catch (e: any) {
      return { success: false, message: e?.message || 'Gagal membuat toko' };
    }
  },

  // Update existing store (partial update)
  update_store: async ({ request, cookies, fetch }) => {
    const form = await request.formData();
    const uuid = (form.get('store_uuid') as string) || '';
    const name = (form.get('name') as string) || undefined;
    const telp = (form.get('telp') as string) || undefined;

    if (!uuid) {
      return { success: false, message: 'Store UUID tidak ditemukan' };
    }

    const token = cookies.get('access_token');
    const cookieStr = token ? `access_token=${token}` : undefined;

    try {
      await patchStoreApi(uuid, { name, telp }, cookieStr, fetch);
      throw redirect(303, '/settings');
    } catch (e: any) {
      return { success: false, message: e?.message || 'Gagal memperbarui toko' };
    }
  },

  // Upload brand image for store and persist brand_url
  upload_store_brand: async ({ request, cookies, fetch }) => {
    const form = await request.formData();
    const file = form.get('brand');
    const uuid = (form.get('store_uuid') as string) || '';

    if (!uuid) {
      return { success: false, message: 'Store UUID tidak ditemukan' };
    }
    if (!(file && file instanceof File)) {
      return { success: false, message: 'No file uploaded' };
    }

    const token = cookies.get('access_token');
    const cookieStr = token ? `access_token=${token}` : undefined;

    try {
      const res = await uploadStoreImageApi(file as File, cookieStr, fetch);
      const imageUrl = res?.data?.image_url || null;
      if (imageUrl) {
        await patchStoreApi(uuid, { brand_url: imageUrl }, cookieStr, fetch);
      }
      throw redirect(303, '/settings');
    } catch (e: any) {
      return { success: false, message: e?.message || 'Failed to upload brand image' };
    }
  }
};