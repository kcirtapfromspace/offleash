<script lang="ts">
	import type { PageData, ActionData } from './$types';
	import { enhance } from '$app/forms';

	export let data: PageData;
	export let form: ActionData;

	let showCreateModal = false;
	let showEditModal = false;
	let editingService: (typeof data.services)[0] | null = null;
	let isSubmitting = false;

	// Form state for create modal
	let newService = {
		name: '',
		description: '',
		duration_minutes: 30,
		price: '25.00'
	};

	// Form state for edit modal
	let editService = {
		name: '',
		description: '',
		duration_minutes: 30,
		price: '25.00',
		is_active: true
	};

	function formatCurrency(cents: number): string {
		return new Intl.NumberFormat('en-US', {
			style: 'currency',
			currency: 'USD'
		}).format(cents / 100);
	}

	function formatDuration(minutes: number): string {
		if (minutes < 60) {
			return `${minutes} min`;
		}
		const hours = Math.floor(minutes / 60);
		const mins = minutes % 60;
		return mins > 0 ? `${hours}h ${mins}m` : `${hours}h`;
	}

	function openEditModal(service: (typeof data.services)[0]) {
		editingService = service;
		editService = {
			name: service.name,
			description: service.description || '',
			duration_minutes: service.duration_minutes,
			price: (service.price_cents / 100).toFixed(2),
			is_active: service.is_active
		};
		showEditModal = true;
	}

	function closeEditModal() {
		showEditModal = false;
		editingService = null;
	}

	function closeCreateModal() {
		showCreateModal = false;
		newService = {
			name: '',
			description: '',
			duration_minutes: 30,
			price: '25.00'
		};
	}

	// Close modals on success
	$: if (form?.success) {
		showCreateModal = false;
		showEditModal = false;
		editingService = null;
		isSubmitting = false;
	}
</script>

