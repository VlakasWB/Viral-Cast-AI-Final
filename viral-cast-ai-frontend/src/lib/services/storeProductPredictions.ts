import type { ApiResponse } from '$lib/types';
import { api } from '$lib/utils/api';
import type { StoreProductPredictionsData } from '$lib/types/store-product-prediction';

export type StoreProductPredictionsResponse = ApiResponse<StoreProductPredictionsData>;

export interface StoreProductPredictionsGenerateOptions {
	payload?: Record<string, unknown>;
	temperature?: number;
	max_tokens?: number;
	cookies?: string;
	fetchImpl?: typeof fetch;
}

export interface StoreProductPredictionsGetOptions {
	cookies?: string;
	fetchImpl?: typeof fetch;
}

export function getStoreProductPredictionsApi(
	options: StoreProductPredictionsGetOptions = {}
): Promise<StoreProductPredictionsResponse> {
	const { cookies, fetchImpl } = options;
	return api.get<StoreProductPredictionsData>('/api/v1/stores/predictions', cookies, fetchImpl);
}

export function generateStoreProductPredictionsApi(
	options: StoreProductPredictionsGenerateOptions = {}
): Promise<StoreProductPredictionsResponse> {
	const { payload = {}, temperature, max_tokens, cookies, fetchImpl } = options;
	const searchParams = new URLSearchParams();

	if (typeof temperature === 'number') {
		searchParams.set('temperature', temperature.toString());
	}

	if (typeof max_tokens === 'number') {
		searchParams.set('max_tokens', max_tokens.toString());
	}

	const query = searchParams.toString();
	const endpoint = `/api/v1/stores/predictions${query ? `?${query}` : ''}`;

	return api.post<StoreProductPredictionsData>(endpoint, payload, cookies, fetchImpl);
}
