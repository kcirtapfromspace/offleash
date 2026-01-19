<script lang="ts">
	import { onMount } from 'svelte';
	import { get } from '$lib/api';
	import { formatCurrency, formatDuration } from '$lib/formatting';
	import type { Service } from '$lib/types';

	let services = $state<Service[]>([]);
	let isLoading = $state(true);
	let error = $state<string | null>(null);

	onMount(async () => {
		await fetchServices();
	});

	async function fetchServices() {
		isLoading = true;
		error = null;
		try {
			services = await get<Service[]>('/services');
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load services';
		} finally {
			isLoading = false;
		}
	}
</script>

<div class="p-6">
	<div class="mb-6">
		<h1 class="text-2xl font-bold text-gray-900">Services</h1>
		<p class="text-gray-600 mt-1">Manage your service offerings</p>
	</div>

	{#if error}
		<div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4">
			{error}
		</div>
	{/if}

	{#if isLoading}
		<div class="bg-white rounded-lg shadow-md overflow-hidden">
			<div class="animate-pulse">
				<div class="h-12 bg-gray-100 border-b border-gray-200"></div>
				{#each [1, 2, 3, 4, 5] as i (i)}
					<div class="flex border-b border-gray-200 p-4">
						<div class="h-4 bg-gray-200 rounded w-1/4 mr-4"></div>
						<div class="h-4 bg-gray-200 rounded w-1/6 mr-4"></div>
						<div class="h-4 bg-gray-200 rounded w-1/6 mr-4"></div>
						<div class="h-4 bg-gray-200 rounded w-1/6"></div>
					</div>
				{/each}
			</div>
		</div>
	{:else if services.length === 0}
		<div class="bg-white rounded-lg shadow-md p-8 text-center">
			<svg
				class="w-16 h-16 mx-auto text-gray-400 mb-4"
				fill="none"
				stroke="currentColor"
				viewBox="0 0 24 24"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"
				/>
			</svg>
			<h3 class="text-xl font-semibold text-gray-700 mb-2">No services found</h3>
			<p class="text-gray-500">Get started by creating your first service.</p>
		</div>
	{:else}
		<div class="bg-white rounded-lg shadow-md overflow-hidden">
			<table class="min-w-full divide-y divide-gray-200">
				<thead class="bg-gray-50">
					<tr>
						<th
							scope="col"
							class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
						>
							Name
						</th>
						<th
							scope="col"
							class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
						>
							Duration
						</th>
						<th
							scope="col"
							class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
						>
							Price
						</th>
						<th
							scope="col"
							class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
						>
							Status
						</th>
					</tr>
				</thead>
				<tbody class="bg-white divide-y divide-gray-200">
					{#each services as service (service.id)}
						<tr class="hover:bg-gray-50">
							<td class="px-6 py-4 whitespace-nowrap">
								<div class="text-sm font-medium text-gray-900">{service.name}</div>
								{#if service.description}
									<div class="text-sm text-gray-500">{service.description}</div>
								{/if}
							</td>
							<td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
								{formatDuration(service.duration_minutes)}
							</td>
							<td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
								{formatCurrency(service.base_price_cents)}
							</td>
							<td class="px-6 py-4 whitespace-nowrap">
								{#if service.is_active}
									<span
										class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-green-100 text-green-800"
									>
										Active
									</span>
								{:else}
									<span
										class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-gray-100 text-gray-800"
									>
										Inactive
									</span>
								{/if}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
</div>
