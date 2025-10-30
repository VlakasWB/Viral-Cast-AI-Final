<script lang="ts">
    import { page } from '$app/stores';
    import { goto } from '$app/navigation';
    import type { PageData } from './$types';
    import { deleteRecipeSetApi } from '$lib/services/recipe.js';
    import SearchBox from '$lib/components/SearchBox.svelte';
    import Button from '$lib/components/ui/Button.svelte';
    import TablePagination from '$lib/components/TablePagination.svelte';

let { data }: { data: PageData } = $props();

    let showDeleteModal = $state(false);
    let recipeToDelete: any = $state(null);
    let isDeleting = $state(false);

    // Filter states
	const initialProductName =
		data.filters.product_name ||
		(data.filters.product_uuid
			? data.products.find((product) => product.uuid === data.filters.product_uuid)?.name ?? ''
			: '');
	let selectedProduct = $state(initialProductName || '');
    let selectedStatus = $state(
        data.filters.is_active !== undefined ? data.filters.is_active.toString() : ''
    );
    let searchQuery = $state(data.filters.search || '');

    // Pagination size state
    let pageSize = $state(data.pagination.limit || 10);

    // Range calculation for display
    const startIndex = $derived.by(() => (data.recipeSets?.length ?? 0) > 0 ? (data.pagination.page - 1) * data.pagination.limit + 1 : 0);
    const endIndex = $derived.by(() => (data.recipeSets?.length ?? 0) > 0 ? Math.min(startIndex + (data.recipeSets?.length ?? 0) - 1, data.pagination.total) : 0);

	// Format date helper
	function formatDate(timestamp: number): string {
		return new Date(timestamp).toLocaleDateString('id-ID', {
			year: 'numeric',
			month: 'short',
			day: 'numeric'
		});
	}

	// Apply filters
    function applyFilters() {
        const params = new URLSearchParams($page.url.searchParams);

		params.delete('product_uuid');
		if (selectedProduct) {
			params.set('name', selectedProduct);
		} else {
			params.delete('name');
		}

		if (selectedStatus !== '') {
			params.set('is_active', selectedStatus);
		} else {
			params.delete('is_active');
		}

        if (searchQuery) {
            params.set('search', searchQuery);
        } else {
            params.delete('search');
        }

        params.set('page', '1'); // Reset to first page
        params.set('limit', String(pageSize));
        goto(`?${params.toString()}`);
    }

	// Clear filters
    function clearFilters() {
        selectedProduct = '';
        selectedStatus = '';
        searchQuery = '';
        goto('/master/recipes');
    }

	// Pagination
    function goToPage(pageNum: number) {
        const params = new URLSearchParams($page.url.searchParams);
        params.set('page', pageNum.toString());
        params.set('limit', String(pageSize));
        params.delete('product_uuid');
        if (selectedProduct) params.set('name', selectedProduct); else params.delete('name');
        if (selectedStatus !== '') params.set('is_active', selectedStatus); else params.delete('is_active');
        if (searchQuery) params.set('search', searchQuery); else params.delete('search');
        goto(`?${params.toString()}`);
    }

    function changeSize(size: number) {
        pageSize = size;
        const params = new URLSearchParams($page.url.searchParams);
        params.set('page', '1');
        params.set('limit', String(size));
        params.delete('product_uuid');
        if (selectedProduct) params.set('name', selectedProduct); else params.delete('name');
        if (selectedStatus !== '') params.set('is_active', selectedStatus); else params.delete('is_active');
        if (searchQuery) params.set('search', searchQuery); else params.delete('search');
        goto(`?${params.toString()}`);
    }

	// Delete recipe set
	function confirmDelete(recipe: any) {
		recipeToDelete = recipe;
		showDeleteModal = true;
	}

	async function deleteRecipe() {
		if (!recipeToDelete) return;

		isDeleting = true;
		try {
			await deleteRecipeSetApi(recipeToDelete.uuid);
			showDeleteModal = false;
			recipeToDelete = null;
			// Refresh page
			goto($page.url.pathname + $page.url.search, { replaceState: true });
		} catch (error) {
			console.error('Error deleting recipe:', error);
			alert('Gagal menghapus resep. Silakan coba lagi.');
		} finally {
			isDeleting = false;
		}
	}

	function cancelDelete() {
		showDeleteModal = false;
		recipeToDelete = null;
	}

	function formatNumericValue(value: unknown): string {
		const normalised = typeof value === 'string' ? Number(value) : value;

		if (typeof normalised === 'number' && !Number.isNaN(normalised)) {
			return Number.isInteger(normalised)
				? normalised.toString()
				: normalised.toLocaleString('id-ID', {
						minimumFractionDigits: 0,
						maximumFractionDigits: 4
				  });
		}

		if (value !== undefined && value !== null && value !== '') {
			return String(value);
		}

		return '';
	}

	function formatYieldQuantity(recipe: any): string {
		if (!recipe) return '-';

		const rawYield =
			recipe.yield_qty ??
			recipe.yield_quantity ??
			recipe.yield?.quantity ??
			recipe.yield?.qty ??
			recipe.yield?.value ??
			recipe.yieldQuantity ??
			null;

		let unit =
			recipe.yield_uom ??
			recipe.yield_unit ??
			recipe.yield_quantity_uom ??
			recipe.yield?.uom ??
			recipe.yield?.unit ??
			recipe.yieldUnit ??
			recipe.yield?.unit_name ??
			'';

		if (rawYield && typeof rawYield === 'object' && !Array.isArray(rawYield)) {
			const objValue =
				rawYield.value ??
				rawYield.amount ??
				rawYield.qty ??
				rawYield.quantity ??
				null;
			unit = rawYield.unit ?? rawYield.uom ?? rawYield.unit_name ?? unit;
			const formatted = formatNumericValue(objValue);
			return formatted ? `${formatted}${unit ? ` ${unit}` : ''}` : '-';
		}

		const formatted = formatNumericValue(rawYield);
		return formatted ? `${formatted}${unit ? ` ${unit}` : ''}` : '-';
	}
