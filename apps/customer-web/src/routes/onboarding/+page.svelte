<script lang="ts">
	import { goto } from '$app/navigation';
	import { env } from '$env/dynamic/public';

	function selectRole(role: 'customer' | 'walker') {
		// Redirect to login/register with role parameter
		// After auth, they'll be routed to the appropriate flow
		if (role === 'customer') {
			goto('/login?role=customer&redirect=/services');
		} else {
			// Staff auth happens on this domain (OAuth registered here), then redirects to admin app
			const adminUrl = env.PUBLIC_ADMIN_URL || 'https://paperwork.offleash.world';
			goto(`/login?app=admin&redirect=${encodeURIComponent(adminUrl + '/dashboard')}`);
		}
	}
</script>

<svelte:head>
	<title>Get Started - OFFLEASH</title>
</svelte:head>

<div class="min-h-screen bg-gray-50 flex flex-col">
	<div class="flex-1 flex flex-col items-center justify-center px-4 py-12">
		<!-- Logo and App Name -->
		<div class="text-center mb-12">
			<div class="w-20 h-20 mx-auto mb-4 rounded-full bg-blue-100 flex items-center justify-center">
				<svg class="w-12 h-12 text-blue-600" fill="currentColor" viewBox="0 0 24 24">
					<path d="M4.5 11.5c.28 0 .5-.34.5-.75v-1c0-.41-.22-.75-.5-.75s-.5.34-.5.75v1c0 .41.22.75.5.75zm3 0c.28 0 .5-.34.5-.75v-1c0-.41-.22-.75-.5-.75s-.5.34-.5.75v1c0 .41.22.75.5.75zm9 0c.28 0 .5-.34.5-.75v-1c0-.41-.22-.75-.5-.75s-.5.34-.5.75v1c0 .41.22.75.5.75zm3 0c.28 0 .5-.34.5-.75v-1c0-.41-.22-.75-.5-.75s-.5.34-.5.75v1c0 .41.22.75.5.75zM12 8c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0-6c1.1 0 2 .9 2 2s-.9 2-2 2-2-.9-2-2 .9-2 2-2zm0 8c-4.42 0-8 1.79-8 4v2h16v-2c0-2.21-3.58-4-8-4z"/>
				</svg>
			</div>
			<h1 class="text-3xl font-bold" style="color: var(--color-primary, #3b82f6)">OFFLEASH</h1>
			<p class="text-gray-500 mt-2">Premium pet care, on demand</p>
		</div>

		<!-- Role Selection -->
		<div class="w-full max-w-md space-y-4">
			<h2 class="text-lg font-semibold text-gray-900 text-center mb-6">How would you like to use the app?</h2>

			<!-- Customer Card -->
			<button
				type="button"
				class="w-full p-4 bg-white rounded-2xl shadow-sm border border-gray-100 hover:shadow-md hover:border-gray-200 transition-all text-left flex items-center gap-4"
				onclick={() => selectRole('customer')}
			>
				<div class="w-14 h-14 rounded-full bg-blue-50 flex items-center justify-center flex-shrink-0">
					<svg class="w-7 h-7 text-blue-600" fill="currentColor" viewBox="0 0 24 24">
						<path d="M18 4c-1.1 0-2 .9-2 2h-2c0-1.1-.9-2-2-2s-2 .9-2 2H8c0-1.1-.9-2-2-2s-2 .9-2 2H2v14h20V6h-2c0-1.1-.9-2-2-2zm0 2c0-.55.45-1 1-1s1 .45 1 1-.45 1-1 1-1-.45-1-1zM6 7c-.55 0-1-.45-1-1s.45-1 1-1 1 .45 1 1-.45 1-1 1zm6-1c-.55 0-1-.45-1-1s.45-1 1-1 1 .45 1 1-.45 1-1 1z"/>
					</svg>
				</div>
				<div class="flex-1 min-w-0">
					<h3 class="font-semibold text-gray-900">My dogs need a walk</h3>
					<p class="text-sm text-gray-500">Book walks, daycare, and more for your pets</p>
				</div>
				<svg class="w-5 h-5 text-gray-300 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
				</svg>
			</button>

			<!-- Walker Card -->
			<button
				type="button"
				class="w-full p-4 bg-white rounded-2xl shadow-sm border border-gray-100 hover:shadow-md hover:border-gray-200 transition-all text-left flex items-center gap-4"
				onclick={() => selectRole('walker')}
			>
				<div class="w-14 h-14 rounded-full bg-green-50 flex items-center justify-center flex-shrink-0">
					<svg class="w-7 h-7 text-green-600" fill="currentColor" viewBox="0 0 24 24">
						<path d="M13.5 5.5c1.1 0 2-.9 2-2s-.9-2-2-2-2 .9-2 2 .9 2 2 2zM9.8 8.9L7 23h2.1l1.8-8 2.1 2v6h2v-7.5l-2.1-2 .6-3C14.8 12 16.8 13 19 13v-2c-1.9 0-3.5-1-4.3-2.4l-1-1.6c-.4-.6-1-1-1.7-1-.3 0-.5.1-.8.1L6 8.3V13h2V9.6l1.8-.7"/>
					</svg>
				</div>
				<div class="flex-1 min-w-0">
					<h3 class="font-semibold text-gray-900">I walk dogs</h3>
					<p class="text-sm text-gray-500">Manage your schedule and bookings</p>
				</div>
				<svg class="w-5 h-5 text-gray-300 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
				</svg>
			</button>
		</div>

		<!-- Already have account -->
		<div class="mt-8 text-center">
			<p class="text-sm text-gray-500">
				Already have an account?
				<a href="/login" class="font-medium hover:underline" style="color: var(--color-primary, #3b82f6)">
					Sign in
				</a>
			</p>
		</div>
	</div>

	<!-- Footer -->
	<div class="text-center py-6">
		<p class="text-sm text-gray-400">Need help?</p>
		<a href="mailto:support@offleash.app" class="text-sm" style="color: var(--color-accent, #10b981)">support@offleash.app</a>
	</div>
</div>
