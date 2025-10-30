import type { ApiResponse } from '$lib/types';

const getApiBaseUrl = () => {
	try {
		return process.env.API_BASE_URL || 'http://localhost:12000';
	} catch {
		return 'http://localhost:12000';
	}
};

export interface RagAnswerRequest {
	query: string;
	document_ids?: string[];
	category_filter?: string;
	max_results?: number;
	similarity_threshold?: number;
	prompt_instructions?: string;
	max_tokens?: number;
	temperature?: number;
}

export interface RagAnswerResponse {
	answer?: string;
	response?: string;
	model?: string;
	tokens_used?: number;
	usage?: {
		total_tokens?: number;
		prompt_tokens?: number;
		completion_tokens?: number;
	};
	sources?: Array<{
		document_id?: string;
		title?: string;
		snippet?: string;
		score?: number;
		url?: string;
	}>;
	success?: boolean;
	message?: string | null;
}

export interface RagDocumentUploadPayload {
	file: File;
	title?: string;
	description?: string;
	category?: string;
	tags?: string | string[];
}

export interface RagDocumentUploadResponse {
	document_id?: string;
	status?: string;
	success?: boolean;
	message?: string | null;
}

export async function ragAnswerApi(
	request: RagAnswerRequest,
	accessToken?: string
): Promise<ApiResponse<RagAnswerResponse>> {
	const API_BASE_URL = getApiBaseUrl();
	const headers: Record<string, string> = {
		'Content-Type': 'application/json'
	};

	if (accessToken) {
		headers['Authorization'] = `Bearer ${accessToken}`;
	}

	const response = await fetch(`${API_BASE_URL}/api/rag/answer`, {
		method: 'POST',
		headers,
		body: JSON.stringify(request)
	});

	if (!response.ok) {
		throw new Error(`HTTP error! status: ${response.status}`);
	}

	return await response.json();
}

export async function uploadRagDocument(
	payload: RagDocumentUploadPayload,
	accessToken?: string
): Promise<ApiResponse<RagDocumentUploadResponse>> {
	const API_BASE_URL = getApiBaseUrl();
	const formData = new FormData();

	const derivedTitle = payload.title ?? payload.file.name.replace(/\.[^/.]+$/, '');
	const derivedCategory = payload.category ?? 'general';

	formData.append('title', derivedTitle);
	formData.append('category', derivedCategory);
	formData.append('file', payload.file);

	if (payload.description) {
		formData.append('description', payload.description);
	}

	if (payload.tags) {
		const tagsValue = Array.isArray(payload.tags) ? payload.tags.join(',') : payload.tags;
		formData.append('tags', tagsValue);
	}

	const headers: Record<string, string> = {};

	if (accessToken) {
		headers['Authorization'] = `Bearer ${accessToken}`;
	}

	const response = await fetch(`${API_BASE_URL}/api/rag/documents/upload`, {
		method: 'POST',
		body: formData,
		headers
	});

	if (!response.ok) {
		throw new Error(`HTTP error! status: ${response.status}`);
	}

	return await response.json();
}

