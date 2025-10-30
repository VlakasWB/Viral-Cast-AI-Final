// Authentication API service
// Handles login, refresh, and logout operations with the backend API

import { API_BASE_URL } from '$env/static/private';

export interface LoginCredentials {
	username: string;
	password: string;
}

export interface AuthResponse {
	access_token: string;
	refresh_token?: string;
	user?: {
		id: string;
		username: string;
		email?: string;
		name?: string;
	};
}

export interface ApiResponse<T> {
	code: number;
	status: string;
	message: string;
	data: T;
	errors: any;
}

export interface ApiError {
	message: string;
	status: number;
}

export class AuthApiError extends Error {
	constructor(
		public status: number,
		message: string
	) {
		super(message);
		this.name = 'AuthApiError';
	}
}

/**
 * Login with username and password
 */
export async function loginApi(credentials: LoginCredentials): Promise<AuthResponse> {
	try {
		const response = await fetch(`${API_BASE_URL}/api/v1/auth/login`, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			credentials: 'include', // Include cookies in request
			body: JSON.stringify(credentials)
		});

		if (!response.ok) {
			const errorData = await response.json().catch(() => ({ message: 'Login failed' }));
			throw new AuthApiError(response.status, errorData.message || 'Login failed');
		}

		const apiResponse: ApiResponse<{ access_token: string; refresh_token?: string }> =
			await response.json();
		return {
			access_token: apiResponse.data.access_token,
			refresh_token: apiResponse.data.refresh_token ?? apiResponse.data.access_token, // Prefer server-provided refresh token, fallback for compatibility
			user: {
				id: credentials.username,
				username: credentials.username,
				name: credentials.username
			}
		};
	} catch (error) {
		if (error instanceof AuthApiError) {
			throw error;
		}
		throw new AuthApiError(500, 'Network error during login');
	}
}

/**
 * Refresh access token using refresh token
 */
export async function refreshTokenApi(
	refreshToken?: string,
	fetchImpl: typeof fetch = fetch
): Promise<AuthResponse> {
	try {
		const headers: Record<string, string> = {
			'Content-Type': 'application/json'
		};

		// Add Authorization header if refresh token is provided
		if (refreshToken) {
			headers['Authorization'] = `Bearer ${refreshToken}`;
		}

		const response = await fetchImpl(`${API_BASE_URL}/api/v1/auth/refresh`, {
			method: 'GET',
			headers,
			credentials: 'include' // Include cookies in request
		});

		if (!response.ok) {
			const errorData = await response.json().catch(() => ({ message: 'Token refresh failed' }));
			throw new AuthApiError(response.status, errorData.message || 'Token refresh failed');
		}

		const apiResponse: ApiResponse<{ access_token: string; refresh_token?: string }> =
			await response.json();
		return {
			access_token: apiResponse.data.access_token,
			refresh_token: apiResponse.data.refresh_token ?? refreshToken // Prefer rotated refresh token if provided
		};
	} catch (error) {
		if (error instanceof AuthApiError) {
			throw error;
		}
		throw new AuthApiError(500, 'Network error during token refresh');
	}
}

/**
 * Logout and invalidate tokens
 */
export async function logoutApi(accessToken?: string): Promise<void> {
	try {
		const headers: Record<string, string> = {
			'Content-Type': 'application/json'
		};

		// Add Authorization header if access token is provided
		if (accessToken) {
			headers['Authorization'] = `Bearer ${accessToken}`;
		}

		const response = await fetch(`${API_BASE_URL}/api/v1/auth/logout`, {
			method: 'POST',
			headers,
			credentials: 'include' // Include cookies in request
		});

		if (!response.ok) {
			const errorData = await response.json().catch(() => ({ message: 'Logout failed' }));
			throw new AuthApiError(response.status, errorData.message || 'Logout failed');
		}
	} catch (error) {
		if (error instanceof AuthApiError) {
			throw error;
		}
		throw new AuthApiError(500, 'Network error during logout');
	}
}

/**
 * Check if user is authenticated by trying to refresh token
 */
export async function checkAuthStatus(refreshToken?: string): Promise<boolean> {
	try {
		await refreshTokenApi(refreshToken);
		return true;
	} catch {
		return false;
	}
}
