<script lang="ts">
import { createEventDispatcher, onDestroy, tick } from 'svelte';

	type Option = { value: string; label: string };

	// ID: Komponen Select khusus dengan dropdown kustom & dukungan pencarian
	// EN: Custom select component with rounded dropdown and optional search
	export let id: string = '';
	export let name: string = '';
	export let label: string = '';
	export let value: string = '';
	export let options: Option[] = [];
	export let required: boolean = false;
	export let disabled: boolean = false;
	export let error: string = '';
	export let placeholder: string = 'Pilih...';
	export let classes: string = '';
	export let searchable: boolean = false;
	export let searchPlaceholder = 'Cari...';
export let emptyState = 'Tidak ada pilihan';
export let loading = false;
export let onSearch: ((query: string) => void | Promise<void>) | null = null;

	const dispatch = createEventDispatcher<{
		change: { value: string };
		select: { option: Option };
		focus: void;
		blur: void;
		search: { query: string };
	}>();

	let isOpen = false;
let searchQuery = '';
let buttonEl: HTMLButtonElement | null = null;
let searchInputEl: HTMLInputElement | null = null;
let remoteLoading = false;

	$: selectedOption = options.find((opt) => opt.value === value);
	$: filteredOptions =
		searchable && searchQuery.trim().length > 0
			? options.filter((opt) =>
					opt.label.toLowerCase().includes(searchQuery.trim().toLowerCase())
				)
			: options;

	async function openDropdown() {
		if (disabled) return;
		isOpen = true;
		await tick();
		if (searchable && searchInputEl) {
			searchInputEl.focus();
			searchInputEl.select();
		}
	}

	function closeDropdown() {
		isOpen = false;
		searchQuery = '';
	}

	function toggleDropdown() {
		isOpen ? closeDropdown() : openDropdown();
	}

	async function handleSearchInput(event: Event) {
		const query = (event.target as HTMLInputElement).value;
		searchQuery = query;
	dispatch('search', { query });
	if (onSearch) {
		const result = onSearch(query);
		if (result instanceof Promise) {
			remoteLoading = true;
			try {
				await result;
			} catch {
				// swallow async search errors to avoid breaking typing UX
			} finally {
				remoteLoading = false;
			}
		}
	}
}

	function selectOption(option: Option) {
		if (option.value === value) {
			closeDropdown();
			return;
		}
		value = option.value;
		dispatch('change', { value: option.value });
		dispatch('select', { option });
		closeDropdown();
	}

	function handleBlur(event: FocusEvent) {
		if (!event.relatedTarget || !(event.relatedTarget instanceof Node)) {
			dispatch('blur');
			return;
		}
		const nextTarget = event.relatedTarget as Node;
		if (!container?.contains(nextTarget)) {
			closeDropdown();
			dispatch('blur');
		}
	}

	let container: HTMLDivElement | null = null;

	function handleDocumentClick(event: MouseEvent) {
		if (!container || container.contains(event.target as Node)) return;
		closeDropdown();
	}

$: {
		if (isOpen) {
			document.addEventListener('click', handleDocumentClick);
		} else {
			document.removeEventListener('click', handleDocumentClick);
		}
	}

	onDestroy(() => {
		document.removeEventListener('click', handleDocumentClick);
	});
</script>

