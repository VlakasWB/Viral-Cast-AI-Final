<script lang="ts">
	import { page } from '$app/stores';
	import type { PageData } from './$types';

	export let data: PageData;

	// Format date helper
	function formatDate(timestamp: number): string {
		return new Date(timestamp).toLocaleDateString('id-ID', {
			year: 'numeric',
			month: 'long',
			day: 'numeric'
		});
	}

	// Format datetime helper
	function formatDateTime(timestamp: number): string {
		return new Date(timestamp).toLocaleString('id-ID', {
			year: 'numeric',
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	// Get active recipe
	$: activeRecipe = data.recipeSets.find((recipe) => recipe.is_active);
	$: inactiveRecipes = data.recipeSets.filter((recipe) => !recipe.is_active);
</script>

<svelte:head>
	<title>Detail Produk: {data.product.name} - Viral Cast AI</title>
</svelte:head>

<div class="container mx-auto px-4 py-6">
	<!-- Breadcrumb -->
	<div class="mb-6 flex items-center gap-2 text-sm text-gray-600">
		<a href="/master/products" class="hover:text-blue-600">Master Produk</a>
		<span>/</span>
		<span>{data.product.name}</span>
	</div>

	<!-- Product Header -->
	<div class="mb-6 rounded-lg border bg-white p-6 shadow-sm">
		<div class="mb-4 flex items-start justify-between">
			<div>
				<h1 class="mb-2 text-2xl font-bold text-gray-900">{data.product.name}</h1>
				<div class="flex items-center gap-4">
					<span
						class="inline-flex rounded-full px-3 py-1 text-sm font-semibold {data.product.status ===
						'ACTIVE'
							? 'bg-green-100 text-green-800'
							: 'bg-red-100 text-red-800'}"
					>
						{data.product.status === 'ACTIVE' ? 'Aktif' : 'Tidak Aktif'}
					</span>
					<span class="text-sm text-gray-600"
						>SKU: <span class="font-medium">{data.product.sku}</span></span
					>
				</div>
			</div>
			<div class="flex gap-2">
				<a
					href="/master/products/{data.product.uuid}/edit"
					class="rounded-lg bg-blue-600 px-4 py-2 font-medium text-white transition-colors hover:bg-blue-700"
				>
					Edit Produk
				</a>
				<a
					href="/master/recipes/new?product_uuid={data.product.uuid}"
					class="rounded-lg bg-green-600 px-4 py-2 font-medium text-white transition-colors hover:bg-green-700"
				>
					+ Buat Resep
				</a>
			</div>
		</div>

		<!-- Product Details Grid -->
		<div class="grid grid-cols-1 gap-6 md:grid-cols-3">
			<div>
				<h3 class="mb-1 text-sm font-medium text-gray-500">Harga</h3>
				<p class="text-lg font-semibold text-gray-900">
					Rp {parseFloat(data.product.price).toLocaleString('id-ID')}
				</p>
			</div>
			<div>
				<h3 class="mb-1 text-sm font-medium text-gray-500">Kategori</h3>
				<p class="text-sm text-gray-900">{data.product.category_uuid}</p>
			</div>
			<div>
				<h3 class="mb-1 text-sm font-medium text-gray-500">Total Resep</h3>
				<p class="text-lg font-semibold text-gray-900">{data.recipeSets.length} resep</p>
			</div>
		</div>

		<!-- Product Image -->
		{#if data.product.image_url}
			<div class="mt-4 border-t border-gray-200 pt-4">
				<h3 class="mb-2 text-sm font-medium text-gray-500">Gambar Produk</h3>
				<img
					src={data.product.image_url}
					alt={data.product.name}
					class="h-32 w-32 rounded-lg border object-cover"
				/>
			</div>
		{/if}

		<!-- Timestamps -->
		<div class="mt-4 border-t border-gray-200 pt-4">
			<div class="grid grid-cols-1 gap-4 text-sm text-gray-600 md:grid-cols-2">
				<div>
					<span class="font-medium">Dibuat:</span>
					{formatDateTime(data.product.created_at)}
				</div>
				<div>
					<span class="font-medium">Diperbarui:</span>
					{formatDateTime(data.product.updated_at)}
				</div>
			</div>
		</div>
	</div>

	<!-- Active Recipe Section -->
	{#if activeRecipe}
		<div class="mb-6 overflow-hidden rounded-lg border bg-white shadow-sm">
			<div class="border-b border-green-200 bg-green-50 px-6 py-4">
				<div class="flex items-center justify-between">
					<h2 class="text-lg font-semibold text-green-900">Resep Aktif</h2>
					<span
						class="inline-flex rounded-full bg-green-100 px-2 py-1 text-xs font-semibold text-green-800"
					>
						Aktif
					</span>
				</div>
			</div>

			<div class="p-6">
				<div class="mb-4 flex items-start justify-between">
					<div>
						<h3 class="mb-2 text-xl font-semibold text-gray-900">{activeRecipe.name}</h3>
						<div class="grid grid-cols-1 gap-4 text-sm md:grid-cols-3">
							<div>
								<span class="text-gray-500">Yield Quantity:</span>
								<span class="ml-1 font-medium text-gray-900">{activeRecipe.yield_qty}</span>
							</div>
							<div>
								<span class="text-gray-500">Periode Efektif:</span>
								<span class="ml-1 font-medium text-gray-900">
									{formatDate(activeRecipe.effective_from)} - {formatDate(
										activeRecipe.effective_to
									)}
								</span>
							</div>
							<div>
								<span class="text-gray-500">Dibuat:</span>
								<span class="ml-1 font-medium text-gray-900"
									>{formatDateTime(activeRecipe.created_at)}</span
								>
							</div>
						</div>
					</div>
					<div class="flex gap-2">
					<a
						href={"/master/recipes/" + activeRecipe.uuid}
						class="text-sm font-medium text-blue-600 hover:text-blue-900"
					>
						Lihat Detail
					</a>
					<a
						href={"/master/recipes/" + activeRecipe.uuid + "/edit"}
						class="text-sm font-medium text-indigo-600 hover:text-indigo-900"
					>
						Edit
					</a>
					</div>
				</div>
			</div>
		</div>
	{:else}
		<div class="mb-6 rounded-lg border bg-white p-6 shadow-sm">
			<div class="text-center">
				<div class="mb-4 text-gray-400">
					<svg class="mx-auto h-12 w-12" fill="none" viewBox="0 0 24 24" stroke="currentColor">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"
						/>
					</svg>
				</div>
				<h3 class="mb-2 text-lg font-medium text-gray-900">Belum ada resep aktif</h3>
				<p class="mb-4 text-gray-600">
					Produk ini belum memiliki resep aktif. Buat resep untuk produk ini.
				</p>
				<a
					href={"/master/recipes/new?product_uuid=" + data.product.uuid}
					class="inline-flex items-center rounded-md bg-blue-600 px-4 py-2 font-medium text-white transition-colors hover:bg-blue-700"
				>
					+ Buat Resep Pertama
				</a>
			</div>
		</div>
	{/if}

	<!-- All Recipes Section -->
	{#if data.recipeSets.length > 0}
		<div class="overflow-hidden rounded-lg border bg-white shadow-sm">
			<div class="border-b border-gray-200 px-6 py-4">
				<div class="flex items-center justify-between">
					<h2 class="text-lg font-semibold text-gray-900">Semua Resep</h2>
					<a
						href={`/master/recipes?name=${encodeURIComponent(data.product.name)}`}
						class="text-sm font-medium text-blue-600 hover:text-blue-900"
					>
						Lihat Semua →
					</a>
				</div>
			</div>

			<div class="overflow-x-auto">
				<table class="min-w-full divide-y divide-gray-200">
					<thead class="bg-gray-50">
						<tr>
							<th
								class="px-6 py-3 text-left text-xs font-medium tracking-wider text-gray-500 uppercase"
							>
								Nama Resep
							</th>
							<th
								class="px-6 py-3 text-left text-xs font-medium tracking-wider text-gray-500 uppercase"
							>
								Yield Qty
							</th>
							<th
								class="px-6 py-3 text-left text-xs font-medium tracking-wider text-gray-500 uppercase"
							>
								Periode Efektif
							</th>
							<th
								class="px-6 py-3 text-left text-xs font-medium tracking-wider text-gray-500 uppercase"
							>
								Status
							</th>
							<th
								class="px-6 py-3 text-left text-xs font-medium tracking-wider text-gray-500 uppercase"
							>
								Aksi
							</th>
						</tr>
					</thead>
					<tbody class="divide-y divide-gray-200 bg-white">
						{#each data.recipeSets as recipe}
							<tr class="hover:bg-gray-50">
								<td class="px-6 py-4 whitespace-nowrap">
									<div class="text-sm font-medium text-gray-900">{recipe.name}</div>
								</td>
								<td class="px-6 py-4 whitespace-nowrap">
									<div class="text-sm text-gray-900">{recipe.yield_qty}</div>
								</td>
								<td class="px-6 py-4 whitespace-nowrap">
									<div class="text-sm text-gray-900">
										{formatDate(recipe.effective_from)} - {formatDate(recipe.effective_to)}
									</div>
								</td>
								<td class="px-6 py-4 whitespace-nowrap">
									<span
										class="inline-flex rounded-full px-2 py-1 text-xs font-semibold {recipe.is_active
											? 'bg-green-100 text-green-800'
											: 'bg-red-100 text-red-800'}"
									>
										{recipe.is_active ? 'Aktif' : 'Tidak Aktif'}
									</span>
								</td>
								<td class="space-x-2 px-6 py-4 text-sm font-medium whitespace-nowrap">
							<a href={"/master/recipes/" + recipe.uuid} class="text-blue-600 hover:text-blue-900">
								Detail
							</a>
							<a
								href={"/master/recipes/" + recipe.uuid + "/edit"}
								class="text-indigo-600 hover:text-indigo-900"
							>
								Edit
							</a>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		</div>
	{/if}
</div>
