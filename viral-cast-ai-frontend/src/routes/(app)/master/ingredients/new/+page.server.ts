import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

// Alias: redirect old path /master/ingredients/new to the new route
export const load: PageServerLoad = async () => {
  throw redirect(308, '/master/ingredient-catalog/new');
};