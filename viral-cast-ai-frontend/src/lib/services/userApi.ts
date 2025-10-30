import type { ApiResponse } from '$lib/types';

// Get API base URL - fallback for client-side
const getApiBaseUrl = () => {
	try {
		// Try to import from server environment
		return process.env.API_BASE_URL || 'http://localhost:12000';
	} catch {
		// Fallback for client-side
		return 'http://localhost:12000';
	}
};

export interface User {
	uuid: string;
	username: string;
	email: string | null;
}

// Get current user information
export async function getCurrentUserApi(accessToken?: string): Promise<ApiResponse<User>> {
	const API_BASE_URL = getApiBaseUrl();
	const headers: Record<string, string> = {
		'Content-Type': 'application/json'
	};

	// Add authorization header if token is provided
	if (accessToken) {
		headers['Authorization'] = `Bearer ${accessToken}`;
	}

	const response = await fetch(`${API_BASE_URL}/api/v1/users/me`, {
		method: 'GET',
		headers
	});

	if (!response.ok) {
		throw new Error(`HTTP error! status: ${response.status}`);
	}

	return await response.json();
}
