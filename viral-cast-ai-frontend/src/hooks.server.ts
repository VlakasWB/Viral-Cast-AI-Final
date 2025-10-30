// [ID] Gabungkan Paraglide + session cookie via sequence.
// [EN] Combine Paraglide + session cookie via sequence.

import { sequence } from '@sveltejs/kit/hooks';
import { paraglideMiddleware } from '$lib/paraglide/server';
// hooks.server.ts (tambahan kecil)
import { redirect, type Handle, type HandleFetch } from '@sveltejs/kit';
import { API_BASE_URL } from '$env/static/private';

// Helper: decode JWT exp dan hitung sisa detik
function decodeJwtExp(token?: string | null): number | null {
	if (!token) return null;
	try {
		const parts = token.split('.');
		if (parts.length < 2) return null;
		const payload = JSON.parse(Buffer.from(parts[1], 'base64').toString('utf8'));
		return typeof payload.exp === 'number' ? payload.exp : null;
	} catch {
		return null;
	}
}

function secondsUntil(expUnixSeconds: number | null): number | null {
	if (expUnixSeconds == null) return null;
	const now = Math.floor(Date.now() / 1000);
	return Math.max(0, expUnixSeconds - now);
}

const PUBLIC_PREFIX = ['/login', '/auth', '/coming-soon', '/maintenance', '/pages']; // sesuaikan
const IS_DEV = process.env.NODE_ENV !== 'production';
// Dev-only pages we allow without auth to unblock local testing
const DEV_PUBLIC_PREFIX = [
    '/master/categories',
    '/master/category-products',
    '/master/ingredient-catalog',
    '/master/ingredient-stock-moves',
    '/master/ingredient-stocks',
    '/master/ingredients',
    '/master/products',
    '/master/recipes',
    '/master/units-of-measurements'
];
const isAsset = (p: string) =>
	p.startsWith('/assets') ||
	p.startsWith('/build') ||
	/\.(?:png|jpg|jpeg|svg|ico|css|js|map|webp|woff2?)$/.test(p);

const handleAuth: Handle = async ({ event, resolve }) => {
    const { url, locals } = event;
    if (
        isAsset(url.pathname) ||
        PUBLIC_PREFIX.some((p) => url.pathname.startsWith(p)) ||
        (IS_DEV && DEV_PUBLIC_PREFIX.some((p) => url.pathname.startsWith(p)))
    ) {
        return resolve(event);
    }
    if (!locals.user) {
        const from = url.pathname + url.search;
        throw redirect(303, `/login?from=${encodeURIComponent(from)}`);
    }
    return resolve(event);
};

// [ID] Parser sesi dengan token; mendukung refresh otomatis
// [EN] Session parser with tokens; supports automatic refresh
import { refreshTokenApi } from '$lib/services/auth.js';

function parseSession(session: string | undefined) {
	if (!session) return null;

	try {
		const sessionData = JSON.parse(session);
		if (sessionData?.user?.id) {
			return sessionData.user;
		}
	} catch {}
	return null;
}

// Function to refresh tokens and update cookies
async function refreshUserTokens(event: any) {
	try {
		const refreshToken = event.cookies.get('refresh_token');

		// Guard: skip refresh if token missing or empty
		if (!refreshToken || typeof refreshToken !== 'string' || refreshToken.trim() === '') {
			return false;
		}

		// Optional: basic JWT-like structure check to avoid noisy 401s on invalid tokens
		const jwtLike = /^[A-Za-z0-9-_]+\.[A-Za-z0-9-_]+\.[A-Za-z0-9-_]+$/.test(refreshToken);
		if (!jwtLike) {
			// Clean invalid cookies silently and skip refresh
			event.cookies.delete('session', { path: '/' });
			event.cookies.delete('access_token', { path: '/' });
			event.cookies.delete('refresh_token', { path: '/' });
			return false;
		}

		const authResponse = await refreshTokenApi(refreshToken, event.fetch);

		// Update session data
		const currentSession = event.cookies.get('session');
		let sessionData: any = {};

		try {
			sessionData = JSON.parse(currentSession || '{}');
		} catch {}

		sessionData.access_token = authResponse.access_token;
		sessionData.refresh_token = authResponse.refresh_token;

		// Hitung maxAge dari exp token bila tersedia
		const accessExp = decodeJwtExp(authResponse.access_token);
		const refreshExp = decodeJwtExp(authResponse.refresh_token || refreshToken || undefined);
		const accessMaxAge = secondsUntil(accessExp) ?? 15 * 60; // fallback 15 menit
		const refreshMaxAge = secondsUntil(refreshExp) ?? 60 * 60; // fallback 60 menit

		// Update cookies
		event.cookies.set('session', JSON.stringify(sessionData), {
			path: '/',
			httpOnly: true,
			sameSite: 'lax',
			secure: process.env.NODE_ENV === 'production',
			maxAge: Math.max(refreshMaxAge, accessMaxAge) // sesi mengikuti yang lebih panjang
		});

		event.cookies.set('access_token', authResponse.access_token, {
			path: '/',
			httpOnly: true,
			sameSite: 'lax',
			secure: process.env.NODE_ENV === 'production',
			maxAge: accessMaxAge
		});

		event.cookies.set('refresh_token', authResponse.refresh_token || refreshToken || '', {
			path: '/',
			httpOnly: true,
			sameSite: 'lax',
			secure: process.env.NODE_ENV === 'production',
			maxAge: refreshMaxAge
		});

		return true;
	} catch (error) {
		// Downgrade to warn to reduce noise during initial visits without a valid session
		console.warn('Token refresh failed:', error);
		// Clear invalid cookies
		event.cookies.delete('session', { path: '/' });
		event.cookies.delete('access_token', { path: '/' });
		event.cookies.delete('refresh_token', { path: '/' });
		return false;
	}
}

