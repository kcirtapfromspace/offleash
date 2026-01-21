<script lang="ts">
	import type { PageData, ActionData } from './$types';

	export let data: PageData;
	export let form: ActionData;

	let showAddForm = false;

	// Form fields
	let name = '';
	let address = '';
	let city = '';
	let state = '';
	let zip_code = '';
	let notes = '';
	let is_default = false;

	function resetForm() {
		name = '';
		address = '';
		city = '';
		state = '';
		zip_code = '';
		notes = '';
		is_default = false;
		showAddForm = false;
	}

	// Reset form on successful submission
	$: if (form?.success) {
		resetForm();
	}
</script>

<div class="max-w-4xl mx-auto">
	<div class="flex items-center justify-between mb-6">
		<div>
			<h1 class="text-2xl font-bold">My Locations</h1>
			<p class="text-gray-600">Saved addresses for your dog walking bookings.</p>
		</div>
		<button
			on:click={() => (showAddForm = !showAddForm)}
			class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
		>
			{showAddForm ? 'Cancel' : '+ Add Location'}
		</button>
	</div>

	{#if form?.error}
		<div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg mb-6">
			{form.error}
		</div>
	{/if}

	{#if form?.success}
		<div class="bg-green-50 border border-green-200 text-green-700 px-4 py-3 rounded-lg mb-6">
			Location added successfully!
		</div>
	{/if}

	{#if data.error}
		<div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg mb-6">
			{data.error}
		</div>
	{/if}

	<!-- Add Location Form -->
	{#if showAddForm}
		<form method="POST" action="?/add" class="bg-white border border-gray-200 rounded-xl p-6 mb-6">
			<h2 class="text-lg font-semibold mb-4">Add New Location</h2>
			<div class="grid gap-4 md:grid-cols-2">
				<div class="md:col-span-2">
					<label for="name" class="block text-sm font-medium text-gray-700 mb-1">Location Name</label>
					<input
						type="text"
						id="name"
						name="name"
						bind:value={name}
						placeholder="e.g., Home, Office, Park"
						class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
						required
					/>
				</div>
				<div class="md:col-span-2">
					<label for="address" class="block text-sm font-medium text-gray-700 mb-1">Street Address</label>
					<input
						type="text"
						id="address"
						name="address"
						bind:value={address}
						placeholder="123 Main St"
						class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
						required
					/>
				</div>
				<div>
					<label for="city" class="block text-sm font-medium text-gray-700 mb-1">City</label>
					<input
						type="text"
						id="city"
						name="city"
						bind:value={city}
						placeholder="Denver"
						class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
						required
					/>
				</div>
				<div class="grid grid-cols-2 gap-4">
					<div>
						<label for="state" class="block text-sm font-medium text-gray-700 mb-1">State</label>
						<input
							type="text"
							id="state"
							name="state"
							bind:value={state}
							placeholder="CO"
							maxlength="2"
							class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
							required
						/>
					</div>
					<div>
						<label for="zip_code" class="block text-sm font-medium text-gray-700 mb-1">ZIP Code</label>
						<input
							type="text"
							id="zip_code"
							name="zip_code"
							bind:value={zip_code}
							placeholder="80202"
							class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
							required
						/>
					</div>
				</div>
				<div class="md:col-span-2">
					<label for="notes" class="block text-sm font-medium text-gray-700 mb-1">Notes (Optional)</label>
					<textarea
						id="notes"
						name="notes"
						bind:value={notes}
						rows="2"
						placeholder="Gate code, parking instructions, etc."
						class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
					></textarea>
				</div>
				<div class="md:col-span-2">
					<label class="flex items-center">
						<input
							type="checkbox"
							name="is_default"
							bind:checked={is_default}
							class="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
						/>
						<span class="ml-2 text-sm text-gray-700">Set as default location</span>
					</label>
				</div>
			</div>
			<div class="mt-6 flex gap-3">
				<button
					type="submit"
					class="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
				>
					Save Location
				</button>
				<button
					type="button"
					on:click={resetForm}
					class="px-6 py-2 bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200 transition-colors"
				>
					Cancel
				</button>
			</div>
		</form>
	{/if}

	<!-- Locations List -->
	{#if data.locations.length === 0}
		<div class="text-center py-12 bg-gray-50 rounded-xl">
			<svg class="w-12 h-12 mx-auto text-gray-400 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z" />
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 11a3 3 0 11-6 0 3 3 0 016 0z" />
			</svg>
			<p class="text-gray-500 mb-4">No locations saved yet.</p>
			<button
				on:click={() => (showAddForm = true)}
				class="text-blue-600 hover:underline font-medium"
			>
				Add your first location
			</button>
		</div>
	{:else}
		<div class="space-y-4">
			{#each data.locations as location}
				<div class="bg-white border border-gray-200 rounded-xl p-5 hover:shadow-sm transition-shadow">
					<div class="flex items-start justify-between">
						<div>
							<div class="flex items-center gap-2 mb-1">
								<h3 class="font-semibold text-gray-900">{location.name}</h3>
								{#if location.is_default}
									<span class="px-2 py-0.5 text-xs bg-blue-100 text-blue-700 rounded-full">Default</span>
								{/if}
							</div>
							<p class="text-gray-600">{location.full_address}</p>
							{#if location.notes}
								<p class="text-sm text-gray-500 mt-2">
									<span class="font-medium">Notes:</span> {location.notes}
								</p>
							{/if}
						</div>
						<svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z" />
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 11a3 3 0 11-6 0 3 3 0 016 0z" />
						</svg>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>
