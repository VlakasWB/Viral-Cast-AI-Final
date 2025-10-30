import { redirect, type Actions } from '@sveltejs/kit';
import { logoutApi } from '$lib/services/auth.js';

export const actions: Actions = {
	default: async ({ cookies }) => {
		try {
			// Get access token from cookies
			const accessToken = cookies.get('access_token');
			// Call logout API to invalidate tokens on server
			await logoutApi(accessToken);
		} catch (error) {
			console.error('Logout API error:', error);
			// Continue with local logout even if API fails
		}

		// Clear all authentication cookies
		cookies.delete('session', { path: '/' });
		cookies.delete('access_token', { path: '/' });
		cookies.delete('refresh_token', { path: '/' });

		throw redirect(303, '/login');
	}
};

export const load = async () => ({});
