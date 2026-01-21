<script lang="ts">
	import type { PageData, ActionData } from './$types';
	import { enhance } from '$app/forms';
	import { goto } from '$app/navigation';

	export let data: PageData;
	export let form: ActionData;

	let searchInput = data.searchQuery || '';
	let selectedBooking: string | null = null;

	function formatDate(isoString: string): string {
		return new Date(isoString).toLocaleDateString('en-US', {
			weekday: 'short',
			month: 'short',
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

	function handleSearch(e: Event) {
		e.preventDefault();
		const params = new URLSearchParams();
		if (data.currentStatus !== 'all') params.set('status', data.currentStatus);
		if (searchInput) params.set('q', searchInput);
		goto(`/bookings?${params.toString()}`);
	}

	function canUpdateStatus(status: string): boolean {
		return !['cancelled', 'completed'].includes(status.toLowerCase());
	}
</script>

<div class="p-6">
	<div class="mb-6">
		<h1 class="text-2xl font-bold text-gray-900">Bookings Management</h1>
		<p class="text-gray-600">View and manage all bookings</p>
	</div>

	{#if form?.error}
		<div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg mb-6">
			{form.error}
		</div>
	{/if}

	{#if form?.success}
		<div class="bg-green-50 border border-green-200 text-green-700 px-4 py-3 rounded-lg mb-6">
			Booking updated successfully
		</div>
	{/if}

	{#if data.error}
		<div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg mb-6">
			{data.error}
		</div>
	{/if}

	<!-- Filters -->
	<div class="bg-white border border-gray-200 rounded-xl p-4 mb-6">
		<div class="flex flex-col md:flex-row gap-4">
			<!-- Search -->
			<form on:submit={handleSearch} class="flex-1">
				<div class="relative">
					<input
						type="text"
						bind:value={searchInput}
						placeholder="Search by customer, walker, or service..."
						class="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
					/>
					<svg class="w-5 h-5 text-gray-400 absolute left-3 top-2.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
					</svg>
				</div>
			</form>

			<!-- Status Filter -->
			<div class="flex items-center gap-2">
				<span class="text-sm text-gray-600">Status:</span>
				<div class="flex gap-1">
					<a
						href="/bookings?status=all{searchInput ? `&q=${searchInput}` : ''}"
						class="px-3 py-1.5 text-sm rounded-lg transition-colors {data.currentStatus === 'all'
							? 'bg-gray-900 text-white'
							: 'bg-gray-100 text-gray-700 hover:bg-gray-200'}"
					>
						All
					</a>
					{#each data.statuses as status}
						<a
							href="/bookings?status={status}{searchInput ? `&q=${searchInput}` : ''}"
							class="px-3 py-1.5 text-sm rounded-lg transition-colors {data.currentStatus === status
								? 'bg-gray-900 text-white'
								: 'bg-gray-100 text-gray-700 hover:bg-gray-200'}"
						>
							{formatStatus(status)}
						</a>
					{/each}
				</div>
			</div>
		</div>
	</div>

	<!-- Bookings Table -->
	<div class="bg-white border border-gray-200 rounded-xl overflow-hidden">
		{#if data.bookings.length === 0}
			<div class="p-8 text-center text-gray-500">
				<svg class="w-12 h-12 mx-auto text-gray-400 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
				</svg>
				<p>No bookings found</p>
			</div>
		{:else}
			<div class="overflow-x-auto">
				<table class="w-full">
					<thead class="bg-gray-50 border-b border-gray-200">
						<tr>
							<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Customer</th>
							<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Service</th>
							<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Walker</th>
							<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Date & Time</th>
							<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Price</th>
							<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Status</th>
							<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Actions</th>
						</tr>
					</thead>
					<tbody class="divide-y divide-gray-200">
						{#each data.bookings as booking}
							<tr class="hover:bg-gray-50">
								<td class="px-6 py-4 whitespace-nowrap">
									<p class="font-medium text-gray-900">{booking.customer_name}</p>
								</td>
								<td class="px-6 py-4 whitespace-nowrap">
									<p class="text-gray-900">{booking.service_name}</p>
								</td>
								<td class="px-6 py-4 whitespace-nowrap">
									<p class="text-gray-900">{booking.walker_name}</p>
								</td>
								<td class="px-6 py-4 whitespace-nowrap">
									<p class="text-gray-900">{formatDate(booking.scheduled_start)}</p>
									<p class="text-sm text-gray-500">{formatTime(booking.scheduled_start)} - {formatTime(booking.scheduled_end)}</p>
								</td>
								<td class="px-6 py-4 whitespace-nowrap">
									<p class="font-medium text-gray-900">{booking.price_display}</p>
								</td>
								<td class="px-6 py-4 whitespace-nowrap">
									<span class="px-2 py-1 text-xs font-medium rounded-full {getStatusColor(booking.status)}">
										{formatStatus(booking.status)}
									</span>
								</td>
								<td class="px-6 py-4 whitespace-nowrap">
									{#if canUpdateStatus(booking.status)}
										<button
											type="button"
											on:click={() => (selectedBooking = selectedBooking === booking.id ? null : booking.id)}
											class="text-blue-600 hover:text-blue-800 text-sm font-medium"
										>
											Update
										</button>
									{:else}
										<span class="text-gray-400 text-sm">-</span>
									{/if}
								</td>
							</tr>
							{#if selectedBooking === booking.id}
								<tr class="bg-gray-50">
									<td colspan="7" class="px-6 py-4">
										<div class="flex items-center gap-4">
											<span class="text-sm font-medium text-gray-700">Update Status:</span>
											<form method="POST" action="?/updateStatus" use:enhance class="flex gap-2">
												<input type="hidden" name="booking_id" value={booking.id} />
												{#if booking.status === 'pending'}
													<button
														type="submit"
														name="status"
														value="confirmed"
														class="px-3 py-1.5 text-sm bg-blue-600 text-white rounded-lg hover:bg-blue-700"
													>
														Confirm
													</button>
												{/if}
												{#if booking.status === 'confirmed'}
													<button
														type="submit"
														name="status"
														value="in_progress"
														class="px-3 py-1.5 text-sm bg-purple-600 text-white rounded-lg hover:bg-purple-700"
													>
														Start
													</button>
												{/if}
												{#if booking.status === 'in_progress'}
													<button
														type="submit"
														name="status"
														value="completed"
														class="px-3 py-1.5 text-sm bg-green-600 text-white rounded-lg hover:bg-green-700"
													>
														Complete
													</button>
												{/if}
											</form>
											{#if ['pending', 'confirmed'].includes(booking.status)}
												<form method="POST" action="?/cancel" use:enhance>
													<input type="hidden" name="booking_id" value={booking.id} />
													<button
														type="submit"
														class="px-3 py-1.5 text-sm text-red-600 border border-red-300 rounded-lg hover:bg-red-50"
													>
														Cancel
													</button>
												</form>
											{/if}
											<button
												type="button"
												on:click={() => (selectedBooking = null)}
												class="px-3 py-1.5 text-sm text-gray-600 hover:text-gray-800"
											>
												Close
											</button>
										</div>
									</td>
								</tr>
							{/if}
						{/each}
					</tbody>
				</table>
			</div>
		{/if}
	</div>

	<!-- Summary -->
	<div class="mt-4 text-sm text-gray-500">
		Showing {data.bookings.length} booking{data.bookings.length !== 1 ? 's' : ''}
	</div>
</div>
