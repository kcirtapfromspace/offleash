<script lang="ts">
	import { enhance } from '$app/forms';
	import { invalidateAll } from '$app/navigation';
	import { page } from '$app/stores';
	import { env } from '$env/dynamic/public';

	let { data, form } = $props();

	let unlinking = $state<string | null>(null);
	let linkingProvider = $state<string | null>(null);
	let linkError = $state<string | null>(null);

	const apiUrl = env.PUBLIC_API_URL || '';
	const orgSlug = $page.url.searchParams.get('org') || 'demo';

	// Provider display info
	const providerInfo: Record<string, { name: string; icon: string; color: string }> = {
		email: { name: 'Email & Password', icon: 'mail', color: 'bg-gray-500' },
		phone: { name: 'Phone Number', icon: 'phone', color: 'bg-green-500' },
		google: { name: 'Google', icon: 'google', color: 'bg-red-500' },
		apple: { name: 'Apple', icon: 'apple', color: 'bg-black' },
		wallet: { name: 'Crypto Wallet', icon: 'wallet', color: 'bg-purple-500' },
	};

	function getProviderInfo(provider: string) {
		return providerInfo[provider] || { name: provider, icon: 'link', color: 'bg-gray-400' };
	}

	// Check which providers are not yet linked
	let linkedProviders = $derived(new Set(data.identities.map((i) => i.provider)));
	let availableProviders = $derived(Object.keys(providerInfo).filter((p) => !linkedProviders.has(p)));

	// Link a new provider
	async function linkWallet() {
		const ethereum = (window as any).ethereum;
		if (!ethereum) {
			linkError = 'No Ethereum wallet detected. Please install MetaMask.';
			return;
		}

		linkingProvider = 'wallet';
		linkError = null;

		try {
			// Request account access
			const accounts = await ethereum.request({ method: 'eth_requestAccounts' });
			if (!accounts || accounts.length === 0) {
				throw new Error('No accounts found');
			}
			const walletAddress = accounts[0].toLowerCase();

			// Get challenge from backend
			const challengeRes = await fetch(`${apiUrl}/auth/wallet/challenge`, {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					org_slug: orgSlug,
					wallet_address: walletAddress,
				}),
			});

			if (!challengeRes.ok) {
				throw new Error('Failed to get signing challenge');
			}

			const { message } = await challengeRes.json();

			// Sign the message
			const signature = await ethereum.request({
				method: 'personal_sign',
				params: [message, walletAddress],
			});

			// For linking, we need an authenticated endpoint
			// For now, this creates a new account - proper linking would require an authenticated endpoint
			linkError = 'Wallet linking requires the link endpoint to be implemented. Contact support.';
		} catch (err: any) {
			if (err.code === 4001) {
				linkError = 'Signature request was rejected.';
			} else {
				linkError = err.message || 'Failed to link wallet';
			}
		} finally {
			linkingProvider = null;
		}
	}
</script>