// // [ID] Contoh verifikasi JWT (aktifkan saat API siap):
// // [EN] JWT verify example (enable when API is ready):
// // import { jwtVerify } from 'jose';
// // async function verifyJWT(token: string) {
// //   const secret = new TextEncoder().encode(process.env.JWT_SECRET!);
// //   const { payload } = await jwtVerify(token, secret, { algorithms: ['HS256'] });
// //   return { id: String(payload.sub), email: payload.email as string | undefined, name: payload.name as string | undefined };
// // }

const handleParaglide: Handle = ({ event, resolve }) =>
	paraglideMiddleware(event.request, ({ request, locale }) => {
		event.request = request;
		return resolve(event, {
			transformPageChunk: ({ html }) => html.replace('%paraglide.lang%', locale)
		});
	});

const handleSession: Handle = async ({ event, resolve }) => {
	const raw = event.cookies.get('session');
	// buang kalau terlalu besar (mis. > 2 KB)
	if (raw && raw.length > 2048) {
		event.cookies.delete('session', { path: '/' });
	}

	// Parse current session
	let user = parseSession(event.cookies.get('session'));

	// Proactive refresh: only when we truly have a valid-looking refresh token
	const accessToken = event.cookies.get('access_token');
	const refreshToken = event.cookies.get('refresh_token');
	const hasValidRefresh = Boolean(
		refreshToken && /^[A-Za-z0-9-_]+\.[A-Za-z0-9-_]+\.[A-Za-z0-9-_]+$/.test(refreshToken)
	);
	if (!accessToken && hasValidRefresh) {
		const refreshed = await refreshUserTokens(event);
		if (refreshed && !user) {
			user = parseSession(event.cookies.get('session'));
		}
	}

	// Proactive refresh: if access token exists but akan kadaluarsa < 60 detik, refresh sekarang
	if (accessToken && hasValidRefresh) {
		const exp = decodeJwtExp(accessToken);
		const remain = secondsUntil(exp);
		if (remain !== null && remain <= 60) {
			const refreshed = await refreshUserTokens(event);
			if (refreshed && !user) {
				user = parseSession(event.cookies.get('session'));
			}
		}
	}

	// If no user but we have a refresh token, try to refresh
	if (!user && hasValidRefresh) {
		const refreshSuccess = await refreshUserTokens(event);
		if (refreshSuccess) {
			user = parseSession(event.cookies.get('session'));
		}
	}

	// Tambahkan fungsi getSession ke locals
	event.locals.getSession = async () => {
		return user;
	};

	event.locals.user = user;
	return resolve(event);
};

// Inject Authorization for API requests and auto-refresh on 401
export const handleFetch: HandleFetch = async ({ event, request, fetch }) => {
	let req = request;
	// Only intercept API calls to backend
	const isApiCall = req.url.startsWith(API_BASE_URL);
	if (!isApiCall) {
		return fetch(req);
	}

	// Skip refresh endpoints to avoid loops
	const urlObj = new URL(req.url);
	const pathname = urlObj.pathname;
	const isRefreshEndpoint = pathname.includes('/api/v1/auth/refresh');
	if (isRefreshEndpoint) {
		return fetch(req);
	}

	// Attach Authorization header from access_token cookie if not present
	const headers = new Headers(req.headers);
	const hasAuth = headers.has('authorization') || headers.has('Authorization');
	const accessToken = event.cookies.get('access_token');
	if (!hasAuth && accessToken) {
		headers.set('Authorization', `Bearer ${accessToken}`);
	}

	// Make initial request
	let response = await fetch(new Request(req, { headers }));

	// If unauthorized and we have a refresh token, try to refresh and retry once
	if (response.status === 401) {
		const refreshToken = event.cookies.get('refresh_token');
		if (refreshToken) {
			const refreshed = await refreshUserTokens(event);
			if (refreshed) {
				const newAccess = event.cookies.get('access_token');
				if (newAccess) {
					headers.set('Authorization', `Bearer ${newAccess}`);
				}
				response = await fetch(new Request(req, { headers }));
			}
		}
	}

	return response;
};

export const handle: Handle = sequence(handleParaglide, handleSession, handleAuth);
