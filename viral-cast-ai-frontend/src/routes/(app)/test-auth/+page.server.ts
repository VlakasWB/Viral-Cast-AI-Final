import { redirect, type Actions } from '@sveltejs/kit';

export const actions: Actions = {
	login: async ({ cookies }) => {
		// Create a test user session
		const sessionData = {
			access_token: 'test-access-token',
			refresh_token: 'test-refresh-token',
			user: {
				id: 'c5f04a05-d8c2-475f-8fe2-bcbf3999f7b9',
				username: 'cashier',
				name: 'Test Cashier',
				email: 'cashier@example.com'
			}
		};

		// Set session cookie with user data
		cookies.set('session', JSON.stringify(sessionData), {
			path: '/',
			httpOnly: true,
			sameSite: 'lax',
			secure: false, // Set to false for development
			maxAge: 60 * 60 * 24 * 7 // 7 days
		});

		// Set access token cookie for API requests
		cookies.set('access_token', 'test-access-token', {
			path: '/',
			httpOnly: true,
			sameSite: 'lax',
			secure: false, // Set to false for development
			maxAge: 60 * 60 * 24 // 24 hours
		});

		// Set refresh token cookie
		cookies.set('refresh_token', 'test-refresh-token', {
			path: '/',
			httpOnly: true,
			sameSite: 'lax',
			secure: false, // Set to false for development
			maxAge: 60 * 60 * 24 * 30 // 30 days
		});

		throw redirect(303, '/products');
	},

	logout: async ({ cookies }) => {
		// Clear all authentication cookies
		cookies.delete('session', { path: '/' });
		cookies.delete('access_token', { path: '/' });
		cookies.delete('refresh_token', { path: '/' });

		throw redirect(303, '/test-auth');
	}
};
