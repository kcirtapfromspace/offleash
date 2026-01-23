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

	// Phone auth state
	let phoneAuthMode = $state(false);
	let phoneNumber = $state('');
	let countryCode = $state('+1');
	let verificationCode = $state('');
	let codeSent = $state(false);
	let phoneLoading = $state(false);
	let phoneError = $state<string | null>(null);
	let resendCooldown = $state(0);
	let resendTimer: ReturnType<typeof setInterval> | null = null;

	// Wallet auth state
	let walletLoading = $state(false);
	let walletError = $state<string | null>(null);
	let hasWallet = $state(false);

	// Check if browser has Ethereum wallet
	$effect(() => {
		if (typeof window !== 'undefined') {
			hasWallet = !!(window as any).ethereum;
		}
	});

	// Check if OAuth is configured
	const googleClientId = env.PUBLIC_GOOGLE_CLIENT_ID || '';
	const appleClientId = env.PUBLIC_APPLE_CLIENT_ID || '';
	const apiUrl = env.PUBLIC_API_URL || '';
	const isGoogleConfigured = Boolean(googleClientId);
	const isAppleConfigured = Boolean(appleClientId);
	const hasOAuthProviders = isGoogleConfigured || isAppleConfigured;

	// Get org slug from URL or default
	const orgSlug = $page.url.searchParams.get('org') || 'demo';

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

			// Call the Rust API directly (not through SvelteKit proxy)
			const res = await fetch(`${apiUrl}/auth/google`, {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					org_slug: orgSlug,
					id_token: idToken,
				}),
			});

			if (!res.ok) {
				const error = await res.text();
				throw new Error(error || 'Google authentication failed');
			}

			const result = await res.json();

			// Store token and redirect
			document.cookie = `token=${result.token}; path=/; max-age=${60 * 60 * 24 * 7}; SameSite=Lax`;
			await goto('/services');
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

			// Call the Rust API directly (not through SvelteKit proxy)
			const res = await fetch(`${apiUrl}/auth/apple`, {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					org_slug: orgSlug,
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

			// Store token and redirect
			document.cookie = `token=${result.token}; path=/; max-age=${60 * 60 * 24 * 7}; SameSite=Lax`;
			await goto('/services');
		} catch (err) {
			if ((err as any)?.error === 'popup_closed_by_user') {
				// User cancelled, not an error
				oauthLoading = null;
				return;
			}
			oauthError = err instanceof Error ? err.message : 'Apple authentication failed';
		} finally {
			oauthLoading = null;
		}
	}

	// Phone auth functions
	function startPhoneAuth() {
		phoneAuthMode = true;
		codeSent = false;
		verificationCode = '';
		phoneError = null;
	}

	function cancelPhoneAuth() {
		phoneAuthMode = false;
		codeSent = false;
		phoneNumber = '';
		verificationCode = '';
		phoneError = null;
		if (resendTimer) {
			clearInterval(resendTimer);
			resendTimer = null;
		}
		resendCooldown = 0;
	}

	function startResendCooldown() {
		resendCooldown = 60;
		if (resendTimer) clearInterval(resendTimer);
		resendTimer = setInterval(() => {
			resendCooldown--;
			if (resendCooldown <= 0) {
				if (resendTimer) clearInterval(resendTimer);
				resendTimer = null;
			}
		}, 1000);
	}

	async function sendPhoneCode() {
		phoneLoading = true;
		phoneError = null;

		const fullPhoneNumber = countryCode + phoneNumber.replace(/\D/g, '');

		try {
			const res = await fetch(`${apiUrl}/auth/phone/send-code`, {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					org_slug: orgSlug,
					phone_number: fullPhoneNumber,
				}),
			});

			if (!res.ok) {
				const error = await res.json().catch(() => ({ message: 'Failed to send code' }));
				throw new Error(error.message || 'Failed to send verification code');
			}

			codeSent = true;
			startResendCooldown();
		} catch (err) {
			phoneError = err instanceof Error ? err.message : 'Failed to send code';
		} finally {
			phoneLoading = false;
		}
	}

	async function verifyPhoneCode() {
		phoneLoading = true;
		phoneError = null;

		const fullPhoneNumber = countryCode + phoneNumber.replace(/\D/g, '');

		try {
			const res = await fetch(`${apiUrl}/auth/phone/verify`, {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					org_slug: orgSlug,
					phone_number: fullPhoneNumber,
					code: verificationCode,
				}),
			});

			if (!res.ok) {
				const error = await res.json().catch(() => ({ message: 'Verification failed' }));
				throw new Error(error.message || 'Invalid verification code');
			}

			const result = await res.json();

			// Store token and redirect
			document.cookie = `token=${result.token}; path=/; max-age=${60 * 60 * 24 * 7}; SameSite=Lax`;
			await goto('/services');
		} catch (err) {
			phoneError = err instanceof Error ? err.message : 'Verification failed';
		} finally {
			phoneLoading = false;
		}
	}

	// Auto-submit when code is complete
	function handleCodeInput(event: Event) {
		const input = event.target as HTMLInputElement;
		verificationCode = input.value.replace(/\D/g, '').slice(0, 6);
		if (verificationCode.length === 6) {
			verifyPhoneCode();
		}
	}

	// Format phone number as user types
	function formatPhoneNumber(event: Event) {
		const input = event.target as HTMLInputElement;
		const cleaned = input.value.replace(/\D/g, '');
		phoneNumber = cleaned;
	}

	// Wallet auth functions
	async function handleWalletLogin() {
		const ethereum = (window as any).ethereum;
		if (!ethereum) {
			walletError = 'No Ethereum wallet detected. Please install MetaMask.';
			return;
		}

		walletLoading = true;
		walletError = null;
		oauthError = null;

		try {
			// Request account access
			const accounts = await ethereum.request({ method: 'eth_requestAccounts' });
			if (!accounts || accounts.length === 0) {
				throw new Error('No accounts found. Please unlock your wallet.');
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
				const error = await challengeRes.json().catch(() => ({ message: 'Failed to get challenge' }));
				throw new Error(error.message || 'Failed to get signing challenge');
			}

			const { message } = await challengeRes.json();

			// Sign the message with the wallet
			const signature = await ethereum.request({
				method: 'personal_sign',
				params: [message, walletAddress],
			});

			// Verify signature with backend
			const verifyRes = await fetch(`${apiUrl}/auth/wallet/verify`, {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					org_slug: orgSlug,
					wallet_address: walletAddress,
					message,
					signature,
				}),
			});

			if (!verifyRes.ok) {
				const error = await verifyRes.json().catch(() => ({ message: 'Verification failed' }));
				throw new Error(error.message || 'Wallet verification failed');
			}

			const result = await verifyRes.json();

			// Store token and redirect
			document.cookie = `token=${result.token}; path=/; max-age=${60 * 60 * 24 * 7}; SameSite=Lax`;
			await goto('/services');
		} catch (err: any) {
			// Handle user rejection
			if (err.code === 4001) {
				walletError = 'You rejected the signature request.';
			} else {
				walletError = err instanceof Error ? err.message : 'Wallet login failed';
			}
		} finally {
			walletLoading = false;
		}
	}
