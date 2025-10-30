<script lang="ts">
	import { aiChatApi } from '$lib/services/aiApi';
	import { ragAnswerApi, uploadRagDocument, type RagAnswerResponse } from '$lib/services/ragApi';
	import { onDestroy, onMount } from 'svelte';
	import { locale, t } from '$lib/stores/i18n';

	// Chat state
	let messages = $state<
		Array<{
			id: string;
			type: 'user' | 'ai';
			content: string;
			timestamp: number;
			tokensUsed?: number;
			model?: string;
		}>
	>([]);

	let currentMessage = $state('');
	let isLoading = $state(false);
	let error = $state('');
	let maxTokens = $state(1000);
	let temperature = $state(0.7);
	let useRag = $state(false);
	let isUploading = $state(false);
	let uploadFeedback = $state<{ type: 'success' | 'error'; message: string } | null>(null);

	const dateLocale = $derived.by(() => ($locale === 'id' ? 'id-ID' : 'en-US'));
	const aiText = $derived.by(() => {
		const _ = $locale;
		return {
			pageTitle: t('ai_chat_page_title'),
			heading: t('ai_chat_heading'),
			subheading: t('ai_chat_subheading'),
			ragNotice: t('ai_chat_rag_notice'),
			ragToggleOn: t('ai_chat_toggle_on'),
			ragToggleOff: t('ai_chat_toggle_off'),
			uploadLabel: t('ai_chat_upload_label'),
			uploadingLabel: t('ai_chat_uploading'),
			settingsButton: t('ai_chat_settings_button'),
			clearButton: t('ai_chat_clear_button'),
			settingsTitle: t('ai_chat_settings_title'),
			maxTokensHelp: t('ai_chat_max_tokens_help'),
			temperatureHelp: t('ai_chat_temperature_help'),
			welcomeTitle: t('ai_chat_welcome_title'),
			welcomeSubtitle: t('ai_chat_welcome_subtitle'),
			loading: t('ai_chat_loading'),
			inputPlaceholder: t('ai_chat_input_placeholder'),
			sending: t('ai_chat_sending'),
			send: t('ai_chat_send'),
			copyTooltip: t('ai_chat_copy_tooltip')
		};
	});

	// Chat settings
	let showSettings = $state(false);
	let feedbackTimeout: ReturnType<typeof setTimeout> | null = null;
	let fileInput: HTMLInputElement | null = null;

	// Load chat history from localStorage
	function loadChatHistory() {
		if (typeof window !== 'undefined') {
			try {
				const saved = localStorage.getItem('ai-chat-messages');
				if (saved) {
					messages = JSON.parse(saved);
				}
			} catch (err) {
				console.error('Failed to load chat history:', err);
			}
		}
	}

	// Save chat history to localStorage
	function saveChatHistory() {
		if (typeof window !== 'undefined') {
			try {
				localStorage.setItem('ai-chat-messages', JSON.stringify(messages));
			} catch (err) {
				console.error('Failed to save chat history:', err);
			}
		}
	}

	// Add message to chat
	function addMessage(type: 'user' | 'ai', content: string, tokensUsed?: number, model?: string) {
		const message = {
			id: Date.now().toString() + Math.random().toString(36).substr(2, 9),
			type,
			content,
			timestamp: Date.now(),
			tokensUsed,
			model
		};

		messages = [...messages, message];
		saveChatHistory();
	}

	// Send message to AI
	async function sendMessage() {
		if (!currentMessage.trim() || isLoading) return;

		const userMessage = currentMessage.trim();
		currentMessage = '';
		error = '';

		// Add user message
		addMessage('user', userMessage);

		// Scroll to bottom
		setTimeout(scrollToBottom, 100);

		isLoading = true;

		try {
			if (useRag) {
				const response = await ragAnswerApi({
					query: userMessage,
					max_tokens: maxTokens,
					temperature: temperature
				});

				if (response.code === 200) {
					const data = response.data;
					const isSuccess = data?.success ?? true;
					if (!isSuccess) {
						throw new Error(
							response.message || data?.message || t('ai_chat_error_rag_generate')
						);
					}

					const ragMessage = formatRagAnswer(data);
					const tokensUsed =
						data?.tokens_used ?? data?.usage?.total_tokens ?? data?.usage?.completion_tokens;

					addMessage('ai', ragMessage, tokensUsed, data?.model);
				} else {
					throw new Error(response.message || t('ai_chat_error_rag_fetch'));
				}
			} else {
				// Call AI API
				const response = await aiChatApi({
					prompt: userMessage,
					max_tokens: maxTokens,
					temperature: temperature
				});

				if (response.code === 200 && response.data.success) {
					// Add AI response
					addMessage('ai', response.data.response, response.data.tokens_used, response.data.model);
				} else {
					throw new Error(response.message || t('ai_chat_error_ai_fetch'));
				}
			}
		} catch (err) {
			console.error('AI chat error:', err);
			const fallback = t('ai_chat_error_send');
			error = err instanceof Error ? err.message || fallback : fallback;

			// Add error message to chat
			addMessage('ai', t('ai_chat_error_with_reason', { message: error }));
		} finally {
			isLoading = false;
			setTimeout(scrollToBottom, 100);
		}
	}

	// Handle Enter key
	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter' && !event.shiftKey) {
			event.preventDefault();
			sendMessage();
		}
	}

	// Scroll to bottom of chat
	function scrollToBottom() {
		const chatContainer = document.getElementById('chat-container');
		if (chatContainer) {
			chatContainer.scrollTop = chatContainer.scrollHeight;
		}
	}

	// Clear chat
	function clearChat() {
		messages = [];
		saveChatHistory();
		error = '';
	}

	function toggleRag() {
		useRag = !useRag;
	}

	function formatRagAnswer(data: RagAnswerResponse | undefined) {
		if (!data) {
			return t('ai_chat_no_rag_response');
		}

		let content =
			data.answer?.trim() ||
			data.response?.trim() ||
			data.message?.trim() ||
			t('ai_chat_no_answer');

		// Remove technical chunk markers like "[doc] (chunk #0)"
		content = content
			.split('\n')
			.filter((line) => !/^\s*\[[^\]]+\]\s*\(chunk\s*#\d+\)\s*\.?\s*$/i.test(line.trim()))
			.join('\n')
			.replace(/\n{3,}/g, '\n\n')
			.trim();

		// ID: Sembunyikan daftar sumber dari respons RAG untuk mencegah ID seperti "3694F193" ditampilkan kepada pengguna.
		// EN: Hide the RAG sources list from the chat output to avoid exposing internal IDs like "3694F193".
		// Catatan/Note: Jika di masa depan ingin menampilkan sumber lagi, pertimbangkan opsi toggle UI atau setting aplikasi.

		return content || t('ai_chat_no_answer');
	}

	async function handleDocumentUpload(event: Event) {
		const target = event.target as HTMLInputElement;
		if (!target.files || target.files.length === 0) {
			return;
		}

		const file = target.files[0];
		isUploading = true;
		uploadFeedback = null;
		error = '';

		const derivedCategory = /\.(xls|xlsx|xlsm)$/i.test(file.name) ? 'excel' : 'document';

		try {
			const response = await uploadRagDocument({
				file,
				category: derivedCategory,
				description: t('ai_chat_upload_description'),
				tags: derivedCategory === 'excel' ? 'excel,rag' : 'document,rag'
			});

			if (response.code === 200) {
				const data = response.data;
				const isSuccess = data?.success ?? true;
				if (!isSuccess) {
					throw new Error(response.message || data?.message || t('ai_chat_upload_failed'));
				}

			const docId = data?.document_id ?? '';

				setUploadFeedback({
					type: 'success',
					message: docId
						? t('ai_chat_upload_success_with_id', { id: docId })
						: t('ai_chat_upload_success')
				});
			} else {
				throw new Error(response.message || t('ai_chat_upload_failed'));
			}
		} catch (err) {
			console.error('RAG document upload error:', err);
			const message =
				err instanceof Error ? err.message : t('ai_chat_upload_error');
			setUploadFeedback({
				type: 'error',
				message
			});
		} finally {
			isUploading = false;
			target.value = '';
		}
	}

	function setUploadFeedback(value: { type: 'success' | 'error'; message: string } | null) {
		uploadFeedback = value;
		if (feedbackTimeout) {
			clearTimeout(feedbackTimeout);
			feedbackTimeout = null;
		}
		if (value) {
			feedbackTimeout = setTimeout(() => {
				uploadFeedback = null;
				feedbackTimeout = null;
			}, 5000);
		}
	}

	// Format timestamp
	function formatTime(timestamp: number) {
		return new Date(timestamp).toLocaleTimeString(dateLocale, {
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	// Copy message content
	async function copyMessage(content: string) {
		try {
			await navigator.clipboard.writeText(content);
		} catch (err) {
			console.error('Failed to copy:', err);
		}
	}

	// Load history on mount
	onMount(() => {
		loadChatHistory();
		setTimeout(scrollToBottom, 100);
	});

	onDestroy(() => {
		if (feedbackTimeout) {
			clearTimeout(feedbackTimeout);
		}
	});
</script>

<svelte:head>
	<title>{aiText.pageTitle}</title>
</svelte:head>

<div class="mx-auto h-[calc(100vh-8rem)] max-w-4xl">
	<input
		type="file"
		class="hidden"
		accept=".doc,.docx,.xls,.xlsx,.xlsm"
		bind:this={fileInput}
		onchange={handleDocumentUpload}
	/>
	<!-- Header -->
	<div class="mb-6 flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
		<div>
			<h1 class="text-3xl font-bold text-gray-900 dark:text-white">{aiText.heading}</h1>
			<p class="mt-2 text-gray-600 dark:text-gray-400">
				{aiText.subheading}
			</p>
			{#if useRag}
				<p class="mt-1 text-xs font-semibold uppercase tracking-wide text-emerald-600 dark:text-emerald-400">
					{aiText.ragNotice}
				</p>
			{/if}
		</div>

		<div class="flex flex-wrap items-center justify-end gap-3">
			<button
				type="button"
				class={`rounded-lg px-4 py-2 text-sm font-medium transition-colors ${
					useRag
						? 'bg-emerald-600 text-white hover:bg-emerald-700'
						: 'border border-gray-300 text-gray-700 hover:bg-gray-50 dark:border-gray-600 dark:text-gray-200 dark:hover:bg-gray-700'
				}`}
				aria-pressed={useRag}
				onclick={toggleRag}
				disabled={isLoading}
			>
				{useRag ? aiText.ragToggleOn : aiText.ragToggleOff}
			</button>

			<button
				type="button"
				class="flex items-center gap-2 rounded-lg border border-gray-300 px-4 py-2 text-sm transition-colors hover:bg-gray-50 disabled:cursor-not-allowed disabled:text-gray-400 dark:border-gray-600 dark:hover:bg-gray-700"
				onclick={() => fileInput?.click()}
				disabled={isUploading}
			>
				{#if isUploading}
					<svg
						class="h-4 w-4 animate-spin text-gray-500"
						xmlns="http://www.w3.org/2000/svg"
						fill="none"
						viewBox="0 0 24 24"
					>
						<circle
							class="opacity-25"
							cx="12"
							cy="12"
							r="10"
							stroke="currentColor"
							stroke-width="4"
						></circle>
						<path
							class="opacity-75"
							fill="currentColor"
							d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
						></path>
					</svg>
					{aiText.uploadingLabel}
				{:else}
					{aiText.uploadLabel}
				{/if}
			</button>

			<button
				type="button"
				class="rounded-lg border border-gray-300 px-4 py-2 text-sm transition-colors hover:bg-gray-50 dark:border-gray-600 dark:hover:bg-gray-700"
				onclick={() => (showSettings = !showSettings)}
			>
				⚙️ {aiText.settingsButton}
			</button>

			{#if messages.length > 0}
				<button
					type="button"
					class="rounded-lg bg-red-600 px-4 py-2 text-sm text-white transition-colors hover:bg-red-700"
					onclick={clearChat}
				>
					🗑️ {aiText.clearButton}
				</button>
			{/if}
		</div>
	</div>

	{#if uploadFeedback}
		<div
			class={`mb-4 rounded-lg border px-4 py-3 text-sm ${
				uploadFeedback.type === 'success'
					? 'border-emerald-200 bg-emerald-50 text-emerald-700 dark:border-emerald-900/40 dark:bg-emerald-900/10 dark:text-emerald-300'
					: 'border-red-200 bg-red-50 text-red-700 dark:border-red-900/40 dark:bg-red-900/10 dark:text-red-300'
			}`}
		>
			{uploadFeedback.message}
		</div>
	{/if}

	<!-- Settings Panel -->
	{#if showSettings}
		<div
			class="mb-6 rounded-xl border border-gray-200 bg-white p-6 shadow-lg dark:border-gray-700 dark:bg-gray-800"
		>
			<h3 class="mb-4 text-lg font-semibold text-gray-900 dark:text-white">{aiText.settingsTitle}</h3>

			<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
				<div>
					<label
						for="maxTokens"
						class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300"
					>
						{t('ai_chat_max_tokens_label', { value: maxTokens, locale: $locale })}
					</label>
					<input
						id="maxTokens"
						type="range"
						min="100"
						max="2000"
						step="100"
						bind:value={maxTokens}
						class="w-full"
					/>
					<div class="mt-1 text-xs text-gray-500">{aiText.maxTokensHelp}</div>
				</div>

				<div>
					<label
						for="temperature"
						class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300"
					>
						{t('ai_chat_temperature_label', { value: temperature, locale: $locale })}
					</label>
					<input
						id="temperature"
						type="range"
						min="0.1"
						max="1.0"
						step="0.1"
						bind:value={temperature}
						class="w-full"
					/>
					<div class="mt-1 text-xs text-gray-500">
						{aiText.temperatureHelp}
					</div>
				</div>
			</div>
		</div>
	{/if}

	<!-- Chat Container -->
	<div
		class="flex h-[calc(100vh-16rem)] flex-col rounded-xl border border-gray-200 bg-white shadow-lg dark:border-gray-700 dark:bg-gray-800"
	>
		<!-- Chat Messages -->
		<div id="chat-container" class="flex-1 space-y-4 overflow-y-auto p-6">
			{#if messages.length === 0}
				<div class="py-12 text-center">
					<div class="mb-4 text-6xl">🤖</div>
					<h3 class="mb-2 text-xl font-semibold text-gray-900 dark:text-white">
						{aiText.welcomeTitle}
					</h3>
					<p class="text-gray-600 dark:text-gray-400">
						{aiText.welcomeSubtitle}
					</p>
				</div>
			{:else}
				{#each messages as message}
					<div class="flex {message.type === 'user' ? 'justify-end' : 'justify-start'}">
						<div class="max-w-[80%] {message.type === 'user' ? 'order-2' : 'order-1'}">
							<!-- Message Bubble -->
							<div
								class="
								{message.type === 'user'
									? 'bg-blue-600 text-white'
									: 'bg-gray-100 text-gray-900 dark:bg-gray-700 dark:text-white'} 
								rounded-2xl px-4 py-3 shadow-sm
							"
							>
								<div class="text-sm leading-relaxed whitespace-pre-wrap">
									{@html message.content
										.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
										.replace(/\n/g, '<br>')}
								</div>
							</div>

							<!-- Message Info -->
							<div
								class="mt-1 flex items-center gap-2 px-2 {message.type === 'user'
									? 'justify-end'
									: 'justify-start'}"
							>
								<span class="text-xs text-gray-500">
									{formatTime(message.timestamp)}
								</span>

								{#if message.type === 'ai' && message.tokensUsed}
									<span class="text-xs text-gray-500">
										{t('ai_chat_token_counter', { count: message.tokensUsed, locale: $locale })}
									</span>
								{/if}

								{#if message.model}
									<span class="text-xs text-gray-500">
										• {message.model}
									</span>
								{/if}

								<button
									type="button"
									class="text-xs text-gray-500 hover:text-gray-700 dark:hover:text-gray-300"
									onclick={() => copyMessage(message.content)}
									title={aiText.copyTooltip}
								>
									📋
								</button>
							</div>
						</div>

						<!-- Avatar -->
						<div
							class="
							{message.type === 'user' ? 'order-1 mr-3' : 'order-2 ml-3'} 
							flex h-8 w-8 flex-shrink-0 items-center justify-center rounded-full text-sm
							{message.type === 'user'
								? 'bg-blue-600 text-white'
								: 'bg-gray-300 text-gray-700 dark:bg-gray-600 dark:text-gray-300'}
						"
						>
							{message.type === 'user' ? '👤' : '🤖'}
						</div>
					</div>
				{/each}
			{/if}

		<!-- Loading indicator -->
		{#if isLoading}
			<div class="flex justify-start">
				<div class="max-w-[80%]">
					<div class="rounded-2xl bg-gray-100 px-4 py-3 shadow-sm dark:bg-gray-700">
						<div class="flex items-center space-x-2">
							<div class="flex space-x-1">
								<div class="h-2 w-2 animate-bounce rounded-full bg-gray-400"></div>
								<div
									class="h-2 w-2 animate-bounce rounded-full bg-gray-400"
									style="animation-delay: 0.1s"
								></div>
								<div
									class="h-2 w-2 animate-bounce rounded-full bg-gray-400"
									style="animation-delay: 0.2s"
								></div>
							</div>
							<span class="text-sm text-gray-600 dark:text-gray-400">{aiText.loading}</span>
						</div>
					</div>
				</div>
				<div
					class="ml-3 flex h-8 w-8 flex-shrink-0 items-center justify-center rounded-full bg-gray-300 text-sm dark:bg-gray-600"
				>
					🤖
				</div>
			</div>
		{/if}
		</div>

		<!-- Error Display -->
		{#if error}
			<div
				class="mx-6 mb-4 rounded-lg border border-red-200 bg-red-50 p-3 dark:border-red-800 dark:bg-red-900/20"
			>
				<div class="text-sm text-red-700 dark:text-red-300">
					❌ {error}
				</div>
			</div>
		{/if}

		<!-- Message Input -->
		<div class="border-t border-gray-200 p-6 dark:border-gray-700">
			<div class="flex items-end gap-3">
				<div class="flex-1">
					<textarea
						bind:value={currentMessage}
						onkeydown={handleKeydown}
						placeholder={aiText.inputPlaceholder}
						class="w-full resize-none rounded-xl border border-gray-300 bg-white px-4 py-3 text-gray-900 focus:border-transparent focus:ring-2 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
						rows="3"
						disabled={isLoading}
					></textarea>
				</div>

				<button
					type="button"
					onclick={sendMessage}
					disabled={!currentMessage.trim() || isLoading}
					class="flex items-center gap-2 rounded-xl bg-blue-600 px-6 py-3 font-medium text-white transition-colors hover:bg-blue-700 disabled:cursor-not-allowed disabled:bg-gray-400"
				>
					{#if isLoading}
						<svg
							class="h-4 w-4 animate-spin"
							xmlns="http://www.w3.org/2000/svg"
							fill="none"
							viewBox="0 0 24 24"
						>
							<circle
								class="opacity-25"
								cx="12"
								cy="12"
								r="10"
								stroke="currentColor"
								stroke-width="4"
							></circle>
							<path
								class="opacity-75"
								fill="currentColor"
								d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
							></path>
						</svg>
						{aiText.sending}
					{:else}
						<svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8"
							></path>
						</svg>
						{aiText.send}
					{/if}
				</button>
			</div>

			<div class="mt-3 flex items-center justify-between text-xs text-gray-500">
				<div>
					{t('ai_chat_footer_status', {
						tokens: maxTokens,
						temperature,
						locale: $locale
					})}
				</div>
				<div>
					{t('ai_chat_message_count', { count: messages.length, locale: $locale })}
				</div>
			</div>
		</div>
	</div>
</div>

<style>
	/* Custom scrollbar for chat */
	#chat-container::-webkit-scrollbar {
		width: 6px;
	}

	#chat-container::-webkit-scrollbar-track {
		background: transparent;
	}

	#chat-container::-webkit-scrollbar-thumb {
		background: rgba(156, 163, 175, 0.5);
		border-radius: 3px;
	}

	#chat-container::-webkit-scrollbar-thumb:hover {
		background: rgba(156, 163, 175, 0.7);
	}
</style>
