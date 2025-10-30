<!-- src/lib/components/ui/ConfirmDialog.svelte -->
<script lang="ts">
	let {
		show = false,
		title = 'Confirmation',
		message = 'Are you sure you want to proceed?',
		confirmText = 'OK',
		cancelText = 'Cancel',
		onConfirm,
		onCancel,
		isLoading = false
	}: {
		show: boolean;
		title?: string;
		message?: string;
		confirmText?: string;
		cancelText?: string;
		onConfirm?: () => void | Promise<void>;
		onCancel?: () => void;
		isLoading?: boolean;
	} = $props();

	async function handleConfirm() {
		if (isLoading) return;
		await onConfirm?.();
	}

	function handleCancel() {
		if (isLoading) return;
		onCancel?.();
	}

	function handleBackdropClick(event: MouseEvent) {
		if (event.target === event.currentTarget) {
			handleCancel();
		}
	}

	function handleBackdropKeydown(event: KeyboardEvent) {
		const isSelf = event.target === event.currentTarget;
		if (event.key === 'Escape') {
			handleCancel();
			return;
		}
		// Allow Enter/Space to close only when focus is on backdrop itself
		if (isSelf && (event.key === 'Enter' || event.key === ' ' || event.key === 'Spacebar')) {
			handleCancel();
		}
	}
</script>

{#if show}
	<div
		class="bg-opacity-50 fixed inset-0 z-50 flex items-center justify-center bg-black"
		onclick={handleBackdropClick}
		onkeydown={handleBackdropKeydown}
		role="dialog"
		tabindex="0"
		aria-modal="true"
		aria-labelledby="dialog-title"
		aria-describedby="dialog-desc"
	>
		<div class="mx-4 w-full max-w-md rounded-lg bg-white p-6 shadow-xl dark:bg-gray-800">
			<h3 id="dialog-title" class="mb-4 text-lg font-medium text-gray-900 dark:text-gray-100">
				{title}
			</h3>

			<p id="dialog-desc" class="mb-6 text-sm text-gray-600 dark:text-gray-300">
				{message}
			</p>

			<div class="flex justify-end gap-3">
				<button
					type="button"
					onclick={handleCancel}
					disabled={isLoading}
					class="rounded-md px-4 py-2 text-sm font-medium transition-colors disabled:cursor-not-allowed disabled:opacity-50 bg-[var(--color-yellow-deep-600)] text-white hover:brightness-105"
				>
					{cancelText}
				</button>

				<button
					type="button"
					onclick={handleConfirm}
					disabled={isLoading}
					class="rounded-md bg-red-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-red-700 disabled:cursor-not-allowed disabled:opacity-50"
				>
					{isLoading ? 'Processing...' : confirmText}
				</button>
			</div>
		</div>
	</div>
{/if}
