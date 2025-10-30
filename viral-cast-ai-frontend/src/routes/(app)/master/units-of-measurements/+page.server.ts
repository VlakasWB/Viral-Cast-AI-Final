import { redirect, fail } from '@sveltejs/kit';
import type { PageServerLoad, Actions } from './$types';
import type { UOM } from '$lib/types/uom';
import { deleteUOMApi, searchUnitsOfMeasureApi } from '$lib/services/uom';

export const load: PageServerLoad = async ({ url, cookies, fetch }) => {
  const pageParam = parseInt(url.searchParams.get('page') ?? '1', 10);
  const limitParam = parseInt(url.searchParams.get('limit') ?? '10', 10);
  const search = (url.searchParams.get('search') ?? '').trim();
  const code = (url.searchParams.get('code') ?? '').trim();
  const name = (url.searchParams.get('name') ?? '').trim();

  const page = Number.isNaN(pageParam) || pageParam < 1 ? 1 : pageParam;
  const limit = Number.isNaN(limitParam) || limitParam < 1 ? 10 : limitParam;
  let currentPage = page;

  // Ensure we have a token before calling the API
  const token = cookies.get('access_token');
  if (!token) {
    throw redirect(303, '/login');
  }
  const cookieStr = `access_token=${token}`;

  let items: UOM[] = [];
  let total = 0;
  let pageCount = 1;
  let pageSize = limit;
  const q = search;

  try {
    const params: { page: number; limit: number; search?: string; code?: string; name?: string } = {
      page,
      limit
    };

    if (search) params.search = search;
    if (code) params.code = code;
    if (name) params.name = name;

    const resp = await searchUnitsOfMeasureApi(params, cookieStr, fetch);
    items = Array.isArray(resp.data) ? resp.data : [];

    const meta = resp.meta ?? {
      total: items.length,
      per_page: limit,
      last_page: Math.max(1, Math.ceil(items.length / limit)),
      current_page: page,
      from: items.length ? 1 : 0,
      to: items.length
    };

    total = Number.isFinite(meta.total) ? meta.total : items.length;
    pageCount = Number.isFinite(meta.last_page)
      ? meta.last_page
      : Math.max(1, Math.ceil((total || items.length) / (meta.per_page || limit || 1)));
    pageSize = Number.isFinite(meta.per_page) && meta.per_page > 0 ? meta.per_page : limit;

    // sync page with backend pagination if provided
    if (Number.isFinite(meta.current_page) && meta.current_page > 0) {
      currentPage = meta.current_page;
    }
  } catch (e: any) {
    if (e?.status === 401) {
      throw redirect(303, '/login');
    }
    console.error('Failed to load UOMs from API:', e);
    items = [];
    total = 0;
    pageCount = 1;
  }

  return {
    items,
    total,
    page: currentPage,
    pageCount,
    size: pageSize,
    q
  };
};

export const actions: Actions = {
  delete: async ({ request, cookies, fetch }) => {
    const formData = await request.formData();
    const uuid = String(formData.get('uuid') ?? '');

    const token = cookies.get('access_token');
    if (!token) {
      return fail(401, { success: false, message: 'Unauthorized' });
    }
    const cookieStr = `access_token=${token}`;

    try {
      const resp = await deleteUOMApi(uuid, cookieStr, fetch);
      return { success: true, message: resp.message };
    } catch (error: any) {
      return { success: false, message: error?.message || 'Delete UOM failed' };
    }
  }
};
