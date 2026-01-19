<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { post, setToken, ApiError } from '$lib/api';
	import { generateCssVariables } from '$lib/stores/branding';

	interface LoginResponse {
		token: string;
	}

	let email = $state('');
	let password = $state('');
	let error = $state<string | null>(null);
	let isLoading = $state(false);

	const branding = $derived($page.data.branding);
	const cssVariables = $derived(generateCssVariables(branding));

	async function handleSubmit(event: SubmitEvent) {
		event.preventDefault();
		error = null;
		isLoading = true;

		try {
			const response = await post<LoginResponse>('/auth/login', { email, password });
			setToken(response.token);
			await goto('/');
		} catch (err) {
			if (err instanceof ApiError) {
				if (err.status === 401) {
					error = 'Invalid email or password';
				} else {
					error = err.message || 'An error occurred. Please try again.';
				}
			} else {
				error = 'An unexpected error occurred. Please try again.';
			}
		} finally {
			isLoading = false;
		}
	}
</script>

<svelte:head>
	<title>Login - {branding.company_name}</title>
</svelte:head>

<div class="login-page" style={cssVariables}>
	<div class="login-card">
		{#if branding.logo_url}
			<img src={branding.logo_url} alt={branding.company_name} class="logo" />
		{:else}
			<h1 class="company-name">{branding.company_name}</h1>
		{/if}

		<h2 class="title">Sign In</h2>

		{#if error}
			<div class="error-message" role="alert">
				{error}
			</div>
		{/if}

		<form onsubmit={handleSubmit}>
			<div class="form-group">
				<label for="email">Email</label>
				<input
					type="email"
					id="email"
					bind:value={email}
					required
					placeholder="you@example.com"
					disabled={isLoading}
					autocomplete="email"
				/>
			</div>

			<div class="form-group">
				<label for="password">Password</label>
				<input
					type="password"
					id="password"
					bind:value={password}
					required
					placeholder="Enter your password"
					disabled={isLoading}
					autocomplete="current-password"
				/>
			</div>

			<button type="submit" class="submit-button" disabled={isLoading}>
				{isLoading ? 'Signing in...' : 'Sign In'}
			</button>
		</form>

		<p class="register-link">
			Don't have an account? <a href="/register">Create one</a>
		</p>
	</div>
</div>

<style>
	.login-page {
		min-height: 100vh;
		display: flex;
		align-items: center;
		justify-content: center;
		background-color: #f3f4f6;
		padding: 1rem;
	}

	.login-card {
		background-color: white;
		padding: 2rem;
		border-radius: 0.5rem;
		box-shadow:
			0 4px 6px -1px rgb(0 0 0 / 0.1),
			0 2px 4px -2px rgb(0 0 0 / 0.1);
		width: 100%;
		max-width: 24rem;
	}

	.logo {
		display: block;
		max-height: 3rem;
		margin: 0 auto 1.5rem;
	}

	.company-name {
		text-align: center;
		font-size: 1.5rem;
		font-weight: 700;
		color: var(--color-primary, #3b82f6);
		margin: 0 0 1.5rem;
	}

	.title {
		text-align: center;
		font-size: 1.25rem;
		font-weight: 600;
		color: #374151;
		margin: 0 0 1.5rem;
	}

	.error-message {
		margin-bottom: 1rem;
		padding: 0.75rem;
		background-color: #fef2f2;
		border: 1px solid #fecaca;
		border-radius: 0.375rem;
		color: #dc2626;
		font-size: 0.875rem;
	}

	.form-group {
		margin-bottom: 1rem;
	}

	.form-group label {
		display: block;
		font-size: 0.875rem;
		font-weight: 500;
		color: #374151;
		margin-bottom: 0.25rem;
	}

	.form-group input {
		width: 100%;
		padding: 0.5rem 0.75rem;
		border: 1px solid #d1d5db;
		border-radius: 0.375rem;
		font-size: 1rem;
		box-sizing: border-box;
	}

	.form-group input:focus {
		outline: none;
		border-color: var(--color-primary, #3b82f6);
		box-shadow: 0 0 0 2px rgb(59 130 246 / 0.2);
	}

	.form-group input:disabled {
		background-color: #f9fafb;
		cursor: not-allowed;
	}

	.submit-button {
		width: 100%;
		padding: 0.625rem 1rem;
		background-color: var(--color-primary, #3b82f6);
		color: white;
		font-weight: 500;
		border: none;
		border-radius: 0.375rem;
		cursor: pointer;
		font-size: 1rem;
		margin-top: 0.5rem;
	}

	.submit-button:hover:not(:disabled) {
		background-color: var(--color-secondary, #1e40af);
	}

	.submit-button:focus {
		outline: none;
		box-shadow: 0 0 0 2px rgb(59 130 246 / 0.5);
	}

	.submit-button:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.register-link {
		text-align: center;
		margin-top: 1.5rem;
		font-size: 0.875rem;
		color: #6b7280;
	}

	.register-link a {
		color: var(--color-primary, #3b82f6);
		text-decoration: none;
		font-weight: 500;
	}

	.register-link a:hover {
		text-decoration: underline;
	}
</style>
