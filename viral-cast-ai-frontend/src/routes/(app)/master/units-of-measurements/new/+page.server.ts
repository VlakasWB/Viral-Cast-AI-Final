import { redirect, fail, type Actions } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { createUOMApi } from '$lib/services/uom.js';

const DEFAULT_REDIRECT = '/master/units-of-measurements';

const resolveRedirectTarget = (target: FormDataEntryValue | string | null | undefined) => {
	if (!target) return DEFAULT_REDIRECT;
	const value = String(target).trim();
	if (!value || !value.startsWith('/') || value.startsWith('//')) {
		return DEFAULT_REDIRECT;
	}
	return value;
};

export const load: PageServerLoad = async ({ url }) => {
	const redirectTo = resolveRedirectTarget(url.searchParams.get('redirectTo'));
	return { redirectTo };
};

export const actions: Actions = {
	default: async ({ request, cookies, fetch }) => {
		const form = await request.formData();
		const name = String(form.get('name') ?? '').trim();
		const code = String(form.get('code') ?? '').trim();
		const redirectTo = resolveRedirectTarget(form.get('redirectTo'));

		// Validation
		if (!name) {
			return fail(400, {
				errors: { name: 'Unit name is required' },
				values: { name, code }
			});
		}

		if (!code) {
			return fail(400, {
				errors: { code: 'Unit code is required' },
				values: { name, code }
			});
		}

		try {
			const requestData = { name, code };
			const token = cookies.get('access_token');
			if (!token) {
				return fail(401, {
					errors: { api: 'Unauthorized' },
					values: { name, code }
				});
			}

			const cookieStr = `access_token=${token}`;
			await createUOMApi(requestData, cookieStr, fetch);

			// On success, redirect to the list page
			throw redirect(303, redirectTo);
		} catch (error: any) {
			console.error('Create UOM failed:', error);
			const status = error?.status ?? 500;
			const message = error?.response?.message || error?.message || 'An unexpected error occurred';
			const fieldErrors = { ...(error?.response?.errors ?? {}) };
			if (!fieldErrors.api) {
				fieldErrors.api = message;
			}
			return fail(status, {
				errors: fieldErrors,
				values: { name, code }
			});
		}
	}
};