</script>

<div class="min-h-screen flex items-center justify-center bg-gray-100">
	<div class="bg-white p-8 rounded-lg shadow-md w-full max-w-md">
		<h1 class="text-2xl font-bold text-center mb-6" style="color: var(--color-primary)">
			Login
		</h1>

		{#if form?.error || oauthError || phoneError || walletError}
			<div class="mb-4 p-3 bg-red-100 border border-red-400 text-red-700 rounded">
				{form?.error || oauthError || phoneError || walletError}
			</div>
		{/if}

		<!-- Phone Auth Mode -->
		{#if phoneAuthMode}
		<div class="space-y-4">
			<button
				type="button"
				onclick={cancelPhoneAuth}
				class="flex items-center text-gray-600 hover:text-gray-900 text-sm mb-2"
			>
				<svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
				</svg>
				Back to login options
			</button>

			{#if !codeSent}
				<!-- Phone number input -->
				<div>
					<label for="phone" class="block text-sm font-medium text-gray-700 mb-1">
						Phone Number
					</label>
					<div class="flex gap-2">
						<select
							bind:value={countryCode}
							disabled={phoneLoading}
							class="px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
						>
							<option value="+1">+1 (US)</option>
							<option value="+44">+44 (UK)</option>
							<option value="+61">+61 (AU)</option>
							<option value="+91">+91 (IN)</option>
							<option value="+86">+86 (CN)</option>
							<option value="+81">+81 (JP)</option>
							<option value="+49">+49 (DE)</option>
							<option value="+33">+33 (FR)</option>
							<option value="+52">+52 (MX)</option>
							<option value="+55">+55 (BR)</option>
						</select>
						<input
							type="tel"
							id="phone"
							value={phoneNumber}
							oninput={formatPhoneNumber}
							required
							disabled={phoneLoading}
							class="flex-1 px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
							placeholder="(555) 123-4567"
						/>
					</div>
					<p class="text-xs text-gray-500 mt-1">We'll send you a verification code</p>
				</div>

				<button
					type="button"
					onclick={sendPhoneCode}
					disabled={phoneLoading || phoneNumber.length < 10}
					class="w-full text-white py-2.5 px-4 rounded-lg focus:outline-none focus:ring-2 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
					style="background-color: var(--color-primary)"
				>
					{phoneLoading ? 'Sending...' : 'Send Verification Code'}
				</button>
			{:else}
				<!-- Verification code input -->
				<div>
					<p class="text-sm text-gray-600 mb-3">
						Enter the 6-digit code sent to {countryCode}{phoneNumber}
					</p>
					<label for="code" class="block text-sm font-medium text-gray-700 mb-1">
						Verification Code
					</label>
					<input
						type="text"
						id="code"
						value={verificationCode}
						oninput={handleCodeInput}
						maxlength="6"
						disabled={phoneLoading}
						class="w-full px-3 py-3 border border-gray-300 rounded text-center text-2xl tracking-widest font-mono focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
						placeholder="000000"
						autocomplete="one-time-code"
					/>
				</div>

				<button
					type="button"
					onclick={verifyPhoneCode}
					disabled={phoneLoading || verificationCode.length !== 6}
					class="w-full text-white py-2.5 px-4 rounded-lg focus:outline-none focus:ring-2 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
					style="background-color: var(--color-primary)"
				>
					{phoneLoading ? 'Verifying...' : 'Verify Code'}
				</button>

				<div class="text-center">
					{#if resendCooldown > 0}
						<p class="text-sm text-gray-500">
							Resend code in {resendCooldown}s
						</p>
					{:else}
						<button
							type="button"
							onclick={sendPhoneCode}
							disabled={phoneLoading}
							class="text-sm hover:underline disabled:opacity-50"
							style="color: var(--color-primary)"
						>
							Resend code
						</button>
					{/if}
				</div>
			{/if}
		</div>
		{:else}
		<!-- Standard login options -->

		<!-- OAuth & Phone Buttons -->
		<div class="space-y-3 mb-6">
			{#if isGoogleConfigured}
			<button
				type="button"
				onclick={handleGoogleLogin}
				disabled={oauthLoading !== null || walletLoading}
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
				disabled={oauthLoading !== null || walletLoading}
				class="w-full flex items-center justify-center gap-3 bg-black text-white py-2.5 px-4 rounded-lg hover:bg-gray-900 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-500 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
			>
				<svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
					<path d="M17.05 20.28c-.98.95-2.05.8-3.08.35-1.09-.46-2.09-.48-3.24 0-1.44.62-2.2.44-3.06-.35C2.79 15.25 3.51 7.59 9.05 7.31c1.35.07 2.29.74 3.08.8 1.18-.24 2.31-.93 3.57-.84 1.51.12 2.65.72 3.4 1.8-3.12 1.87-2.38 5.98.48 7.13-.57 1.5-1.31 2.99-2.54 4.09l.01-.01zM12.03 7.25c-.15-2.23 1.66-4.07 3.74-4.25.29 2.58-2.34 4.5-3.74 4.25z"/>
				</svg>
				{oauthLoading === 'apple' ? 'Connecting...' : 'Continue with Apple'}
			</button>
			{/if}

			<!-- Phone auth button (always shown) -->
			<button
				type="button"
				onclick={startPhoneAuth}
				disabled={oauthLoading !== null || walletLoading}
				class="w-full flex items-center justify-center gap-3 bg-white border border-gray-300 text-gray-700 py-2.5 px-4 rounded-lg hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-500 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
			>
				<svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
					<path stroke-linecap="round" stroke-linejoin="round" d="M3 5a2 2 0 012-2h3.28a1 1 0 01.948.684l1.498 4.493a1 1 0 01-.502 1.21l-2.257 1.13a11.042 11.042 0 005.516 5.516l1.13-2.257a1 1 0 011.21-.502l4.493 1.498a1 1 0 01.684.949V19a2 2 0 01-2 2h-1C9.716 21 3 14.284 3 6V5z" />
				</svg>
				Continue with Phone
			</button>

			<!-- Wallet auth button (shown when wallet detected) -->
			{#if hasWallet}
			<button
				type="button"
				onclick={handleWalletLogin}
				disabled={oauthLoading !== null || walletLoading}
				class="w-full flex items-center justify-center gap-3 bg-gradient-to-r from-purple-600 to-blue-500 text-white py-2.5 px-4 rounded-lg hover:from-purple-700 hover:to-blue-600 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-purple-500 disabled:opacity-50 disabled:cursor-not-allowed transition-all"
			>
				<svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
					<path stroke-linecap="round" stroke-linejoin="round" d="M17 9V7a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2m2 4h10a2 2 0 002-2v-6a2 2 0 00-2-2H9a2 2 0 00-2 2v6a2 2 0 002 2zm7-5a2 2 0 11-4 0 2 2 0 014 0z" />
				</svg>
				{walletLoading ? 'Connecting Wallet...' : 'Continue with Wallet'}
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
					placeholder="you@example.com"
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
				class="w-full text-white py-2 px-4 rounded focus:outline-none focus:ring-2 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed"
				style="background-color: var(--color-primary)"
			>
				{isLoading ? 'Logging in...' : 'Login'}
			</button>
		</form>

		<p class="mt-4 text-center text-gray-600">
			Don't have an account?
			<a href="/register" class="hover:underline" style="color: var(--color-primary)">
				Register
			</a>
		</p>
		{/if}
	</div>
</div>
