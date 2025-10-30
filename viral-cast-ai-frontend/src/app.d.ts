// [ID] Menambah tipe untuk data user di request lifecycle
// [EN] Add typing for user info across requests
declare global {
	namespace App {
		interface Locals {
			user: { id: string; email: string; name?: string } | null;
			getSession: () => Promise<{ id: string; email: string; name?: string } | null>;
		}
	}
}
export {};
