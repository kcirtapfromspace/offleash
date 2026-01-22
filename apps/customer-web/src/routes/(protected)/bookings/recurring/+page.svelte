<script lang="ts">
	import type { PageData } from './$types';

	export let data: PageData;

	function formatTime(time: string): string {
		const [hours, minutes] = time.split(':');
		const hour = parseInt(hours);
		const ampm = hour >= 12 ? 'PM' : 'AM';
		const displayHour = hour % 12 || 12;
		return `${displayHour}:${minutes} ${ampm}`;
	}

	function formatNextOccurrence(dateStr: string | null): string {
		if (!dateStr) return 'No upcoming';
		const date = new Date(dateStr);
		return date.toLocaleDateString('en-US', {
			weekday: 'short',
			month: 'short',
			day: 'numeric',
			hour: 'numeric',
			minute: '2-digit'
		});
	}
</script>

<div class="max-w-4xl mx-auto">
	<div class="flex justify-between items-center mb-6">
		<div>
			<h1 class="text-2xl font-bold">Recurring Bookings</h1>
			<p class="text-gray-600">Manage your recurring booking series</p>
		</div>
		<a
			href="/bookings/new"
			class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
		>
			+ New Booking
		</a>
	</div>

	{#if data.error}
		<div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg mb-6">
			{data.error}
		</div>
	{/if}

	{#if data.series.length === 0}
		<div class="bg-white border border-gray-200 rounded-xl p-12 text-center">
			<div class="w-16 h-16 bg-gray-100 rounded-full flex items-center justify-center mx-auto mb-4">
				<svg class="w-8 h-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"
					/>
				</svg>
			</div>
			<h3 class="text-lg font-medium text-gray-900 mb-2">No recurring bookings</h3>
			<p class="text-gray-500 mb-6">Create your first recurring booking to have regular walks scheduled automatically.</p>
			<a
				href="/bookings/new"
				class="inline-block px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
			>
				Create Recurring Booking
			</a>
		</div>
	{:else}
		<div class="space-y-4">
			{#each data.series as item}
				<a
					href="/bookings/recurring/{item.id}"
					class="block bg-white border border-gray-200 rounded-xl p-6 hover:border-blue-300 hover:shadow-md transition-all"
				>
					<div class="flex justify-between items-start">
						<div class="flex-1">
							<div class="flex items-center gap-3 mb-2">
								<h3 class="text-lg font-semibold text-gray-900">{item.service_name}</h3>
								{#if item.is_active}
									<span class="px-2 py-1 text-xs font-medium bg-green-100 text-green-700 rounded-full">
										Active
									</span>
								{:else}
									<span class="px-2 py-1 text-xs font-medium bg-gray-100 text-gray-600 rounded-full">
										Inactive
									</span>
								{/if}
							</div>
							<p class="text-gray-600 mb-3">
								{item.frequency} on {item.day_of_week_name}s at {formatTime(item.time_of_day)}
							</p>
							<div class="flex items-center gap-4 text-sm text-gray-500">
								<span>Walker: {item.walker_name}</span>
								<span class="text-gray-300">|</span>
								<span>{item.total_bookings} bookings</span>
								<span class="text-gray-300">|</span>
								<span>{item.price_display} each</span>
							</div>
						</div>
						<div class="text-right">
							{#if item.next_occurrence}
								<p class="text-sm text-gray-500">Next</p>
								<p class="font-medium text-gray-900">{formatNextOccurrence(item.next_occurrence)}</p>
							{:else}
								<p class="text-sm text-gray-400">No upcoming bookings</p>
							{/if}
						</div>
					</div>
				</a>
			{/each}
		</div>
	{/if}

	<div class="mt-8 text-center">
		<a href="/bookings" class="text-blue-600 hover:underline">
			← Back to all bookings
		</a>
	</div>
</div>
