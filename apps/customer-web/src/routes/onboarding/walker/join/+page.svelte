<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';

	let isLoading = $state(false);
	let error = $state('');
	let successMessage = $state('');
	let joinedTenantName = $state('');

	// Check for invite token in URL
	const inviteToken = $derived(page.url.searchParams.get('token'));

	$effect(() => {
		if (inviteToken) {
			joinTenant(inviteToken);
		}
	});

	async function joinTenant(token: string) {
		isLoading = true;
		error = '';

		try {
			const response = await fetch('/api/walker/join-tenant', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ inviteToken: token })
			});

			const data = await response.json();

			if (response.ok && data.success) {
				joinedTenantName = data.tenantName || 'the business';
				successMessage = `You've successfully joined ${joinedTenantName}. You can now start accepting bookings.`;
			} else {
				error = data.error || data.message || 'This invitation is invalid or has expired.';
			}
		} catch (e) {
			error = 'An unexpected error occurred. Please try again.';
		} finally {
			isLoading = false;
		}
	}

	function getStarted() {
		goto('/services');
	}
</script>

<svelte:head>
	<title>Join a Business - OFFLEASH</title>
</svelte:head>

<div class="min-h-screen bg-gray-50">
	<div class="max-w-md mx-auto px-4 py-8">
		<!-- Back Button -->
		<a
			href="/onboarding/walker"
			class="inline-flex items-center gap-1 text-sm mb-8 hover:underline"
			style="color: var(--color-primary, #3b82f6)"
		>
			<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
			</svg>
			Back
		</a>

		{#if isLoading}
			<!-- Loading State -->
			<div class="text-center py-16">
				<svg class="animate-spin w-12 h-12 mx-auto mb-4 text-blue-600" fill="none" viewBox="0 0 24 24">
					<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
					<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
				</svg>
				<h2 class="text-xl font-semibold text-gray-900">Joining Business...</h2>
				<p class="text-gray-500 mt-2">Please wait while we add you to the team.</p>
			</div>
		{:else if successMessage}
			<!-- Success State -->
			<div class="text-center py-16">
				<div class="w-16 h-16 mx-auto mb-4 rounded-full bg-green-100 flex items-center justify-center">
					<svg class="w-8 h-8 text-green-600" fill="currentColor" viewBox="0 0 24 24">
						<path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41L9 16.17z"/>
					</svg>
				</div>
				<h2 class="text-xl font-semibold text-gray-900">Welcome!</h2>
				<p class="text-gray-600 mt-2">{successMessage}</p>
				<button
					onclick={getStarted}
					class="mt-6 px-6 py-3 rounded-xl font-semibold text-white"
					style="background-color: var(--color-primary, #3b82f6)"
				>
					Get Started
				</button>
			</div>
		{:else if error}
			<!-- Error State -->
			<div class="text-center py-16">
				<div class="w-16 h-16 mx-auto mb-4 rounded-full bg-red-100 flex items-center justify-center">
					<svg class="w-8 h-8 text-red-600" fill="currentColor" viewBox="0 0 24 24">
						<path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
					</svg>
				</div>
				<h2 class="text-xl font-semibold text-gray-900">Something went wrong</h2>
				<p class="text-gray-600 mt-2">{error}</p>
				<a
					href="/onboarding/walker"
					class="mt-6 inline-block px-6 py-3 rounded-xl font-semibold text-white"
					style="background-color: var(--color-primary, #3b82f6)"
				>
					Go Back
				</a>
			</div>
		{:else}
			<!-- No Token State - Show Instructions -->
			<div class="text-center">
				<div class="w-16 h-16 mx-auto mb-4 rounded-full bg-blue-100 flex items-center justify-center">
					<svg class="w-8 h-8 text-blue-600" fill="currentColor" viewBox="0 0 24 24">
						<path d="M20 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4l-8 5-8-5V6l8 5 8-5v2z"/>
					</svg>
				</div>
				<h1 class="text-2xl font-bold text-gray-900">Need an Invitation</h1>
				<p class="text-gray-500 mt-2 mb-8">To join a business, you'll need an invitation from the business owner.</p>

				<!-- Instructions -->
				<div class="bg-gray-100 rounded-xl p-6 text-left space-y-4">
					<div class="flex items-start gap-4">
						<div class="w-6 h-6 rounded-full bg-blue-600 text-white text-sm font-bold flex items-center justify-center flex-shrink-0">
							1
						</div>
						<p class="text-sm text-gray-700">Ask the business owner to send you an invite</p>
					</div>
					<div class="flex items-start gap-4">
						<div class="w-6 h-6 rounded-full bg-blue-600 text-white text-sm font-bold flex items-center justify-center flex-shrink-0">
							2
						</div>
						<p class="text-sm text-gray-700">Check your email or text messages for the invitation link</p>
					</div>
					<div class="flex items-start gap-4">
						<div class="w-6 h-6 rounded-full bg-blue-600 text-white text-sm font-bold flex items-center justify-center flex-shrink-0">
							3
						</div>
						<p class="text-sm text-gray-700">Tap the link to automatically join the team</p>
					</div>
				</div>

				<div class="mt-6 flex items-center justify-center gap-2 text-sm text-gray-500">
					<svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
						<path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z"/>
					</svg>
					Invitation links expire after 7 days
				</div>
			</div>
		{/if}
	</div>
</div>
