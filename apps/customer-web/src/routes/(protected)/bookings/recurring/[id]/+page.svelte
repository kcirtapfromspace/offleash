<script lang="ts">
	import type { PageData, ActionData } from './$types';

	export let data: PageData;
	export let form: ActionData;

	let showCancelModal = false;
	let cancelScope = 'all_future';

	function formatTime(time: string): string {
		const [hours, minutes] = time.split(':');
		const hour = parseInt(hours);
		const ampm = hour >= 12 ? 'PM' : 'AM';
		const displayHour = hour % 12 || 12;
		return `${displayHour}:${minutes} ${ampm}`;
	}

	function formatDateTime(dateStr: string): string {
		const date = new Date(dateStr);
		return date.toLocaleDateString('en-US', {
			weekday: 'short',
			month: 'short',
			day: 'numeric',
			hour: 'numeric',
			minute: '2-digit'
		});
	}

	function getStatusColor(status: string): string {
		switch (status) {
			case 'pending':
				return 'bg-yellow-100 text-yellow-800';
			case 'confirmed':
				return 'bg-blue-100 text-blue-800';
			case 'completed':
				return 'bg-green-100 text-green-800';
			case 'cancelled':
				return 'bg-red-100 text-red-800';
			default:
				return 'bg-gray-100 text-gray-800';
		}
	}

	$: upcomingBookings = data.bookings.filter(
		(b) => new Date(b.scheduled_start) > new Date() && b.status !== 'cancelled'
	);
	$: pastBookings = data.bookings.filter(
		(b) => new Date(b.scheduled_start) <= new Date() || b.status === 'cancelled'
	);
</script>

