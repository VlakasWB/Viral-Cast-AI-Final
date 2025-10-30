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

export interface AIChatRequest {
	prompt: string;
	max_tokens?: number;
	temperature?: number;
}

export interface AIChatResponse {
	response: string;
	tokens_used: number;
	tokens_remaining: number;
	model: string;
	success: boolean;
	message: string | null;
}

// AI Chat API call
export async function aiChatApi(
	request: AIChatRequest,
	accessToken?: string
): Promise<ApiResponse<AIChatResponse>> {
	const API_BASE_URL = getApiBaseUrl();
	const headers: Record<string, string> = {
		'Content-Type': 'application/json'
	};

	// Add authorization header if token is provided
	if (accessToken) {
		headers['Authorization'] = `Bearer ${accessToken}`;
	}

	const response = await fetch(`${API_BASE_URL}/api/ai/chat`, {
		method: 'POST',
		headers,
		body: JSON.stringify(request)
	});

	if (!response.ok) {
		throw new Error(`HTTP error! status: ${response.status}`);
	}

	return await response.json();
}
