<script lang="ts">
	import { enhance } from '$app/forms';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Select from '$lib/components/ui/Select.svelte';
	import TextArea from '$lib/components/ui/TextArea.svelte';
	import { goto } from '$app/navigation';

	let {
		data
	}: {
		data: {
			uoms: Array<{ uuid: string; code: string; name: string }>;
		};
	} = $props();

	let name = $state('');
	let description = $state('');
	let baseUomUuid = $state('');
let minimumStock = $state('0');
let shelfLifeDays = $state('0');
let isSubmitting = $state(false);
let errors = $state<Record<string, string>>({});
const uomOptions = $derived.by(() =>
	data.uoms.map((uom) => ({
		value: uom.uuid,
		label: `${uom.name} (${uom.code})`
	}))
);

	function validateForm() {
		const newErrors: Record<string, string> = {};

		if (!name.trim()) {
			newErrors.name = 'Nama katalog bahan tidak boleh kosong';
		}

		if (!baseUomUuid) {
			newErrors.baseUomUuid = 'Satuan dasar harus dipilih';
		}

		if (isNaN(parseFloat(minimumStock)) || parseFloat(minimumStock) < 0) {
			newErrors.minimumStock = 'Stok minimum harus berupa angka positif';
		}

		if (isNaN(parseInt(shelfLifeDays)) || parseInt(shelfLifeDays) < 0) {
			newErrors.shelfLifeDays = 'Masa simpan harus berupa angka positif';
		}

		errors = newErrors;
		return Object.keys(newErrors).length === 0;
	}

	function handleCancel() {
		goto('/master/ingredient-catalog');
	}
</script>

<div class="container mx-auto p-4">
	<div class="bg-white rounded-lg shadow-md p-6">
		<h1 class="text-2xl font-bold mb-6">Tambah Katalog Bahan</h1>

		<form method="POST" use:enhance={() => {
			isSubmitting = true;
			
			return async ({ result }) => {
				isSubmitting = false;
				
				if (result.type === 'success') {
					goto('/master/ingredient-catalog');
				}
			};
		}}>
			<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
				<div>
					<Input
						label="Nama Katalog Bahan"
						name="name"
						bind:value={name}
						placeholder="Masukkan nama katalog bahan"
						required
						error={errors.name}
					/>
				</div>

				<div>
					<Select
						label="Satuan Dasar"
						name="baseUomUuid"
						bind:value={baseUomUuid}
						required
						error={errors.baseUomUuid}
						options={uomOptions}
						placeholder="Pilih Satuan"
					/>
				</div>

				<div>
					<Input
						label="Stok Minimum"
						name="minimumStock"
						type="number"
						step="0.001"
						min="0"
						bind:value={minimumStock}
						placeholder="0.000"
						error={errors.minimumStock}
					/>
				</div>

				<div>
					<Input
						label="Masa Simpan (Hari)"
						name="shelfLifeDays"
						type="number"
						min="0"
						bind:value={shelfLifeDays}
						placeholder="0"
						error={errors.shelfLifeDays}
					/>
				</div>

				<div class="md:col-span-2">
					<TextArea
						label="Deskripsi"
						name="description"
						bind:value={description}
						placeholder="Masukkan deskripsi katalog bahan (opsional)"
						rows={4}
					/>
				</div>
			</div>

			<div class="flex justify-end space-x-4 mt-8">
				<Button type="button" variant="outline" on:click={handleCancel}>
					Batal
				</Button>
				<Button type="submit" disabled={isSubmitting}>
					{isSubmitting ? 'Menyimpan...' : 'Simpan'}
				</Button>
			</div>
		</form>
	</div>
</div>
