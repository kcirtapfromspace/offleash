<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
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

	async function handleSubmit(e: SubmitEvent) {
		e.preventDefault();

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

<div
	class="min-h-screen flex items-center justify-center bg-gray-100 py-12 px-4"
	style={cssVariables}
>
	<div class="bg-white p-8 rounded-lg shadow-md w-full max-w-md">
		{#if branding.logo_url}
			<img src={branding.logo_url} alt={branding.company_name} class="h-12 mx-auto mb-6" />
		{:else}
			<h1 class="text-2xl font-bold text-center mb-2" style="color: var(--color-primary)">
				{branding.company_name}
			</h1>
		{/if}

		<h2 class="text-xl font-semibold text-center text-gray-700 mb-6">Create your account</h2>

		{#if errors.general}
			<div class="mb-4 p-3 bg-red-100 border border-red-400 text-red-700 rounded text-sm">
				{errors.general}
			</div>
		{/if}

		<form onsubmit={handleSubmit}>
			<div class="grid grid-cols-2 gap-4 mb-4">
				<div>
					<label for="first_name" class="block text-sm font-medium text-gray-700 mb-1">
						First Name
					</label>
					<input
						type="text"
						id="first_name"
						bind:value={first_name}
						required
						class="w-full px-3 py-2 border rounded focus:outline-none focus:ring-2 {errors.first_name
							? 'border-red-500'
							: 'border-gray-300'}"
						style="--tw-ring-color: var(--color-primary)"
						placeholder="John"
						disabled={isLoading}
					/>
					{#if errors.first_name}
						<p class="mt-1 text-xs text-red-600">{errors.first_name}</p>
					{/if}
				</div>

				<div>
					<label for="last_name" class="block text-sm font-medium text-gray-700 mb-1">
						Last Name
					</label>
					<input
						type="text"
						id="last_name"
						bind:value={last_name}
						required
						class="w-full px-3 py-2 border rounded focus:outline-none focus:ring-2 {errors.last_name
							? 'border-red-500'
							: 'border-gray-300'}"
						style="--tw-ring-color: var(--color-primary)"
						placeholder="Doe"
						disabled={isLoading}
					/>
					{#if errors.last_name}
						<p class="mt-1 text-xs text-red-600">{errors.last_name}</p>
					{/if}
				</div>
			</div>

			<div class="mb-4">
				<label for="email" class="block text-sm font-medium text-gray-700 mb-1">Email</label>
				<input
					type="email"
					id="email"
					bind:value={email}
					required
					class="w-full px-3 py-2 border rounded focus:outline-none focus:ring-2 {errors.email
						? 'border-red-500'
						: 'border-gray-300'}"
					style="--tw-ring-color: var(--color-primary)"
					placeholder="john@example.com"
					disabled={isLoading}
				/>
				{#if errors.email}
					<p class="mt-1 text-xs text-red-600">{errors.email}</p>
				{/if}
			</div>

			<div class="mb-4">
				<label for="phone" class="block text-sm font-medium text-gray-700 mb-1">Phone</label>
				<input
					type="tel"
					id="phone"
					bind:value={phone}
					required
					class="w-full px-3 py-2 border rounded focus:outline-none focus:ring-2 {errors.phone
						? 'border-red-500'
						: 'border-gray-300'}"
					style="--tw-ring-color: var(--color-primary)"
					placeholder="(555) 123-4567"
					disabled={isLoading}
				/>
				{#if errors.phone}
					<p class="mt-1 text-xs text-red-600">{errors.phone}</p>
				{/if}
			</div>

			<div class="mb-4">
				<label for="password" class="block text-sm font-medium text-gray-700 mb-1">Password</label>
				<input
					type="password"
					id="password"
					bind:value={password}
					required
					class="w-full px-3 py-2 border rounded focus:outline-none focus:ring-2 {errors.password
						? 'border-red-500'
						: 'border-gray-300'}"
					style="--tw-ring-color: var(--color-primary)"
					placeholder="At least 8 characters"
					disabled={isLoading}
				/>
				{#if errors.password}
					<p class="mt-1 text-xs text-red-600">{errors.password}</p>
				{/if}
			</div>

			<div class="mb-6">
				<label for="confirm_password" class="block text-sm font-medium text-gray-700 mb-1">
					Confirm Password
				</label>
				<input
					type="password"
					id="confirm_password"
					bind:value={confirm_password}
					required
					class="w-full px-3 py-2 border rounded focus:outline-none focus:ring-2 {errors.confirm_password
						? 'border-red-500'
						: 'border-gray-300'}"
					style="--tw-ring-color: var(--color-primary)"
					placeholder="Confirm your password"
					disabled={isLoading}
				/>
				{#if errors.confirm_password}
					<p class="mt-1 text-xs text-red-600">{errors.confirm_password}</p>
				{/if}
			</div>

			<button
				type="submit"
				disabled={isLoading}
				class="w-full text-white py-2 px-4 rounded focus:outline-none focus:ring-2 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
				style="background-color: var(--color-primary); --tw-ring-color: var(--color-primary)"
			>
				{isLoading ? 'Creating account...' : 'Create Account'}
			</button>
		</form>

		<p class="mt-6 text-center text-sm text-gray-600">
			Already have an account?
			<a href="/login" class="font-medium hover:underline" style="color: var(--color-primary)">
				Sign in
			</a>
		</p>

		{#if branding.support_email}
			<p class="mt-4 text-center text-xs text-gray-500">
				Need help?
				<a
					href="mailto:{branding.support_email}"
					class="hover:underline"
					style="color: var(--color-accent)"
				>
					Contact support
				</a>
			</p>
		{/if}
	</div>
</div>
