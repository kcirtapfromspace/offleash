<script lang="ts">
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	// Check if user is admin/owner (can access branding settings)
	const isAdmin = $derived(data.membership?.role === 'admin' || data.membership?.role === 'owner');
	// Check if user is owner (can delete organization)
	const isOwner = $derived(data.membership?.role === 'owner');
</script>

<div class="p-6 max-w-2xl">
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

		<!-- Branding (for tenant admins only) -->
		{#if isAdmin}
			<a href="/settings/branding" class="block p-4 hover:bg-gray-50 transition-colors">
				<div class="flex items-center justify-between">
					<div class="flex items-center gap-3">
						<div class="w-10 h-10 rounded-full bg-purple-100 flex items-center justify-center text-purple-600">
							<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
								<path stroke-linecap="round" stroke-linejoin="round" d="M7 21a4 4 0 01-4-4V5a2 2 0 012-2h4a2 2 0 012 2v12a4 4 0 01-4 4zm0 0h12a2 2 0 002-2v-4a2 2 0 00-2-2h-2.343M11 7.343l1.657-1.657a2 2 0 012.828 0l2.829 2.829a2 2 0 010 2.828l-8.486 8.485M7 17h.01" />
							</svg>
						</div>
						<div>
							<p class="font-medium">Branding</p>
							<p class="text-sm text-gray-500">Customize your business appearance</p>
						</div>
					</div>
					<svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
					</svg>
				</div>
			</a>
		{/if}

		<!-- Profile -->
		<a href="/settings/profile" class="block p-4 hover:bg-gray-50 transition-colors">
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

		<!-- Organization Settings (for owners only) -->
		{#if isOwner}
			<a href="/settings/organization" class="block p-4 hover:bg-gray-50 transition-colors">
				<div class="flex items-center justify-between">
					<div class="flex items-center gap-3">
						<div class="w-10 h-10 rounded-full bg-red-100 flex items-center justify-center text-red-600">
							<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
								<path stroke-linecap="round" stroke-linejoin="round" d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4" />
							</svg>
						</div>
						<div>
							<p class="font-medium">Organization</p>
							<p class="text-sm text-gray-500">Manage or delete your organization</p>
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