</script>

<svelte:head>
	<title>Recipes - Viral Cast AI</title>
</svelte:head>

<main class="w-full px-4 py-6">
	<!-- Header -->
	<div class="mb-6 flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">Recipes</h1>
			<p class="mt-1 text-gray-600 dark:text-gray-300">
				Manage product recipes and ingredient compositions
			</p>
		</div>
		<a
			href="/master/recipes/new"
			class="rounded-lg bg-blue-600 px-4 py-2 font-medium text-white transition-colors hover:bg-blue-700"
		>
			+ New Recipe
		</a>
	</div>

    <!-- Filters -->
    <div class="mb-6 rounded-lg border bg-white p-4 shadow-sm dark:border-gray-700 dark:bg-gray-800">
        <div class="grid grid-cols-1 gap-4 md:grid-cols-4">
            <div>
                <label for="search" class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-200">Search Recipes</label>
                <SearchBox value={searchQuery} onSearch={(q) => { searchQuery = q; applyFilters(); }} />
            </div>
            <div>
                <label
                    for="product-filter"
                    class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-200"
                >
                    Filter by Product
                </label>
                <select
                    id="product-filter"
                    bind:value={selectedProduct}
                    class="w-full rounded-md border border-gray-300 px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:outline-none dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100"
                >
                    <option value="">All Products</option>
                    {#each data.products as product}
                        <option value={product.name}>{product.name}</option>
                    {/each}
                </select>
            </div>

            <div>
                <label
                    for="status-filter"
                    class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-200"
                >
                    Status
                </label>
                <select
                    id="status-filter"
                    bind:value={selectedStatus}
                    class="w-full rounded-md border border-gray-300 px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:outline-none dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100"
                >
                    <option value="">All Status</option>
                    <option value="true">Active</option>
                    <option value="false">Inactive</option>
                </select>
            </div>

            <div class="flex items-end gap-2">
                <button
                    on:click={applyFilters}
                    class="rounded-md bg-blue-600 px-4 py-2 font-medium text-white transition-colors hover:bg-blue-700"
                >
                    Apply Filters
                </button>
                <button
                    on:click={clearFilters}
                    class="rounded-md bg-gray-500 px-4 py-2 font-medium text-white transition-colors hover:bg-gray-600"
                >
                    Reset
                </button>
            </div>
        </div>
    </div>

	<!-- Recipe Sets Table -->
	<div
		class="overflow-hidden rounded-lg border bg-white shadow-sm dark:border-gray-700 dark:bg-gray-800"
	>
		<div class="overflow-x-auto">
			<table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
				<thead class="bg-gray-50 dark:bg-gray-900">
					<tr>
						<th
							class="px-6 py-3 text-left text-xs font-medium tracking-wider text-gray-500 uppercase dark:text-gray-400"
						>
							Recipe Name
						</th>
						<th
							class="px-6 py-3 text-left text-xs font-medium tracking-wider text-gray-500 uppercase"
						>
							Yield Qty
						</th>
						<th
							class="px-6 py-3 text-left text-xs font-medium tracking-wider text-gray-500 uppercase dark:text-gray-400"
						>
							Effective Period
						</th>
						<th
							class="px-6 py-3 text-left text-xs font-medium tracking-wider text-gray-500 uppercase dark:text-gray-400"
						>
							Status
						</th>
						<th
							class="px-6 py-3 text-left text-xs font-medium tracking-wider text-gray-500 uppercase dark:text-gray-400"
						>
							Actions
						</th>
					</tr>
				</thead>
				<tbody class="divide-y divide-gray-200 bg-white dark:divide-gray-700 dark:bg-gray-800">
					{#each data.recipeSets as recipe}
						<tr class="hover:bg-gray-50 dark:hover:bg-gray-700">
							<td class="px-6 py-4 whitespace-nowrap">
								<div class="text-sm font-medium text-gray-900 dark:text-gray-100">
									{recipe.name}
								</div>
							</td>
							<td class="px-6 py-4 whitespace-nowrap">
								<div class="text-sm text-gray-900 dark:text-gray-100">{formatYieldQuantity(recipe)}</div>
							</td>
							<td class="px-6 py-4 whitespace-nowrap">
								<div class="text-sm text-gray-900 dark:text-gray-100">
									{formatDate(recipe.effective_from)} - {formatDate(recipe.effective_to)}
								</div>
							</td>
							<td class="px-6 py-4 whitespace-nowrap">
								<span
									class="inline-flex rounded-full px-2 py-1 text-xs font-semibold {recipe.is_active
										? 'bg-green-100 text-green-800'
										: 'bg-red-100 text-red-800'}"
								>
									{recipe.is_active ? 'Active' : 'Inactive'}
								</span>
							</td>
							<td class="px-6 py-4 text-sm font-medium whitespace-nowrap">
								<div class="flex items-center gap-2">
								<Button label="View" color="sky" href={"/master/recipes/" + recipe.uuid} />
								<Button label="Edit" color="violet" href={"/master/recipes/" + recipe.uuid + "/edit"} />
									<Button label="Delete" color="red" onClick={() => confirmDelete(recipe)} />
								</div>
							</td>
						</tr>
					{:else}
						<tr>
							<td colspan="6" class="px-6 py-4 text-center text-gray-500 dark:text-gray-400">
								No recipes found
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>

        <!-- Pagination -->
        <div class="border-t border-gray-200 bg-white px-4 py-3 sm:px-6 dark:border-gray-700 dark:bg-gray-800">
            <TablePagination
                page={data.pagination.page}
                pageCount={data.pagination.total_pages}
                total={data.pagination.total}
                start={startIndex}
                end={endIndex}
                size={pageSize}
                sizes={[3, 5, 10, 25, 50]}
                onChangePage={goToPage}
                onChangeSize={changeSize}
            />
        </div>
	</div>
</main>

<!-- Delete Confirmation Modal -->
{#if showDeleteModal}
	<div class="bg-opacity-50 fixed inset-0 z-50 h-full w-full overflow-y-auto bg-gray-600">
		<div
			class="relative top-20 mx-auto w-96 rounded-md border bg-white p-5 shadow-lg dark:border-gray-700 dark:bg-gray-800"
		>
			<div class="mt-3 text-center">
				<h3 class="text-lg font-medium text-gray-900 dark:text-gray-100">Delete Confirmation</h3>
				<div class="mt-2 px-7 py-3">
					<p class="text-sm text-gray-500 dark:text-gray-300">
						Are you sure you want to delete recipe "{recipeToDelete?.name}"? This action cannot be
						undone.
					</p>
				</div>
				<div class="mt-4 flex justify-center gap-4">
					<Button label="Cancel" color="deepYellow" onClick={cancelDelete} disabled={isDeleting} />
					<button
						on:click={deleteRecipe}
						disabled={isDeleting}
						class="rounded-md bg-red-600 px-4 py-2 text-base font-medium text-white shadow-sm hover:bg-red-700 focus:ring-2 focus:ring-red-500 focus:outline-none disabled:opacity-50"
					>
						{isDeleting ? 'Deleting...' : 'Delete'}
					</button>
				</div>
			</div>
		</div>
	</div>
{/if}
