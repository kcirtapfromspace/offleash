<script lang="ts">
	import type { PageData, ActionData } from './$types';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';

	export let data: PageData;
	export let form: ActionData;

	let selectedServiceId = data.selectedServiceId || '';
	// Auto-select the default location if no location is specified in URL
	let selectedLocationId = data.selectedLocationId || data.locations.find(l => l.is_default)?.id || '';
	let selectedDate = data.selectedDate || '';
	let selectedSlot: { walkerId: string; start: string } | null = null;
	let notes = '';

	// Recurring booking options
	let isRecurring = false;
	let recurringFrequency = 'weekly';
	let endConditionType: 'occurrences' | 'date' = 'occurrences';
	let endOccurrences = 12;
	let endDate = '';

	// Toast notification state
	interface Toast {
		id: number;
		type: 'success' | 'error' | 'warning';
		message: string;
		details?: string;
	}
	let toasts: Toast[] = [];
	let toastId = 0;

	function addToast(type: Toast['type'], message: string, details?: string) {
		const id = ++toastId;
		toasts = [...toasts, { id, type, message, details }];
		// Auto dismiss after 8 seconds
		setTimeout(() => dismissToast(id), 8000);
	}

	function dismissToast(id: number) {
		toasts = toasts.filter(t => t.id !== id);
	}

	// Handle form response
	$: if (form?.success && form?.seriesId) {
		// Show success toast with conflicts info
		if (form.conflicts && form.conflicts.length > 0) {
			addToast(
				'warning',
				form.message || `Created ${form.bookingsCreated} bookings`,
				`${form.conflicts.length} dates had conflicts and were skipped.`
			);
		} else {
			addToast('success', 'Recurring booking created successfully!');
		}
		// Redirect after showing toast
		setTimeout(() => {
			goto(`/bookings/recurring/${form.seriesId}`);
		}, 2000);
	}

	$: if (form?.error) {
		addToast('error', form.error, form.errorType === 'api_error' ? 'Please try again or contact support.' : undefined);
	}

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

	function formatTravelTime(minutes: number): string {
		if (minutes < 1) return '<1 min away';
		if (minutes === 1) return '1 min away';
		return `${minutes} min away`;
	}

	$: selectedService = data.services.find((s) => s.id === selectedServiceId);
	$: canShowSlots = selectedServiceId && selectedDate && selectedLocationId;

	// Get selected slot's travel info
	$: selectedSlotData = (() => {
		const slot = selectedSlot;
		if (!slot) return null;
		return data.availability
			.find((w) => w.walkerId === slot.walkerId)
			?.slots.find((s) => s.start === slot.start) ?? null;
	})();

	// Calculate total price for recurring bookings
	$: totalOccurrences = endConditionType === 'occurrences' ? endOccurrences : estimatedOccurrences;
	$: estimatedOccurrences = calculateEstimatedOccurrences();
	$: totalPrice = selectedService
		? (selectedService.price_cents * totalOccurrences) / 100
		: 0;

	function calculateEstimatedOccurrences(): number {
		if (!selectedDate || !endDate) return 0;
		const start = new Date(selectedDate);
		const end = new Date(endDate);
		const diffDays = Math.ceil((end.getTime() - start.getTime()) / (1000 * 60 * 60 * 24));
		switch (recurringFrequency) {
			case 'weekly':
				return Math.floor(diffDays / 7) + 1;
			case 'bi_weekly':
				return Math.floor(diffDays / 14) + 1;
			case 'monthly':
				return Math.floor(diffDays / 30) + 1;
			default:
				return 0;
		}
	}

	function getFrequencyLabel(freq: string): string {
		switch (freq) {
			case 'weekly':
				return 'Weekly';
			case 'bi_weekly':
				return 'Every 2 weeks';
			case 'monthly':
				return 'Monthly';
			default:
				return freq;
		}
	}

	// Generate max end date (1 year from start)
	$: maxEndDate = selectedDate
		? new Date(new Date(selectedDate).getTime() + 365 * 24 * 60 * 60 * 1000)
				.toISOString()
				.split('T')[0]
		: '';
</script>

