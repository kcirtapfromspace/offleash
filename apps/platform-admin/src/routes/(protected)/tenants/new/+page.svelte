<script lang="ts">
	import { enhance } from '$app/forms';

	let { form } = $props();
	let isLoading = $state(false);
</script>

<div>
	<h1 class="text-2xl font-bold text-white mb-6">Create New Tenant</h1>

	{#if form?.error}
		<div class="mb-4 p-3 bg-red-900/50 border border-red-500 text-red-200 rounded">
			{form.error}
		</div>
	{/if}

	<form
		method="POST"
		class="max-w-xl bg-slate-800 rounded-lg p-6"
		use:enhance={() => {
			isLoading = true;
			return async ({ update }) => {
				await update();
				isLoading = false;
			};
		}}
	>
		<div class="mb-4">
			<label for="name" class="block text-sm font-medium text-slate-300 mb-1">
				Organization Name
			</label>
			<input
				type="text"
				id="name"
				name="name"
				required
				class="w-full px-3 py-2 bg-slate-700 border border-slate-600 text-white rounded focus:outline-none focus:ring-2 focus:ring-indigo-500"
				disabled={isLoading}
				value={form?.name ?? ''}
			/>
		</div>

		<div class="mb-4">
			<label for="slug" class="block text-sm font-medium text-slate-300 mb-1">
				Subdomain (slug)
			</label>
			<input
				type="text"
				id="slug"
				name="slug"
				required
				pattern="[a-z0-9-]+"
				class="w-full px-3 py-2 bg-slate-700 border border-slate-600 text-white rounded focus:outline-none focus:ring-2 focus:ring-indigo-500"
				placeholder="acme"
				disabled={isLoading}
				value={form?.slug ?? ''}
			/>
			<p class="text-slate-400 text-xs mt-1">Lowercase letters, numbers, and hyphens only</p>
		</div>

		<div class="mb-4">
			<label for="adminEmail" class="block text-sm font-medium text-slate-300 mb-1">
				Admin Email
			</label>
			<input
				type="email"
				id="adminEmail"
				name="adminEmail"
				required
				class="w-full px-3 py-2 bg-slate-700 border border-slate-600 text-white rounded focus:outline-none focus:ring-2 focus:ring-indigo-500"
				disabled={isLoading}
				value={form?.adminEmail ?? ''}
			/>
		</div>

		<div class="mb-4">
			<label for="adminPassword" class="block text-sm font-medium text-slate-300 mb-1">
				Admin Password
			</label>
			<input
				type="password"
				id="adminPassword"
				name="adminPassword"
				required
				minlength="8"
				class="w-full px-3 py-2 bg-slate-700 border border-slate-600 text-white rounded focus:outline-none focus:ring-2 focus:ring-indigo-500"
				disabled={isLoading}
			/>
		</div>

		<div class="mb-6">
			<label for="subscriptionTier" class="block text-sm font-medium text-slate-300 mb-1">
				Subscription Tier
			</label>
			<select
				id="subscriptionTier"
				name="subscriptionTier"
				class="w-full px-3 py-2 bg-slate-700 border border-slate-600 text-white rounded focus:outline-none focus:ring-2 focus:ring-indigo-500"
				disabled={isLoading}
			>
				<option value="starter">Starter</option>
				<option value="growth">Growth</option>
				<option value="scale">Scale</option>
			</select>
		</div>

		<div class="flex space-x-4">
			<button
				type="submit"
				disabled={isLoading}
				class="px-4 py-2 bg-indigo-600 text-white rounded hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed"
			>
				{isLoading ? 'Creating...' : 'Create Tenant'}
			</button>
			<a
				href="/tenants"
				class="px-4 py-2 text-slate-300 hover:text-white"
			>
				Cancel
			</a>
		</div>
	</form>
</div>
