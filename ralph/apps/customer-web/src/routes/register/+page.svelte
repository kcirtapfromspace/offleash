<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { post, setToken, ApiError } from '$lib/api';
	import { generateCssVariables } from '$lib/stores/branding';

	interface RegisterRequest {
		first_name: string;
		last_name: string;
		email: string;
		phone: string;
		password: string;
	}

	interface RegisterResponse {
		token: string;
	}

	interface FormErrors {
		first_name?: string;
		last_name?: string;
		email?: string;
		phone?: string;
		password?: string;
		confirm_password?: string;
		general?: string;
	}

	let first_name = $state('');
	let last_name = $state('');
	let email = $state('');
	let phone = $state('');
	let password = $state('');
	let confirm_password = $state('');
	let isLoading = $state(false);
	let errors = $state<FormErrors>({});

	const branding = $derived($page.data.branding);
	const cssVariables = $derived(generateCssVariables(branding));

	function validateForm(): boolean {
		const newErrors: FormErrors = {};

		if (!first_name.trim()) {
			newErrors.first_name = 'First name is required';
		}

		if (!last_name.trim()) {
			newErrors.last_name = 'Last name is required';
		}

		if (!email.trim()) {
			newErrors.email = 'Email is required';
		} else if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
			newErrors.email = 'Please enter a valid email address';
		}

		if (!phone.trim()) {
			newErrors.phone = 'Phone number is required';
		}

		if (!password) {
			newErrors.password = 'Password is required';
		} else if (password.length < 8) {
			newErrors.password = 'Password must be at least 8 characters';
		}

		if (!confirm_password) {
			newErrors.confirm_password = 'Please confirm your password';
		} else if (password !== confirm_password) {
			newErrors.confirm_password = 'Passwords do not match';
		}

		errors = newErrors;
		return Object.keys(newErrors).length === 0;
	}

	async function handleSubmit(event: SubmitEvent) {
		event.preventDefault();

		if (!validateForm()) {
			return;
		}

		isLoading = true;
		errors = {};

		try {
			const response = await post<RegisterResponse>('/auth/register', {
				first_name: first_name.trim(),
				last_name: last_name.trim(),
				email: email.trim(),
				phone: phone.trim(),
				password
			} as RegisterRequest);

			setToken(response.token);
			await goto('/');
		} catch (err) {
			if (err instanceof ApiError) {
				if (err.status === 409) {
					errors = { email: 'An account with this email already exists' };
				} else if (err.status === 400) {
					errors = { general: err.message || 'Invalid registration data' };
				} else {
					errors = { general: err.message || 'Registration failed. Please try again.' };
				}
			} else {
				errors = { general: 'An unexpected error occurred. Please try again.' };
			}
		} finally {
			isLoading = false;
		}
	}
</script>

<svelte:head>
	<title>Register - {branding.company_name}</title>
</svelte:head>

<div class="register-page" style={cssVariables}>
	<div class="register-card">
		{#if branding.logo_url}
			<img src={branding.logo_url} alt={branding.company_name} class="logo" />
		{:else}
			<h1 class="company-name">{branding.company_name}</h1>
		{/if}

		<h2 class="title">Create your account</h2>

		{#if errors.general}
			<div class="error-message" role="alert">
				{errors.general}
			</div>
		{/if}

		<form onsubmit={handleSubmit}>
			<div class="name-row">
				<div class="form-group">
					<label for="first_name">First Name</label>
					<input
						type="text"
						id="first_name"
						bind:value={first_name}
						required
						placeholder="John"
						disabled={isLoading}
						autocomplete="given-name"
						class:input-error={errors.first_name}
					/>
					{#if errors.first_name}
						<span class="field-error">{errors.first_name}</span>
					{/if}
				</div>

				<div class="form-group">
					<label for="last_name">Last Name</label>
					<input
						type="text"
						id="last_name"
						bind:value={last_name}
						required
						placeholder="Doe"
						disabled={isLoading}
						autocomplete="family-name"
						class:input-error={errors.last_name}
					/>
					{#if errors.last_name}
						<span class="field-error">{errors.last_name}</span>
					{/if}
				</div>
			</div>

			<div class="form-group">
				<label for="email">Email</label>
				<input
					type="email"
					id="email"
					bind:value={email}
					required
					placeholder="john@example.com"
					disabled={isLoading}
					autocomplete="email"
					class:input-error={errors.email}
				/>
				{#if errors.email}
					<span class="field-error">{errors.email}</span>
				{/if}
			</div>

			<div class="form-group">
				<label for="phone">Phone</label>
				<input
					type="tel"
					id="phone"
					bind:value={phone}
					required
					placeholder="(555) 123-4567"
					disabled={isLoading}
					autocomplete="tel"
					class:input-error={errors.phone}
				/>
				{#if errors.phone}
					<span class="field-error">{errors.phone}</span>
				{/if}
			</div>

			<div class="form-group">
				<label for="password">Password</label>
				<input
					type="password"
					id="password"
					bind:value={password}
					required
					placeholder="At least 8 characters"
					disabled={isLoading}
					autocomplete="new-password"
					class:input-error={errors.password}
				/>
				{#if errors.password}
					<span class="field-error">{errors.password}</span>
				{/if}
			</div>

			<div class="form-group">
				<label for="confirm_password">Confirm Password</label>
				<input
					type="password"
					id="confirm_password"
					bind:value={confirm_password}
					required
					placeholder="Confirm your password"
					disabled={isLoading}
					autocomplete="new-password"
					class:input-error={errors.confirm_password}
				/>
				{#if errors.confirm_password}
					<span class="field-error">{errors.confirm_password}</span>
				{/if}
			</div>

			<button type="submit" class="submit-button" disabled={isLoading}>
				{isLoading ? 'Creating account...' : 'Create Account'}
			</button>
		</form>

		<p class="login-link">
			Already have an account? <a href="/login">Sign in</a>
		</p>

		{#if branding.support_email}
			<p class="support-link">
				Need help? <a href="mailto:{branding.support_email}">Contact support</a>
			</p>
		{/if}
	</div>
</div>

<style>
	.register-page {
		min-height: 100vh;
		display: flex;
		align-items: center;
		justify-content: center;
		background-color: #f3f4f6;
		padding: 1rem;
	}

	.register-card {
		background-color: white;
		padding: 2rem;
		border-radius: 0.5rem;
		box-shadow:
			0 4px 6px -1px rgb(0 0 0 / 0.1),
			0 2px 4px -2px rgb(0 0 0 / 0.1);
		width: 100%;
		max-width: 28rem;
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

	.name-row {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 1rem;
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

	.form-group input.input-error {
		border-color: #ef4444;
	}

	.form-group input.input-error:focus {
		box-shadow: 0 0 0 2px rgb(239 68 68 / 0.2);
	}

	.field-error {
		display: block;
		font-size: 0.75rem;
		color: #dc2626;
		margin-top: 0.25rem;
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

	.login-link {
		text-align: center;
		margin-top: 1.5rem;
		font-size: 0.875rem;
		color: #6b7280;
	}

	.login-link a {
		color: var(--color-primary, #3b82f6);
		text-decoration: none;
		font-weight: 500;
	}

	.login-link a:hover {
		text-decoration: underline;
	}

	.support-link {
		text-align: center;
		margin-top: 1rem;
		font-size: 0.75rem;
		color: #9ca3af;
	}

	.support-link a {
		color: var(--color-accent, #10b981);
		text-decoration: none;
	}

	.support-link a:hover {
		text-decoration: underline;
	}
</style>
