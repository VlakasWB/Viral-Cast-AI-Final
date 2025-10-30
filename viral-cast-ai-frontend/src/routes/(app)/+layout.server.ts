import { redirect } from '@sveltejs/kit';
import type { LayoutServerLoad } from './$types';
import { getMyProfileApi } from '$lib/services/profile.js';

// Build absolute URLs for image paths returned by API
const API_BASE_URL = process.env.API_BASE_URL || 'http://localhost:12000';
function resolveImageUrl(u?: string): string | null {
	if (!u || typeof u !== 'string') return null;
	const trimmed = String(u).trim().replace(/^`+|`+$/g, '');
	if (/^https?:\/\//i.test(trimmed)) return trimmed;
	const path = trimmed.startsWith('/') ? trimmed : `/${trimmed}`;
	return `${API_BASE_URL}${path}`;
}

export const load: LayoutServerLoad = async ({ locals, url, cookies, fetch }) => {
	// Require authentication for (app) shell
	if (!locals.user) {
		const from = url.pathname + url.search;
		throw redirect(303, `/login?from=${encodeURIComponent(from)}`);
	}

	// Fetch minimal profile for header (name + avatar)
	const token = cookies.get('access_token');
	const cookieStr = token ? `access_token=${token}` : undefined;

	let profile: any = null;
	try {
		const res = await getMyProfileApi(cookieStr, fetch);
		profile = (res?.data?.profile) ?? res?.data ?? null;
	} catch {
		profile = null;
	}

	// Normalize avatar URL to absolute for reliable display
	if (profile) {
		profile.photo_profile = resolveImageUrl(profile.photo_profile ?? undefined);
	}

	return { user: locals.user, profile };
};
