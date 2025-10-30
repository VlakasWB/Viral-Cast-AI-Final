// Store API service for viewing store details
import { browser } from '$app/environment';
import type { ImageUploadResponse } from '$lib/types/product.js';
import type { Store, CreateStoreRequest, UpdateStoreRequest, StoreSingleResponse } from '$lib/types/store.js';

export interface StoreResponse<T = any> {
  code: number;
  status: string;
  message: string;
  data: T;
  errors?: any;
}

// Ambil base URL API (server & client)
const getApiBaseUrl = () => {
  try {
    return process.env.API_BASE_URL || 'http://localhost:12000';
  } catch {
    return 'http://localhost:12000';
  }
};

class StoreApiError extends Error {
  constructor(message: string, public status: number, public response?: any) {
    super(message);
    this.name = 'StoreApiError';
  }
}

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

  try {
    const response = await fetchImpl(url, config);
    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new StoreApiError(
        errorData.message || `HTTP ${response.status}: ${response.statusText}`,
        response.status,
        errorData
      );
    }
    return await response.json();
  } catch (error) {
    if (error instanceof StoreApiError) throw error;
    throw new StoreApiError(error instanceof Error ? error.message : 'Network error occurred', 0);
  }
}

// GET /api/v1/stores
export async function getStoresApi(
  cookies?: string,
  fetchImpl?: typeof fetch
): Promise<StoreResponse> {
  return makeRequest<StoreResponse>(`/api/v1/stores`, { method: 'GET' }, cookies, fetchImpl);
}

// GET /api/v1/stores/{uuid}
export async function getStoreByUuidApi(
  uuid: string,
  cookies?: string,
  fetchImpl?: typeof fetch
): Promise<StoreResponse> {
  return makeRequest<StoreResponse>(`/api/v1/stores/${uuid}`, { method: 'GET' }, cookies, fetchImpl);
}

export { StoreApiError };

// POST /api/v1/stores
export async function createStoreApi(
  data: CreateStoreRequest,
  cookies?: string,
  fetchImpl?: typeof fetch
): Promise<StoreSingleResponse> {
  return makeRequest<StoreSingleResponse>(
    '/api/v1/stores',
    { method: 'POST', body: JSON.stringify(data) },
    cookies,
    fetchImpl
  );
}

// PUT /api/v1/stores/{uuid}
export async function updateStoreApi(
  uuid: string,
  data: UpdateStoreRequest,
  cookies?: string,
  fetchImpl?: typeof fetch
): Promise<StoreSingleResponse> {
  return makeRequest<StoreSingleResponse>(
    `/api/v1/stores/${uuid}`,
    { method: 'PUT', body: JSON.stringify(data) },
    cookies,
    fetchImpl
  );
}

// PATCH /api/v1/stores/{uuid}
export async function patchStoreApi(
  uuid: string,
  data: UpdateStoreRequest,
  cookies?: string,
  fetchImpl?: typeof fetch
): Promise<StoreSingleResponse> {
  return makeRequest<StoreSingleResponse>(
    `/api/v1/stores/${uuid}`,
    { method: 'PATCH', body: JSON.stringify(data) },
    cookies,
    fetchImpl
  );
}

// POST /api/v1/images/upload/store
export async function uploadStoreImageApi(
  file: File,
  cookies?: string,
  fetchImpl?: typeof fetch
): Promise<ImageUploadResponse> {
  const API_BASE_URL = getApiBaseUrl();
  const url = `${API_BASE_URL}/api/v1/images/upload/store`;

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

  const formData = new FormData();
  formData.append('image', file);

  const config: RequestInit = {
    method: 'POST',
    body: formData,
    headers: {
      ...(accessToken && { Authorization: `Bearer ${accessToken}` })
    },
    credentials: 'include'
  };

  const response = await (fetchImpl || fetch)(url, config);
  if (!response.ok) {
    const errorData = await response.json().catch(() => ({}));
    throw new StoreApiError(
      errorData.message || `HTTP ${response.status}: ${response.statusText}`,
      response.status,
      errorData
    );
  }
  return await response.json();
}