<div class={`form-group mb-3 ${classes}`} bind:this={container}>
	{#if label}
		<label for={id} class="form-label">
			{label}
			{#if required}
				<span class="text-danger">*</span>
			{/if}
		</label>
	{/if}

	<!-- Hidden select untuk compatibility form submission -->
	<select
		id={id}
		name={name}
		class="sr-only native-select"
		bind:value
		{required}
		disabled={disabled}
		tabindex="-1"
		aria-hidden="true"
	>
		<option value="">{placeholder}</option>
		{#each options as option}
			<option value={option.value}>{option.label}</option>
		{/each}
	</select>

	<button
		class={`custom-select-trigger ${error ? 'is-invalid' : ''} ${disabled ? 'is-disabled' : ''}`}
		type="button"
		class:open={isOpen}
		disabled={disabled}
		aria-expanded={isOpen}
		on:click={toggleDropdown}
		on:focus={() => dispatch('focus')}
		on:blur={handleBlur}
		bind:this={buttonEl}
	>
		<span class={`value ${selectedOption ? 'has-value' : ''}`}>
			{selectedOption ? selectedOption.label : placeholder}
		</span>
		<svg class="chevron" viewBox="0 0 20 20" fill="none" aria-hidden="true">
			<path
				d="M5 7.5l5 5 5-5"
				stroke="currentColor"
				stroke-width="1.5"
				stroke-linecap="round"
				stroke-linejoin="round"
			/>
		</svg>
	</button>

	{#if isOpen}
		<div class="dropdown" role="listbox">
			{#if searchable}
				<div class="search-bar">
					<input
						type="search"
						class="search-input"
						placeholder={searchPlaceholder}
						bind:this={searchInputEl}
						value={searchQuery}
						on:input={handleSearchInput}
					/>
					{#if loading || remoteLoading}
						<div class="spinner" aria-hidden="true"></div>
					{/if}
				</div>
			{/if}

			{#if filteredOptions.length === 0 && !(loading || remoteLoading)}
				<div class="empty-state">{emptyState}</div>
			{:else}
				<ul class="option-list">
					{#each filteredOptions as option (option.value)}
						<li>
							<button
								type="button"
								class={`option ${option.value === value ? 'selected' : ''}`}
								on:click={() => selectOption(option)}
							>
								{option.label}
							</button>
						</li>
					{/each}
				</ul>
			{/if}
		</div>
	{/if}

	{#if error}
		<div class="invalid-feedback">{error}</div>
	{/if}
</div>

<style>
	.form-group {
		position: relative;
		display: flex;
		flex-direction: column;
		gap: 0.4rem;
	}

	.native-select {
		position: absolute;
		width: 1px;
		height: 1px;
		clip: rect(0 0 0 0);
		clip-path: inset(50%);
		margin: -1px;
		border: 0;
		padding: 0;
		overflow: hidden;
	}

	.custom-select-trigger {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 0.5rem;
		width: 100%;
		border-radius: var(--radius-pill, 999px);
		border: 1px solid var(--input-border, rgba(0, 0, 0, 0.12));
		background:
			linear-gradient(135deg, color-mix(in srgb, var(--input-bg, #ffffff) 75%, #ffffff) 0%, var(--input-bg, #ffffff) 100%);
		color: var(--color-text, var(--text));
		font-weight: 600;
		padding: 0.65rem 1.25rem;
		min-height: 2.8rem;
		transition:
			border-color var(--transition-fast, 150ms ease-out),
			box-shadow var(--transition-fast, 150ms ease-out),
			background var(--transition-fast, 150ms ease-out),
			transform 120ms ease-out;
		position: relative;
	}

	.custom-select-trigger:hover:not(.is-disabled) {
		border-color: var(--input-focus-border, color-mix(in srgb, var(--accent) 35%, transparent));
	}

	.custom-select-trigger.open {
		border-color: var(--input-focus-border, var(--accent));
		box-shadow: 0 0 0 3px var(--input-focus-ring, rgba(255, 62, 0, 0.2));
	}

	.custom-select-trigger.is-disabled {
		cursor: not-allowed;
		opacity: 0.6;
	}

	.value {
		flex: 1;
		text-align: left;
		color: var(--color-text-muted, rgba(15, 23, 42, 0.55));
	}

	.value.has-value {
		color: var(--color-text, var(--text));
	}

	.chevron {
		width: 1rem;
		height: 1rem;
		color: var(--color-text-muted, rgba(15, 23, 42, 0.5));
		transition: transform 150ms ease;
	}

	.custom-select-trigger.open .chevron {
		transform: rotate(180deg);
	}

	.dropdown {
		position: absolute;
		top: 100%;
		left: 0;
		right: 0;
		margin-top: 0.4rem;
		border-radius: var(--radius-xl, 20px);
		border: 1px solid color-mix(in srgb, rgba(15, 23, 42, 0.12) 75%, transparent);
		background: color-mix(in srgb, var(--color-surface, #ffffff) 95%, rgba(255, 255, 255, 0.88));
		box-shadow:
			0 18px 40px rgba(15, 23, 42, 0.18),
			0 2px 6px rgba(15, 23, 42, 0.08);
		padding: 0.6rem;
		z-index: 20;
		backdrop-filter: blur(18px);
	}

	.search-bar {
		position: relative;
		margin-bottom: 0.5rem;
	}

	.search-input {
		width: 100%;
		border-radius: var(--radius-pill, 999px);
		border: 1px solid color-mix(in srgb, rgba(15, 23, 42, 0.16) 70%, transparent);
		padding: 0.5rem 0.95rem;
		background: color-mix(in srgb, rgba(240, 244, 255, 0.85) 65%, white);
		color: var(--color-text, #0f172a);
		font-size: 0.92rem;
		outline: none;
		transition:
			border-color var(--transition-fast, 150ms ease-out),
			box-shadow var(--transition-fast, 150ms ease-out);
	}

	.search-input:focus {
		border-color: var(--input-focus-border, var(--accent));
		box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent) 22%, transparent);
	}

	.spinner {
		position: absolute;
		right: 0.9rem;
		top: 50%;
		transform: translateY(-50%);
		width: 1rem;
		height: 1rem;
		border-radius: 999px;
		border: 2px solid rgba(100, 116, 139, 0.35);
		border-top-color: var(--accent);
		animation: spin 650ms linear infinite;
	}

	@keyframes spin {
		to {
			transform: translateY(-50%) rotate(360deg);
		}
	}

	.option-list {
		max-height: 14rem;
		overflow-y: auto;
		padding: 0.25rem;
		margin: 0;
		list-style: none;
		display: flex;
		flex-direction: column;
		gap: 0.35rem;
		scrollbar-width: thin;
	}

	.option {
		width: 100%;
		text-align: left;
		border: 0;
		border-radius: var(--radius-pill, 999px);
		padding: 0.55rem 0.95rem;
		background: color-mix(in srgb, rgba(239, 246, 255, 0.65) 55%, white);
		color: var(--color-text, #0f172a);
		font-weight: 600;
		transition:
			background 150ms ease,
			transform 120ms ease,
			color 150ms ease;
	}

	.option:hover,
	.option:focus-visible {
		background: color-mix(in srgb, var(--accent) 20%, rgba(255, 255, 255, 0.9));
		color: color-mix(in srgb, var(--accent) 65%, #0f172a);
	}

	.option.selected {
		background: color-mix(in srgb, var(--accent) 28%, rgba(255, 255, 255, 0.95));
		color: color-mix(in srgb, var(--accent) 80%, #0f172a);
	}

	.empty-state {
		padding: 1rem;
		text-align: center;
		font-size: 0.92rem;
		color: var(--color-text-muted, rgba(15, 23, 42, 0.6));
	}

	.invalid-feedback {
		color: var(--color-error, #dc2626);
		font-size: 0.85rem;
		margin-top: 0.25rem;
	}

    /*
      ID: Gunakan selector berbasis data-mode agar konsisten dengan sistem tema app
      EN: Use data-mode based selector to align with app's theming system
      Catatan: Mengganti prefers-color-scheme agar light mode tidak ikut OS dark
    */
    :global(html[data-mode='dark']) .custom-select-trigger {
        border-color: rgba(148, 163, 184, 0.35);
        background: rgba(15, 23, 42, 0.72);
        color: rgba(226, 232, 240, 0.92);
    }

    :global(html[data-mode='dark']) .value {
        color: rgba(148, 163, 184, 0.8);
    }

    :global(html[data-mode='dark']) .dropdown {
        border-color: rgba(148, 163, 184, 0.25);
        background: color-mix(in srgb, rgba(15, 23, 42, 0.9) 80%, rgba(30, 41, 59, 0.85));
        box-shadow:
            0 22px 50px rgba(2, 6, 23, 0.65),
            0 2px 6px rgba(15, 23, 42, 0.28);
    }

    :global(html[data-mode='dark']) .search-input {
        border-color: rgba(148, 163, 184, 0.35);
        background: rgba(30, 41, 59, 0.6);
        color: rgba(226, 232, 240, 0.92);
    }

    :global(html[data-mode='dark']) .option {
        background: rgba(30, 41, 59, 0.65);
        color: rgba(226, 232, 240, 0.92);
    }

    :global(html[data-mode='dark']) .option:hover,
    :global(html[data-mode='dark']) .option:focus-visible {
        background: color-mix(in srgb, var(--accent) 28%, rgba(15, 23, 42, 0.4));
        color: rgba(248, 250, 252, 0.92);
    }

    :global(html[data-mode='dark']) .option.selected {
        background: color-mix(in srgb, var(--accent) 38%, rgba(15, 23, 42, 0.35));
        color: rgba(248, 250, 252, 0.98);
    }

    :global(html[data-mode='dark']) .empty-state {
        color: rgba(148, 163, 184, 0.9);
    }
</style>