<div class="max-w-2xl mx-auto p-4">
	<div class="mb-6">
		<a href="/settings" class="text-sm text-gray-600 hover:text-gray-900 flex items-center gap-1">
			<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
			</svg>
			Back to Settings
		</a>
	</div>

	<h1 class="text-2xl font-bold mb-2">Connected Accounts</h1>
	<p class="text-gray-600 mb-6">Manage your sign-in methods. You can add or remove ways to access your account.</p>

	{#if form?.error || linkError}
		<div class="mb-4 p-3 bg-red-100 border border-red-400 text-red-700 rounded">
			{form?.error || linkError}
		</div>
	{/if}

	{#if form?.success}
		<div class="mb-4 p-3 bg-green-100 border border-green-400 text-green-700 rounded">
			Account unlinked successfully.
		</div>
	{/if}

	<!-- Linked accounts -->
	<div class="bg-white rounded-lg shadow divide-y">
		<div class="p-4 bg-gray-50 rounded-t-lg">
			<h2 class="font-semibold text-gray-700">Linked Sign-in Methods</h2>
		</div>

		{#if data.identities.length === 0}
			<div class="p-6 text-center text-gray-500">
				No linked accounts found. This shouldn't happen.
			</div>
		{:else}
			{#each data.identities as identity}
				{@const info = getProviderInfo(identity.provider)}
				<div class="p-4 flex items-center justify-between">
					<div class="flex items-center gap-3">
						<div class={`w-10 h-10 rounded-full ${info.color} flex items-center justify-center text-white`}>
							{#if identity.provider === 'google'}
								<svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
									<path d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/>
									<path d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/>
								</svg>
							{:else if identity.provider === 'apple'}
								<svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
									<path d="M17.05 20.28c-.98.95-2.05.8-3.08.35-1.09-.46-2.09-.48-3.24 0-1.44.62-2.2.44-3.06-.35C2.79 15.25 3.51 7.59 9.05 7.31c1.35.07 2.29.74 3.08.8 1.18-.24 2.31-.93 3.57-.84 1.51.12 2.65.72 3.4 1.8-3.12 1.87-2.38 5.98.48 7.13-.57 1.5-1.31 2.99-2.54 4.09l.01-.01zM12.03 7.25c-.15-2.23 1.66-4.07 3.74-4.25.29 2.58-2.34 4.5-3.74 4.25z"/>
								</svg>
							{:else if identity.provider === 'phone'}
								<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
									<path stroke-linecap="round" stroke-linejoin="round" d="M3 5a2 2 0 012-2h3.28a1 1 0 01.948.684l1.498 4.493a1 1 0 01-.502 1.21l-2.257 1.13a11.042 11.042 0 005.516 5.516l1.13-2.257a1 1 0 011.21-.502l4.493 1.498a1 1 0 01.684.949V19a2 2 0 01-2 2h-1C9.716 21 3 14.284 3 6V5z" />
								</svg>
							{:else if identity.provider === 'wallet'}
								<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
									<path stroke-linecap="round" stroke-linejoin="round" d="M17 9V7a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2m2 4h10a2 2 0 002-2v-6a2 2 0 00-2-2H9a2 2 0 00-2 2v6a2 2 0 002 2zm7-5a2 2 0 11-4 0 2 2 0 014 0z" />
								</svg>
							{:else}
								<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
									<path stroke-linecap="round" stroke-linejoin="round" d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
								</svg>
							{/if}
						</div>
						<div>
							<p class="font-medium">{info.name}</p>
							<p class="text-sm text-gray-500">
								{identity.provider_email || identity.provider_user_id}
							</p>
						</div>
					</div>
					<div>
						{#if identity.can_unlink}
							<form method="POST" action="?/unlink" use:enhance={() => {
								unlinking = identity.id;
								return async ({ update }) => {
									await update();
									unlinking = null;
									await invalidateAll();
								};
							}}>
								<input type="hidden" name="identityId" value={identity.id} />
								<button
									type="submit"
									disabled={unlinking === identity.id}
									class="text-sm text-red-600 hover:text-red-800 disabled:opacity-50"
								>
									{unlinking === identity.id ? 'Removing...' : 'Remove'}
								</button>
							</form>
						{:else}
							<span class="text-sm text-gray-400">Primary</span>
						{/if}
					</div>
				</div>
			{/each}
		{/if}
	</div>

	<!-- Add new methods -->
	{#if availableProviders.length > 0}
		<div class="mt-6 bg-white rounded-lg shadow">
			<div class="p-4 bg-gray-50 rounded-t-lg">
				<h2 class="font-semibold text-gray-700">Add Sign-in Method</h2>
			</div>
			<div class="p-4 space-y-3">
				{#if availableProviders.includes('wallet')}
					<button
						onclick={linkWallet}
						disabled={linkingProvider === 'wallet'}
						class="w-full flex items-center justify-center gap-3 bg-gradient-to-r from-purple-600 to-blue-500 text-white py-2.5 px-4 rounded-lg hover:from-purple-700 hover:to-blue-600 disabled:opacity-50 disabled:cursor-not-allowed transition-all"
					>
						<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
							<path stroke-linecap="round" stroke-linejoin="round" d="M17 9V7a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2m2 4h10a2 2 0 002-2v-6a2 2 0 00-2-2H9a2 2 0 00-2 2v6a2 2 0 002 2zm7-5a2 2 0 11-4 0 2 2 0 014 0z" />
						</svg>
						{linkingProvider === 'wallet' ? 'Connecting...' : 'Connect Wallet'}
					</button>
				{/if}

				{#if availableProviders.includes('phone')}
					<p class="text-sm text-gray-500 text-center">
						To add phone authentication, contact support.
					</p>
				{/if}

				{#if availableProviders.includes('google') || availableProviders.includes('apple')}
					<p class="text-sm text-gray-500 text-center">
						Additional OAuth providers can be linked after initial setup.
					</p>
				{/if}
			</div>
		</div>
	{/if}

	<div class="mt-6 p-4 bg-yellow-50 rounded-lg border border-yellow-200">
		<p class="text-sm text-yellow-800">
			<strong>Note:</strong> You must keep at least one sign-in method connected to your account.
			If you want to remove your primary method, please add another one first.
		</p>
	</div>
</div>
