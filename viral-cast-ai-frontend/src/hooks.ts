import { deLocalizeUrl } from '$lib/paraglide/runtime';

export const reroute = (request) => deLocalizeUrl(request.url).pathname;
// [ID] Stub transport kosong; aman diabaikan jika tidak perlu.
// [EN] Optional empty transport stub to silence warnings.
import type { Transport } from '@sveltejs/kit';
export const transport: Transport = {};
