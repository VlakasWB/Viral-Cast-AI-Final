import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import { loginApi, refreshTokenApi, logoutApi, checkAuthStatus, AuthApiError } from './auth.js';

// Mock fetch globally
const mockFetch = vi.fn();
global.fetch = mockFetch;

describe('Authentication Service', () => {
	beforeEach(() => {
		mockFetch.mockClear();
	});

	afterEach(() => {
		vi.restoreAllMocks();
	});

	describe('loginApi', () => {
		it('should successfully login with valid credentials', async () => {
			const mockResponse = {
				code: 200,
				status: 'OK',
				message: 'Login success',
				data: {
					access_token: 'mock-access-token-12345'
				},
				errors: {}
			};

			mockFetch.mockResolvedValueOnce({
				ok: true,
				status: 200,
				json: () => Promise.resolve(mockResponse)
			});

			const credentials = {
				username: 'admin_vcai_2',
				password: 'admin'
			};

			const result = await loginApi(credentials);

			expect(mockFetch).toHaveBeenCalledWith('http://localhost:12000/api/v1/auth/login', {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				},
				credentials: 'include',
				body: JSON.stringify(credentials)
			});

			expect(result).toEqual({
				access_token: 'mock-access-token-12345',
				refresh_token: 'mock-access-token-12345', // Using access token as refresh token
				user: {
					id: 'admin_vcai_2',
					username: 'admin_vcai_2',
					name: 'admin_vcai_2'
				}
			});
		});

		it('should throw AuthApiError on login failure', async () => {
			const mockErrorResponse = {
				message: 'Invalid credentials'
			};

			mockFetch.mockResolvedValueOnce({
				ok: false,
				status: 401,
				json: () => Promise.resolve(mockErrorResponse)
			});

			const credentials = {
				username: 'invalid',
				password: 'invalid'
			};

			await expect(loginApi(credentials)).rejects.toThrow(AuthApiError);
		});

		it('should handle network errors', async () => {
			mockFetch.mockRejectedValueOnce(new Error('Network error'));

			const credentials = {
				username: 'admin_vcai_2',
				password: 'admin'
			};

			await expect(loginApi(credentials)).rejects.toThrow(AuthApiError);
			await expect(loginApi(credentials)).rejects.toThrow('Network error during login');
		});
	});

	describe('refreshTokenApi', () => {
		it('should successfully refresh token with valid refresh token', async () => {
			const mockResponse = {
				code: 200,
				status: 'OK',
				message: 'Token refreshed',
				data: {
					access_token: 'new-access-token-67890'
				},
				errors: {}
			};

			mockFetch.mockResolvedValueOnce({
				ok: true,
				status: 200,
				json: () => Promise.resolve(mockResponse)
			});

			const refreshToken = 'mock-refresh-token';
			const result = await refreshTokenApi(refreshToken);

			expect(mockFetch).toHaveBeenCalledWith('http://localhost:12000/api/v1/auth/refresh', {
				method: 'GET',
				headers: {
					'Content-Type': 'application/json',
					Authorization: 'Bearer mock-refresh-token'
				},
				credentials: 'include'
			});

			expect(result).toEqual({
				access_token: 'new-access-token-67890',
				refresh_token: 'mock-refresh-token'
			});
		});

		it('should work without refresh token parameter', async () => {
			const mockResponse = {
				code: 200,
				status: 'OK',
				message: 'Token refreshed',
				data: {
					access_token: 'new-access-token-67890'
				},
				errors: {}
			};

			mockFetch.mockResolvedValueOnce({
				ok: true,
				status: 200,
				json: () => Promise.resolve(mockResponse)
			});

			const result = await refreshTokenApi();

			expect(mockFetch).toHaveBeenCalledWith('http://localhost:12000/api/v1/auth/refresh', {
				method: 'GET',
				headers: {
					'Content-Type': 'application/json'
				},
				credentials: 'include'
			});

			expect(result).toEqual({
				access_token: 'new-access-token-67890',
				refresh_token: undefined
			});
		});

		it('should throw AuthApiError on refresh failure', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: false,
				status: 403,
				json: () => Promise.resolve({ message: 'Could not refresh access token' })
			});

			await expect(refreshTokenApi('invalid-token')).rejects.toThrow(AuthApiError);
		});
	});

	describe('logoutApi', () => {
		it('should successfully logout with access token', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: true,
				status: 200
			});

			const accessToken = 'mock-access-token';
			await logoutApi(accessToken);

			expect(mockFetch).toHaveBeenCalledWith('http://localhost:12000/api/v1/auth/logout', {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json',
					Authorization: 'Bearer mock-access-token'
				},
				credentials: 'include'
			});
		});

		it('should work without access token parameter', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: true,
				status: 200
			});

			await logoutApi();

			expect(mockFetch).toHaveBeenCalledWith('http://localhost:12000/api/v1/auth/logout', {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				},
				credentials: 'include'
			});
		});

		it('should throw AuthApiError on logout failure', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: false,
				status: 403,
				json: () => Promise.resolve({ message: 'Token is invalid or session has expired' })
			});

			await expect(logoutApi('invalid-token')).rejects.toThrow(AuthApiError);
		});

		it('should handle network error during logout', async () => {
			mockFetch.mockRejectedValueOnce(new Error('Network error'));

			await expect(logoutApi('invalid-token')).rejects.toThrow(AuthApiError);
			await expect(logoutApi('invalid-token')).rejects.toThrow('Network error during logout');
		});
	});

	describe('checkAuthStatus', () => {
		it('should return true when refresh is successful', async () => {
			const mockResponse = {
				code: 200,
				status: 'OK',
				message: 'Token refreshed',
				data: {
					access_token: 'new-access-token'
				},
				errors: {}
			};

			mockFetch.mockResolvedValueOnce({
				ok: true,
				status: 200,
				json: () => Promise.resolve(mockResponse)
			});

			const result = await checkAuthStatus('valid-refresh-token');
			expect(result).toBe(true);
		});

		it('should return false when refresh fails', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: false,
				status: 403,
				json: () => Promise.resolve({ message: 'Could not refresh access token' })
			});

			const result = await checkAuthStatus('invalid-refresh-token');
			expect(result).toBe(false);
		});

		it('should return false on network error', async () => {
			mockFetch.mockRejectedValueOnce(new Error('Network error'));

			const result = await checkAuthStatus('some-token');
			expect(result).toBe(false);
		});
	});

	describe('AuthApiError', () => {
		it('should create error with correct properties', () => {
			const error = new AuthApiError(404, 'Not found');

			expect(error.status).toBe(404);
			expect(error.message).toBe('Not found');
			expect(error).toBeInstanceOf(Error);
		});
	});
});
