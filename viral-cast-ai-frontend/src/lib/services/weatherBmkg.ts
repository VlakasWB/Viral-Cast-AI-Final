import { browser } from '$app/environment';

// Types for BMKG Weather Prediction
export interface WeatherLocation {
  province_code?: string;
  province_name?: string;
  regency_code?: string;
  regency_name?: string;
  district_code?: string;
  district_name?: string;
  village_code?: string;
  village_name?: string;
}

export interface WeatherForecastEntry {
  datetime_utc?: string;
  local_datetime?: string;
  weather?: string;
  weather_code?: string;
  t?: number; // temperature (°C)
  hu?: number; // humidity (%)
  tp?: number; // rainfall (mm)
  wd?: string; // wind direction
  ws?: number; // wind speed (m/s)
  tcc?: number; // total cloud cover (%)
  icon_url?: string;
}

export interface WeatherPrediction {
  region_code: string;
  lokasi?: WeatherLocation;
  prakiraan_cuaca?: WeatherForecastEntry[];
  last_updated?: string;
}

export interface WeatherPredictionResponse {
  success: boolean;
  message: string;
  data: WeatherPrediction;
}

// Base URL helper
const getApiBaseUrl = () => {
  try {
    return process.env.API_BASE_URL || 'http://localhost:12000';
  } catch {
    return 'http://localhost:12000';
  }
};

class WeatherApiError extends Error {
  constructor(
    message: string,
    public status: number,
    public response?: any
  ) {
    super(message);
    this.name = 'WeatherApiError';
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
    const isUnauthorized = response.status === 401;
    const msg = String(errorData?.message || '');
    const tokenInvalid = /Token is invalid|session has expired/i.test(msg);
    if (browser && (isUnauthorized || tokenInvalid)) {
      const from = `${window.location.pathname}${window.location.search}`;
      window.location.href = `/login?from=${encodeURIComponent(from)}`;
    }
    throw new WeatherApiError(
      errorData.message || `HTTP ${response.status}: ${response.statusText}`,
      response.status,
      errorData
    );
  }
  return await response.json();
}

export async function getWeatherPredictionApi(
  region_code: string,
  cookies?: string,
  fetchImpl?: typeof fetch
): Promise<WeatherPredictionResponse> {
  const searchParams = new URLSearchParams({ region_code });
  const endpoint = `/api/v1/weather_bmkg/prediction?${searchParams.toString()}`;
  return makeRequest<WeatherPredictionResponse>(endpoint, { method: 'GET' }, cookies, fetchImpl);
}

export { WeatherApiError };