import { browser } from '$app/environment';

// Tipe respons API
export interface ApiResponse<T = any> {
  code: number;
  status: string;
  message: string;
  data: T;
  errors?: any;
}

// Kelas error API
export class ApiError extends Error {
  constructor(
    public status: number,
    message: string,
    public response?: any
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

// Mendapatkan base URL API (berfungsi di server dan client)
const getApiBaseUrl = () => {
  try {
    return process.env.API_BASE_URL || 'http://localhost:12000';
  } catch {
    return 'http://localhost:12000';
  }
};

// Fungsi untuk membuat request API
async function makeRequest<TData = any>(
  endpoint: string,
  options: RequestInit = {},
  cookies?: string,
  fetchImpl: typeof fetch = fetch
): Promise<ApiResponse<TData>> {
  const API_BASE_URL = getApiBaseUrl();
  const url = `${API_BASE_URL}${endpoint}`;
  
  // Mendapatkan access token dari cookie (browser atau server)
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
  
  // Menyiapkan headers
  const headers = {
    'Content-Type': 'application/json',
    ...((options.headers || {}) as Record<string, string>),
    ...(accessToken && { Authorization: `Bearer ${accessToken}` })
  } as Record<string, string>;
  
  // Menyiapkan konfigurasi request
  const config: RequestInit = {
    ...options,
    headers,
    credentials: 'include'
  };
  
  try {
    const response = await fetchImpl(url, config);
    
    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new ApiError(
        response.status,
        errorData.message || `HTTP ${response.status}: ${response.statusText}`,
        errorData
      );
    }
    
    return await response.json();
  } catch (error) {
    if (error instanceof ApiError) {
      throw error;
    }
    throw new ApiError(0, error instanceof Error ? error.message : 'Network error occurred');
  }
}

// API client dengan metode HTTP standar
export const api = {
  get: <TData = any>(endpoint: string, cookies?: string, fetchImpl?: typeof fetch): Promise<ApiResponse<TData>> => 
    makeRequest<TData>(endpoint, { method: 'GET' }, cookies, fetchImpl),
    
  post: <TData = any>(endpoint: string, data?: any, cookies?: string, fetchImpl?: typeof fetch): Promise<ApiResponse<TData>> => 
    makeRequest<TData>(endpoint, { 
      method: 'POST',
      body: data ? JSON.stringify(data) : undefined
    }, cookies, fetchImpl),
    
  put: <TData = any>(endpoint: string, data?: any, cookies?: string, fetchImpl?: typeof fetch): Promise<ApiResponse<TData>> => 
    makeRequest<TData>(endpoint, { 
      method: 'PUT',
      body: data ? JSON.stringify(data) : undefined
    }, cookies, fetchImpl),
    
  patch: <TData = any>(endpoint: string, data?: any, cookies?: string, fetchImpl?: typeof fetch): Promise<ApiResponse<TData>> => 
    makeRequest<TData>(endpoint, { 
      method: 'PATCH',
      body: data ? JSON.stringify(data) : undefined
    }, cookies, fetchImpl),
    
  delete: <TData = any>(endpoint: string, cookies?: string, fetchImpl?: typeof fetch): Promise<ApiResponse<TData>> => 
    makeRequest<TData>(endpoint, { method: 'DELETE' }, cookies, fetchImpl)
};