<div class="max-w-4xl mx-auto">
	{#if form?.error}
		<div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg mb-6">
			{form.error}
		</div>
	{/if}

	{#if data.error}
		<div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg mb-6">
			{data.error}
		</div>
	{/if}

	{#if data.series}
		<!-- Header -->
		<div class="flex justify-between items-start mb-6">
			<div>
				<a href="/bookings/recurring" class="text-blue-600 hover:underline text-sm mb-2 inline-block">
					← Back to recurring bookings
				</a>
				<h1 class="text-2xl font-bold">{data.service_name}</h1>
				<p class="text-gray-600">Recurring Series</p>
			</div>
			{#if data.series.is_active}
				<button
					on:click={() => (showCancelModal = true)}
					class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors"
				>
					Cancel Series
				</button>
			{:else}
				<span class="px-4 py-2 bg-gray-100 text-gray-600 rounded-lg">
					Series Cancelled
				</span>
			{/if}
		</div>

		<!-- Series Details -->
		<div class="bg-white border border-gray-200 rounded-xl p-6 mb-6">
			<h2 class="text-lg font-semibold mb-4">Series Details</h2>
			<dl class="grid grid-cols-2 gap-4">
				<div>
					<dt class="text-sm text-gray-500">Frequency</dt>
					<dd class="font-medium">{data.series.frequency}</dd>
				</div>
				<div>
					<dt class="text-sm text-gray-500">Day & Time</dt>
					<dd class="font-medium">{data.series.day_of_week_name}s at {formatTime(data.series.time_of_day)}</dd>
				</div>
				<div>
					<dt class="text-sm text-gray-500">Walker</dt>
					<dd class="font-medium">{data.walker_name}</dd>
				</div>
				<div>
					<dt class="text-sm text-gray-500">Location</dt>
					<dd class="font-medium">{data.location_address}</dd>
				</div>
				<div>
					<dt class="text-sm text-gray-500">Price per booking</dt>
					<dd class="font-medium text-green-600">{data.series.price_display}</dd>
				</div>
				<div>
					<dt class="text-sm text-gray-500">Status</dt>
					<dd>
						{#if data.series.is_active}
							<span class="px-2 py-1 text-xs font-medium bg-green-100 text-green-700 rounded-full">
								Active
							</span>
						{:else}
							<span class="px-2 py-1 text-xs font-medium bg-gray-100 text-gray-600 rounded-full">
								Inactive
							</span>
						{/if}
					</dd>
				</div>
				{#if data.series.default_notes}
					<div class="col-span-2">
						<dt class="text-sm text-gray-500">Notes</dt>
						<dd class="font-medium">{data.series.default_notes}</dd>
					</div>
				{/if}
			</dl>
		</div>

		<!-- Upcoming Bookings -->
		<div class="bg-white border border-gray-200 rounded-xl p-6 mb-6">
			<h2 class="text-lg font-semibold mb-4">Upcoming Bookings ({upcomingBookings.length})</h2>
			{#if upcomingBookings.length === 0}
				<p class="text-gray-500 text-center py-4">No upcoming bookings</p>
			{:else}
				<div class="space-y-3">
					{#each upcomingBookings as booking}
						<a
							href="/bookings/{booking.id}"
							class="flex justify-between items-center p-4 bg-gray-50 rounded-lg hover:bg-gray-100 transition-colors"
						>
							<div>
								<p class="font-medium">{formatDateTime(booking.scheduled_start)}</p>
								<p class="text-sm text-gray-500">Booking #{booking.occurrence_number}</p>
							</div>
							<div class="flex items-center gap-3">
								<span class="px-2 py-1 text-xs font-medium rounded-full {getStatusColor(booking.status)}">
									{booking.status}
								</span>
								<span class="text-gray-400">→</span>
							</div>
						</a>
					{/each}
				</div>
			{/if}
		</div>

		<!-- Past Bookings -->
		{#if pastBookings.length > 0}
			<div class="bg-white border border-gray-200 rounded-xl p-6">
				<h2 class="text-lg font-semibold mb-4">Past & Cancelled Bookings ({pastBookings.length})</h2>
				<div class="space-y-3">
					{#each pastBookings as booking}
						<a
							href="/bookings/{booking.id}"
							class="flex justify-between items-center p-4 bg-gray-50 rounded-lg hover:bg-gray-100 transition-colors opacity-60"
						>
							<div>
								<p class="font-medium">{formatDateTime(booking.scheduled_start)}</p>
								<p class="text-sm text-gray-500">Booking #{booking.occurrence_number}</p>
							</div>
							<div class="flex items-center gap-3">
								<span class="px-2 py-1 text-xs font-medium rounded-full {getStatusColor(booking.status)}">
									{booking.status}
								</span>
								<span class="text-gray-400">→</span>
							</div>
						</a>
					{/each}
				</div>
			</div>
		{/if}
	{/if}
</div>

<!-- Cancel Modal -->
{#if showCancelModal}
	<div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
		<div class="bg-white rounded-xl p-6 max-w-md w-full mx-4">
			<h3 class="text-lg font-semibold mb-4">Cancel Recurring Series</h3>
			<p class="text-gray-600 mb-4">
				Choose how you want to cancel this recurring series:
			</p>

			<form method="POST" action="?/cancel">
				<div class="space-y-3 mb-6">
					<label class="flex items-start gap-3 p-3 border border-gray-200 rounded-lg cursor-pointer hover:border-blue-500 transition-colors">
						<input
							type="radio"
							name="scope"
							value="all_future"
							bind:group={cancelScope}
							class="mt-1"
						/>
						<div>
							<p class="font-medium">Cancel future bookings</p>
							<p class="text-sm text-gray-500">Only cancel upcoming bookings. Past bookings will remain.</p>
						</div>
					</label>
					<label class="flex items-start gap-3 p-3 border border-gray-200 rounded-lg cursor-pointer hover:border-blue-500 transition-colors">
						<input
							type="radio"
							name="scope"
							value="entire_series"
							bind:group={cancelScope}
							class="mt-1"
						/>
						<div>
							<p class="font-medium">Cancel entire series</p>
							<p class="text-sm text-gray-500">Cancel all pending and confirmed bookings in this series.</p>
						</div>
					</label>
				</div>

				<div class="flex gap-3">
					<button
						type="button"
						on:click={() => (showCancelModal = false)}
						class="flex-1 px-4 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 transition-colors"
					>
						Keep Series
					</button>
					<button
						type="submit"
						class="flex-1 px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors"
					>
						Cancel Series
					</button>
				</div>
			</form>
		</div>
	</div>
{/if}
