<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { get, ApiError } from '$lib/api';
	import { generateCssVariables } from '$lib/stores/branding';
	import { onMount } from 'svelte';

	interface Service {
		id: string;
		name: string;
		description: string | null;
		duration_minutes: number;
		price_cents: number;
		price_display: string;
		is_active: boolean;
	}

	interface TimeSlot {
		start_time: string;
		end_time: string;
		walker_id: string;
		walker_name?: string;
	}

	interface Location {
		id: string;
		name: string;
		address: string;
		city: string;
		state: string;
		zip: string;
	}

	const serviceId = $derived($page.params.serviceId);
	const branding = $derived($page.data.branding);
	const cssVariables = $derived(generateCssVariables(branding));

	let service = $state<Service | null>(null);
	let locations = $state<Location[]>([]);
	let selectedLocationId = $state<string | null>(null);
	let selectedDate = $state<string | null>(null);
	let timeSlots = $state<TimeSlot[]>([]);
	let selectedSlot = $state<TimeSlot | null>(null);

	let isLoadingService = $state(true);
	let isLoadingLocations = $state(true);
	let isLoadingSlots = $state(false);
	let error = $state<string | null>(null);
	let slotsError = $state<string | null>(null);

	// Generate dates for the next 14 days
	const availableDates = $derived(() => {
		const dates: { date: string; display: string; dayName: string; dayNum: string; month: string }[] = [];
		const today = new Date();

		for (let i = 0; i < 14; i++) {
			const date = new Date(today);
			date.setDate(today.getDate() + i);

			const dateStr = date.toISOString().split('T')[0];
			const dayName = date.toLocaleDateString('en-US', { weekday: 'short' });
			const dayNum = date.getDate().toString();
			const month = date.toLocaleDateString('en-US', { month: 'short' });
			const display = date.toLocaleDateString('en-US', { weekday: 'long', month: 'long', day: 'numeric' });

			dates.push({ date: dateStr, display, dayName, dayNum, month });
		}

		return dates;
	});

	onMount(() => {
		fetchService();
		fetchLocations();
	});

	async function fetchService() {
		try {
			isLoadingService = true;
			error = null;
			service = await get<Service>(`/services/${serviceId}`);
		} catch (err) {
			if (err instanceof ApiError) {
				error = err.message || 'Failed to load service';
			} else {
				error = 'An unexpected error occurred';
			}
		} finally {
			isLoadingService = false;
		}
	}

	async function fetchLocations() {
		try {
			isLoadingLocations = true;
			locations = await get<Location[]>('/locations');
			if (locations.length > 0) {
				selectedLocationId = locations[0].id;
			}
		} catch (err) {
			// Locations might not be required, continue without them
			locations = [];
		} finally {
			isLoadingLocations = false;
		}
	}

	async function fetchTimeSlots() {
		if (!selectedDate || !selectedLocationId) {
			timeSlots = [];
			return;
		}

		try {
			isLoadingSlots = true;
			slotsError = null;
			selectedSlot = null;

			const params = new URLSearchParams();
			params.set('date', selectedDate);
			if (serviceId) params.set('service_id', serviceId);
			params.set('location_id', selectedLocationId);

			timeSlots = await get<TimeSlot[]>(`/availability?${params.toString()}`);
		} catch (err) {
			if (err instanceof ApiError) {
				slotsError = err.message || 'Failed to load available times';
			} else {
				slotsError = 'An unexpected error occurred';
			}
			timeSlots = [];
		} finally {
			isLoadingSlots = false;
		}
	}

	function handleDateSelect(date: string) {
		selectedDate = date;
		fetchTimeSlots();
	}

	function handleSlotSelect(slot: TimeSlot) {
		selectedSlot = slot;
	}

	function formatTime(isoString: string): string {
		const date = new Date(isoString);
		return date.toLocaleTimeString('en-US', { hour: 'numeric', minute: '2-digit', hour12: true });
	}

	function formatPrice(cents: number): string {
		return `$${(cents / 100).toFixed(2)}`;
	}

	function formatDuration(minutes: number): string {
		if (minutes < 60) {
			return `${minutes} min`;
		}
		const hours = Math.floor(minutes / 60);
		const remainingMinutes = minutes % 60;
		if (remainingMinutes === 0) {
			return hours === 1 ? '1 hour' : `${hours} hours`;
		}
		return `${hours}h ${remainingMinutes}m`;
	}

	function proceedToConfirmation() {
		if (!selectedSlot || !selectedDate || !selectedLocationId) return;

		// Store booking details in session storage for the confirmation page
		const bookingDetails = {
			serviceId,
			serviceName: service?.name,
			serviceDuration: service?.duration_minutes,
			servicePrice: service?.price_cents,
			date: selectedDate,
			startTime: selectedSlot.start_time,
			endTime: selectedSlot.end_time,
			walkerId: selectedSlot.walker_id,
			walkerName: selectedSlot.walker_name,
			locationId: selectedLocationId,
			locationName: locations.find(l => l.id === selectedLocationId)?.name
		};

		if (typeof window !== 'undefined') {
			sessionStorage.setItem('pendingBooking', JSON.stringify(bookingDetails));
		}

		goto(`/book/${serviceId}/confirm`);
	}

	function goBack() {
		goto('/services');
	}
