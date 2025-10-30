import { browser } from '$app/environment';
import type {
  Province,
  Regency,
  District,
  Village,
  RegionsListResponse,
  SingleRegionResponse
} from '$lib/types/regions';

// Ambil base URL API (server & client)
const getApiBaseUrl = () => {
  try {
    return process.env.API_BASE_URL || 'http://localhost:12000';
  } catch {
    return 'http://localhost:12000';
  }
};

async function makeRequest<T>(
  endpoint: string,
  options: RequestInit = {},
  cookies?: string,
  fetchImpl: typeof fetch = fetch
): Promise<T> {
  const API_BASE_URL = getApiBaseUrl();
  const url = `${API_BASE_URL}${endpoint}`;

  // Ambil access token dari cookie (browser atau server)
  let accessToken: string | undefined;
  if (browser) {
    accessToken = document.cookie
      .split('; ')
      .find((row) => row.startsWith('access_token='))
      ?.split('=')[1];
  } else if (cookies) {
    accessToken = cookies
      .split('; ')
      .find((row) => row.startsWith('access_token='))
      ?.split('=')[1];
  }

  const defaultHeaders: HeadersInit = {
    'Accept': 'application/json',
    'Content-Type': 'application/json',
    ...(accessToken && { Authorization: `Bearer ${accessToken}` })
  };

  const config: RequestInit = {
    ...options,
    headers: {
      ...defaultHeaders,
      ...options.headers
    },
    credentials: 'include'
  };

  const response = await fetchImpl(url, config);
  if (!response.ok) {
    const errorData = await response.json().catch(() => ({}));
    throw new RegionsApiError(
      errorData.message || `HTTP ${response.status}: ${response.statusText}`,
      response.status,
      errorData
    );
  }
  return await response.json();
}

// ========== REGIONS API: REAL BACKEND CALLS ==========
export async function getProvincesApi(
  params?: { limit?: number; offset?: number; code?: string; search?: string },
  cookies?: string,
  fetchImpl?: typeof fetch
): Promise<RegionsListResponse<Province>> {
  // If code provided, use path endpoint: /provinces/{code}
  if (params?.code) {
    const endpoint = `/api/v1/regions/provinces/${encodeURIComponent(params.code)}`;
    const rawSingle = await makeRequest<SingleRegionResponse<any>>(endpoint, { method: 'GET' }, cookies, fetchImpl);
    const p = rawSingle?.data;
    const data: Province[] = p
      ? [
          {
            uuid: p.uuid ?? p.code,
            code: p.code,
            name: p.name
          }
        ]
      : [];
    return {
      success: rawSingle.success,
      message: rawSingle.message,
      data,
      pagination: { total: data.length, limit: data.length || 0, offset: 0, has_next: false }
    };
  }

  // Otherwise list endpoint with optional filters
  const searchParams = new URLSearchParams();
  if (params?.search) searchParams.set('search', params.search);
  if (params?.limit) searchParams.set('limit', String(params.limit));
  if (params?.offset) searchParams.set('offset', String(params.offset));

  const endpoint = `/api/v1/regions/provinces${searchParams.size ? `?${searchParams.toString()}` : ''}`;
  const raw = await makeRequest<RegionsListResponse<any>>(endpoint, { method: 'GET' }, cookies, fetchImpl);

  const data: Province[] = (raw?.data || []).map((p: any) => ({
    uuid: p.uuid ?? p.code,
    code: p.code,
    name: p.name
  }));

  return { ...raw, data };
}

function looksLikeUuid(value?: string): boolean {
  return !!value && /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(value);
}

