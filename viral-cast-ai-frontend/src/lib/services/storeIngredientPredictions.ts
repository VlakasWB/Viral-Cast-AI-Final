import type { ApiResponse } from '$lib/types';
import { api } from '$lib/utils/api';
import type { StoreIngredientPredictionsData } from '$lib/types/store-ingredient-prediction';

export type StoreIngredientPredictionsResponse = ApiResponse<StoreIngredientPredictionsData>;

export interface StoreIngredientPredictionsGenerateOptions {
	temperature?: number;
	max_tokens?: number;
	cookies?: string;
	fetchImpl?: typeof fetch;
}

export interface StoreIngredientPredictionsGetOptions {
	cookies?: string;
	fetchImpl?: typeof fetch;
}

export function getStoreIngredientPredictionsApi(
	options: StoreIngredientPredictionsGetOptions = {}
): Promise<StoreIngredientPredictionsResponse> {
	const { cookies, fetchImpl } = options;
	return api.get<StoreIngredientPredictionsData>(
		'/api/v1/stores/ingredient-predictions',
		cookies,
		fetchImpl
	);
}

export function generateStoreIngredientPredictionsApi(
	options: StoreIngredientPredictionsGenerateOptions = {}
): Promise<StoreIngredientPredictionsResponse> {
	const { temperature, max_tokens, cookies, fetchImpl } = options;
	const searchParams = new URLSearchParams();

	if (typeof temperature === 'number') {
		searchParams.set('temperature', temperature.toString());
	}

	if (typeof max_tokens === 'number') {
		searchParams.set('max_tokens', max_tokens.toString());
	}

	const query = searchParams.toString();
	const endpoint = `/api/v1/stores/ingredient-predictions${query ? `?${query}` : ''}`;

	return api.get<StoreIngredientPredictionsData>(endpoint, cookies, fetchImpl);
}