</script>

<svelte:head>
	<title>Book {service?.name || 'Service'} - {branding.company_name}</title>
</svelte:head>

<div class="booking-page" style={cssVariables}>
	<header class="header">
		<button type="button" class="back-button" onclick={goBack} aria-label="Go back to services">
			<svg fill="none" stroke="currentColor" viewBox="0 0 24 24" width="24" height="24">
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
			</svg>
		</button>
		{#if branding.logo_url}
			<img src={branding.logo_url} alt={branding.company_name} class="logo" />
		{:else}
			<span class="company-name">{branding.company_name}</span>
		{/if}
		<div class="header-spacer"></div>
	</header>

	<main class="main">
		{#if isLoadingService}
			<div class="loading-state">
				<div class="spinner"></div>
				<p>Loading service...</p>
			</div>
		{:else if error}
			<div class="error-message" role="alert">
				<p>{error}</p>
				<button onclick={fetchService} class="retry-button">Try Again</button>
			</div>
		{:else if service}
			<!-- Service Summary -->
			<div class="service-summary">
				<h1 class="service-name">{service.name}</h1>
				<div class="service-meta">
					<span class="service-duration">
						<svg fill="none" stroke="currentColor" viewBox="0 0 24 24" width="16" height="16">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
						</svg>
						{formatDuration(service.duration_minutes)}
					</span>
					<span class="service-price">{service.price_display || formatPrice(service.price_cents)}</span>
				</div>
			</div>

			<!-- Location Selection -->
			{#if !isLoadingLocations && locations.length > 0}
				<section class="section">
					<h2 class="section-title">Select Location</h2>
					<div class="location-select-wrapper">
						<select
							class="location-select"
							bind:value={selectedLocationId}
							onchange={() => { if (selectedDate) fetchTimeSlots(); }}
						>
							{#each locations as location (location.id)}
								<option value={location.id}>{location.name} - {location.address}</option>
							{/each}
						</select>
					</div>
				</section>
			{/if}

			<!-- Date Selection -->
			<section class="section">
				<h2 class="section-title">Select Date</h2>
				<div class="date-picker">
					{#each availableDates() as { date, dayName, dayNum, month } (date)}
						<button
							type="button"
							class="date-button"
							class:selected={selectedDate === date}
							onclick={() => handleDateSelect(date)}
						>
							<span class="date-day">{dayName}</span>
							<span class="date-num">{dayNum}</span>
							<span class="date-month">{month}</span>
						</button>
					{/each}
				</div>
			</section>

			<!-- Time Slots -->
			{#if selectedDate}
				<section class="section">
					<h2 class="section-title">Select Time</h2>

					{#if isLoadingSlots}
						<div class="slots-loading">
							<div class="spinner small"></div>
							<p>Loading available times...</p>
						</div>
					{:else if slotsError}
						<div class="slots-error">
							<p>{slotsError}</p>
							<button onclick={fetchTimeSlots} class="retry-button small">Try Again</button>
						</div>
					{:else if timeSlots.length === 0}
						<div class="no-slots">
							<svg fill="none" stroke="currentColor" viewBox="0 0 24 24" width="48" height="48">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
							</svg>
							<h3>No Available Times</h3>
							<p>There are no available time slots for this date. Please try a different date.</p>
						</div>
					{:else}
						<div class="time-slots">
							{#each timeSlots as slot (slot.start_time)}
								<button
									type="button"
									class="time-slot"
									class:selected={selectedSlot?.start_time === slot.start_time}
									onclick={() => handleSlotSelect(slot)}
								>
									<span class="slot-time">{formatTime(slot.start_time)}</span>
									{#if slot.walker_name}
										<span class="slot-walker">with {slot.walker_name}</span>
									{/if}
								</button>
							{/each}
						</div>
					{/if}
				</section>
			{/if}

			<!-- Continue Button -->
			{#if selectedSlot}
				<div class="continue-section">
					<div class="selection-summary">
						<p>
							<strong>{availableDates().find(d => d.date === selectedDate)?.display}</strong>
							at <strong>{formatTime(selectedSlot.start_time)}</strong>
						</p>
					</div>
					<button
						type="button"
						class="continue-button"
						onclick={proceedToConfirmation}
					>
						Continue to Confirmation
					</button>
				</div>
			{/if}
		{/if}
	</main>
</div>

<style>
	.booking-page {
		min-height: 100vh;
		background-color: #f3f4f6;
	}

	.header {
		background-color: white;
		padding: 1rem;
		box-shadow: 0 1px 3px 0 rgb(0 0 0 / 0.1);
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.back-button {
		background: none;
		border: none;
		padding: 0.5rem;
		cursor: pointer;
		color: #374151;
		border-radius: 0.375rem;
	}

	.back-button:hover {
		background-color: #f3f4f6;
	}

	.logo {
		max-height: 2rem;
	}

	.company-name {
		font-size: 1.25rem;
		font-weight: 700;
		color: var(--color-primary, #3b82f6);
	}

	.header-spacer {
		width: 44px;
	}

	.main {
		max-width: 48rem;
		margin: 0 auto;
		padding: 1.5rem 1rem 6rem;
	}

	.loading-state {
		text-align: center;
		padding: 3rem 1rem;
	}

	.spinner {
		width: 2.5rem;
		height: 2.5rem;
		border: 3px solid #e5e7eb;
		border-top-color: var(--color-primary, #3b82f6);
		border-radius: 50%;
		animation: spin 1s linear infinite;
		margin: 0 auto 1rem;
	}

	.spinner.small {
		width: 1.5rem;
		height: 1.5rem;
		border-width: 2px;
		margin: 0 auto 0.5rem;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	.error-message {
		background-color: #fef2f2;
		border: 1px solid #fecaca;
		border-radius: 0.5rem;
		padding: 2rem;
		text-align: center;
		color: #dc2626;
	}

	.error-message p {
		margin: 0 0 1rem;
	}

	.retry-button {
		background-color: var(--color-primary, #3b82f6);
		color: white;
		padding: 0.5rem 1rem;
		border: none;
		border-radius: 0.375rem;
		font-weight: 500;
		cursor: pointer;
	}

	.retry-button:hover {
		background-color: var(--color-secondary, #1e40af);
	}

	.retry-button.small {
		padding: 0.375rem 0.75rem;
		font-size: 0.875rem;
	}

	.service-summary {
		background-color: white;
		border-radius: 0.75rem;
		padding: 1.5rem;
		margin-bottom: 1.5rem;
		box-shadow: 0 1px 3px 0 rgb(0 0 0 / 0.1);
	}

	.service-name {
		font-size: 1.5rem;
		font-weight: 700;
		color: #111827;
		margin: 0 0 0.75rem;
	}

	.service-meta {
		display: flex;
		align-items: center;
		gap: 1rem;
	}

	.service-duration {
		display: flex;
		align-items: center;
		gap: 0.375rem;
		color: #6b7280;
		font-size: 0.875rem;
	}

	.service-duration svg {
		color: var(--color-accent, #10b981);
	}

	.service-price {
		font-size: 1.125rem;
		font-weight: 700;
		color: var(--color-primary, #3b82f6);
	}

	.section {
		background-color: white;
		border-radius: 0.75rem;
		padding: 1.5rem;
		margin-bottom: 1rem;
		box-shadow: 0 1px 3px 0 rgb(0 0 0 / 0.1);
	}

	.section-title {
		font-size: 1rem;
		font-weight: 600;
		color: #374151;
		margin: 0 0 1rem;
	}

	.location-select-wrapper {
		position: relative;
	}

	.location-select {
		width: 100%;
		padding: 0.75rem 1rem;
		border: 1px solid #d1d5db;
		border-radius: 0.5rem;
		font-size: 1rem;
		background-color: white;
		cursor: pointer;
		appearance: none;
		background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 24 24' stroke='%236b7280'%3E%3Cpath stroke-linecap='round' stroke-linejoin='round' stroke-width='2' d='M19 9l-7 7-7-7'%3E%3C/path%3E%3C/svg%3E");
		background-repeat: no-repeat;
		background-position: right 0.75rem center;
		background-size: 1.25rem;
		padding-right: 2.5rem;
	}

	.location-select:focus {
		outline: none;
		border-color: var(--color-primary, #3b82f6);
		box-shadow: 0 0 0 2px rgb(59 130 246 / 0.2);
	}

	.date-picker {
		display: flex;
		gap: 0.5rem;
		overflow-x: auto;
		padding-bottom: 0.5rem;
		-webkit-overflow-scrolling: touch;
		scrollbar-width: thin;
	}

	.date-picker::-webkit-scrollbar {
		height: 4px;
	}

	.date-picker::-webkit-scrollbar-thumb {
		background-color: #d1d5db;
		border-radius: 2px;
	}

	.date-button {
		flex-shrink: 0;
		display: flex;
		flex-direction: column;
		align-items: center;
		padding: 0.75rem 1rem;
		background-color: #f9fafb;
		border: 2px solid transparent;
		border-radius: 0.5rem;
		cursor: pointer;
		transition: all 0.15s ease;
		min-width: 4.5rem;
	}

	.date-button:hover {
		background-color: #f3f4f6;
		border-color: #d1d5db;
	}

	.date-button.selected {
		background-color: var(--color-primary, #3b82f6);
		border-color: var(--color-primary, #3b82f6);
		color: white;
	}

	.date-day {
		font-size: 0.75rem;
		font-weight: 500;
		text-transform: uppercase;
		opacity: 0.8;
	}

	.date-num {
		font-size: 1.25rem;
		font-weight: 700;
		line-height: 1.2;
	}

	.date-month {
		font-size: 0.75rem;
		opacity: 0.8;
	}

	.slots-loading,
	.slots-error {
		text-align: center;
		padding: 2rem;
		color: #6b7280;
	}

	.slots-error {
		color: #dc2626;
	}

	.no-slots {
		text-align: center;
		padding: 2rem;
		color: #6b7280;
	}

	.no-slots svg {
		margin: 0 auto 1rem;
		color: #9ca3af;
	}

	.no-slots h3 {
		font-size: 1.125rem;
		font-weight: 600;
		color: #374151;
		margin: 0 0 0.5rem;
	}

	.no-slots p {
		margin: 0;
		font-size: 0.875rem;
	}

	.time-slots {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 0.5rem;
	}

	@media (min-width: 640px) {
		.time-slots {
			grid-template-columns: repeat(4, 1fr);
		}
	}

	.time-slot {
		display: flex;
		flex-direction: column;
		align-items: center;
		padding: 0.75rem 0.5rem;
		background-color: #f9fafb;
		border: 2px solid transparent;
		border-radius: 0.5rem;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.time-slot:hover {
		background-color: #f3f4f6;
		border-color: #d1d5db;
	}

	.time-slot.selected {
		background-color: var(--color-primary, #3b82f6);
		border-color: var(--color-primary, #3b82f6);
		color: white;
	}

	.slot-time {
		font-weight: 600;
		font-size: 0.9375rem;
	}

	.slot-walker {
		font-size: 0.75rem;
		opacity: 0.8;
		margin-top: 0.125rem;
	}

	.continue-section {
		position: fixed;
		bottom: 0;
		left: 0;
		right: 0;
		background-color: white;
		padding: 1rem;
		box-shadow: 0 -4px 6px -1px rgb(0 0 0 / 0.1);
	}

	.selection-summary {
		text-align: center;
		margin-bottom: 0.75rem;
		color: #374151;
		font-size: 0.875rem;
	}

	.selection-summary p {
		margin: 0;
	}

	.continue-button {
		width: 100%;
		max-width: 24rem;
		margin: 0 auto;
		display: block;
		padding: 0.875rem 1.5rem;
		background-color: var(--color-primary, #3b82f6);
		color: white;
		font-weight: 600;
		font-size: 1rem;
		border: none;
		border-radius: 0.5rem;
		cursor: pointer;
		transition: background-color 0.15s ease;
	}

	.continue-button:hover {
		background-color: var(--color-secondary, #1e40af);
	}

	.continue-button:focus {
		outline: none;
		box-shadow: 0 0 0 3px rgb(59 130 246 / 0.5);
	}
</style>
