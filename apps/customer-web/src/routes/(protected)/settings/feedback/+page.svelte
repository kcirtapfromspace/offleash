<script lang="ts">
	import { enhance } from '$app/forms';

	let { data, form } = $props();

	let feedbackType = $state<'bug' | 'feature'>('bug');
	let title = $state('');
	let description = $state('');
	let isSubmitting = $state(false);

	const isFormValid = $derived(title.trim().length >= 5 && description.trim().length >= 20);
</script>

<svelte:head>
	<title>Send Feedback - OFFLEASH</title>
</svelte:head>

<div class="max-w-2xl mx-auto p-4">
	<a
		href="/settings"
		class="inline-flex items-center gap-1 text-sm mb-6 hover:underline"
		style="color: var(--color-primary, #3b82f6)"
	>
		<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
			<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
		</svg>
		Back to Settings
	</a>

	<h1 class="text-2xl font-bold mb-2">Send Feedback</h1>
	<p class="text-gray-500 mb-6">Help us improve OFFLEASH by reporting bugs or suggesting new features.</p>

	{#if form?.success}
		<div class="bg-green-50 border border-green-200 rounded-xl p-6 text-center">
			<div class="w-12 h-12 mx-auto mb-4 rounded-full bg-green-100 flex items-center justify-center">
				<svg class="w-6 h-6 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
				</svg>
			</div>
			<h2 class="text-lg font-semibold text-green-800 mb-2">Thank you for your feedback!</h2>
			<p class="text-green-700 mb-4">We've received your {form.feedbackType === 'bug' ? 'bug report' : 'feature request'} and will review it soon.</p>
			{#if form.issueUrl}
				<a
					href={form.issueUrl}
					target="_blank"
					rel="noopener noreferrer"
					class="inline-flex items-center gap-2 text-sm text-green-600 hover:underline"
				>
					<svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
						<path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
					</svg>
					View on GitHub
				</a>
			{/if}
			<div class="mt-4">
				<a
					href="/settings/feedback"
					class="text-sm text-gray-600 hover:underline"
				>
					Submit another
				</a>
			</div>
		</div>
	{:else}
		<form
			method="POST"
			use:enhance={() => {
				isSubmitting = true;
				return async ({ update }) => {
					await update();
					isSubmitting = false;
				};
			}}
			class="space-y-6"
		>
			<!-- Feedback Type -->
			<div>
				<label class="block text-sm font-medium text-gray-700 mb-3">What type of feedback?</label>
				<div class="grid grid-cols-2 gap-3">
					<button
						type="button"
						class="p-4 rounded-xl border-2 transition-all text-left {feedbackType === 'bug'
							? 'border-red-500 bg-red-50'
							: 'border-gray-200 hover:border-gray-300'}"
						onclick={() => (feedbackType = 'bug')}
					>
						<div class="flex items-center gap-3">
							<div class="w-10 h-10 rounded-full {feedbackType === 'bug' ? 'bg-red-100' : 'bg-gray-100'} flex items-center justify-center">
								<svg class="w-5 h-5 {feedbackType === 'bug' ? 'text-red-600' : 'text-gray-500'}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
								</svg>
							</div>
							<div>
								<p class="font-medium {feedbackType === 'bug' ? 'text-red-900' : 'text-gray-900'}">Bug Report</p>
								<p class="text-xs {feedbackType === 'bug' ? 'text-red-600' : 'text-gray-500'}">Something isn't working</p>
							</div>
						</div>
					</button>

					<button
						type="button"
						class="p-4 rounded-xl border-2 transition-all text-left {feedbackType === 'feature'
							? 'border-blue-500 bg-blue-50'
							: 'border-gray-200 hover:border-gray-300'}"
						onclick={() => (feedbackType = 'feature')}
					>
						<div class="flex items-center gap-3">
							<div class="w-10 h-10 rounded-full {feedbackType === 'feature' ? 'bg-blue-100' : 'bg-gray-100'} flex items-center justify-center">
								<svg class="w-5 h-5 {feedbackType === 'feature' ? 'text-blue-600' : 'text-gray-500'}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
								</svg>
							</div>
							<div>
								<p class="font-medium {feedbackType === 'feature' ? 'text-blue-900' : 'text-gray-900'}">Feature Request</p>
								<p class="text-xs {feedbackType === 'feature' ? 'text-blue-600' : 'text-gray-500'}">Suggest an improvement</p>
							</div>
						</div>
					</button>
				</div>
				<input type="hidden" name="feedbackType" value={feedbackType} />
			</div>

			<!-- Title -->
			<div>
				<label for="title" class="block text-sm font-medium text-gray-700 mb-2">
					{feedbackType === 'bug' ? 'What went wrong?' : 'What would you like to see?'}
				</label>
				<input
					type="text"
					id="title"
					name="title"
					bind:value={title}
					placeholder={feedbackType === 'bug' ? 'e.g., Cannot save payment method' : 'e.g., Add dark mode support'}
					class="w-full px-4 py-3 border border-gray-300 rounded-xl focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
					required
					minlength="5"
				/>
				<p class="mt-1 text-xs text-gray-500">Minimum 5 characters</p>
			</div>

			<!-- Description -->
			<div>
				<label for="description" class="block text-sm font-medium text-gray-700 mb-2">
					{feedbackType === 'bug' ? 'Please describe the issue in detail' : 'Tell us more about your idea'}
				</label>
				<textarea
					id="description"
					name="description"
					bind:value={description}
					rows="5"
					placeholder={feedbackType === 'bug'
						? 'What were you trying to do? What happened instead? Any error messages?'
						: 'How would this feature help you? Any specific details about how it should work?'}
					class="w-full px-4 py-3 border border-gray-300 rounded-xl focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none resize-none"
					required
					minlength="20"
				></textarea>
				<p class="mt-1 text-xs text-gray-500">Minimum 20 characters ({description.length}/20)</p>
			</div>

			<!-- Error Message -->
			{#if form?.error}
				<div class="p-4 bg-red-50 text-red-700 rounded-xl text-sm">
					{form.error}
				</div>
			{/if}

			<!-- Submit Button -->
			<button
				type="submit"
				disabled={!isFormValid || isSubmitting}
				class="w-full py-3 px-4 rounded-xl font-semibold text-white transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
				style="background-color: {isFormValid ? 'var(--color-primary, #3b82f6)' : '#9ca3af'}"
			>
				{#if isSubmitting}
					<span class="flex items-center justify-center gap-2">
						<svg class="animate-spin w-5 h-5" fill="none" viewBox="0 0 24 24">
							<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
							<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
						</svg>
						Submitting...
					</span>
				{:else}
					Submit Feedback
				{/if}
			</button>

			<!-- Privacy Note -->
			<p class="text-xs text-gray-500 text-center">
				Your feedback will be submitted to our public issue tracker. Your email ({data.user?.email}) will be included for follow-up.
			</p>
		</form>
	{/if}
</div>
