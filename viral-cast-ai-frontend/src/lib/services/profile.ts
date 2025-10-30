import { browser } from '$app/environment';
import type {
  Profile,
  CreateProfileRequest,
  UpdateProfileRequest,
  ProfileResponse
} from '$lib/types/profile.js';
import type { ImageUploadResponse } from '$lib/types/product.js';

// Ambil base URL API (server & client)
const getApiBaseUrl = () => {
  try {
    return process.env.API_BASE_URL || 'http://localhost:12000';
  } catch {
    return 'http://localhost:12000';
  }
};

class ProfileApiError extends Error {
  constructor(
    message: string,
    public status: number,
    public response?: any
  ) {
    super(message);
    this.name = 'ProfileApiError';
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
      throw new ProfileApiError(
        errorData.message || `HTTP ${response.status}: ${response.statusText}`,
        response.status,
        errorData
      );
    }

    return await response.json();
  } catch (error) {
    if (error instanceof ProfileApiError) {
      throw error;
    }
    throw new ProfileApiError(error instanceof Error ? error.message : 'Network error occurred', 0);
  }
}

// GET /api/v1/profiles (My Profile via JWT)
export async function getMyProfileApi(
  cookies?: string,
  fetchImpl?: typeof fetch
): Promise<ProfileResponse> {
  return makeRequest<ProfileResponse>(
    '/api/v1/profiles',
    { method: 'GET' },
    cookies,
    fetchImpl
  );
}

// POST /api/v1/profiles (Create)
export async function createProfileApi(
  data: CreateProfileRequest,
  cookies?: string,
  fetchImpl?: typeof fetch
): Promise<ProfileResponse> {
  return makeRequest<ProfileResponse>(
    '/api/v1/profiles',
    {
      method: 'POST',
      body: JSON.stringify(data)
    },
    cookies,
    fetchImpl
  );
}

// PUT /api/v1/profiles (Full Update)
export async function updateProfileApi(
  data: CreateProfileRequest | UpdateProfileRequest,
  cookies?: string,
  fetchImpl?: typeof fetch
): Promise<ProfileResponse> {
  return makeRequest<ProfileResponse>(
    '/api/v1/profiles',
    {
      method: 'PUT',
      body: JSON.stringify(data)
    },
    cookies,
    fetchImpl
  );
}

// PATCH /api/v1/profiles (Partial Update)
export async function patchProfileApi(
  data: UpdateProfileRequest,
  cookies?: string,
  fetchImpl?: typeof fetch
): Promise<ProfileResponse> {
  return makeRequest<ProfileResponse>(
    '/api/v1/profiles',
    {
      method: 'PATCH',
      body: JSON.stringify(data)
    },
    cookies,
    fetchImpl
  );
}

export { ProfileApiError };

// POST /api/v1/images/upload/user
export async function uploadUserProfilePhotoApi(
  file: File,
  cookies?: string,
  fetchImpl?: typeof fetch
): Promise<ImageUploadResponse> {
  const API_BASE_URL = getApiBaseUrl();
  const url = `${API_BASE_URL}/api/v1/images/upload/user/profile-photo`;

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

  try {
    const response = await (fetchImpl || fetch)(url, config);
    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new ProfileApiError(
        errorData.message || `HTTP ${response.status}: ${response.statusText}`,
        response.status,
        errorData
      );
    }
    return await response.json();
  } catch (error) {
    if (error instanceof ProfileApiError) {
      throw error;
    }
    throw new ProfileApiError(error instanceof Error ? error.message : 'Network error occurred', 0);
  }
}

export async function uploadUserBackgroundApi(
  file: File,
  cookies?: string,
  fetchImpl?: typeof fetch
): Promise<ImageUploadResponse> {
  const API_BASE_URL = getApiBaseUrl();
  const url = `${API_BASE_URL}/api/v1/images/upload/user/background`;

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

  try {
    const response = await (fetchImpl || fetch)(url, config);
    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new ProfileApiError(
        errorData.message || `HTTP ${response.status}: ${response.statusText}`,
        response.status,
        errorData
      );
    }
    return await response.json();
  } catch (error) {
    if (error instanceof ProfileApiError) {
      throw error;
    }
    throw new ProfileApiError(error instanceof Error ? error.message : 'Network error occurred', 0);
  }
}