<!-- Toast Notifications -->
{#if toasts.length > 0}
	<div class="fixed top-4 right-4 z-50 space-y-2 max-w-sm">
		{#each toasts as toast (toast.id)}
			<div
				class="rounded-lg shadow-lg p-4 flex items-start gap-3 animate-slide-in {
					toast.type === 'success' ? 'bg-green-50 border border-green-200' :
					toast.type === 'warning' ? 'bg-yellow-50 border border-yellow-200' :
					'bg-red-50 border border-red-200'
				}"
			>
				<!-- Icon -->
				<div class="flex-shrink-0">
					{#if toast.type === 'success'}
						<svg class="w-5 h-5 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
						</svg>
					{:else if toast.type === 'warning'}
						<svg class="w-5 h-5 text-yellow-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
						</svg>
					{:else}
						<svg class="w-5 h-5 text-red-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
						</svg>
					{/if}
				</div>

				<!-- Content -->
				<div class="flex-1">
					<p class="font-medium {
						toast.type === 'success' ? 'text-green-800' :
						toast.type === 'warning' ? 'text-yellow-800' :
						'text-red-800'
					}">{toast.message}</p>
					{#if toast.details}
						<p class="text-sm mt-1 {
							toast.type === 'success' ? 'text-green-600' :
							toast.type === 'warning' ? 'text-yellow-600' :
							'text-red-600'
						}">{toast.details}</p>
					{/if}
				</div>

				<!-- Dismiss button -->
				<button
					aria-label="Dismiss notification"
					onclick={() => dismissToast(toast.id)}
					class="flex-shrink-0 {
						toast.type === 'success' ? 'text-green-400 hover:text-green-600' :
						toast.type === 'warning' ? 'text-yellow-400 hover:text-yellow-600' :
						'text-red-400 hover:text-red-600'
					}"
				>
					<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
					</svg>
				</button>
			</div>
		{/each}
	</div>
{/if}

