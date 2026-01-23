<script lang="ts">
	import { enhance } from '$app/forms';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { env } from '$env/dynamic/public';

	let { form } = $props();
	let isLoading = $state(false);
	let oauthLoading = $state<'google' | 'apple' | null>(null);
	let oauthError = $state<string | null>(null);

	// Check if OAuth is configured
	const googleClientId = env.PUBLIC_GOOGLE_CLIENT_ID || '';
	const appleClientId = env.PUBLIC_APPLE_CLIENT_ID || '';
	const apiUrl = env.PUBLIC_API_URL || '';
	const isGoogleConfigured = Boolean(googleClientId);
	const isAppleConfigured = Boolean(appleClientId);

	onMount(() => {
		// Load Google Identity Services
		if (isGoogleConfigured && typeof window !== 'undefined') {
			const script = document.createElement('script');
			script.src = 'https://accounts.google.com/gsi/client';
			script.async = true;
			script.defer = true;
			script.onload = initializeGoogle;
			document.head.appendChild(script);
		}

		// Load Apple JS SDK
		if (isAppleConfigured && typeof window !== 'undefined') {
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
			redirectURI: `${$page.url.origin}/auth/apple/callback`,
			usePopup: true,
		});
	}

	async function handleGoogleLogin() {
		if (!isGoogleConfigured) {
			oauthError = 'Google Sign-In is not configured';
			return;
		}

		oauthLoading = 'google';
		oauthError = null;

		try {
			// Trigger Google One Tap or popup
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
			oauthError = err instanceof Error ? err.message : 'Google login failed';
			oauthLoading = null;
		}
	}

	async function handleGoogleCallback(response: any) {
		try {
			const idToken = response.credential || response.access_token;
			if (!idToken) {
				throw new Error('No token received from Google');
			}

			// Call the API
			const res = await fetch(`${apiUrl}/auth/google`, {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					id_token: idToken,
				}),
			});

			if (!res.ok) {
				const error = await res.text();
				throw new Error(error || 'Google authentication failed');
			}

			const result = await res.json();

			// Check for admin access
			const adminMemberships = result.memberships?.filter(
				(m: any) => m.role === 'admin' || m.role === 'owner'
			) ?? [];

			if (adminMemberships.length === 0) {
				oauthError = 'You do not have admin access to any organization';
				oauthLoading = null;
				return;
			}

			// Store cookies and redirect
			document.cookie = `token=${result.token}; path=/; max-age=${60 * 60 * 24 * 7}; SameSite=Lax`;
			document.cookie = `user=${JSON.stringify(result.user)}; path=/; max-age=${60 * 60 * 24 * 7}; SameSite=Lax`;
			document.cookie = `memberships=${JSON.stringify(adminMemberships)}; path=/; max-age=${60 * 60 * 24 * 7}; SameSite=Lax`;
			if (result.membership) {
				document.cookie = `membership=${JSON.stringify(result.membership)}; path=/; max-age=${60 * 60 * 24 * 7}; SameSite=Lax`;
			}

			await goto('/dashboard');
		} catch (err) {
			oauthError = err instanceof Error ? err.message : 'Google authentication failed';
		} finally {
			oauthLoading = null;
		}
	}

	async function handleAppleLogin() {
		if (!isAppleConfigured) {
			oauthError = 'Apple Sign-In is not configured';
			return;
		}

		oauthLoading = 'apple';
		oauthError = null;

		try {
			const appleResponse = await (window as any).AppleID.auth.signIn();

			// Call the API
			const res = await fetch(`${apiUrl}/auth/apple`, {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					id_token: appleResponse.authorization.id_token,
					first_name: appleResponse.user?.name?.firstName,
					last_name: appleResponse.user?.name?.lastName,
				}),
			});

			if (!res.ok) {
				const error = await res.text();
				throw new Error(error || 'Apple authentication failed');
			}

			const result = await res.json();

			// Check for admin access
			const adminMemberships = result.memberships?.filter(
				(m: any) => m.role === 'admin' || m.role === 'owner'
			) ?? [];

			if (adminMemberships.length === 0) {
				oauthError = 'You do not have admin access to any organization';
				oauthLoading = null;
				return;
			}

			// Store cookies and redirect
			document.cookie = `token=${result.token}; path=/; max-age=${60 * 60 * 24 * 7}; SameSite=Lax`;
			document.cookie = `user=${JSON.stringify(result.user)}; path=/; max-age=${60 * 60 * 24 * 7}; SameSite=Lax`;
			document.cookie = `memberships=${JSON.stringify(adminMemberships)}; path=/; max-age=${60 * 60 * 24 * 7}; SameSite=Lax`;
			if (result.membership) {
				document.cookie = `membership=${JSON.stringify(result.membership)}; path=/; max-age=${60 * 60 * 24 * 7}; SameSite=Lax`;
			}

			await goto('/dashboard');
		} catch (err) {
			if ((err as any)?.error === 'popup_closed_by_user') {
				oauthLoading = null;
				return;
			}
			oauthError = err instanceof Error ? err.message : 'Apple authentication failed';
		} finally {
			oauthLoading = null;
		}
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
				onclick={handleGoogleLogin}
				disabled={oauthLoading !== null}
				class="w-full flex items-center justify-center gap-3 bg-white border border-gray-300 text-gray-700 py-2.5 px-4 rounded-lg hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-500 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
			>
				<svg class="w-5 h-5" viewBox="0 0 24 24">
					<path fill="#4285F4" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/>
					<path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/>
					<path fill="#FBBC05" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"/>
					<path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"/>
				</svg>
				{oauthLoading === 'google' ? 'Connecting...' : 'Continue with Google'}
			</button>
			{/if}

			{#if isAppleConfigured}
			<button
				type="button"
				onclick={handleAppleLogin}
				disabled={oauthLoading !== null}
				class="w-full flex items-center justify-center gap-3 bg-black text-white py-2.5 px-4 rounded-lg hover:bg-gray-900 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-500 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
			>
				<svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
					<path d="M17.05 20.28c-.98.95-2.05.8-3.08.35-1.09-.46-2.09-.48-3.24 0-1.44.62-2.2.44-3.06-.35C2.79 15.25 3.51 7.59 9.05 7.31c1.35.07 2.29.74 3.08.8 1.18-.24 2.31-.93 3.57-.84 1.51.12 2.65.72 3.4 1.8-3.12 1.87-2.38 5.98.48 7.13-.57 1.5-1.31 2.99-2.54 4.09l.01-.01zM12.03 7.25c-.15-2.23 1.66-4.07 3.74-4.25.29 2.58-2.34 4.5-3.74 4.25z"/>
				</svg>
				{oauthLoading === 'apple' ? 'Connecting...' : 'Continue with Apple'}
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
