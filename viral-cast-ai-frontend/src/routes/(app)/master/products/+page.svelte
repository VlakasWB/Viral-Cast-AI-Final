<script lang="ts">
    import TablePagination from '$lib/components/TablePagination.svelte';
	import SearchBox from '$lib/components/SearchBox.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import type { Product } from '$lib/types/product.js';
    import type { StoreProductPredictionsData } from '$lib/types/store-product-prediction';
import { goto, invalidateAll } from '$app/navigation';
	import { buildImageUrl } from '$lib/utils/imageUrl.js';

let {
	data
}: {
    data: {
        items: Product[];
        total: number;
        page: number;
        pageCount: number;
        search: string;
        size: number;
        predictions: StoreProductPredictionsData | null;
        predictionsMessage?: string;
    };
	} = $props();

	// Server-driven search & pagination
    let searchQuery = $state(data.search || '');
    let activeSearchQuery = $state(data.search || '');
    let pageSize = $state(data.size || 10);
	let showSearch = $state(true);

    let isRefreshingPredictions = $state(false);
    let predictionError = $state('');
    let predictionSuccess = $state('');

    const predictions = $derived.by(() => data.predictions ?? null);
    const predictionProducts = $derived.by(() => predictions?.products ?? []);
    const hasPredictionProducts = $derived.by(() => predictionProducts.length > 0);

	// Delete confirmation dialog state
	let showDeleteDialog = $state(false);
	let productToDelete = $state<Product | null>(null);
	let isDeleting = $state(false);

	// Server-driven pagination values
    const filteredItems = $derived.by(() => data.items || []);
    const totalPages = $derived.by(() => data.pageCount || 1);
    const currentDisplayPage = $derived.by(() => data.page || 1);
    const startIndex = $derived.by(() => (data.items?.length ?? 0) > 0 ? (data.page - 1) * data.size + 1 : 0);
    const endIndex = $derived.by(() => (data.items?.length ?? 0) > 0 ? Math.min(startIndex + (data.items?.length ?? 0) - 1, data.total) : 0);

	// Pagination function
    function go(page: number) {
        const s = activeSearchQuery ? `&search=${encodeURIComponent(activeSearchQuery)}` : '';
        const size = `&size=${pageSize}`;
        goto(`/master/products?page=${page}${size}${s}`);
    }

	// Delete function
	function handleDelete(uuid: string) {
		const product = data.items.find((p) => p.uuid === uuid);
		if (product) {
			productToDelete = product;
			showDeleteDialog = true;
		}
	}

	async function confirmDelete() {
		if (!productToDelete) return;

		isDeleting = true;
		try {
			const formData = new FormData();
			formData.append('uuid', productToDelete.uuid);

			const response = await fetch('?/delete', {
				method: 'POST',
				body: formData
			});

			if (response.ok) {
				showDeleteDialog = false;
				productToDelete = null;
			} else {
				alert('Failed to delete product');
			}
		} catch (error) {
			console.error('Error deleting product:', error);
			alert('Error deleting product');
		} finally {
			isDeleting = false;
		}
	}

	function cancelDelete() {
		showDeleteDialog = false;
		productToDelete = null;
	}

	// Search function
    function handleSearch(query: string) {
        activeSearchQuery = query;
        const size = `&size=${pageSize}`;
        goto(`/master/products?page=1${size}${query ? `&search=${encodeURIComponent(query)}` : ''}`);
    }

    function clearSearch() {
        searchQuery = '';
        activeSearchQuery = '';
        goto(`/master/products?page=1&size=${pageSize}`);
    }

    function changeSize(size: number) {
        pageSize = size;
        const s = activeSearchQuery ? `&search=${encodeURIComponent(activeSearchQuery)}` : '';
        goto(`/master/products?page=1&size=${size}${s}`);
    }

    async function refreshPredictions() {
        if (isRefreshingPredictions) {
            return;
        }

        isRefreshingPredictions = true;
        predictionError = '';
        predictionSuccess = '';

        try {
            const response = await fetch('?/predict', {
                method: 'POST'
            });

            const payload = await response.json().catch(() => ({}));

            if (!response.ok) {
                predictionError =
                    payload?.message ||
                    'Gagal memuat prediksi produk. Mohon coba kembali.';
                return;
            }

            predictionSuccess =
                payload?.message ?? 'Prediksi produk berhasil diperbarui.';
            await invalidateAll();
        } catch (error) {
            console.error('Error refreshing product predictions:', error);
            predictionError =
                error instanceof Error
                    ? error.message
                    : 'Gagal memuat prediksi produk. Mohon coba kembali.';
        } finally {
            isRefreshingPredictions = false;
        }
    }

    function formatProbability(value?: number): string {
        if (typeof value !== 'number' || Number.isNaN(value)) {
            return '-';
        }

        const percentage = Math.round(value * 100);
        return `${percentage}%`;
    }

    function formatRecommendedQty(value?: number | string | null): string {
        const numericValue =
            typeof value === 'number'
                ? value
                : value !== null && value !== undefined
                    ? Number.parseFloat(value)
                    : Number.NaN;

        if (!Number.isFinite(numericValue)) {
            return '-';
        }

        return new Intl.NumberFormat('id-ID', {
            minimumFractionDigits: Number.isInteger(numericValue) ? 0 : 2,
            maximumFractionDigits: 2
        }).format(numericValue);
    }

    function formatGeneratedAt(value?: number): string {
        if (typeof value !== 'number' || Number.isNaN(value) || value <= 0) {
            return '-';
        }

        return new Date(value).toLocaleString('id-ID', {
            year: 'numeric',
            month: 'short',
            day: 'numeric',
            hour: '2-digit',
            minute: '2-digit'
        });
    }

    function formatWeatherValue(value?: number): string {
        if (typeof value !== 'number' || Number.isNaN(value)) {
            return '-';
        }

        return value.toFixed(1);
    }

	function handleAddProduct() {
		goto('/master/products/new');
	}

	function handleEditProduct(uuid: string) {
		goto(`/master/products/${uuid}/edit`);
	}

	// Format timestamp to readable date
	function formatDate(timestamp: number): string {
		return new Date(timestamp).toLocaleDateString('en-US', {
			year: 'numeric',
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	// Format price to currency
	function formatPrice(price: string): string {
		const numPrice = parseFloat(price);
		return new Intl.NumberFormat('id-ID', {
			style: 'currency',
			currency: 'IDR',
			minimumFractionDigits: 0
		}).format(numPrice);
	}

	// Get status badge color
	function getStatusColor(status: string): string {
		return status === 'ACTIVE' ? 'emerald' : 'gray';
	}

	// Build complete image URL
	function getImageUrl(imageUrl: string | null): string | null {
		return buildImageUrl(imageUrl);
	}

	const columns = [
		{ key: 'image', label: 'Image' },
		{ key: 'name', label: 'Name' },
		{ key: 'sku', label: 'SKU' },
		{ key: 'price', label: 'Price' },
		{ key: 'status', label: 'Status' },
		{ key: 'created_at', label: 'Created At' },
		{ key: 'actions', label: 'Actions' }
	] as const;
</script>

<section class="space-y-4">
	<h1 class="text-xl font-semibold">Products</h1>
	<!-- Header bar: title + add button (search hidden for now) -->
	<div class="mb-4 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
		<div class="flex items-center gap-2">
			{#if showSearch}
				<SearchBox value={searchQuery} onSearch={handleSearch} />
				{#if activeSearchQuery}
					<Button label="Clear" color="gray" onClick={clearSearch} />
				<span class="text-sm text-gray-600 dark:text-gray-400">
					Total {data.total} products for "{activeSearchQuery}"
				</span>
			{:else}
				<span class="text-sm text-gray-600 dark:text-gray-400"> Press Enter to search </span>
			{/if}
		{:else}
			<span class="text-sm text-gray-600 dark:text-gray-400">
				Total {data.total} products
			</span>
		{/if}
		</div>
		<Button label="Add Product" color="emerald" onClick={handleAddProduct} />
	</div>

	<!-- Table -->
	<div
		class="overflow-x-auto rounded-[var(--radius-card)] border bg-white/60 backdrop-blur-md dark:bg-white/10" style="border-color: var(--ui-border);"
	>
		<table class="min-w-full text-sm">
			<thead class="bg-gray-50 dark:bg-gray-900">
				<tr>
					{#each columns as c}
						<th class="px-4 py-2 text-left font-semibold">{c.label}</th>
					{/each}
				</tr>
			</thead>
			<tbody>
			{#each data.items as row (row.uuid)}
					<tr class="odd:bg-gray-50 dark:odd:bg-gray-800">
						<td class="px-4 py-2">
							{#if getImageUrl(row.image_url)}
								<img
									src={getImageUrl(row.image_url)}
									alt={row.name}
									class="h-12 w-12 rounded-lg object-cover"
									loading="lazy"
								/>
							{:else}
								<div
									class="flex h-12 w-12 items-center justify-center rounded-lg bg-gray-200 dark:bg-gray-700"
								>
									<span class="text-xs text-gray-500">No Image</span>
								</div>
							{/if}
						</td>
						<td class="px-4 py-2 font-medium">{row.name}</td>
						<td class="px-4 py-2 text-gray-600 dark:text-gray-400">{row.sku}</td>
						<td class="px-4 py-2 text-gray-600 dark:text-gray-400">{formatPrice(row.price)}</td>
						<td class="px-4 py-2">
							<span
								class="inline-flex items-center rounded-full px-2 py-1 text-xs font-medium {row.status ===
								'ACTIVE'
									? 'bg-emerald-100 text-emerald-800 dark:bg-emerald-900 dark:text-emerald-200'
									: 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-200'}"
							>
								{row.status}
							</span>
						</td>
						<td class="px-4 py-2 text-gray-600 dark:text-gray-400">{formatDate(row.created_at)}</td>
						<td class="px-4 py-2">
							<div class="flex items-center gap-2">
								<Button label="Edit" color="violet" onClick={() => handleEditProduct(row.uuid)} />
								<Button label="Delete" color="red" onClick={() => handleDelete(row.uuid)} />
							</div>
						</td>
					</tr>
				{/each}

				{#if data.items.length === 0}
					<tr>
						<td class="px-4 py-6 opacity-60" colspan={columns.length}>
							{activeSearchQuery ? 'No products match your search' : 'No products available'}
						</td>
					</tr>
				{/if}
			</tbody>
		</table>
	</div>

    <!-- Pagination -->
    <div class="flex justify-end">
        <TablePagination
            page={currentDisplayPage}
            pageCount={totalPages}
            total={data.total}
            start={startIndex}
            end={endIndex}
            size={pageSize}
            sizes={[3, 5, 10, 25, 50]}
            onChangePage={go}
            onChangeSize={changeSize}
        />
    </div>

    <!-- Store product predictions -->
    <div class="mt-8 space-y-4">
        <div class="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
            <div>
                <h2 class="text-lg font-semibold">Store Product Predictions</h2>
                {#if data.predictionsMessage}
                    <p class="text-sm text-gray-600 dark:text-gray-400">{data.predictionsMessage}</p>
                {/if}
            </div>
            <Button
                label={isRefreshingPredictions ? 'Refreshing...' : 'Refresh Predictions'}
                color="violet"
                disabled={isRefreshingPredictions}
                onClick={refreshPredictions}
            />
        </div>

        {#if predictionError}
            <div
                class="rounded-lg border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700 dark:border-red-900/40 dark:bg-red-900/10 dark:text-red-300"
            >
                {predictionError}
            </div>
        {/if}

        {#if predictionSuccess}
            <div
                class="rounded-lg border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-700 dark:border-emerald-900/40 dark:bg-emerald-900/10 dark:text-emerald-300"
            >
                {predictionSuccess}
            </div>
        {/if}

        {#if predictions}
            <div
                class="space-y-4 rounded-[var(--radius-card)] border bg-white/60 p-4 backdrop-blur-md dark:bg-white/10"
                style="border-color: var(--ui-border);"
            >
                <div class="grid gap-4 md:grid-cols-2">
                    <div class="space-y-1">
                        <p class="text-sm font-medium uppercase tracking-wide text-gray-500 dark:text-gray-400">
                            Generated At
                        </p>
                        <p class="text-base">{formatGeneratedAt(predictions.generated_at_ms)}</p>
                        {#if predictions.llm_summary}
                            <p class="text-base leading-relaxed text-gray-700 dark:text-gray-200">
                                {predictions.llm_summary}
                            </p>
                        {/if}
                        {#if predictions.llm_model}
                            <p class="text-sm text-gray-500 dark:text-gray-500">
                                Model: {predictions.llm_model}
                            </p>
                        {/if}
                    </div>
                    <div class="rounded-lg bg-gray-50 p-4 dark:bg-gray-900/40">
                        <p class="text-sm font-semibold text-gray-700 dark:text-gray-200">Weather</p>
                        <div class="mt-2 grid grid-cols-2 gap-2 text-sm text-gray-600 dark:text-gray-300">
                            <div>
                                <p class="font-medium text-gray-700 dark:text-gray-200">Summary</p>
                                <p>{predictions.weather?.summary ?? '-'}</p>
                            </div>
                            <div>
                                <p class="font-medium text-gray-700 dark:text-gray-200">Temp Min (°C)</p>
                                <p>{formatWeatherValue(predictions.weather?.temp_min_c)}</p>
                            </div>
                            <div>
                                <p class="font-medium text-gray-700 dark:text-gray-200">Temp Max (°C)</p>
                                <p>{formatWeatherValue(predictions.weather?.temp_max_c)}</p>
                            </div>
                            <div>
                                <p class="font-medium text-gray-700 dark:text-gray-200">Humidity (%)</p>
                                <p>{formatWeatherValue(predictions.weather?.humidity_avg)}</p>
                            </div>
                            <div>
                                <p class="font-medium text-gray-700 dark:text-gray-200">Precip. (mm)</p>
                                <p>{formatWeatherValue(predictions.weather?.precipitation_total_mm)}</p>
                            </div>
                        </div>
                    </div>
                </div>

                <div>
                    <h3 class="text-base font-semibold text-gray-700 dark:text-gray-200">
                        Recommended Products
                    </h3>

                    {#if hasPredictionProducts}
                        <div class="mt-3 overflow-x-auto">
                            <table class="min-w-full text-sm">
                                <thead>
                                    <tr class="text-left text-gray-500 dark:text-gray-400">
                                        <th class="px-3 py-2 font-semibold">Product</th>
                                        <th class="px-3 py-2 font-semibold">SKU</th>
                                        <th class="px-3 py-2 font-semibold">Demand</th>
                                        <th class="px-3 py-2 font-semibold">Probability</th>
                                        <th class="px-3 py-2 font-semibold">Recommended Qty</th>
                                        <th class="px-3 py-2 font-semibold">Reasoning</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {#each predictionProducts as item, index (item.product_uuid ?? index.toString())}
                                        <tr class="border-b last:border-b-0 dark:border-gray-800/60">
                                            <td class="px-3 py-2 text-sm font-medium">
                                                {item.product_name ?? '-'}
                                            </td>
                                            <td class="px-3 py-2 text-sm text-gray-600 dark:text-gray-400">
                                                {item.product_sku ?? '-'}
                                            </td>
                                            <td class="px-3 py-2">
                                                <span class="rounded-full bg-emerald-100 px-2 py-1 text-sm font-medium text-emerald-800 dark:bg-emerald-900/40 dark:text-emerald-300">
                                                    {item.demand_label ?? '-'}
                                                </span>
                                            </td>
                                            <td class="px-3 py-2 text-sm text-gray-600 dark:text-gray-400">
                                                {formatProbability(item.demand_probability)}
                                            </td>
                                            <td class="px-3 py-2 text-sm text-gray-600 dark:text-gray-400">
                                                {formatRecommendedQty(item.recommended_stock_qty)}
                                            </td>
                                            <td class="px-3 py-2 text-base leading-relaxed text-gray-700 dark:text-gray-200">
                                                {item.llm_reasoning ?? '-'}
                                            </td>
                                        </tr>
                                    {/each}
                                </tbody>
                            </table>
                        </div>
                    {:else}
                        <p class="mt-2 text-sm text-gray-600 dark:text-gray-400">
                            Tidak ada rekomendasi produk dalam hasil prediksi.
                        </p>
                    {/if}
                </div>
            </div>
        {:else}
            <p class="text-sm text-gray-600 dark:text-gray-400">
                Belum ada hasil prediksi untuk store ini.
            </p>
        {/if}
    </div>
</section>

<!-- Delete Confirmation Dialog -->
<ConfirmDialog
	show={showDeleteDialog}
	title="Delete Product"
	message="Are you sure you want to delete this product? This action cannot be undone."
	confirmText="OK"
	cancelText="Cancel"
	isLoading={isDeleting}
	onConfirm={confirmDelete}
	onCancel={cancelDelete}
/>
