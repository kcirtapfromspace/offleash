<script lang="ts">
	import { enhance } from '$app/forms';
	import { page } from '$app/stores';
	import { env } from '$env/dynamic/public';

	let { form } = $props();
	let isLoading = $state(false);
	let oauthError = $state<string | null>(null);

	// Check if OAuth is configured
	const googleClientId = env.PUBLIC_GOOGLE_CLIENT_ID || '';
	const appleClientId = env.PUBLIC_APPLE_CLIENT_ID || '';
	const isGoogleConfigured = Boolean(googleClientId);
	const isAppleConfigured = Boolean(appleClientId);

	// Customer web URL for OAuth (OAuth only works on root domain)
	// Default to offleash.world since that's where Apple/Google OAuth redirect URIs are registered
	const customerWebUrl = env.PUBLIC_CUSTOMER_URL || 'https://offleash.world';

	// Get the redirect path from URL params or default to /dashboard
	const redirectPath = $page.url.searchParams.get('redirect') || '/dashboard';

	// Redirect to customer-web for OAuth authentication
	// OAuth providers (Apple, Google) only have the root domain registered
	// Don't pass provider param - let user click OAuth button on customer-web (proper gesture context)
	function handleOAuthLogin(_provider: 'google' | 'apple') {
		const authUrl = new URL('/login', customerWebUrl);
		authUrl.searchParams.set('app', 'admin');
		authUrl.searchParams.set('redirect', window.location.origin + redirectPath);
		window.location.href = authUrl.toString();
	}
</script>

<div class="min-h-screen flex items-center justify-center bg-gray-100">
	<div class="bg-white p-8 rounded-lg shadow-md w-full max-w-md">
		<h1 class="text-2xl font-bold text-center mb-6">Admin Login</h1>

		{#if form?.error || oauthError}
			<div class="mb-4 p-3 bg-red-100 border border-red-400 text-red-700 rounded">
				{form?.error || oauthError}
			</div>
		{/if}

		<!-- OAuth Buttons -->
		{#if isGoogleConfigured || isAppleConfigured}
		<div class="space-y-3 mb-6">
			{#if isGoogleConfigured}
			<button
				type="button"
				onclick={() => handleOAuthLogin('google')}
				class="w-full flex items-center justify-center gap-3 bg-white border border-gray-300 text-gray-700 py-2.5 px-4 rounded-lg hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-500 transition-colors"
			>
				<svg class="w-5 h-5" viewBox="0 0 24 24">
					<path fill="#4285F4" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/>
					<path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/>
					<path fill="#FBBC05" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"/>
					<path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"/>
				</svg>
				Continue with Google
			</button>
			{/if}

			{#if isAppleConfigured}
			<button
				type="button"
				onclick={() => handleOAuthLogin('apple')}
				class="w-full flex items-center justify-center gap-3 bg-black text-white py-2.5 px-4 rounded-lg hover:bg-gray-900 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-500 transition-colors"
			>
				<svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
					<path d="M17.05 20.28c-.98.95-2.05.8-3.08.35-1.09-.46-2.09-.48-3.24 0-1.44.62-2.2.44-3.06-.35C2.79 15.25 3.51 7.59 9.05 7.31c1.35.07 2.29.74 3.08.8 1.18-.24 2.31-.93 3.57-.84 1.51.12 2.65.72 3.4 1.8-3.12 1.87-2.38 5.98.48 7.13-.57 1.5-1.31 2.99-2.54 4.09l.01-.01zM12.03 7.25c-.15-2.23 1.66-4.07 3.74-4.25.29 2.58-2.34 4.5-3.74 4.25z"/>
				</svg>
				Continue with Apple
			</button>
			{/if}
		</div>

		<!-- Divider -->
		<div class="relative mb-6">
			<div class="absolute inset-0 flex items-center">
				<div class="w-full border-t border-gray-300"></div>
			</div>
			<div class="relative flex justify-center text-sm">
				<span class="px-2 bg-white text-gray-500">or continue with email</span>
			</div>
		</div>
		{/if}

		<!-- Email/Password Form -->
		<form
			method="POST"
			use:enhance={() => {
				isLoading = true;
				return async ({ update }) => {
					await update();
					isLoading = false;
				};
			}}
		>
			<div class="mb-4">
				<label for="email" class="block text-sm font-medium text-gray-700 mb-1">
					Email
				</label>
				<input
					type="email"
					id="email"
					name="email"
					required
					class="w-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
					placeholder="admin@example.com"
					disabled={isLoading}
					value={form?.email ?? ''}
				/>
			</div>

			<div class="mb-6">
				<label for="password" class="block text-sm font-medium text-gray-700 mb-1">
					Password
				</label>
				<input
					type="password"
					id="password"
					name="password"
					required
					class="w-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
					placeholder="Enter your password"
					disabled={isLoading}
				/>
			</div>

			<button
				type="submit"
				disabled={isLoading}
				class="w-full bg-blue-600 text-white py-2 px-4 rounded hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed"
			>
				{isLoading ? 'Logging in...' : 'Login'}
			</button>
		</form>
	</div>
</div>
