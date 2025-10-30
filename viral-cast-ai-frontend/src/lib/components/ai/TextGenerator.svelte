<script lang="ts">
	import Button from '$lib/components/ui/Button.svelte';
	import { aiChatApi } from '$lib/services/aiApi';

	// ---- state form
	let topic = $state('');
	let tone: 'neutral' | 'friendly' | 'professional' | 'playful' | 'persuasive' | 'casual' =
		$state('neutral');
	let length: 'short' | 'medium' | 'long' = $state('medium');
	let keywords = $state('');
	let result = $state('');
	let loading = $state(false);
	let error = $state('');
	let tokensUsed = $state(0);
	let tokensRemaining = $state(0);
	let aiModel = $state('');
	let showHistory = $state(false);
	let chatHistory = $state<
		Array<{
			id: string;
			topic: string;
			tone: string;
			length: string;
			keywords: string;
			prompt: string;
			response: string;
			timestamp: number;
			tokensUsed: number;
			model: string;
		}>
	>([]);

	const tones = [
		{ id: 'neutral', label: 'Neutral' },
		{ id: 'friendly', label: 'Friendly' },
		{ id: 'professional', label: 'Professional' },
		{ id: 'playful', label: 'Playful' },
		{ id: 'persuasive', label: 'Persuasive' },
		{ id: 'casual', label: 'Casual' }
	] as const;

	const lengths = [
		{ id: 'short', label: 'Short' },
		{ id: 'medium', label: 'Medium' },
		{ id: 'long', label: 'Long' }
	] as const;

	function targetWords() {
		switch (length) {
			case 'short':
				return 70;
			case 'long':
				return 280;
			default:
				return 150;
		}
	}

	// Load chat history from localStorage
	function loadChatHistory() {
		if (typeof window !== 'undefined') {
			try {
				const saved = localStorage.getItem('ai-chat-history');
				if (saved) {
					chatHistory = JSON.parse(saved);
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
				localStorage.setItem('ai-chat-history', JSON.stringify(chatHistory));
			} catch (err) {
				console.error('Failed to save chat history:', err);
			}
		}
	}

	// Add chat to history
	function addToHistory(prompt: string, response: string) {
		const chatItem = {
			id: Date.now().toString(),
			topic,
			tone,
			length,
			keywords,
			prompt,
			response,
			timestamp: Date.now(),
			tokensUsed,
			model: aiModel
		};

		chatHistory = [chatItem, ...chatHistory.slice(0, 49)]; // Keep last 50 chats
		saveChatHistory();
	}

	// Load from history item
	function loadFromHistory(item: (typeof chatHistory)[0]) {
		topic = item.topic;
		tone = item.tone as typeof tone;
		length = item.length as typeof length;
		keywords = item.keywords;
		result = item.response;
		tokensUsed = item.tokensUsed;
		aiModel = item.model;
		error = '';
	}

	// Clear history
	function clearHistory() {
		chatHistory = [];
		saveChatHistory();
	}

	// Format timestamp
	function formatTimestamp(timestamp: number) {
		return new Date(timestamp).toLocaleString();
	}

	// Load history on component mount
	$effect(() => {
		loadChatHistory();
	});

	async function generate() {
		if (!topic.trim()) {
			error = 'Please enter a topic to generate content.';
			return;
		}

		loading = true;
		error = '';

		try {
			// Build prompt based on form inputs
			const keywordsList = keywords
				.split(',')
				.map((s) => s.trim())
				.filter(Boolean);

			let prompt = `Generate ${length} content about "${topic}" with a ${tone} tone.`;

			if (keywordsList.length > 0) {
				prompt += ` Include these keywords: ${keywordsList.join(', ')}.`;
			}

			// Add length guidance
			const wordsTarget = targetWords();
			prompt += ` Target approximately ${wordsTarget} words.`;

			// Call AI API
			const response = await aiChatApi({
				prompt,
				max_tokens: Math.max(wordsTarget * 2, 500),
				temperature: tone === 'playful' ? 0.8 : tone === 'professional' ? 0.3 : 0.5
			});

			if (response.code === 200 && response.data.success) {
				result = response.data.response;
				tokensUsed = response.data.tokens_used;
				tokensRemaining = response.data.tokens_remaining;
				aiModel = response.data.model;

				// Add to chat history
				addToHistory(prompt, response.data.response);
			} else {
				throw new Error(response.message || 'Failed to generate content');
			}
		} catch (err) {
			console.error('AI generation error:', err);
			error = err instanceof Error ? err.message : 'Failed to generate content. Please try again.';
		} finally {
			loading = false;
		}
	}

	function clearAll() {
		topic = '';
		keywords = '';
		result = '';
		error = '';
		tokensUsed = 0;
		tokensRemaining = 0;
		aiModel = '';
	}

	async function copy() {
		if (!result) return;
		const nav = (globalThis as any)?.navigator;
		if (!nav?.clipboard?.writeText) return;
		await nav.clipboard.writeText(result);
	}

	// id untuk a11y
	const idTopic = 'ai-topic';
	const idTone = 'ai-tone';
	const idLength = 'ai-length';
	const idKeywords = 'ai-keywords';
</script>

<div class="grid gap-6 lg:grid-cols-2">
	<!-- left: controls -->
	<div
		class="rounded-[var(--radius-card)] border border-[var(--ui-border)] bg-[var(--color-surface)] p-5"
	>
		<div class="grid gap-4">
			<div>
				<label class="mb-1 block text-sm" for={idTopic}>Topic</label>
				<input
					id={idTopic}
					class="form-input"
					bind:value={topic}
					placeholder="e.g. Launch announcement"
				/>
			</div>

			<div class="grid gap-4 sm:grid-cols-2">
				<div>
					<label class="mb-1 block text-sm" for={idTone}>Tone</label>
					<select id={idTone} class="form-input" bind:value={tone} aria-label="tone">
						{#each tones as t}
							<option value={t.id}>{t.label}</option>
						{/each}
					</select>
				</div>

				<div>
					<label class="mb-1 block text-sm" for={idLength}>Length</label>
					<select id={idLength} class="form-input" bind:value={length} aria-label="length">
						{#each lengths as l}
							<option value={l.id}>{l.label}</option>
						{/each}
					</select>
				</div>
			</div>

			<div>
				<label class="mb-1 block text-sm" for={idKeywords}>Keywords (comma separated)</label>
				<input
					id={idKeywords}
					class="form-input"
					bind:value={keywords}
					placeholder="e.g. fast, secure, open-source"
				/>
			</div>

			<div class="flex flex-wrap items-center gap-3 pt-2">
				<Button
					label={loading ? 'Generating…' : 'Generate'}
					color="accent"
					onClick={generate}
					disabled={loading || !topic.trim()}
				/>
				<button
					type="button"
					class="inline-flex items-center justify-center rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-4 py-2.5 text-sm hover:bg-[color-mix(in_oklch,var(--color-surface),black_3%)]"
					onclick={clearAll}
				>
					Clear
				</button>
				<button
					type="button"
					class="inline-flex items-center justify-center rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-4 py-2.5 text-sm hover:bg-[color-mix(in_oklch,var(--color-surface),black_3%)]"
					onclick={() => (showHistory = !showHistory)}
				>
					{showHistory ? 'Hide' : 'Show'} History ({chatHistory.length})
				</button>
			</div>

			<!-- Chat History -->
			{#if showHistory}
				<div class="mt-4 border-t border-[var(--ui-border)] pt-4">
					<div class="mb-3 flex items-center justify-between">
						<h3 class="text-sm font-medium">Chat History</h3>
						{#if chatHistory.length > 0}
							<button
								type="button"
								class="text-xs text-red-600 hover:text-red-800 dark:text-red-400"
								onclick={clearHistory}
							>
								Clear All
							</button>
						{/if}
					</div>

					{#if chatHistory.length === 0}
						<p class="text-sm opacity-60">No chat history yet.</p>
					{:else}
						<div class="max-h-60 space-y-2 overflow-y-auto">
							{#each chatHistory as item}
								<button
									type="button"
									class="w-full cursor-pointer rounded-lg border border-[var(--ui-border)] p-3 text-left hover:bg-[color-mix(in_oklch,var(--color-surface),black_2%)]"
									onclick={() => loadFromHistory(item)}
									onkeydown={(e) => e.key === 'Enter' && loadFromHistory(item)}
									tabindex="0"
								>
									<div class="truncate text-sm font-medium">{item.topic}</div>
									<div class="mt-1 text-xs opacity-60">
										{item.tone} • {item.length} • {formatTimestamp(item.timestamp)}
									</div>
									{#if item.tokensUsed > 0}
										<div class="mt-1 text-xs opacity-50">
											{item.tokensUsed} tokens • {item.model}
										</div>
									{/if}
								</button>
							{/each}
						</div>
					{/if}
				</div>
			{/if}
		</div>
	</div>

	<!-- right: output -->
	<div
		class="rounded-[var(--radius-card)] border border-[var(--ui-border)] bg-[var(--color-surface)] p-5"
	>
		<div class="mb-3 flex items-center justify-between gap-3">
			<div class="text-sm opacity-70">
				Output
				{#if aiModel}
					<span class="ml-2 text-xs opacity-50">({aiModel})</span>
				{/if}
			</div>
			<div class="flex items-center gap-2">
				<button
					type="button"
					class="inline-flex items-center justify-center rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-4 py-2.5 text-sm hover:bg-[color-mix(in_oklch,var(--color-surface),black_3%)] disabled:opacity-50"
					onclick={copy}
					disabled={!result}
				>
					Copy
				</button>
				<button
					type="button"
					class="inline-flex items-center justify-center rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-4 py-2.5 text-sm hover:bg-[color-mix(in_oklch,var(--color-surface),black_3%)] disabled:opacity-50"
					onclick={generate}
					disabled={loading || !topic.trim()}
				>
					Regenerate
				</button>
			</div>
		</div>

		<!-- Token Usage Info -->
		{#if tokensUsed > 0}
			<div class="mb-3 text-xs opacity-60">
				Tokens used: {tokensUsed.toLocaleString()} | Remaining: {tokensRemaining.toLocaleString()}
			</div>
		{/if}

		<!-- Error Display -->
		{#if error}
			<div
				class="mb-3 rounded-lg border border-red-200 bg-red-50 p-3 text-sm text-red-700 dark:border-red-800 dark:bg-red-900/20 dark:text-red-300"
			>
				{error}
			</div>
		{/if}

		<div
			class="min-h-[260px] rounded-xl border border-[var(--ui-border)] bg-transparent p-4 text-[.95rem] leading-relaxed whitespace-pre-wrap"
		>
			{#if result}
				{@html result.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>').replace(/\n/g, '<br>')}
			{:else}
				<p class="opacity-60">
					Your AI-generated text will appear here. Enter a topic and click
					<strong>Generate</strong> to start.
				</p>
			{/if}
		</div>
	</div>
</div>
