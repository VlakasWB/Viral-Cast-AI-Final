import { fail, redirect, type Actions } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { loginApi } from '$lib/services/auth.js';

export const actions: Actions = {
	default: async ({ request, cookies, url }) => {
		const form = await request.formData();
		const username = String(form.get('email') ?? ''); // Using email field as username
		const password = String(form.get('password') ?? '');

		if (!username || !password) {
			return fail(400, { message: 'Username and password are required' });
		}

		let authResponse;

		try {
			// Call the API to authenticate
			authResponse = await loginApi({ username, password });
		} catch (error: any) {
			console.error('Login error:', error);
			return fail(400, {
				message: error.message || 'Login failed. Please try again.'
			});
		}

		// Store user session data in cookie
		const sessionData = {
			access_token: authResponse.access_token,
			refresh_token: authResponse.refresh_token,
			user: authResponse.user || { id: username, username, name: username }
		};

		// Set session cookie with user data
		cookies.set('session', JSON.stringify(sessionData), {
			path: '/',
			httpOnly: true,
			sameSite: 'lax',
			secure: process.env.NODE_ENV === 'production',
			maxAge: 60 * 60 * 24 * 7 // 7 days
		});

		// Set access token cookie for API requests
		cookies.set('access_token', authResponse.access_token, {
			path: '/',
			httpOnly: true,
			sameSite: 'lax',
			secure: process.env.NODE_ENV === 'production',
			maxAge: 60 * 60 * 24 // 24 hours
		});

		// Set refresh token cookie
		cookies.set('refresh_token', authResponse.refresh_token || '', {
			path: '/',
			httpOnly: true,
			sameSite: 'lax',
			secure: process.env.NODE_ENV === 'production',
			maxAge: 60 * 60 * 24 * 30 // 30 days
		});

		// Redirect after successful login (outside try-catch)
		const to = url.searchParams.get('from') ?? '/';
		throw redirect(303, to);
	}
};

export const load: PageServerLoad = async ({ locals, url }) => {
	// If user is already authenticated, redirect to dashboard or intended page
	if (locals.user) {
		const from = url.searchParams.get('from');
		const redirectTo = from ? decodeURIComponent(from) : '/';
		throw redirect(302, redirectTo);
	}

	return {};
};
