<script lang="ts">
	import type { PageData, ActionData } from './$types';
	import { goto } from '$app/navigation';

	export let data: PageData;
	export let form: ActionData;

	let selectedServiceId = data.selectedServiceId || '';
	let selectedLocationId = data.selectedLocationId || '';
	let selectedDate = data.selectedDate || '';
	let selectedSlot: { walkerId: string; start: string } | null = null;
	let notes = '';

	// Get min date (today) and max date (30 days from now)
	const today = new Date();
	const minDate = today.toISOString().split('T')[0];
	const maxDate = new Date(today.getTime() + 30 * 24 * 60 * 60 * 1000).toISOString().split('T')[0];

	function updateUrl() {
		const params = new URLSearchParams();
		if (selectedServiceId) params.set('service', selectedServiceId);
		if (selectedDate) params.set('date', selectedDate);
		if (selectedLocationId) params.set('location', selectedLocationId);
		goto(`/bookings/new?${params.toString()}`, { replaceState: true, noScroll: true });
	}

	function handleServiceChange() {
		selectedSlot = null;
		updateUrl();
	}

	function handleDateChange() {
		selectedSlot = null;
		updateUrl();
	}

	function handleLocationChange() {
		selectedSlot = null;
		updateUrl();
	}

	function selectSlot(walkerId: string, start: string) {
		selectedSlot = { walkerId, start };
	}

	function formatTime(isoString: string): string {
		return new Date(isoString).toLocaleTimeString('en-US', {
			hour: 'numeric',
			minute: '2-digit',
			hour12: true
		});
	}

	function formatDuration(minutes: number): string {
		if (minutes < 60) return `${minutes} min`;
		const hours = Math.floor(minutes / 60);
		const mins = minutes % 60;
		return mins > 0 ? `${hours}h ${mins}m` : `${hours}h`;
	}

	$: selectedService = data.services.find((s) => s.id === selectedServiceId);
	$: canShowSlots = selectedServiceId && selectedDate && selectedLocationId;
</script>

