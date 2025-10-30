// [ID] "Database" in-memory untuk dev/test (SSR). Aman dipakai di local dev.
// [EN] In-memory "database" for dev/test (SSR). Replace with real DB in prod.

import type { Product } from '$lib/types';

let rows: Product[] = [
	{ id: crypto.randomUUID(), name: 'Alpha', price: 15000, stock: 12, active: true },
	{ id: crypto.randomUUID(), name: 'Beta', price: 20000, stock: 5, active: false },
	{ id: crypto.randomUUID(), name: 'Gamma', price: 9500, stock: 32, active: true }
];

export type ListParams = { page?: number; size?: number; q?: string };

export async function listProducts({ page = 1, size = 10, q = '' }: ListParams) {
	const filtered = !q
		? rows
		: rows.filter((r) => JSON.stringify(r).toLowerCase().includes(q.toLowerCase()));

	const pageCount = Math.max(1, Math.ceil(filtered.length / size));
	const start = (page - 1) * size;
	const items = filtered.slice(start, start + size);
	return { items, total: filtered.length, page, pageCount };
}

export async function getProduct(id: string) {
	return rows.find((r) => r.id === id) ?? null;
}

export async function createProduct(input: Omit<Product, 'id'>) {
	const row: Product = { id: crypto.randomUUID(), ...input };
	rows.unshift(row);
	return row;
}

export async function updateProduct(id: string, patch: Partial<Omit<Product, 'id'>>) {
	const i = rows.findIndex((r) => r.id === id);
	if (i === -1) return null;
	rows[i] = { ...rows[i], ...patch };
	return rows[i];
}

export async function deleteProduct(id: string) {
	const len = rows.length;
	rows = rows.filter((r) => r.id !== id);
	return rows.length < len;
}

// [ID] Helper validasi & parse FormData -> Product partial.
// [EN] Helper to validate & parse FormData -> Product partial.
export function parseProductForm(data: FormData) {
	const name = String(data.get('name') ?? '').trim();
	const price = Number(data.get('price') ?? '0');
	const stock = Number(data.get('stock') ?? '0');
	const active =
		String(data.get('active') ?? '') === 'on' || String(data.get('active') ?? '') === 'true';

	const errors: Record<string, string> = {};
	if (!name) errors.name = 'Name is required';
	if (!Number.isFinite(price) || price < 0) errors.price = 'Price must be >= 0';
	if (!Number.isInteger(stock) || stock < 0) errors.stock = 'Stock must be >= 0';

	return { values: { name, price, stock, active }, errors };
}
