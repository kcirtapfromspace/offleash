<script lang="ts">
	import type { PageData, ActionData } from './$types';

	export let data: PageData;
	export let form: ActionData;

	let showAddForm = false;
	let editingLocation: typeof data.locations[0] | null = null;
	let showDeleteConfirm: string | null = null;

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
		editingLocation = null;
	}

	function startEdit(location: typeof data.locations[0]) {
		editingLocation = location;
		name = location.name;
		address = location.address;
		city = location.city;
		state = location.state;
		zip_code = location.zip_code;
		notes = location.notes || '';
		is_default = location.is_default;
		showAddForm = false;
	}

	function cancelEdit() {
		editingLocation = null;
		resetForm();
	}

	// Reset form on successful submission
	$: if (form?.success) {
		resetForm();
		showDeleteConfirm = null;
	}
</script>

<div class="max-w-4xl mx-auto">
	<div class="flex items-center justify-between mb-6">
		<div>
			<h1 class="text-2xl font-bold">My Locations</h1>
			<p class="text-gray-600">Saved addresses for your dog walking bookings.</p>
		</div>
		<button
			on:click={() => { showAddForm = !showAddForm; editingLocation = null; resetForm(); }}
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

	{#if form?.success && !form?.deleted && !form?.updated}
		<div class="bg-green-50 border border-green-200 text-green-700 px-4 py-3 rounded-lg mb-6">
			Location added successfully!
		</div>
	{/if}

	{#if form?.success && form?.updated}
		<div class="bg-green-50 border border-green-200 text-green-700 px-4 py-3 rounded-lg mb-6">
			Location updated successfully!
		</div>
	{/if}

	{#if form?.success && form?.deleted}
		<div class="bg-green-50 border border-green-200 text-green-700 px-4 py-3 rounded-lg mb-6">
			Location deleted successfully!
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
				<!-- Edit Form for this location -->
				{#if editingLocation?.id === location.id}
					<form method="POST" action="?/update" class="bg-white border-2 border-blue-400 rounded-xl p-6">
						<input type="hidden" name="id" value={location.id} />
						<h2 class="text-lg font-semibold mb-4">Edit Location</h2>
						<div class="grid gap-4 md:grid-cols-2">
							<div class="md:col-span-2">
								<label for="edit-name" class="block text-sm font-medium text-gray-700 mb-1">Location Name</label>
								<input
									type="text"
									id="edit-name"
									name="name"
									bind:value={name}
									placeholder="e.g., Home, Office, Park"
									class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
									required
								/>
							</div>
							<div class="md:col-span-2">
								<label for="edit-address" class="block text-sm font-medium text-gray-700 mb-1">Street Address</label>
								<input
									type="text"
									id="edit-address"
									name="address"
									bind:value={address}
									placeholder="123 Main St"
									class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
									required
								/>
							</div>
							<div>
								<label for="edit-city" class="block text-sm font-medium text-gray-700 mb-1">City</label>
								<input
									type="text"
									id="edit-city"
									name="city"
									bind:value={city}
									placeholder="Denver"
									class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
									required
								/>
							</div>
							<div class="grid grid-cols-2 gap-4">
								<div>
									<label for="edit-state" class="block text-sm font-medium text-gray-700 mb-1">State</label>
									<input
										type="text"
										id="edit-state"
										name="state"
										bind:value={state}
										placeholder="CO"
										maxlength="2"
										class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
										required
									/>
								</div>
								<div>
									<label for="edit-zip_code" class="block text-sm font-medium text-gray-700 mb-1">ZIP Code</label>
									<input
										type="text"
										id="edit-zip_code"
										name="zip_code"
										bind:value={zip_code}
										placeholder="80202"
										class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
										required
									/>
								</div>
							</div>
							<div class="md:col-span-2">
								<label for="edit-notes" class="block text-sm font-medium text-gray-700 mb-1">Notes (Optional)</label>
								<textarea
									id="edit-notes"
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
								Save Changes
							</button>
							<button
								type="button"
								on:click={cancelEdit}
								class="px-6 py-2 bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200 transition-colors"
							>
								Cancel
							</button>
						</div>
					</form>
				{:else}
					<!-- Display location card -->
					<div class="bg-white border border-gray-200 rounded-xl p-5 hover:shadow-sm transition-shadow">
						<div class="flex items-start justify-between">
							<div class="flex-1">
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
							<div class="flex items-center gap-2">
								<!-- Set as Default button (if not already default) -->
								{#if !location.is_default}
									<form method="POST" action="?/setDefault" class="inline">
										<input type="hidden" name="id" value={location.id} />
										<button
											type="submit"
											class="p-2 text-gray-400 hover:text-blue-600 hover:bg-blue-50 rounded-lg transition-colors"
											title="Set as default"
										>
											<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
												<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z" />
											</svg>
										</button>
									</form>
								{/if}
								<!-- Edit button -->
								<button
									on:click={() => startEdit(location)}
									class="p-2 text-gray-400 hover:text-blue-600 hover:bg-blue-50 rounded-lg transition-colors"
									title="Edit location"
								>
									<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
									</svg>
								</button>
								<!-- Delete button -->
								<button
									on:click={() => showDeleteConfirm = location.id}
									class="p-2 text-gray-400 hover:text-red-600 hover:bg-red-50 rounded-lg transition-colors"
									title="Delete location"
								>
									<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
									</svg>
								</button>
							</div>
						</div>

						<!-- Delete Confirmation -->
						{#if showDeleteConfirm === location.id}
							<div class="mt-4 p-4 bg-red-50 border border-red-200 rounded-lg">
								<p class="text-red-800 mb-3">Are you sure you want to delete "{location.name}"?</p>
								<div class="flex gap-3">
									<form method="POST" action="?/delete">
										<input type="hidden" name="id" value={location.id} />
										<button
											type="submit"
											class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors"
										>
											Yes, Delete
										</button>
									</form>
									<button
										on:click={() => showDeleteConfirm = null}
										class="px-4 py-2 bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200 transition-colors"
									>
										Cancel
									</button>
								</div>
							</div>
						{/if}
					</div>
				{/if}
			{/each}
		</div>
	{/if}
</div>
