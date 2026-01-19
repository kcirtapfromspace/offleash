<script lang="ts">
	import { enhance } from '$app/forms';

	let { form } = $props();
	let isLoading = $state(false);
</script>

<div class="min-h-screen flex items-center justify-center bg-gray-100">
	<div class="bg-white p-8 rounded-lg shadow-md w-full max-w-md">
		<h1 class="text-2xl font-bold text-center mb-6" style="color: var(--color-primary)">
			Create Account
		</h1>

		{#if form?.error}
			<div class="mb-4 p-3 bg-red-100 border border-red-400 text-red-700 rounded">
				{form.error}
			</div>
		{/if}

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
			<div class="grid grid-cols-2 gap-4 mb-4">
				<div>
					<label for="firstName" class="block text-sm font-medium text-gray-700 mb-1">
						First Name
					</label>
					<input
						type="text"
						id="firstName"
						name="firstName"
						required
						class="w-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
						disabled={isLoading}
						value={form?.firstName ?? ''}
					/>
				</div>
				<div>
					<label for="lastName" class="block text-sm font-medium text-gray-700 mb-1">
						Last Name
					</label>
					<input
						type="text"
						id="lastName"
						name="lastName"
						required
						class="w-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
						disabled={isLoading}
						value={form?.lastName ?? ''}
					/>
				</div>
			</div>

			<div class="mb-4">
				<label for="email" class="block text-sm font-medium text-gray-700 mb-1">
					Email
				</label>
				<input
					type="email"
					id="email"
					name="email"
					required
					class="w-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
					placeholder="you@example.com"
					disabled={isLoading}
					value={form?.email ?? ''}
				/>
			</div>

			<div class="mb-4">
				<label for="phone" class="block text-sm font-medium text-gray-700 mb-1">
					Phone
				</label>
				<input
					type="tel"
					id="phone"
					name="phone"
					class="w-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
					placeholder="(555) 123-4567"
					disabled={isLoading}
					value={form?.phone ?? ''}
				/>
			</div>

			<div class="mb-4">
				<label for="password" class="block text-sm font-medium text-gray-700 mb-1">
					Password
				</label>
				<input
					type="password"
					id="password"
					name="password"
					required
					minlength="8"
					class="w-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
					disabled={isLoading}
				/>
			</div>

			<div class="mb-6">
				<label for="confirmPassword" class="block text-sm font-medium text-gray-700 mb-1">
					Confirm Password
				</label>
				<input
					type="password"
					id="confirmPassword"
					name="confirmPassword"
					required
					class="w-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
					disabled={isLoading}
				/>
			</div>

			<button
				type="submit"
				disabled={isLoading}
				class="w-full text-white py-2 px-4 rounded focus:outline-none focus:ring-2 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed"
				style="background-color: var(--color-primary)"
			>
				{isLoading ? 'Creating account...' : 'Register'}
			</button>
		</form>

		<p class="mt-4 text-center text-gray-600">
			Already have an account?
			<a href="/login" class="hover:underline" style="color: var(--color-primary)">
				Login
			</a>
		</p>
	</div>
</div>
