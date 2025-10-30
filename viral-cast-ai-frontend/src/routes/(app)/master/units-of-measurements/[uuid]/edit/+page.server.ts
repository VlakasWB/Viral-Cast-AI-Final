import { error, redirect, fail, type Actions } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { getUOMApi, updateUOMApi } from '$lib/services/uom.js';

const DEFAULT_REDIRECT = '/master/units-of-measurements';

function resolveRedirectTarget(target: FormDataEntryValue | string | null | undefined) {
	if (!target) return DEFAULT_REDIRECT;
	const value = String(target).trim();
	// allow only same-origin absolute paths
	if (!value || !value.startsWith('/') || value.startsWith('//')) {
		return DEFAULT_REDIRECT;
	}
	return value;
}

export const load: PageServerLoad = async ({ params, cookies, fetch, url }) => {
	const uuid = params.uuid;
	if (!uuid) throw error(400, 'Missing UOM UUID');
	const redirectTo = resolveRedirectTarget(url.searchParams.get('redirectTo'));

	try {
		const accessToken = cookies.get('access_token');
		if (!accessToken) {
			throw redirect(303, '/login');
		}
		const cookieStr = `access_token=${accessToken}`;
		const response = await getUOMApi(uuid, cookieStr, fetch);
		if (!response.data) throw error(404, 'Unit of measurement not found');
		return { uom: response.data, redirectTo };
	} catch (err: any) {
		if (err?.status === 401) {
			throw redirect(303, '/login');
		}
		console.error('Error loading unit of measurement:', err);
		throw error(404, 'Unit of measurement not found');
	}
};

export const actions: Actions = {
	default: async ({ request, cookies, fetch }) => {
		const form = await request.formData();
		const uuid = String(form.get('uuid') ?? '');
		const name = String(form.get('name') ?? '').trim();
		const code = String(form.get('code') ?? '').trim();
		const redirectTo = resolveRedirectTarget(form.get('redirectTo'));

		if (!uuid) throw error(400, 'Missing UOM UUID');

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
			const updateData = { name, code };

			const accessToken = cookies.get('access_token');
			if (!accessToken) {
				return fail(401, {
					errors: { api: 'Unauthorized' },
					values: { name, code }
				});
			}

			const cookieStr = `access_token=${accessToken}`;
			await updateUOMApi(uuid, updateData, cookieStr, fetch);

			// On success, redirect to the list page
			throw redirect(303, redirectTo);
		} catch (err: any) {
			console.error('Error updating unit of measurement:', err);
			const status = err?.status ?? 500;
			const message =
				err?.response?.message || err?.message || 'Failed to update unit of measurement';
			const fieldErrors = { ...(err?.response?.errors ?? {}) };
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