<div class="max-w-2xl mx-auto">
	<h1 class="text-2xl font-bold mb-2">Book a Service</h1>
	<p class="text-gray-600 mb-8">Complete the steps below to schedule your booking.</p>

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

	<form method="POST" action="?/book" class="space-y-8">
		<!-- Step 1: Select Service -->
		<div class="bg-white border border-gray-200 rounded-xl p-6">
			<h2 class="text-lg font-semibold mb-4 flex items-center">
				<span class="w-7 h-7 bg-blue-600 text-white rounded-full flex items-center justify-center text-sm mr-3">1</span>
				Select Service
			</h2>
			<select
				name="service_id"
				bind:value={selectedServiceId}
				on:change={handleServiceChange}
				class="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
				required
			>
				<option value="">Choose a service...</option>
				{#each data.services as service}
					<option value={service.id}>
						{service.name} - {service.price_display} ({formatDuration(service.duration_minutes)})
					</option>
				{/each}
			</select>
		</div>

		<!-- Step 2: Select Location -->
		<div class="bg-white border border-gray-200 rounded-xl p-6">
			<h2 class="text-lg font-semibold mb-4 flex items-center">
				<span class="w-7 h-7 bg-blue-600 text-white rounded-full flex items-center justify-center text-sm mr-3">2</span>
				Select Pickup Location
			</h2>
			{#if data.locations.length === 0}
				<div class="text-center py-6 bg-gray-50 rounded-lg">
					<p class="text-gray-600 mb-3">You haven't added any locations yet.</p>
					<a href="/locations" class="text-blue-600 hover:underline font-medium">Add a location first</a>
				</div>
			{:else}
				<select
					name="location_id"
					bind:value={selectedLocationId}
					on:change={handleLocationChange}
					class="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
					required
				>
					<option value="">Choose a location...</option>
					{#each data.locations as location}
						<option value={location.id}>
							{location.name} - {location.full_address}
						</option>
					{/each}
				</select>
				<a href="/locations" class="text-sm text-blue-600 hover:underline mt-2 inline-block">+ Add new location</a>
			{/if}
		</div>

		<!-- Step 3: Select Date -->
		<div class="bg-white border border-gray-200 rounded-xl p-6">
			<h2 class="text-lg font-semibold mb-4 flex items-center">
				<span class="w-7 h-7 bg-blue-600 text-white rounded-full flex items-center justify-center text-sm mr-3">3</span>
				Select Date
			</h2>
			<input
				type="date"
				bind:value={selectedDate}
				on:change={handleDateChange}
				min={minDate}
				max={maxDate}
				class="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
				required
			/>
		</div>

		<!-- Step 4: Select Time -->
		<div class="bg-white border border-gray-200 rounded-xl p-6">
			<h2 class="text-lg font-semibold mb-4 flex items-center">
				<span class="w-7 h-7 bg-blue-600 text-white rounded-full flex items-center justify-center text-sm mr-3">4</span>
				Select Time
			</h2>

			{#if !canShowSlots}
				<p class="text-gray-500 text-center py-6">Please complete the steps above to see available times.</p>
			{:else if data.availability.length === 0}
				<div class="text-center py-6 bg-yellow-50 rounded-lg">
					<p class="text-yellow-800">No available time slots for this date. Please try another date.</p>
				</div>
			{:else}
				{#each data.availability as walker}
					<div class="mb-4">
						<h3 class="text-sm font-medium text-gray-700 mb-2">Available with {walker.walkerName}</h3>
						<div class="grid grid-cols-3 sm:grid-cols-4 gap-2">
							{#each walker.slots as slot}
								<button
									type="button"
									on:click={() => selectSlot(walker.walkerId, slot.start)}
									class="px-3 py-2 text-sm rounded-lg border transition-colors {selectedSlot?.start === slot.start && selectedSlot?.walkerId === walker.walkerId
										? 'bg-blue-600 text-white border-blue-600'
										: 'bg-white text-gray-700 border-gray-300 hover:border-blue-500'}"
								>
									{formatTime(slot.start)}
								</button>
							{/each}
						</div>
					</div>
				{/each}

				{#if selectedSlot}
					<input type="hidden" name="walker_id" value={selectedSlot.walkerId} />
					<input type="hidden" name="start_time" value={selectedSlot.start} />
				{/if}
			{/if}
		</div>

		<!-- Step 5: Notes (Optional) -->
		<div class="bg-white border border-gray-200 rounded-xl p-6">
			<h2 class="text-lg font-semibold mb-4 flex items-center">
				<span class="w-7 h-7 bg-gray-400 text-white rounded-full flex items-center justify-center text-sm mr-3">5</span>
				Additional Notes (Optional)
			</h2>
			<textarea
				name="notes"
				bind:value={notes}
				rows="3"
				placeholder="Any special instructions for the walker..."
				class="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
			></textarea>
		</div>

		<!-- Summary & Submit -->
		{#if selectedService && selectedSlot}
			<div class="bg-blue-50 border border-blue-200 rounded-xl p-6">
				<h2 class="text-lg font-semibold mb-4">Booking Summary</h2>
				<dl class="space-y-2 text-sm">
					<div class="flex justify-between">
						<dt class="text-gray-600">Service:</dt>
						<dd class="font-medium">{selectedService.name}</dd>
					</div>
					<div class="flex justify-between">
						<dt class="text-gray-600">Duration:</dt>
						<dd class="font-medium">{formatDuration(selectedService.duration_minutes)}</dd>
					</div>
					<div class="flex justify-between">
						<dt class="text-gray-600">Date & Time:</dt>
						<dd class="font-medium">{new Date(selectedSlot.start).toLocaleString()}</dd>
					</div>
					<div class="flex justify-between text-lg pt-2 border-t border-blue-200">
						<dt class="font-semibold">Total:</dt>
						<dd class="font-bold text-green-600">{selectedService.price_display}</dd>
					</div>
				</dl>
			</div>
		{/if}

		<button
			type="submit"
			disabled={!selectedSlot}
			class="w-full py-4 bg-blue-600 text-white font-semibold rounded-xl hover:bg-blue-700 transition-colors disabled:bg-gray-300 disabled:cursor-not-allowed"
		>
			Confirm Booking
		</button>
	</form>
</div>
