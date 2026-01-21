<script lang="ts">
	import type { PageData, ActionData } from './$types';
	import { enhance } from '$app/forms';

	export let data: PageData;
	export let form: ActionData;

	let cancelling = false;
	let showCancelConfirm = false;

	function formatDate(isoString: string): string {
		return new Date(isoString).toLocaleDateString('en-US', {
			weekday: 'long',
			month: 'long',
			day: 'numeric',
			year: 'numeric'
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

	function canCancel(status: string): boolean {
		return ['pending', 'confirmed'].includes(status.toLowerCase());
	}
</script>

<div class="max-w-2xl mx-auto">
	<!-- Back Link -->
	<a href="/bookings" class="inline-flex items-center text-gray-600 hover:text-gray-900 mb-6">
		<svg class="w-5 h-5 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
			<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
		</svg>
		Back to Bookings
	</a>

	{#if form?.error}
		<div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg mb-6">
			{form.error}
		</div>
	{/if}

	{#if form?.success}
		<div class="bg-green-50 border border-green-200 text-green-700 px-4 py-3 rounded-lg mb-6">
			{form.message}
		</div>
	{/if}

	<!-- Booking Card -->
	<div class="bg-white border border-gray-200 rounded-xl overflow-hidden">
		<!-- Header -->
		<div class="bg-gray-50 px-6 py-4 border-b border-gray-200">
			<div class="flex items-center justify-between">
				<h1 class="text-xl font-bold text-gray-900">Booking Details</h1>
				<span class="px-3 py-1 text-sm font-medium rounded-full {getStatusColor(data.booking.status)}">
					{formatStatus(data.booking.status)}
				</span>
			</div>
		</div>

		<!-- Content -->
		<div class="p-6 space-y-6">
			<!-- Service Info -->
			<div>
				<h2 class="text-sm font-medium text-gray-500 uppercase tracking-wide mb-2">Service</h2>
				<div class="flex items-start justify-between">
					<div>
						<p class="text-lg font-semibold text-gray-900">{data.service?.name || 'Service'}</p>
						{#if data.service?.description}
							<p class="text-gray-600 text-sm mt-1">{data.service.description}</p>
						{/if}
					</div>
					<p class="text-lg font-bold text-gray-900">{data.booking.price_display}</p>
				</div>
			</div>

			<!-- Walker Info -->
			<div>
				<h2 class="text-sm font-medium text-gray-500 uppercase tracking-wide mb-2">Walker</h2>
				<div class="flex items-center">
					<div class="w-10 h-10 bg-blue-100 rounded-full flex items-center justify-center mr-3">
						<svg class="w-5 h-5 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
						</svg>
					</div>
					<p class="font-medium text-gray-900">{data.walkerName}</p>
				</div>
			</div>

			<!-- Date & Time -->
			<div>
				<h2 class="text-sm font-medium text-gray-500 uppercase tracking-wide mb-2">Date & Time</h2>
				<div class="space-y-2">
					<div class="flex items-center text-gray-700">
						<svg class="w-5 h-5 mr-3 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
						</svg>
						{formatDate(data.booking.scheduled_start)}
					</div>
					<div class="flex items-center text-gray-700">
						<svg class="w-5 h-5 mr-3 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
						</svg>
						{formatTime(data.booking.scheduled_start)} - {formatTime(data.booking.scheduled_end)}
					</div>
				</div>
			</div>

			<!-- Location -->
			<div>
				<h2 class="text-sm font-medium text-gray-500 uppercase tracking-wide mb-2">Location</h2>
				<div class="flex items-start">
					<svg class="w-5 h-5 mr-3 text-gray-400 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z" />
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 11a3 3 0 11-6 0 3 3 0 016 0z" />
					</svg>
					<div>
						{#if data.location}
							<p class="font-medium text-gray-900">{data.location.name}</p>
							<p class="text-gray-600">{data.location.full_address}</p>
						{:else}
							<p class="text-gray-600">Location details unavailable</p>
						{/if}
					</div>
				</div>
			</div>

			<!-- Notes -->
			{#if data.booking.notes}
				<div>
					<h2 class="text-sm font-medium text-gray-500 uppercase tracking-wide mb-2">Notes</h2>
					<p class="text-gray-700 bg-gray-50 rounded-lg p-3">{data.booking.notes}</p>
				</div>
			{/if}
		</div>

		<!-- Actions -->
		{#if canCancel(data.booking.status)}
			<div class="px-6 py-4 bg-gray-50 border-t border-gray-200">
				{#if !showCancelConfirm}
					<button
						type="button"
						on:click={() => (showCancelConfirm = true)}
						class="px-4 py-2 text-red-600 border border-red-300 rounded-lg hover:bg-red-50 transition-colors"
					>
						Cancel Booking
					</button>
				{:else}
					<div class="bg-red-50 border border-red-200 rounded-lg p-4">
						<p class="text-red-800 font-medium mb-3">Are you sure you want to cancel this booking?</p>
						<div class="flex gap-3">
							<form
								method="POST"
								action="?/cancel"
								use:enhance={() => {
									cancelling = true;
									return async ({ update }) => {
										await update();
										cancelling = false;
										showCancelConfirm = false;
									};
								}}
							>
								<button
									type="submit"
									disabled={cancelling}
									class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors disabled:opacity-50"
								>
									{cancelling ? 'Cancelling...' : 'Yes, Cancel'}
								</button>
							</form>
							<button
								type="button"
								on:click={() => (showCancelConfirm = false)}
								class="px-4 py-2 bg-white border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 transition-colors"
							>
								No, Keep It
							</button>
						</div>
					</div>
				{/if}
			</div>
		{/if}
	</div>
</div>
