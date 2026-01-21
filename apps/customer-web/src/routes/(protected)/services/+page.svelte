<script lang="ts">
	import type { PageData } from './$types';

	export let data: PageData;

	function formatDuration(minutes: number): string {
		if (minutes < 60) return `${minutes} min`;
		const hours = Math.floor(minutes / 60);
		const mins = minutes % 60;
		return mins > 0 ? `${hours}h ${mins}m` : `${hours}h`;
	}
</script>

<div class="max-w-4xl mx-auto">
	<h1 class="text-2xl font-bold mb-2">Our Services</h1>
	<p class="text-gray-600 mb-8">Choose a service to book your dog's next adventure.</p>

	{#if data.error}
		<div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg mb-6">
			{data.error}
		</div>
	{/if}

	{#if data.services.length === 0 && !data.error}
		<div class="text-center py-12 bg-gray-50 rounded-lg">
			<p class="text-gray-500">No services available at the moment.</p>
		</div>
	{:else}
		<div class="grid gap-6 md:grid-cols-2">
			{#each data.services as service}
				<div class="bg-white border border-gray-200 rounded-xl p-6 shadow-sm hover:shadow-md transition-shadow">
					<div class="flex justify-between items-start mb-3">
						<h2 class="text-xl font-semibold text-gray-900">{service.name}</h2>
						<span class="text-lg font-bold text-green-600">{service.price_display}</span>
					</div>

					{#if service.description}
						<p class="text-gray-600 mb-4">{service.description}</p>
					{/if}

					<div class="flex items-center justify-between">
						<span class="text-sm text-gray-500">
							<svg class="w-4 h-4 inline mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
							</svg>
							{formatDuration(service.duration_minutes)}
						</span>
						<a
							href="/bookings/new?service={service.id}"
							class="inline-flex items-center px-4 py-2 bg-blue-600 text-white text-sm font-medium rounded-lg hover:bg-blue-700 transition-colors"
						>
							Book Now
							<svg class="w-4 h-4 ml-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
							</svg>
						</a>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>
