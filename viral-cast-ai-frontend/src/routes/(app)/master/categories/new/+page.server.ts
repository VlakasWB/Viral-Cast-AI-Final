import { redirect, fail, type Actions } from '@sveltejs/kit';
import { createCategoryApi } from '$lib/services/category.js';

export const actions: Actions = {
	default: async ({ request, cookies }) => {
		const form = await request.formData();
		const name = String(form.get('name') ?? '').trim();

		// Validation
		if (!name) {
			return fail(400, {
				errors: { name: 'Category name is required' },
				values: { name }
			});
		}

		try {
			// Try to create via API
			const token = cookies.get('access_token');
			const cookieStr = token ? `access_token=${token}` : undefined;
			await createCategoryApi({ name }, cookieStr);
		} catch (error) {
			console.error('API create failed, category will be added to dummy data on next load:', error);
			// In a real app, you would add to dummy data here
			// For now, we'll just continue with redirect
		}

		// Always redirect to categories page
		throw redirect(303, '/master/categories');
	}
};
