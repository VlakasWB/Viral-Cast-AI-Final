<script lang="ts">
	// Reusable category form: create/edit. Posts to parent route action.
	import type { Category } from '$lib/types/category.js';
	import Button from '$lib/components/ui/Button.svelte';

	let {
		initial,
		submitLabel = 'Save',
		cancelHref = '/master/categories',
		includeUuidHidden = false
	}: {
		initial?: Partial<Category>;
		submitLabel?: string;
		cancelHref?: string;
		includeUuidHidden?: boolean; // For edit page
	} = $props();

	let name = $state(initial?.name ?? '');
	let uuid = $state(initial?.uuid ?? '');
</script>

<form method="POST" class="space-y-3">
	{#if includeUuidHidden}<input type="hidden" name="uuid" value={uuid} />{/if}

	<div class="grid gap-1">
		<label for="category-name" class="text-sm opacity-80">Name</label>
		<input id="category-name" class="form-input" name="name" required bind:value={name} />
	</div>

	<div class="flex gap-2 pt-2">
		<Button label="Cancel" color="deepYellow" href={cancelHref} />
		<Button label={submitLabel} color="sky" type="submit" />
	</div>
</form>
