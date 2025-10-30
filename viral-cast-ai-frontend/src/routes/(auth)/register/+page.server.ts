import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ locals, url }) => {
	// If user is already authenticated, redirect to dashboard or intended page
	if (locals.user) {
		const from = url.searchParams.get('from');
		const redirectTo = from ? decodeURIComponent(from) : '/';
		throw redirect(302, redirectTo);
	}

	return {};
};