export async function getRegenciesApi(
  params?: { province_uuid?: string; province_code?: string; code?: string; limit?: number; offset?: number; search?: string },
  cookies?: string,
  fetchImpl?: typeof fetch
): Promise<RegionsListResponse<Regency>> {
  // If code provided, use path endpoint: /regencies/{code}
  if (params?.code) {
    const endpoint = `/api/v1/regions/regencies/${encodeURIComponent(params.code)}`;
    const rawSingle = await makeRequest<SingleRegionResponse<any>>(endpoint, { method: 'GET' }, cookies, fetchImpl);
    const r = rawSingle?.data;
    const data: Regency[] = r
      ? [
          {
            uuid: r.uuid ?? r.code,
            code: r.code,
            name: r.name,
            province_uuid: r.province_uuid ?? r.province_code,
            province_name: r.province_name,
            province_code: r.province_code
          }
        ]
      : [];
    return {
      success: rawSingle.success,
      message: rawSingle.message,
      data,
      pagination: { total: data.length, limit: data.length || 0, offset: 0, has_next: false }
    };
  }

  // Otherwise list endpoint with optional filters
  const searchParams = new URLSearchParams();
  if (params?.province_uuid) {
    searchParams.set('province_uuid', params.province_uuid);
    if (!looksLikeUuid(params.province_uuid)) {
      searchParams.set('province_code', params.province_uuid);
    }
  }
  if (params?.province_code) searchParams.set('province_code', params.province_code);
  if (params?.search) searchParams.set('search', params.search);
  if (params?.limit) searchParams.set('limit', String(params.limit));
  if (params?.offset) searchParams.set('offset', String(params.offset));

  const endpoint = `/api/v1/regions/regencies${searchParams.size ? `?${searchParams.toString()}` : ''}`;
  const raw = await makeRequest<RegionsListResponse<any>>(endpoint, { method: 'GET' }, cookies, fetchImpl);

  const data: Regency[] = (raw?.data || []).map((r: any) => ({
    uuid: r.uuid ?? r.code,
    code: r.code,
    name: r.name,
    province_uuid: r.province_uuid ?? r.province_code,
    province_name: r.province_name,
    province_code: r.province_code
  }));

  return { ...raw, data };
}

export async function getDistrictsApi(
  params?: { regency_uuid?: string; regency_code?: string; province_uuid?: string; province_code?: string; code?: string; limit?: number; offset?: number; search?: string },
  cookies?: string,
  fetchImpl?: typeof fetch
): Promise<RegionsListResponse<District>> {
  // If code provided, use path endpoint: /districts/{code}
  if (params?.code) {
    const endpoint = `/api/v1/regions/districts/${encodeURIComponent(params.code)}`;
    const rawSingle = await makeRequest<SingleRegionResponse<any>>(endpoint, { method: 'GET' }, cookies, fetchImpl);
    const d = rawSingle?.data;
    const data: District[] = d
      ? [
          {
            uuid: d.uuid ?? d.code,
            code: d.code,
            name: d.name,
            regency_uuid: d.regency_uuid ?? d.regency_code,
            regency_name: d.regency_name,
            province_uuid: d.province_uuid ?? d.province_code,
            province_name: d.province_name,
            regency_code: d.regency_code,
            province_code: d.province_code
          }
        ]
      : [];
    return {
      success: rawSingle.success,
      message: rawSingle.message,
      data,
      pagination: { total: data.length, limit: data.length || 0, offset: 0, has_next: false }
    };
  }

  // Otherwise list endpoint with optional filters
  const searchParams = new URLSearchParams();
  if (params?.regency_uuid) {
    searchParams.set('regency_uuid', params.regency_uuid);
    if (!looksLikeUuid(params.regency_uuid)) {
      searchParams.set('regency_code', params.regency_uuid);
    }
  }
  if (params?.regency_code) searchParams.set('regency_code', params.regency_code);
  if (params?.province_uuid) {
    searchParams.set('province_uuid', params.province_uuid);
    if (!looksLikeUuid(params.province_uuid)) {
      searchParams.set('province_code', params.province_uuid);
    }
  }
  if (params?.province_code) searchParams.set('province_code', params.province_code);
  if (params?.search) searchParams.set('search', params.search);
  if (params?.limit) searchParams.set('limit', String(params.limit));
  if (params?.offset) searchParams.set('offset', String(params.offset));

  const endpoint = `/api/v1/regions/districts${searchParams.size ? `?${searchParams.toString()}` : ''}`;
  const raw = await makeRequest<RegionsListResponse<any>>(endpoint, { method: 'GET' }, cookies, fetchImpl);

  const data: District[] = (raw?.data || []).map((d: any) => ({
    uuid: d.uuid ?? d.code,
    code: d.code,
    name: d.name,
    regency_uuid: d.regency_uuid ?? d.regency_code,
    regency_name: d.regency_name,
    province_uuid: d.province_uuid ?? d.province_code,
    province_name: d.province_name,
    regency_code: d.regency_code,
    province_code: d.province_code
  }));

  return { ...raw, data };
}

