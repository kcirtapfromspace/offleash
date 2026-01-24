<script lang="ts">
	import { goto } from '$app/navigation';
	import { enhance } from '$app/forms';
	import { page } from '$app/state';

	let businessName = $state('');
	let isLoading = $state(false);
	let error = $state('');

	// Check if user came from settings
	const fromSettings = $derived(page.url.searchParams.get('from') === 'settings');
	const backUrl = $derived(fromSettings ? '/onboarding/walker?from=settings' : '/onboarding/walker');

	const slug = $derived(
		businessName
			.toLowerCase()
			.replace(/\s+/g, '-')
			.replace(/[^a-z0-9-]/g, '')
	);

	const isFormValid = $derived(businessName.trim().length >= 3);

	async function handleSubmit(event: Event) {
		event.preventDefault();
		if (!isFormValid || isLoading) return;

		isLoading = true;
		error = '';

		try {
			const response = await fetch('/api/walker/create-tenant', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					business_name: businessName.trim(),
					slug: slug
				})
			});

			const data = await response.json();

			if (response.ok && data.success) {
				// Redirect to the admin dashboard for the new business
				// Use the appropriate domain based on environment
				const adminUrl = window.location.hostname.includes('localhost')
					? '/services' // Local dev - stay on same domain for now
					: `https://paperwork.offleash.world`;

				window.location.href = adminUrl;
			} else {
				error = data.error || 'Failed to create business';
			}
		} catch (e) {
			error = 'An unexpected error occurred. Please try again.';
		} finally {
			isLoading = false;
		}
	}
</script>

<svelte:head>
	<title>Create Your Business - OFFLEASH</title>
</svelte:head>

<div class="min-h-screen bg-gray-50 pb-safe">
	<div class="max-w-md mx-auto px-4 py-8 pb-24">
		<!-- Back Button -->
		<a
			href={backUrl}
			class="inline-flex items-center gap-1 text-sm mb-8 hover:underline"
			style="color: var(--color-primary, #3b82f6)"
		>
			<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
			</svg>
			Back
		</a>

		<!-- Header -->
		<div class="text-center mb-8">
			<div class="w-16 h-16 mx-auto mb-4 rounded-full bg-blue-100 flex items-center justify-center">
				<svg class="w-8 h-8 text-blue-600" fill="currentColor" viewBox="0 0 24 24">
					<path d="M20 4H4v2h16V4zm1 10v-2l-1-5H4l-1 5v2h1v6h10v-6h4v6h2v-6h1zm-9 4H6v-4h6v4z"/>
				</svg>
			</div>
			<h1 class="text-2xl font-bold text-gray-900">Create Your Business</h1>
			<p class="text-gray-500 mt-2">Set up your dog walking business in just a few steps</p>
		</div>

		<!-- Form -->
		<form onsubmit={handleSubmit} class="space-y-6">
			<!-- Business Name -->
			<div>
				<label for="businessName" class="block text-sm font-medium text-gray-700 mb-2">
					Business Name
				</label>
				<input
					type="text"
					id="businessName"
					bind:value={businessName}
					placeholder="e.g., Happy Paws Dog Walking"
					class="w-full px-4 py-3 border border-gray-300 rounded-xl focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
				/>
				{#if businessName}
					<p class="mt-2 text-sm text-gray-500">
						Your URL: offleash.app/<span class="font-medium">{slug}</span>
					</p>
				{/if}
			</div>

			<!-- Info Box -->
			<div class="p-4 rounded-xl" style="background-color: rgba(16, 185, 129, 0.1)">
				<div class="flex gap-3">
					<svg class="w-5 h-5 flex-shrink-0 mt-0.5" style="color: var(--color-accent, #10b981)" fill="currentColor" viewBox="0 0 24 24">
						<path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z"/>
					</svg>
					<div>
						<p class="text-sm font-semibold text-gray-900 mb-2">What you'll get:</p>
						<ul class="text-sm text-gray-600 space-y-1">
							<li class="flex items-center gap-2">
								<span class="text-gray-400">•</span>
								Your own booking page
							</li>
							<li class="flex items-center gap-2">
								<span class="text-gray-400">•</span>
								Client management tools
							</li>
							<li class="flex items-center gap-2">
								<span class="text-gray-400">•</span>
								Schedule & availability settings
							</li>
							<li class="flex items-center gap-2">
								<span class="text-gray-400">•</span>
								Payment processing
							</li>
						</ul>
					</div>
				</div>
			</div>

			<!-- Error Message -->
			{#if error}
				<div class="p-4 bg-red-50 text-red-700 rounded-xl text-sm">
					{error}
				</div>
			{/if}

			<!-- Submit Button -->
			<button
				type="submit"
				disabled={!isFormValid || isLoading}
				class="w-full py-3 px-4 rounded-xl font-semibold text-white transition-colors disabled:cursor-not-allowed border-2"
				style="background-color: {isFormValid ? 'var(--color-primary, #3b82f6)' : '#d1d5db'}; border-color: {isFormValid ? 'var(--color-primary, #3b82f6)' : '#9ca3af'};"
			>
				{#if isLoading}
					<span class="flex items-center justify-center gap-2">
						<svg class="animate-spin w-5 h-5" fill="none" viewBox="0 0 24 24">
							<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
							<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
						</svg>
						Creating...
					</span>
				{:else}
					Create Business
				{/if}
			</button>
		</form>
	</div>
</div>