<div class="max-w-2xl mx-auto">
	<h1 class="text-2xl font-bold mb-2">Book a Service</h1>
	<p class="text-gray-600 mb-8">Complete the steps below to schedule your booking.</p>

	{#if form?.error}
		<div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg mb-6">
			<div class="flex items-start gap-2">
				<svg class="w-5 h-5 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
				</svg>
				<div>
					<p class="font-medium">{form.error}</p>
					{#if form.errorType === 'api_error'}
						<p class="text-sm mt-1">Please try again or contact support if the problem persists.</p>
					{/if}
				</div>
			</div>
		</div>
	{/if}

	<!-- Success with conflicts inline display -->
	{#if form?.success && form?.conflicts && form.conflicts.length > 0}
		<div class="bg-yellow-50 border border-yellow-200 rounded-lg mb-6 overflow-hidden">
			<div class="px-4 py-3 flex items-start gap-2">
				<svg class="w-5 h-5 text-yellow-600 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
				</svg>
				<div class="flex-1">
					<p class="font-medium text-yellow-800">
						Created {form.bookingsCreated} of {form.totalPlanned} bookings
					</p>
					<p class="text-sm text-yellow-700 mt-1">
						{form.conflicts.length} dates had conflicts and were skipped:
					</p>
				</div>
			</div>
			<div class="px-4 pb-3">
				<details class="group">
					<summary class="text-sm text-yellow-700 cursor-pointer hover:text-yellow-800">
						View conflicting dates ({form.conflicts.length})
					</summary>
					<ul class="mt-2 space-y-1 text-sm">
						{#each form.conflicts as conflict}
							<li class="flex items-center gap-2 text-yellow-700 bg-yellow-100 rounded px-2 py-1">
								<span class="font-medium">{conflict.date}</span>
								<span class="text-yellow-600">- {conflict.reason}</span>
							</li>
						{/each}
					</ul>
				</details>
			</div>
			<div class="px-4 py-3 bg-yellow-100 border-t border-yellow-200">
				<a
					href="/bookings/recurring/{form.seriesId}"
					class="inline-flex items-center gap-2 text-yellow-800 hover:text-yellow-900 font-medium"
				>
					View your recurring series
					<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
					</svg>
				</a>
			</div>
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
				onchange={handleServiceChange}
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
					onchange={handleLocationChange}
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
				onchange={handleDateChange}
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
					<div class="mb-6">
						<h3 class="text-sm font-medium text-gray-700 mb-3">Available with {walker.walkerName}</h3>
						<div class="grid grid-cols-2 sm:grid-cols-3 gap-3">
							{#each walker.slots as slot}
								<button
									type="button"
									onclick={() => selectSlot(walker.walkerId, slot.start)}
									class="relative px-3 py-3 text-sm rounded-lg border transition-colors text-left {selectedSlot?.start === slot.start && selectedSlot?.walkerId === walker.walkerId
										? 'bg-blue-600 text-white border-blue-600'
										: slot.is_tight
											? 'bg-yellow-50 text-gray-700 border-yellow-300 hover:border-yellow-500'
											: 'bg-white text-gray-700 border-gray-300 hover:border-blue-500'}"
								>
									<div class="font-medium">{formatTime(slot.start)}</div>
									{#if slot.travel_minutes !== null && slot.travel_minutes !== undefined}
										<div class="text-xs mt-1 flex items-center gap-1 {selectedSlot?.start === slot.start && selectedSlot?.walkerId === walker.walkerId ? 'text-blue-100' : 'text-gray-500'}">
											<!-- Car/travel icon -->
											<svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
												<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z" />
												<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 11a3 3 0 11-6 0 3 3 0 016 0z" />
											</svg>
											<span>{formatTravelTime(slot.travel_minutes)}</span>
										</div>
									{/if}
									{#if slot.is_tight}
										<!-- Warning indicator -->
										<div class="absolute top-1 right-1" title={slot.warning || 'Schedule may be tight'}>
											<svg class="w-4 h-4 {selectedSlot?.start === slot.start && selectedSlot?.walkerId === walker.walkerId ? 'text-yellow-200' : 'text-yellow-500'}" fill="currentColor" viewBox="0 0 20 20">
												<path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
											</svg>
										</div>
									{/if}
								</button>
							{/each}
						</div>
						{#if walker.travelBufferMinutes}
							<p class="text-xs text-gray-500 mt-2">
								<svg class="w-3 h-3 inline mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
								</svg>
								{walker.travelBufferMinutes} min travel buffer between appointments
							</p>
						{/if}
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

		<!-- Step 6: Make Recurring (Optional) -->
		<div class="bg-white border border-gray-200 rounded-xl p-6">
			<h2 class="text-lg font-semibold mb-4 flex items-center">
				<span class="w-7 h-7 bg-gray-400 text-white rounded-full flex items-center justify-center text-sm mr-3">6</span>
				Make This Recurring (Optional)
			</h2>

			<label class="flex items-center gap-3 cursor-pointer mb-4">
				<input
					type="checkbox"
					bind:checked={isRecurring}
					name="is_recurring"
					class="w-5 h-5 rounded border-gray-300 text-blue-600 focus:ring-blue-500"
				/>
				<span class="text-gray-700">Create recurring bookings</span>
			</label>

			{#if isRecurring}
				<div class="space-y-4 pt-4 border-t border-gray-200">
					<!-- Frequency Selection -->
					<div>
						<label for="recurring_frequency" class="block text-sm font-medium text-gray-700 mb-2">Frequency</label>
						<select
							id="recurring_frequency"
							name="recurring_frequency"
							bind:value={recurringFrequency}
							class="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
						>
							<option value="weekly">Weekly</option>
							<option value="bi_weekly">Every 2 weeks</option>
							<option value="monthly">Monthly</option>
						</select>
					</div>

					<!-- End Condition -->
					<fieldset>
						<legend class="block text-sm font-medium text-gray-700 mb-2">Ends</legend>
						<div class="space-y-3">
							<label class="flex items-center gap-3 cursor-pointer">
								<input
									type="radio"
									name="end_condition_type"
									value="occurrences"
									bind:group={endConditionType}
									class="w-4 h-4 text-blue-600 focus:ring-blue-500"
								/>
								<span class="text-gray-700">After</span>
								<input
									type="number"
									name="end_occurrences"
									bind:value={endOccurrences}
									min="2"
									max="52"
									disabled={endConditionType !== 'occurrences'}
									class="w-20 px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 disabled:bg-gray-100"
								/>
								<span class="text-gray-700">bookings</span>
							</label>
							<label class="flex items-center gap-3 cursor-pointer">
								<input
									type="radio"
									name="end_condition_type"
									value="date"
									bind:group={endConditionType}
									class="w-4 h-4 text-blue-600 focus:ring-blue-500"
								/>
								<span class="text-gray-700">On date</span>
								<input
									type="date"
									name="end_date"
									bind:value={endDate}
									min={selectedDate}
									max={maxEndDate}
									disabled={endConditionType !== 'date'}
									class="px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 disabled:bg-gray-100"
								/>
							</label>
						</div>
					</fieldset>

					<!-- Preview -->
					{#if selectedDate && selectedSlot}
						<div class="bg-blue-50 rounded-lg p-4 mt-4">
							<p class="text-sm text-blue-800">
								<strong>{totalOccurrences}</strong> bookings will be created on
								<strong>{new Date(selectedDate).toLocaleDateString('en-US', { weekday: 'long' })}s</strong>
								{getFrequencyLabel(recurringFrequency).toLowerCase()}.
							</p>
						</div>
					{/if}
				</div>
			{/if}
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
					{#if selectedSlotData?.travel_minutes !== null && selectedSlotData?.travel_minutes !== undefined}
						<div class="flex justify-between">
							<dt class="text-gray-600">Walker arrival:</dt>
							<dd class="font-medium flex items-center gap-1">
								<svg class="w-4 h-4 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z" />
								</svg>
								{formatTravelTime(selectedSlotData.travel_minutes)}
								{#if selectedSlotData.travel_from}
									<span class="text-gray-500 text-xs">({selectedSlotData.travel_from})</span>
								{/if}
							</dd>
						</div>
					{/if}
					{#if selectedSlotData?.is_tight && selectedSlotData?.warning}
						<div class="bg-yellow-100 rounded-lg p-2 flex items-start gap-2">
							<svg class="w-4 h-4 text-yellow-600 flex-shrink-0 mt-0.5" fill="currentColor" viewBox="0 0 20 20">
								<path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
							</svg>
							<span class="text-yellow-800 text-xs">{selectedSlotData.warning}</span>
						</div>
					{/if}
					{#if isRecurring}
						<div class="flex justify-between">
							<dt class="text-gray-600">Frequency:</dt>
							<dd class="font-medium">{getFrequencyLabel(recurringFrequency)}</dd>
						</div>
						<div class="flex justify-between">
							<dt class="text-gray-600">Total Bookings:</dt>
							<dd class="font-medium">{totalOccurrences}</dd>
						</div>
						<div class="flex justify-between">
							<dt class="text-gray-600">Price per booking:</dt>
							<dd class="font-medium">{selectedService.price_display}</dd>
						</div>
						<div class="flex justify-between text-lg pt-2 border-t border-blue-200">
							<dt class="font-semibold">Total (all bookings):</dt>
							<dd class="font-bold text-green-600">${totalPrice.toFixed(2)}</dd>
						</div>
					{:else}
						<div class="flex justify-between text-lg pt-2 border-t border-blue-200">
							<dt class="font-semibold">Total:</dt>
							<dd class="font-bold text-green-600">{selectedService.price_display}</dd>
						</div>
					{/if}
				</dl>
			</div>
		{/if}

		<button
			type="submit"
			disabled={!selectedSlot}
			class="w-full py-4 bg-blue-600 text-white font-semibold rounded-xl hover:bg-blue-700 transition-colors disabled:bg-gray-300 disabled:cursor-not-allowed"
		>
			{isRecurring ? `Create ${totalOccurrences} Recurring Bookings` : 'Confirm Booking'}
		</button>
	</form>
</div>

<style>
	@keyframes slide-in {
		from {
			transform: translateX(100%);
			opacity: 0;
		}
		to {
			transform: translateX(0);
			opacity: 1;
		}
	}

	:global(.animate-slide-in) {
		animation: slide-in 0.3s ease-out;
	}
</style>
