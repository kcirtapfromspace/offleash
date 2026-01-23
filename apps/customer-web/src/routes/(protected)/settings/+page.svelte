<script lang="ts">
	// Settings hub page - links to various settings sections
	let { data } = $props();

	// Check if user has any walker/owner/admin memberships
	const hasWalkerBusiness = $derived(
		data.memberships?.some(
			(m: { role: string }) => m.role === 'walker' || m.role === 'owner' || m.role === 'admin'
		) ?? false
	);
</script>

<div class="max-w-2xl mx-auto p-4">
	<h1 class="text-2xl font-bold mb-6">Settings</h1>

	<div class="bg-white rounded-lg shadow divide-y">
		<!-- Connected Accounts -->
		<a href="/settings/connected-accounts" class="block p-4 hover:bg-gray-50 transition-colors">
			<div class="flex items-center justify-between">
				<div class="flex items-center gap-3">
					<div class="w-10 h-10 rounded-full bg-blue-100 flex items-center justify-center text-blue-600">
						<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
							<path stroke-linecap="round" stroke-linejoin="round" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
						</svg>
					</div>
					<div>
						<p class="font-medium">Connected Accounts</p>
						<p class="text-sm text-gray-500">Manage your sign-in methods</p>
					</div>
				</div>
				<svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
				</svg>
			</div>
		</a>

		<!-- Profile (links to existing profile page if any) -->
		<a href="/profile" class="block p-4 hover:bg-gray-50 transition-colors">
			<div class="flex items-center justify-between">
				<div class="flex items-center gap-3">
					<div class="w-10 h-10 rounded-full bg-green-100 flex items-center justify-center text-green-600">
						<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
							<path stroke-linecap="round" stroke-linejoin="round" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
						</svg>
					</div>
					<div>
						<p class="font-medium">Profile</p>
						<p class="text-sm text-gray-500">Edit your personal information</p>
					</div>
				</div>
				<svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
				</svg>
			</div>
		</a>

		<!-- Payment Methods -->
		<a href="/payment-methods" class="block p-4 hover:bg-gray-50 transition-colors">
			<div class="flex items-center justify-between">
				<div class="flex items-center gap-3">
					<div class="w-10 h-10 rounded-full bg-purple-100 flex items-center justify-center text-purple-600">
						<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
							<path stroke-linecap="round" stroke-linejoin="round" d="M3 10h18M7 15h1m4 0h1m-7 4h12a3 3 0 003-3V8a3 3 0 00-3-3H6a3 3 0 00-3 3v8a3 3 0 003 3z" />
						</svg>
					</div>
					<div>
						<p class="font-medium">Payment Methods</p>
						<p class="text-sm text-gray-500">Manage your payment cards</p>
					</div>
				</div>
				<svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
				</svg>
			</div>
		</a>

		<!-- Saved Locations -->
		<a href="/locations" class="block p-4 hover:bg-gray-50 transition-colors">
			<div class="flex items-center justify-between">
				<div class="flex items-center gap-3">
					<div class="w-10 h-10 rounded-full bg-yellow-100 flex items-center justify-center text-yellow-600">
						<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
							<path stroke-linecap="round" stroke-linejoin="round" d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z" />
							<path stroke-linecap="round" stroke-linejoin="round" d="M15 11a3 3 0 11-6 0 3 3 0 016 0z" />
						</svg>
					</div>
					<div>
						<p class="font-medium">Saved Locations</p>
						<p class="text-sm text-gray-500">Manage your addresses</p>
					</div>
				</div>
				<svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
				</svg>
			</div>
		</a>
	</div>

	<!-- Walker Business Section -->
	<h2 class="text-lg font-semibold mt-8 mb-4 text-gray-700">Walker Business</h2>
	<div class="bg-white rounded-lg shadow divide-y">
		{#if hasWalkerBusiness}
			<!-- Already has a business -->
			<a href="/onboarding/walker" class="block p-4 hover:bg-gray-50 transition-colors">
				<div class="flex items-center justify-between">
					<div class="flex items-center gap-3">
						<div class="w-10 h-10 rounded-full bg-green-100 flex items-center justify-center text-green-600">
							<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
								<path stroke-linecap="round" stroke-linejoin="round" d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4" />
							</svg>
						</div>
						<div>
							<p class="font-medium">Manage Businesses</p>
							<p class="text-sm text-gray-500">Create or join another business</p>
						</div>
					</div>
					<svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
					</svg>
				</div>
			</a>
		{:else}
			<!-- No business yet -->
			<a href="/onboarding/walker" class="block p-4 hover:bg-gray-50 transition-colors">
				<div class="flex items-center justify-between">
					<div class="flex items-center gap-3">
						<div class="w-10 h-10 rounded-full bg-blue-100 flex items-center justify-center text-blue-600">
							<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
								<path stroke-linecap="round" stroke-linejoin="round" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
							</svg>
						</div>
						<div>
							<p class="font-medium">Become a Walker</p>
							<p class="text-sm text-gray-500">Start or join a dog walking business</p>
						</div>
					</div>
					<svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
					</svg>
				</div>
			</a>
		{/if}
	</div>
</div>
