<script lang="ts">
	import { enhance } from '$app/forms';

	let { form } = $props();
	let isLoading = $state(false);
</script>

<div class="min-h-screen flex items-center justify-center bg-slate-900">
	<div class="bg-slate-800 p-8 rounded-lg shadow-xl w-full max-w-md">
		<h1 class="text-2xl font-bold text-center mb-2 text-white">Platform Admin</h1>
		<p class="text-slate-400 text-center mb-6">OFFLEASH Management Console</p>

		{#if form?.error}
			<div class="mb-4 p-3 bg-red-900/50 border border-red-500 text-red-200 rounded">
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
			<div class="mb-4">
				<label for="email" class="block text-sm font-medium text-slate-300 mb-1">
					Email
				</label>
				<input
					type="email"
					id="email"
					name="email"
					required
					class="w-full px-3 py-2 bg-slate-700 border border-slate-600 text-white rounded focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent"
					placeholder="admin@offleash.io"
					disabled={isLoading}
					value={form?.email ?? ''}
				/>
			</div>

			<div class="mb-6">
				<label for="password" class="block text-sm font-medium text-slate-300 mb-1">
					Password
				</label>
				<input
					type="password"
					id="password"
					name="password"
					required
					class="w-full px-3 py-2 bg-slate-700 border border-slate-600 text-white rounded focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent"
					placeholder="Enter your password"
					disabled={isLoading}
				/>
			</div>

			<button
				type="submit"
				disabled={isLoading}
				class="w-full bg-indigo-600 text-white py-2 px-4 rounded hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 focus:ring-offset-slate-800 disabled:opacity-50 disabled:cursor-not-allowed"
			>
				{isLoading ? 'Logging in...' : 'Login'}
			</button>
		</form>
	</div>
</div>