export async function getVillagesApi(
  params?: { district_uuid?: string; district_code?: string; regency_uuid?: string; regency_code?: string; province_uuid?: string; province_code?: string; code?: string; limit?: number; offset?: number; search?: string },
  cookies?: string,
  fetchImpl?: typeof fetch
): Promise<RegionsListResponse<Village>> {
  // If code provided, use path endpoint: /villages/{code}
  if (params?.code) {
    const endpoint = `/api/v1/regions/villages/${encodeURIComponent(params.code)}`;
    const rawSingle = await makeRequest<SingleRegionResponse<any>>(endpoint, { method: 'GET' }, cookies, fetchImpl);
    const v = rawSingle?.data;
    const data: Village[] = v
      ? [
          {
            uuid: v.uuid ?? v.code,
            code: v.code,
            name: v.name,
            district_uuid: v.district_uuid ?? v.district_code,
            district_name: v.district_name,
            regency_uuid: v.regency_uuid ?? v.regency_code,
            regency_name: v.regency_name,
            province_uuid: v.province_uuid ?? v.province_code,
            province_name: v.province_name,
            district_code: v.district_code,
            regency_code: v.regency_code,
            province_code: v.province_code
          }
        ]
      : [];
    return {
      success: rawSingle.success,
      message: rawSingle.message,
      data,
      pagination: { total: data.length, limit: data.length || 0, offset: 0, has_next: false }
    };
  }

  // Otherwise list endpoint with optional filters
  const searchParams = new URLSearchParams();
  if (params?.district_uuid) {
    searchParams.set('district_uuid', params.district_uuid);
    if (!looksLikeUuid(params.district_uuid)) {
      searchParams.set('district_code', params.district_uuid);
    }
  }
  if (params?.district_code) searchParams.set('district_code', params.district_code);
  if (params?.regency_uuid) {
    searchParams.set('regency_uuid', params.regency_uuid);
    if (!looksLikeUuid(params.regency_uuid)) {
      searchParams.set('regency_code', params.regency_uuid);
    }
  }
  if (params?.regency_code) searchParams.set('regency_code', params.regency_code);
  if (params?.province_uuid) {
    searchParams.set('province_uuid', params.province_uuid);
    if (!looksLikeUuid(params.province_uuid)) {
      searchParams.set('province_code', params.province_uuid);
    }
  }
  if (params?.province_code) searchParams.set('province_code', params.province_code);
  if (params?.search) searchParams.set('search', params.search);
  if (params?.limit) searchParams.set('limit', String(params.limit));
  if (params?.offset) searchParams.set('offset', String(params.offset));

  const endpoint = `/api/v1/regions/villages${searchParams.size ? `?${searchParams.toString()}` : ''}`;
  const raw = await makeRequest<RegionsListResponse<any>>(endpoint, { method: 'GET' }, cookies, fetchImpl);

  const data: Village[] = (raw?.data || []).map((v: any) => ({
    uuid: v.uuid ?? v.code,
    code: v.code,
    name: v.name,
    district_uuid: v.district_uuid ?? v.district_code,
    district_name: v.district_name,
    regency_uuid: v.regency_uuid ?? v.regency_code,
    regency_name: v.regency_name,
    province_uuid: v.province_uuid ?? v.province_code,
    province_name: v.province_name,
    district_code: v.district_code,
    regency_code: v.regency_code,
    province_code: v.province_code
  }));

  return { ...raw, data };
}

class RegionsApiError extends Error {
    constructor(
        message: string,
        public status: number,
        public response?: any
    ) {
        super(message);
        this.name = 'RegionsApiError';
    }
}

export { RegionsApiError };
