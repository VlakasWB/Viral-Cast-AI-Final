// Error handling utilities to prevent 500 errors and provide better UX

export interface ErrorInfo {
	message: string;
	status: number;
	code?: string;
	details?: any;
}

export class AppError extends Error {
	public status: number;
	public code?: string;
	public details?: any;

	constructor(message: string, status: number = 500, code?: string, details?: any) {
		super(message);
		this.name = 'AppError';
		this.status = status;
		this.code = code;
		this.details = details;
	}
}

// Safe API call wrapper
export async function safeApiCall<T>(
	apiCall: () => Promise<T>,
	fallbackValue?: T,
	errorMessage?: string
): Promise<T | null> {
	try {
		return await apiCall();
	} catch (error) {
		console.error('API call failed:', error);

		if (fallbackValue !== undefined) {
			return fallbackValue;
		}

		// Don't throw 500 errors, return null instead
		return null;
	}
}

// Safe data access with fallbacks
export function safeGet<T>(obj: any, path: string, fallback: T): T {
	try {
		const keys = path.split('.');
		let result = obj;

		for (const key of keys) {
			if (result == null || typeof result !== 'object') {
				return fallback;
			}
			result = result[key];
		}

		return result !== undefined ? result : fallback;
	} catch {
		return fallback;
	}
}

// Validate required parameters
export function validateRequired(params: Record<string, any>, required: string[]): void {
	const missing = required.filter((key) => !params[key]);
	if (missing.length > 0) {
		throw new AppError(`Missing required parameters: ${missing.join(', ')}`, 400, 'MISSING_PARAMS');
	}
}

// Handle async operations safely
export async function handleAsync<T>(
	operation: () => Promise<T>,
	errorHandler?: (error: any) => T
): Promise<T | null> {
	try {
		return await operation();
	} catch (error) {
		console.error('Async operation failed:', error);

		if (errorHandler) {
			return errorHandler(error);
		}

		return null;
	}
}

// Format error for user display
export function formatErrorMessage(error: any): string {
	if (error instanceof AppError) {
		return error.message;
	}

	if (error?.message) {
		return error.message;
	}

	if (typeof error === 'string') {
		return error;
	}

	return 'Terjadi kesalahan yang tidak diketahui';
}

// Check if error should be retried
export function isRetryableError(error: any): boolean {
	const retryableStatuses = [408, 429, 502, 503, 504];
	const status = error?.status || error?.response?.status;
	return retryableStatuses.includes(status);
}

// Retry mechanism with exponential backoff
export async function retryOperation<T>(
	operation: () => Promise<T>,
	maxRetries: number = 3,
	baseDelay: number = 1000
): Promise<T> {
	let lastError: any;

	for (let attempt = 0; attempt <= maxRetries; attempt++) {
		try {
			return await operation();
		} catch (error) {
			lastError = error;

			if (attempt === maxRetries || !isRetryableError(error)) {
				throw error;
			}

			// Exponential backoff
			const delay = baseDelay * Math.pow(2, attempt);
			await new Promise((resolve) => setTimeout(resolve, delay));
		}
	}

	throw lastError;
}

// Log errors for monitoring
export function logError(error: any, context?: string): void {
	const errorInfo = {
		message: formatErrorMessage(error),
		status: error?.status || 500,
		code: error?.code,
		context,
		timestamp: new Date().toISOString(),
		stack: error?.stack
	};

	console.error('Application Error:', errorInfo);

	// In production, send to monitoring service
	if (typeof window !== 'undefined' && window.location.hostname !== 'localhost') {
		// Send to error tracking service (e.g., Sentry, LogRocket, etc.)
		// Example: Sentry.captureException(error, { extra: errorInfo });
	}
}