<div class="p-6">
	<div class="mb-6 flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold text-gray-900">Services</h1>
			<p class="text-gray-600">Manage your service offerings</p>
		</div>
		<button
			on:click={() => (showCreateModal = true)}
			class="inline-flex items-center px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors font-medium"
		>
			<svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
			</svg>
			Add Service
		</button>
	</div>

	{#if form?.error}
		<div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg mb-6">
			{form.error}
		</div>
	{/if}

	{#if form?.success}
		<div class="bg-green-50 border border-green-200 text-green-700 px-4 py-3 rounded-lg mb-6">
			{form.message || 'Success!'}
		</div>
	{/if}

	{#if data.error}
		<div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg mb-6">
			{data.error}
		</div>
	{/if}

	<!-- Stats -->
	<div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
		<div class="bg-white border border-gray-200 rounded-xl p-4">
			<p class="text-sm font-medium text-gray-500">Total Services</p>
			<p class="text-2xl font-bold text-gray-900">{data.services.length}</p>
		</div>
		<div class="bg-white border border-gray-200 rounded-xl p-4">
			<p class="text-sm font-medium text-gray-500">Active</p>
			<p class="text-2xl font-bold text-green-600">{data.services.filter((s) => s.is_active).length}</p>
		</div>
		<div class="bg-white border border-gray-200 rounded-xl p-4">
			<p class="text-sm font-medium text-gray-500">Inactive</p>
			<p class="text-2xl font-bold text-gray-400">{data.services.filter((s) => !s.is_active).length}</p>
		</div>
	</div>

	<!-- Services Grid -->
	{#if data.services.length === 0}
		<div class="bg-white border border-gray-200 rounded-xl p-8 text-center">
			<svg class="w-12 h-12 mx-auto text-gray-400 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"
				/>
			</svg>
			<p class="text-gray-500 mb-4">No services yet</p>
			<button
				on:click={() => (showCreateModal = true)}
				class="inline-flex items-center px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors font-medium"
			>
				<svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
				</svg>
				Create Your First Service
			</button>
		</div>
	{:else}
		<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
			{#each data.services as service}
				<div
					class="bg-white border border-gray-200 rounded-xl overflow-hidden hover:shadow-md transition-shadow {!service.is_active
						? 'opacity-60'
						: ''}"
				>
					<div class="p-6">
						<div class="flex items-start justify-between mb-4">
							<div>
								<h3 class="font-semibold text-gray-900 text-lg">{service.name}</h3>
								{#if service.description}
									<p class="text-sm text-gray-500 mt-1">{service.description}</p>
								{/if}
							</div>
							<span
								class="px-2 py-1 text-xs font-medium rounded-full {service.is_active
									? 'bg-green-100 text-green-800'
									: 'bg-gray-100 text-gray-600'}"
							>
								{service.is_active ? 'Active' : 'Inactive'}
							</span>
						</div>

						<div class="grid grid-cols-2 gap-4">
							<div>
								<p class="text-xs text-gray-500 uppercase tracking-wide">Duration</p>
								<p class="text-lg font-semibold text-gray-900">{formatDuration(service.duration_minutes)}</p>
							</div>
							<div>
								<p class="text-xs text-gray-500 uppercase tracking-wide">Price</p>
								<p class="text-lg font-semibold text-purple-600">{service.price_display}</p>
							</div>
						</div>
					</div>

					<div class="px-6 py-3 bg-gray-50 border-t border-gray-200 flex justify-between items-center">
						<button
							on:click={() => openEditModal(service)}
							class="text-sm font-medium text-purple-600 hover:text-purple-800"
						>
							Edit
						</button>
						<form method="POST" action="?/toggleActive" use:enhance>
							<input type="hidden" name="service_id" value={service.id} />
							<input type="hidden" name="is_active" value={service.is_active.toString()} />
							<button
								type="submit"
								class="text-sm font-medium {service.is_active
									? 'text-red-600 hover:text-red-800'
									: 'text-green-600 hover:text-green-800'}"
							>
								{service.is_active ? 'Deactivate' : 'Activate'}
							</button>
						</form>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

<!-- Create Service Modal -->
{#if showCreateModal}
	<div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
		<div class="bg-white rounded-xl max-w-md w-full p-6">
			<div class="flex items-center justify-between mb-6">
				<h2 class="text-xl font-bold text-gray-900">Add New Service</h2>
				<button on:click={closeCreateModal} class="text-gray-400 hover:text-gray-600">
					<svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
					</svg>
				</button>
			</div>

			<form
				method="POST"
				action="?/create"
				use:enhance={() => {
					isSubmitting = true;
					return async ({ result, update }) => {
						isSubmitting = false;
						if (result.type === 'success') {
							closeCreateModal();
						}
						await update();
					};
				}}
			>
				{#if form?.error && showCreateModal}
					<div class="mb-4 p-3 bg-red-50 border border-red-200 text-red-700 rounded-lg text-sm">
						{form.error}
					</div>
				{/if}

				<div class="space-y-4">
					<div>
						<label for="name" class="block text-sm font-medium text-gray-700 mb-1">Service Name</label>
						<input
							type="text"
							id="name"
							name="name"
							bind:value={newService.name}
							required
							class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-purple-500"
							placeholder="e.g., 30 Minute Walk"
						/>
					</div>

					<div>
						<label for="description" class="block text-sm font-medium text-gray-700 mb-1">Description (Optional)</label>
						<textarea
							id="description"
							name="description"
							bind:value={newService.description}
							rows="2"
							class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-purple-500"
							placeholder="A brief description of the service..."
						></textarea>
					</div>

					<div class="grid grid-cols-2 gap-4">
						<div>
							<label for="duration_minutes" class="block text-sm font-medium text-gray-700 mb-1">Duration (min)</label>
							<input
								type="number"
								id="duration_minutes"
								name="duration_minutes"
								bind:value={newService.duration_minutes}
								min="5"
								step="5"
								required
								class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-purple-500"
							/>
						</div>
						<div>
							<label for="price" class="block text-sm font-medium text-gray-700 mb-1">Price ($)</label>
							<input
								type="number"
								id="price"
								name="price"
								bind:value={newService.price}
								min="0"
								step="0.01"
								required
								class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-purple-500"
							/>
						</div>
					</div>
				</div>

				<div class="mt-6 flex gap-3">
					<button
						type="button"
						on:click={closeCreateModal}
						class="flex-1 px-4 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 transition-colors font-medium"
					>
						Cancel
					</button>
					<button
						type="submit"
						disabled={isSubmitting}
						class="flex-1 px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors font-medium disabled:opacity-50"
					>
						{isSubmitting ? 'Creating...' : 'Create Service'}
					</button>
				</div>
			</form>
		</div>
	</div>
{/if}

<!-- Edit Service Modal -->
{#if showEditModal && editingService}
	<div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
		<div class="bg-white rounded-xl max-w-md w-full p-6">
			<div class="flex items-center justify-between mb-6">
				<h2 class="text-xl font-bold text-gray-900">Edit Service</h2>
				<button on:click={closeEditModal} class="text-gray-400 hover:text-gray-600">
					<svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
					</svg>
				</button>
			</div>

			{#if form?.error && showEditModal}
				<div class="mb-4 p-3 bg-red-50 border border-red-200 text-red-700 rounded-lg text-sm">
					{form.error}
				</div>
			{/if}

			<form
				method="POST"
				action="?/update"
				use:enhance={() => {
					isSubmitting = true;
					return async ({ result, update }) => {
						isSubmitting = false;
						if (result.type === 'success') {
							closeEditModal();
						}
						await update();
					};
				}}
			>
				<input type="hidden" name="service_id" value={editingService.id} />

				<div class="space-y-4">
					<div>
						<label for="edit_name" class="block text-sm font-medium text-gray-700 mb-1">Service Name</label>
						<input
							type="text"
							id="edit_name"
							name="name"
							bind:value={editService.name}
							required
							class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-purple-500"
						/>
					</div>

					<div>
						<label for="edit_description" class="block text-sm font-medium text-gray-700 mb-1"
							>Description (Optional)</label
						>
						<textarea
							id="edit_description"
							name="description"
							bind:value={editService.description}
							rows="2"
							class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-purple-500"
						></textarea>
					</div>

					<div class="grid grid-cols-2 gap-4">
						<div>
							<label for="edit_duration_minutes" class="block text-sm font-medium text-gray-700 mb-1"
								>Duration (min)</label
							>
							<input
								type="number"
								id="edit_duration_minutes"
								name="duration_minutes"
								bind:value={editService.duration_minutes}
								min="5"
								step="5"
								required
								class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-purple-500"
							/>
						</div>
						<div>
							<label for="edit_price" class="block text-sm font-medium text-gray-700 mb-1">Price ($)</label>
							<input
								type="number"
								id="edit_price"
								name="price"
								bind:value={editService.price}
								min="0"
								step="0.01"
								required
								class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-purple-500"
							/>
						</div>
					</div>

					<div class="flex items-center">
						<input
							type="checkbox"
							id="edit_is_active"
							name="is_active"
							value="true"
							checked={editService.is_active}
							on:change={(e) => (editService.is_active = e.currentTarget.checked)}
							class="w-4 h-4 text-purple-600 border-gray-300 rounded focus:ring-purple-500"
						/>
						<label for="edit_is_active" class="ml-2 text-sm text-gray-700">Service is active</label>
					</div>
				</div>

				<div class="mt-6 flex gap-3">
					<button
						type="button"
						on:click={closeEditModal}
						class="flex-1 px-4 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 transition-colors font-medium"
					>
						Cancel
					</button>
					<button
						type="submit"
						disabled={isSubmitting}
						class="flex-1 px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors font-medium disabled:opacity-50"
					>
						{isSubmitting ? 'Saving...' : 'Save Changes'}
					</button>
				</div>
			</form>
		</div>
	</div>
{/if}
