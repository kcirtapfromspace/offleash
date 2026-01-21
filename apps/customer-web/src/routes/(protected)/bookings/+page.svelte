<script lang="ts">
	import type { PageData } from './$types';

	export let data: PageData;

	function formatDate(isoString: string): string {
		return new Date(isoString).toLocaleDateString('en-US', {
			weekday: 'short',
			month: 'short',
			day: 'numeric'
		});
	}

	function formatTime(isoString: string): string {
		return new Date(isoString).toLocaleTimeString('en-US', {
			hour: 'numeric',
			minute: '2-digit',
			hour12: true
		});
	}

	function getStatusColor(status: string): string {
		switch (status.toLowerCase()) {
			case 'pending':
				return 'bg-yellow-100 text-yellow-800';
			case 'confirmed':
				return 'bg-blue-100 text-blue-800';
			case 'in_progress':
				return 'bg-purple-100 text-purple-800';
			case 'completed':
				return 'bg-green-100 text-green-800';
			case 'cancelled':
				return 'bg-red-100 text-red-800';
			default:
				return 'bg-gray-100 text-gray-800';
		}
	}

	function formatStatus(status: string): string {
		return status.replace('_', ' ').replace(/\b\w/g, (l) => l.toUpperCase());
	}
</script>

<div class="max-w-4xl mx-auto">
	<div class="flex items-center justify-between mb-6">
		<h1 class="text-2xl font-bold">My Bookings</h1>
		<a
			href="/bookings/new"
			class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
		>
			+ New Booking
		</a>
	</div>

	<!-- Filter Tabs -->
	<div class="flex space-x-2 mb-6">
		<a
			href="/bookings?filter=upcoming"
			class="px-4 py-2 rounded-lg font-medium transition-colors {data.filter === 'upcoming'
				? 'bg-blue-600 text-white'
				: 'bg-gray-100 text-gray-600 hover:bg-gray-200'}"
		>
			Upcoming
		</a>
		<a
			href="/bookings?filter=past"
			class="px-4 py-2 rounded-lg font-medium transition-colors {data.filter === 'past'
				? 'bg-blue-600 text-white'
				: 'bg-gray-100 text-gray-600 hover:bg-gray-200'}"
		>
			Past
		</a>
	</div>

	{#if data.error}
		<div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg mb-6">
			{data.error}
		</div>
	{/if}

	{#if data.bookings.length === 0}
		<div class="text-center py-12 bg-gray-50 rounded-xl">
			<svg class="w-12 h-12 mx-auto text-gray-400 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
			</svg>
			<p class="text-gray-500 mb-4">
				{data.filter === 'upcoming' ? 'No upcoming bookings.' : 'No past bookings.'}
			</p>
			{#if data.filter === 'upcoming'}
				<a href="/services" class="text-blue-600 hover:underline font-medium">
					Browse services to book
				</a>
			{/if}
		</div>
	{:else}
		<div class="space-y-4">
			{#each data.bookings as booking}
				<a
					href="/bookings/{booking.id}"
					class="block bg-white border border-gray-200 rounded-xl p-5 hover:shadow-md transition-shadow"
				>
					<div class="flex items-start justify-between mb-3">
						<div>
							<h3 class="font-semibold text-gray-900 text-lg">{booking.service_name}</h3>
							<p class="text-gray-600">with {booking.walker_name}</p>
						</div>
						<span class="px-3 py-1 text-sm font-medium rounded-full {getStatusColor(booking.status)}">
							{formatStatus(booking.status)}
						</span>
					</div>

					<div class="grid grid-cols-2 gap-4 text-sm">
						<div class="flex items-center text-gray-600">
							<svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
							</svg>
							{formatDate(booking.scheduled_start)}
						</div>
						<div class="flex items-center text-gray-600">
							<svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
							</svg>
							{formatTime(booking.scheduled_start)} - {formatTime(booking.scheduled_end)}
						</div>
						<div class="flex items-center text-gray-600">
							<svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z" />
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 11a3 3 0 11-6 0 3 3 0 016 0z" />
							</svg>
							<span class="truncate">{booking.location_address}</span>
						</div>
						<div class="flex items-center text-gray-900 font-medium">
							<svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
							</svg>
							{booking.price_display}
						</div>
					</div>
				</a>
			{/each}
		</div>
	{/if}
</div>
