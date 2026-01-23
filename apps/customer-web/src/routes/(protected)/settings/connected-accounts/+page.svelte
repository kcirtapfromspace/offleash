<script lang="ts">
	import { enhance, deserialize } from '$app/forms';
	import { invalidateAll } from '$app/navigation';
	import { page } from '$app/stores';
	import { env } from '$env/dynamic/public';
	import { onMount } from 'svelte';
	import type { ActionResult } from '@sveltejs/kit';

	let { data, form } = $props();

	let unlinking = $state<string | null>(null);
	let linkingProvider = $state<string | null>(null);
	let linkError = $state<string | null>(null);
	let linkSuccess = $state<string | null>(null);
	let showPasswordForm = $state(false);
	let showChangePasswordForm = $state(false);

	const apiUrl = env.PUBLIC_API_URL || '';
	const googleClientId = env.PUBLIC_GOOGLE_CLIENT_ID || '';
	const appleClientId = env.PUBLIC_APPLE_CLIENT_ID || '';

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
	let linkedProviders = $derived(new Set(data.identities.map((i: any) => i.provider)));
	let hasEmailLinked = $derived(linkedProviders.has('email'));

	// Hidden form refs for OAuth linking
	let googleForm: HTMLFormElement;
	let appleForm: HTMLFormElement;

	onMount(() => {
		// Load Google Identity Services if not linked and configured
		if (!linkedProviders.has('google') && googleClientId && typeof window !== 'undefined') {
			const script = document.createElement('script');
			script.src = 'https://accounts.google.com/gsi/client';
			script.async = true;
			script.defer = true;
			script.onload = initializeGoogle;
			document.head.appendChild(script);
		}

		// Load Apple JS SDK if not linked and configured
		if (!linkedProviders.has('apple') && appleClientId && typeof window !== 'undefined') {
			const script = document.createElement('script');
			script.src = 'https://appleid.cdn-apple.com/appleauth/static/jsapi/appleid/1/en_US/appleid.auth.js';
			script.async = true;
			script.defer = true;
			script.onload = initializeApple;
			document.head.appendChild(script);
		}
	});

	function initializeGoogle() {
		if (!(window as any).google) return;

		(window as any).google.accounts.id.initialize({
			client_id: googleClientId,
			callback: handleGoogleCallback,
			auto_select: false,
		});
	}

	function initializeApple() {
		if (!(window as any).AppleID) return;

		(window as any).AppleID.auth.init({
			clientId: appleClientId,
			scope: 'name email',
			redirectURI: $page.url.origin,
			usePopup: true,
		});
	}

	async function handleLinkGoogle() {
		if (!googleClientId) {
			linkError = 'Google Sign-In is not configured';
			return;
		}

		linkingProvider = 'google';
		linkError = null;
		linkSuccess = null;

		try {
			(window as any).google.accounts.id.prompt((notification: any) => {
				if (notification.isNotDisplayed() || notification.isSkippedMoment()) {
					// Fall back to popup
					(window as any).google.accounts.oauth2.initTokenClient({
						client_id: googleClientId,
						scope: 'openid email profile',
						callback: handleGoogleCallback,
					}).requestAccessToken();
				}
			});
		} catch (err) {
			linkError = err instanceof Error ? err.message : 'Google linking failed';
			linkingProvider = null;
		}
	}

	async function handleGoogleCallback(response: any) {
		try {
			const idToken = response.credential || response.access_token;
			if (!idToken) {
				throw new Error('No token received from Google');
			}

			// Submit the form with the token
			const formData = new FormData();
			formData.set('id_token', idToken);

			const res = await fetch('?/linkGoogle', {
				method: 'POST',
				body: formData,
				headers: {
					'x-sveltekit-action': 'true'
				}
			});

			const result: ActionResult = deserialize(await res.text());

			if (result.type === 'failure') {
				linkError = result.data?.error as string || 'Failed to link Google account';
			} else if (result.type === 'success') {
				linkSuccess = 'Google account linked successfully!';
				await invalidateAll();
			} else if (result.type === 'error') {
				linkError = result.error?.message || 'Failed to link Google account';
			}
		} catch (err) {
			linkError = err instanceof Error ? err.message : 'Google linking failed';
		} finally {
			linkingProvider = null;
		}
	}

	async function handleLinkApple() {
		if (!appleClientId) {
			linkError = 'Apple Sign-In is not configured';
			return;
		}

		linkingProvider = 'apple';
		linkError = null;
		linkSuccess = null;

		try {
			const appleResponse = await (window as any).AppleID.auth.signIn();

			// Submit the form with the token
			const formData = new FormData();
			formData.set('id_token', appleResponse.authorization.id_token);

			const res = await fetch('?/linkApple', {
				method: 'POST',
				body: formData,
				headers: {
					'x-sveltekit-action': 'true'
				}
			});

			const result: ActionResult = deserialize(await res.text());

			if (result.type === 'failure') {
				linkError = result.data?.error as string || 'Failed to link Apple account';
			} else if (result.type === 'success') {
				linkSuccess = 'Apple account linked successfully!';
				await invalidateAll();
			} else if (result.type === 'error') {
				linkError = result.error?.message || 'Failed to link Apple account';
			}
		} catch (err: any) {
			if (err?.error === 'popup_closed_by_user') {
				linkingProvider = null;
				return;
			}
			linkError = err instanceof Error ? err.message : 'Apple linking failed';
		} finally {
			linkingProvider = null;
		}
	}

	async function handleLinkWallet() {
		const ethereum = (window as any).ethereum;
		if (!ethereum) {
			linkError = 'No Ethereum wallet detected. Please install MetaMask.';
			return;
		}

		linkingProvider = 'wallet';
		linkError = null;
		linkSuccess = null;

		try {
			const accounts = await ethereum.request({ method: 'eth_requestAccounts' });
			if (!accounts || accounts.length === 0) {
				throw new Error('No accounts found');
			}
			// Wallet linking would need backend support
			linkError = 'Wallet linking is not yet supported.';
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

	function clearMessages() {
		linkError = null;
		linkSuccess = null;
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
	<p class="text-gray-600 mb-6">Manage your sign-in methods. You can add multiple ways to access your account.</p>

	{#if form?.error || linkError}
		<div class="mb-4 p-3 bg-red-100 border border-red-400 text-red-700 rounded flex justify-between items-center">
			<span>{form?.error || linkError}</span>
			<button onclick={clearMessages} class="text-red-700 hover:text-red-900">&times;</button>
		</div>
	{/if}

	{#if form?.success || linkSuccess}
		<div class="mb-4 p-3 bg-green-100 border border-green-400 text-green-700 rounded flex justify-between items-center">
			<span>{form?.message || linkSuccess || 'Success!'}</span>
			<button onclick={clearMessages} class="text-green-700 hover:text-green-900">&times;</button>
		</div>
	{/if}

	<!-- Linked accounts -->
	<div class="bg-white rounded-lg shadow divide-y">
		<div class="p-4 bg-gray-50 rounded-t-lg">
			<h2 class="font-semibold text-gray-700">Linked Sign-in Methods</h2>
		</div>

		{#if data.identities.length === 0}
			<div class="p-6 text-center text-gray-500">
				No linked accounts found.
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
					<div class="flex items-center gap-2">
						{#if identity.provider === 'email' && identity.can_unlink}
							<button
								onclick={() => showChangePasswordForm = !showChangePasswordForm}
								class="text-sm text-blue-600 hover:text-blue-800"
							>
								Change Password
							</button>
						{/if}
						{#if identity.can_unlink}
							<form method="POST" action="?/unlink" use:enhance={() => {
								unlinking = identity.id;
								clearMessages();
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

	<!-- Change Password Form -->
	{#if showChangePasswordForm && hasEmailLinked}
		<div class="mt-4 bg-white rounded-lg shadow p-4">
			<h3 class="font-semibold mb-4">Change Password</h3>
			<form method="POST" action="?/changePassword" use:enhance={() => {
				clearMessages();
				return async ({ update }) => {
					await update();
					showChangePasswordForm = false;
					await invalidateAll();
				};
			}}>
				<div class="space-y-4">
					<div>
						<label for="currentPassword" class="block text-sm font-medium text-gray-700 mb-1">
							Current Password
						</label>
						<input
							type="password"
							id="currentPassword"
							name="currentPassword"
							required
							class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
						/>
					</div>
					<div>
						<label for="newPassword" class="block text-sm font-medium text-gray-700 mb-1">
							New Password
						</label>
						<input
							type="password"
							id="newPassword"
							name="newPassword"
							required
							minlength="8"
							class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
						/>
					</div>
					<div>
						<label for="confirmPasswordChange" class="block text-sm font-medium text-gray-700 mb-1">
							Confirm New Password
						</label>
						<input
							type="password"
							id="confirmPasswordChange"
							name="confirmPassword"
							required
							minlength="8"
							class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
						/>
					</div>
					<div class="flex gap-2">
						<button
							type="submit"
							class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
						>
							Update Password
						</button>
						<button
							type="button"
							onclick={() => showChangePasswordForm = false}
							class="px-4 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300"
						>
							Cancel
						</button>
					</div>
				</div>
			</form>
		</div>
	{/if}

	<!-- Add new methods -->
	<div class="mt-6 bg-white rounded-lg shadow">
		<div class="p-4 bg-gray-50 rounded-t-lg">
			<h2 class="font-semibold text-gray-700">Add Sign-in Method</h2>
		</div>
		<div class="p-4 space-y-3">
			<!-- Google -->
			{#if !linkedProviders.has('google') && googleClientId}
				<button
					onclick={handleLinkGoogle}
					disabled={linkingProvider === 'google'}
					class="w-full flex items-center justify-center gap-3 bg-white border border-gray-300 text-gray-700 py-2.5 px-4 rounded-lg hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
				>
					<svg class="w-5 h-5" viewBox="0 0 24 24">
						<path fill="#4285F4" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/>
						<path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/>
						<path fill="#FBBC05" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"/>
						<path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"/>
					</svg>
					{linkingProvider === 'google' ? 'Connecting...' : 'Link Google Account'}
				</button>
			{/if}

			<!-- Apple -->
			{#if !linkedProviders.has('apple') && appleClientId}
				<button
					onclick={handleLinkApple}
					disabled={linkingProvider === 'apple'}
					class="w-full flex items-center justify-center gap-3 bg-black text-white py-2.5 px-4 rounded-lg hover:bg-gray-900 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
				>
					<svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
						<path d="M17.05 20.28c-.98.95-2.05.8-3.08.35-1.09-.46-2.09-.48-3.24 0-1.44.62-2.2.44-3.06-.35C2.79 15.25 3.51 7.59 9.05 7.31c1.35.07 2.29.74 3.08.8 1.18-.24 2.31-.93 3.57-.84 1.51.12 2.65.72 3.4 1.8-3.12 1.87-2.38 5.98.48 7.13-.57 1.5-1.31 2.99-2.54 4.09l.01-.01zM12.03 7.25c-.15-2.23 1.66-4.07 3.74-4.25.29 2.58-2.34 4.5-3.74 4.25z"/>
					</svg>
					{linkingProvider === 'apple' ? 'Connecting...' : 'Link Apple Account'}
				</button>
			{/if}

			<!-- Email/Password -->
			{#if !linkedProviders.has('email')}
				{#if !showPasswordForm}
					<button
						onclick={() => { showPasswordForm = true; clearMessages(); }}
						class="w-full flex items-center justify-center gap-3 bg-gray-600 text-white py-2.5 px-4 rounded-lg hover:bg-gray-700 transition-colors"
					>
						<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
							<path stroke-linecap="round" stroke-linejoin="round" d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
						</svg>
						Add Password Login
					</button>
				{:else}
					<div class="border border-gray-200 rounded-lg p-4">
						<h3 class="font-medium mb-3">Set a Password</h3>
						<p class="text-sm text-gray-600 mb-4">
							Add a password so you can also sign in with your email address.
						</p>
						<form method="POST" action="?/linkEmail" use:enhance={() => {
							clearMessages();
							return async ({ update }) => {
								await update();
								showPasswordForm = false;
								await invalidateAll();
							};
						}}>
							<div class="space-y-3">
								<div>
									<label for="password" class="block text-sm font-medium text-gray-700 mb-1">
										Password
									</label>
									<input
										type="password"
										id="password"
										name="password"
										required
										minlength="8"
										class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
										placeholder="At least 8 characters"
									/>
								</div>
								<div>
									<label for="confirmPassword" class="block text-sm font-medium text-gray-700 mb-1">
										Confirm Password
									</label>
									<input
										type="password"
										id="confirmPassword"
										name="confirmPassword"
										required
										minlength="8"
										class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
									/>
								</div>
								<div class="flex gap-2">
									<button
										type="submit"
										class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
									>
										Set Password
									</button>
									<button
										type="button"
										onclick={() => showPasswordForm = false}
										class="px-4 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300"
									>
										Cancel
									</button>
								</div>
							</div>
						</form>
					</div>
				{/if}
			{/if}

			<!-- No more providers to add -->
			{#if linkedProviders.has('google') && linkedProviders.has('apple') && linkedProviders.has('email')}
				<p class="text-sm text-gray-500 text-center py-2">
					All available sign-in methods are linked.
				</p>
			{/if}

			<!-- OAuth not configured message -->
			{#if !googleClientId && !appleClientId && !linkedProviders.has('email')}
				<p class="text-sm text-gray-500 text-center">
					No additional sign-in methods are available.
				</p>
			{/if}
		</div>
	</div>

	<div class="mt-6 p-4 bg-blue-50 rounded-lg border border-blue-200">
		<p class="text-sm text-blue-800">
			<strong>Tip:</strong> Adding multiple sign-in methods gives you backup ways to access your account
			if you lose access to one.
		</p>
	</div>

	<div class="mt-4 p-4 bg-yellow-50 rounded-lg border border-yellow-200">
		<p class="text-sm text-yellow-800">
			<strong>Note:</strong> You must keep at least one sign-in method connected to your account.
		</p>
	</div>
</div>
