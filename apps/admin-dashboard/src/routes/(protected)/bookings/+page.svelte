<script lang="ts">
	import type { PageData, ActionData } from './$types';
	import { enhance } from '$app/forms';
	import { goto } from '$app/navigation';

	export let data: PageData;
	export let form: ActionData;

	let searchInput = data.searchQuery || '';
	let selectedBooking: string | null = null;
	let detailBooking: typeof data.bookings[0] | null = null;

	// Derived from data
	$: isAdmin = data.isAdmin;
	$: canAcceptBookings = data.canAcceptBookings;

	function openBookingDetail(booking: typeof data.bookings[0]) {
		detailBooking = booking;
	}

	function closeBookingDetail() {
		detailBooking = null;
	}

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
		<h1 class="text-2xl font-bold text-gray-900">{isAdmin ? 'Bookings Management' : 'My Bookings'}</h1>
		<p class="text-gray-600">{isAdmin ? 'View and manage all bookings' : 'View your upcoming and past bookings'}</p>
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
							{#if isAdmin}
								<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Walker</th>
							{/if}
							<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Date & Time</th>
							{#if isAdmin}
								<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Price</th>
							{/if}
							<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Status</th>
							<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Actions</th>
						</tr>
					</thead>
					<tbody class="divide-y divide-gray-200">
						{#each data.bookings as booking}
							<tr
								class="hover:bg-gray-50 cursor-pointer"
								on:click={() => openBookingDetail(booking)}
								on:keydown={(e) => e.key === 'Enter' && openBookingDetail(booking)}
								tabindex="0"
								role="button"
							>
								<td class="px-6 py-4 whitespace-nowrap">
									<p class="font-medium text-gray-900">{booking.customer_name}</p>
								</td>
								<td class="px-6 py-4 whitespace-nowrap">
									<p class="text-gray-900">{booking.service_name}</p>
								</td>
								{#if isAdmin}
									<td class="px-6 py-4 whitespace-nowrap">
										<p class="text-gray-900">{booking.walker_name}</p>
									</td>
								{/if}
								<td class="px-6 py-4 whitespace-nowrap">
									<p class="text-gray-900">{formatDate(booking.scheduled_start)}</p>
									<p class="text-sm text-gray-500">{formatTime(booking.scheduled_start)} - {formatTime(booking.scheduled_end)}</p>
								</td>
								{#if isAdmin}
									<td class="px-6 py-4 whitespace-nowrap">
										<p class="font-medium text-gray-900">{booking.price_display}</p>
									</td>
								{/if}
								<td class="px-6 py-4 whitespace-nowrap">
									<span class="px-2 py-1 text-xs font-medium rounded-full {getStatusColor(booking.status)}">
										{formatStatus(booking.status)}
									</span>
								</td>
								<td class="px-6 py-4 whitespace-nowrap">
									{#if canUpdateStatus(booking.status) && (isAdmin || canAcceptBookings || booking.status !== 'pending')}
										<button
											type="button"
											on:click|stopPropagation={() => (selectedBooking = selectedBooking === booking.id ? null : booking.id)}
											class="text-blue-600 hover:text-blue-800 text-sm font-medium"
										>
											Update
										</button>
									{:else}
										<button
											type="button"
											on:click|stopPropagation={() => openBookingDetail(booking)}
											class="text-gray-500 hover:text-gray-700 text-sm"
										>
											View
										</button>
									{/if}
								</td>
							</tr>
							{#if selectedBooking === booking.id}
								<tr class="bg-gray-50" on:click|stopPropagation on:keydown|stopPropagation>
									<td colspan={isAdmin ? 7 : 5} class="px-6 py-4">
										<div class="flex items-center gap-4">
											<span class="text-sm font-medium text-gray-700">Update Status:</span>
											<form method="POST" action="?/updateStatus" use:enhance class="flex gap-2">
												<input type="hidden" name="booking_id" value={booking.id} />
												{#if booking.status === 'pending' && (isAdmin || canAcceptBookings)}
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
											{#if ['pending', 'confirmed'].includes(booking.status) && isAdmin}
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

<!-- Booking Detail Modal -->
{#if detailBooking}
	<div
		class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
		on:click={closeBookingDetail}
		on:keydown={(e) => e.key === 'Escape' && closeBookingDetail()}
		role="dialog"
		aria-modal="true"
	>
		<div
			class="bg-white rounded-xl shadow-xl w-full max-w-lg mx-4 max-h-[90vh] overflow-y-auto"
			on:click|stopPropagation
			on:keydown|stopPropagation
		>
			<!-- Header -->
			<div class="p-4 border-b border-gray-200 flex items-center justify-between sticky top-0 bg-white">
				<div>
					<h3 class="text-lg font-semibold text-gray-900">Booking Details</h3>
					<span class="px-2 py-0.5 text-xs font-medium rounded-full {getStatusColor(detailBooking.status)}">
						{formatStatus(detailBooking.status)}
					</span>
				</div>
				<button
					type="button"
					on:click={closeBookingDetail}
					class="text-gray-400 hover:text-gray-600 p-1"
					aria-label="Close"
				>
					<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
					</svg>
				</button>
			</div>

			<div class="p-4 space-y-6">
				<!-- Service & Time -->
				<div>
					<h4 class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2">Service</h4>
					<div class="bg-gray-50 rounded-lg p-3">
						<p class="font-medium text-gray-900">{detailBooking.service_name}</p>
						<p class="text-sm text-gray-600 mt-1">
							{formatDate(detailBooking.scheduled_start)}
						</p>
						<p class="text-sm text-gray-600">
							{formatTime(detailBooking.scheduled_start)} - {formatTime(detailBooking.scheduled_end)}
						</p>
						{#if isAdmin}
							<p class="text-sm font-medium text-gray-900 mt-2">{detailBooking.price_display}</p>
						{/if}
					</div>
				</div>

				<!-- Customer Info -->
				<div>
					<h4 class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2">Customer</h4>
					<div class="bg-gray-50 rounded-lg p-3">
						<div class="flex items-center gap-3">
							<div class="w-10 h-10 bg-blue-100 rounded-full flex items-center justify-center">
								<svg class="w-5 h-5 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
								</svg>
							</div>
							<div>
								<p class="font-medium text-gray-900">{detailBooking.customer_name}</p>
								{#if detailBooking.customer_email}
									<p class="text-sm text-gray-600">{detailBooking.customer_email}</p>
								{/if}
								{#if detailBooking.customer_phone}
									<a href="tel:{detailBooking.customer_phone}" class="text-sm text-blue-600 hover:underline">
										{detailBooking.customer_phone}
									</a>
								{/if}
							</div>
						</div>
					</div>
				</div>

				<!-- Dogs (if available) -->
				{#if detailBooking.dogs && detailBooking.dogs.length > 0}
					<div>
						<h4 class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2">
							Dogs ({detailBooking.dogs.length})
						</h4>
						<div class="space-y-2">
							{#each detailBooking.dogs as dog}
								<div class="bg-gray-50 rounded-lg p-3 flex items-center gap-3">
									<div class="w-10 h-10 bg-amber-100 rounded-full flex items-center justify-center">
										<svg class="w-5 h-5 text-amber-600" fill="currentColor" viewBox="0 0 24 24">
											<path d="M4.5 11.5c.28 0 .5-.34.5-.75v-1c0-.41-.22-.75-.5-.75s-.5.34-.5.75v1c0 .41.22.75.5.75zm15 0c.28 0 .5-.34.5-.75v-1c0-.41-.22-.75-.5-.75s-.5.34-.5.75v1c0 .41.22.75.5.75zM12 2c-4 0-8 3-8 8v4c0 2.21 1.79 4 4 4h8c2.21 0 4-1.79 4-4v-4c0-5-4-8-8-8z"/>
										</svg>
									</div>
									<div class="flex-1">
										<p class="font-medium text-gray-900">{dog.name}</p>
										<p class="text-sm text-gray-600">
											{#if dog.breed}{dog.breed}{/if}
											{#if dog.breed && dog.size} · {/if}
											{#if dog.size}{dog.size}{/if}
										</p>
										{#if dog.notes}
											<p class="text-sm text-gray-500 mt-1">{dog.notes}</p>
										{/if}
									</div>
								</div>
							{/each}
						</div>
					</div>
				{/if}

				<!-- Location -->
				<div>
					<h4 class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2">Location</h4>
					<div class="bg-gray-50 rounded-lg p-3">
						<div class="flex items-start gap-3">
							<svg class="w-5 h-5 text-gray-400 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z" />
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 11a3 3 0 11-6 0 3 3 0 016 0z" />
							</svg>
							<div class="flex-1">
								<p class="text-gray-900">{detailBooking.location_address}</p>
								{#if detailBooking.location_notes}
									<p class="text-sm text-gray-500 mt-1">{detailBooking.location_notes}</p>
								{/if}
								<a
									href="https://maps.google.com/?q={encodeURIComponent(detailBooking.location_address)}"
									target="_blank"
									rel="noopener noreferrer"
									class="inline-flex items-center gap-1 text-sm text-blue-600 hover:underline mt-2"
								>
									<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
									</svg>
									Open in Maps
								</a>
							</div>
						</div>
					</div>
				</div>

				<!-- Notes -->
				{#if detailBooking.notes}
					<div>
						<h4 class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2">Notes</h4>
						<div class="bg-gray-50 rounded-lg p-3">
							<p class="text-gray-700 whitespace-pre-wrap">{detailBooking.notes}</p>
						</div>
					</div>
				{/if}
			</div>

			<!-- Footer Actions -->
			<div class="p-4 border-t border-gray-200 flex justify-between items-center">
				{#if canUpdateStatus(detailBooking.status) && (isAdmin || canAcceptBookings || detailBooking.status !== 'pending')}
					<form method="POST" action="?/updateStatus" use:enhance={() => {
						return async ({ result, update }) => {
							if (result.type === 'success') {
								closeBookingDetail();
								window.location.reload();
							} else {
								await update();
							}
						};
					}} class="flex gap-2">
						<input type="hidden" name="booking_id" value={detailBooking.id} />
						{#if detailBooking.status === 'pending' && (isAdmin || canAcceptBookings)}
							<button
								type="submit"
								name="status"
								value="confirmed"
								class="px-4 py-2 text-sm bg-blue-600 text-white rounded-lg hover:bg-blue-700"
							>
								Confirm Booking
							</button>
						{/if}
						{#if detailBooking.status === 'confirmed'}
							<button
								type="submit"
								name="status"
								value="in_progress"
								class="px-4 py-2 text-sm bg-purple-600 text-white rounded-lg hover:bg-purple-700"
							>
								Start Walk
							</button>
						{/if}
						{#if detailBooking.status === 'in_progress'}
							<button
								type="submit"
								name="status"
								value="completed"
								class="px-4 py-2 text-sm bg-green-600 text-white rounded-lg hover:bg-green-700"
							>
								Complete
							</button>
						{/if}
					</form>
				{:else}
					<div></div>
				{/if}
				<button
					type="button"
					on:click={closeBookingDetail}
					class="px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 rounded-lg"
				>
					Close
				</button>
			</div>
		</div>
	</div>
{/if}
