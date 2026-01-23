<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';

	// Check if user came from settings
	const fromSettings = $derived(page.url.searchParams.get('from') === 'settings');
	const backUrl = $derived(fromSettings ? '/settings' : '/onboarding');
	const backLabel = $derived(fromSettings ? 'Back to settings' : 'Back to role selection');
</script>

<svelte:head>
	<title>Walker Setup - OFFLEASH</title>
</svelte:head>

<div class="min-h-screen bg-gray-50 flex flex-col">
	<div class="flex-1 flex flex-col items-center justify-center px-4 py-12">
		<!-- Header -->
		<div class="text-center mb-12">
			<div class="w-16 h-16 mx-auto mb-4 rounded-full bg-blue-100 flex items-center justify-center">
				<svg class="w-8 h-8 text-blue-600" fill="currentColor" viewBox="0 0 24 24">
					<path d="M12 7V3H2v18h20V7H12zM6 19H4v-2h2v2zm0-4H4v-2h2v2zm0-4H4V9h2v2zm0-4H4V5h2v2zm4 12H8v-2h2v2zm0-4H8v-2h2v2zm0-4H8V9h2v2zm0-4H8V5h2v2zm10 12h-8v-2h2v-2h-2v-2h2v-2h-2V9h8v10zm-2-8h-2v2h2v-2zm0 4h-2v2h2v-2z"/>
				</svg>
			</div>
			<h1 class="text-3xl font-bold text-gray-900">Welcome, Walker!</h1>
			<p class="text-gray-500 mt-2">How would you like to get started?</p>
		</div>

		<!-- Options -->
		<div class="w-full max-w-md space-y-4">
			<!-- Create Business Card -->
			<button
				type="button"
				class="w-full p-4 bg-white rounded-2xl shadow-sm border border-gray-100 hover:shadow-md hover:border-gray-200 transition-all text-left flex items-center gap-4"
				onclick={() => goto(fromSettings ? '/onboarding/walker/create?from=settings' : '/onboarding/walker/create')}
			>
				<div class="w-14 h-14 rounded-full bg-blue-50 flex items-center justify-center flex-shrink-0">
					<svg class="w-7 h-7 text-blue-600" fill="currentColor" viewBox="0 0 24 24">
						<path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
					</svg>
				</div>
				<div class="flex-1 min-w-0">
					<h3 class="font-semibold text-gray-900">Start my own business</h3>
					<p class="text-sm text-gray-500">Create your dog walking business and manage your own clients</p>
				</div>
				<svg class="w-5 h-5 text-gray-300 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
				</svg>
			</button>

			<!-- Join Business Card -->
			<button
				type="button"
				class="w-full p-4 bg-white rounded-2xl shadow-sm border border-gray-100 hover:shadow-md hover:border-gray-200 transition-all text-left flex items-center gap-4"
				onclick={() => goto(fromSettings ? '/onboarding/walker/join?from=settings' : '/onboarding/walker/join')}
			>
				<div class="w-14 h-14 rounded-full bg-green-50 flex items-center justify-center flex-shrink-0">
					<svg class="w-7 h-7 text-green-600" fill="currentColor" viewBox="0 0 24 24">
						<path d="M15 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm-9-2V7H4v3H1v2h3v3h2v-3h3v-2H6zm9 4c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z"/>
					</svg>
				</div>
				<div class="flex-1 min-w-0">
					<h3 class="font-semibold text-gray-900">Join an existing business</h3>
					<p class="text-sm text-gray-500">Work with an established dog walking company</p>
				</div>
				<svg class="w-5 h-5 text-gray-300 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
				</svg>
			</button>
		</div>

		<!-- Back link -->
		<a
			href={backUrl}
			class="mt-8 text-sm flex items-center gap-1 hover:underline"
			style="color: var(--color-primary, #3b82f6)"
		>
			<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
			</svg>
			{backLabel}
		</a>
	</div>

	<!-- Footer -->
	<div class="text-center py-6">
		<p class="text-sm text-gray-400">You can change this later in settings</p>
	</div>
</div>
