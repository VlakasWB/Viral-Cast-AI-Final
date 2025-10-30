// Client-side error handling to prevent unhandled errors
import { logError } from '$lib/utils/errorHandler';

// Handle unhandled promise rejections
if (typeof window !== 'undefined') {
    const redirectToLoginIfInvalidToken = (reason: any) => {
        const message = reason?.message || reason?.response?.message || String(reason || '');
        const status = reason?.status || reason?.response?.status;
        const tokenInvalid = /Token is invalid|session has expired/i.test(message || '');
        if (status === 401 || tokenInvalid) {
            const from = `${window.location.pathname}${window.location.search}`;
            window.location.href = `/login?from=${encodeURIComponent(from)}`;
            return true;
        }
        return false;
    };

    window.addEventListener('unhandledrejection', (event) => {
        console.error('Unhandled promise rejection:', event.reason);
        logError(event.reason, 'Unhandled Promise Rejection');

        // Prevent the default browser behavior (showing error in console)
        event.preventDefault();

        // Redirect to login if token is invalid or expired
        if (redirectToLoginIfInvalidToken(event.reason)) {
            return;
        }

		// Show user-friendly message instead of crashing
		if (event.reason?.message) {
			console.warn('An error occurred, but the application will continue running.');
		}
	});

	// Handle uncaught errors
    window.addEventListener('error', (event) => {
        console.error('Uncaught error:', event.error);
        logError(event.error, 'Uncaught Error');

        // Prevent the error from crashing the app
        event.preventDefault();

        // Redirect to login if token is invalid or expired
        if (redirectToLoginIfInvalidToken(event.error)) {
            return true;
        }

        return true;
    });

	// Handle resource loading errors (skip benign image/static asset failures)
	window.addEventListener(
		'error',
		(event) => {
			const target: any = event.target;
			if (target && target !== window) {
				const tag = (target.tagName || '').toLowerCase();
				const url = target?.src || target?.href;
				// Ignore image load failures and common static uploads to reduce noise
				if (tag === 'img' || (typeof url === 'string' && /\/uploads\//.test(url))) {
					return;
				}
				console.warn('Resource loading error:', target);
				logError(new Error(`Failed to load resource: ${url}`), 'Resource Loading Error');
			}
		},
		true
	);
}
