<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { get, post, ApiError } from '$lib/api';
	import { generateCssVariables } from '$lib/stores/branding';
	import { onMount } from 'svelte';

	interface Service {
		id: string;
		name: string;
		description: string | null;
		duration_minutes: number;
		price_cents: number;
		price_display: string;
	}

	interface Location {
		id: string;
		name: string | null;
		address: string;
		city: string;
		state: string;
		zip: string;
	}

	interface BookingRequest {
		walker_id: string;
		service_id: string;
		location_id: string;
		start_time: string;
		notes?: string;
	}

	interface BookingResponse {
		id: string;
		customer_id: string;
		walker_id: string;
		service_id: string;
		location_id: string;
		status: string;
		scheduled_start: string;
		scheduled_end: string;
		price_cents: number;
		price_display: string;
		notes: string | null;
	}

	let service = $state<Service | null>(null);
	let location = $state<Location | null>(null);
	let isLoading = $state(true);
	let isSubmitting = $state(false);
	let error = $state<string | null>(null);
	let notes = $state('');
	let bookingSuccess = $state(false);
	let createdBooking = $state<BookingResponse | null>(null);

	const branding = $derived($page.data.branding);
	const cssVariables = $derived(generateCssVariables(branding));

	// Get booking parameters from URL
	const serviceId = $derived($page.url.searchParams.get('service_id'));
	const locationId = $derived($page.url.searchParams.get('location_id'));
	const walkerId = $derived($page.url.searchParams.get('walker_id'));
	const startTime = $derived($page.url.searchParams.get('start_time'));

	onMount(() => {
		loadBookingDetails();
	});

	async function loadBookingDetails() {
		if (!serviceId || !locationId || !startTime) {
			error = 'Missing booking details. Please start the booking process again.';
			isLoading = false;
			return;
		}

		try {
			isLoading = true;
			error = null;

			const [serviceData, locationData] = await Promise.all([
				get<Service>(`/services/${serviceId}`),
				get<Location>(`/locations/${locationId}`)
			]);

			service = serviceData;
			location = locationData;
		} catch (err) {
			if (err instanceof ApiError) {
				error = err.message || 'Failed to load booking details';
			} else {
				error = 'An unexpected error occurred';
			}
		} finally {
			isLoading = false;
		}
	}

	function formatDate(isoString: string): string {
		const date = new Date(isoString);
		return date.toLocaleDateString('en-US', {
			weekday: 'long',
			year: 'numeric',
			month: 'long',
			day: 'numeric'
		});
	}

	function formatTime(isoString: string): string {
		const date = new Date(isoString);
		return date.toLocaleTimeString('en-US', {
			hour: 'numeric',
			minute: '2-digit',
			hour12: true
		});
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

	function formatAddress(loc: Location): string {
		const parts = [loc.address, loc.city, `${loc.state} ${loc.zip}`];
		return parts.filter(Boolean).join(', ');
	}

	async function handleConfirmBooking() {
		if (!serviceId || !locationId || !startTime) {
			error = 'Missing required booking information';
			return;
		}

		try {
			isSubmitting = true;
			error = null;

			const bookingRequest: BookingRequest = {
				walker_id: walkerId || '',
				service_id: serviceId,
				location_id: locationId,
				start_time: startTime
			};

			if (notes.trim()) {
				bookingRequest.notes = notes.trim();
			}

			const response = await post<BookingResponse>('/bookings', bookingRequest);
			createdBooking = response;
			bookingSuccess = true;
		} catch (err) {
			if (err instanceof ApiError) {
				error = err.message || 'Failed to create booking';
			} else {
				error = 'An unexpected error occurred while creating your booking';
			}
		} finally {
			isSubmitting = false;
		}
	}

	function handleViewBookings() {
		goto('/bookings');
	}

	function handleBookAnother() {
		goto('/services');
	}
</script>

<svelte:head>
	<title>{bookingSuccess ? 'Booking Confirmed' : 'Confirm Booking'} - {branding.company_name}</title>
</svelte:head>

<div class="confirmation-page" style={cssVariables}>
	<header class="header">
		{#if branding.logo_url}
			<img src={branding.logo_url} alt={branding.company_name} class="logo" />
		{:else}
			<h1 class="company-name">{branding.company_name}</h1>
		{/if}
	</header>

	<main class="main">
		{#if bookingSuccess && createdBooking}
			<!-- Success State -->
			<div class="success-container">
				<div class="success-icon">
					<svg fill="none" stroke="currentColor" viewBox="0 0 24 24" width="64" height="64">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
						/>
					</svg>
				</div>
				<h2 class="success-title">Booking Confirmed!</h2>
				<p class="success-subtitle">Your booking has been successfully created.</p>

				<div class="booking-details-card">
					<h3 class="details-title">Booking Details</h3>

					<div class="detail-row">
						<span class="detail-label">Confirmation #</span>
						<span class="detail-value confirmation-number">{createdBooking.id.slice(0, 8).toUpperCase()}</span>
					</div>

					<div class="detail-row">
						<span class="detail-label">Service</span>
						<span class="detail-value">{service?.name}</span>
					</div>

					<div class="detail-row">
						<span class="detail-label">Date</span>
						<span class="detail-value">{formatDate(createdBooking.scheduled_start)}</span>
					</div>

					<div class="detail-row">
						<span class="detail-label">Time</span>
						<span class="detail-value">
							{formatTime(createdBooking.scheduled_start)} - {formatTime(createdBooking.scheduled_end)}
						</span>
					</div>

					{#if location}
						<div class="detail-row">
							<span class="detail-label">Location</span>
							<span class="detail-value">{formatAddress(location)}</span>
						</div>
					{/if}

					<div class="detail-row">
						<span class="detail-label">Total</span>
						<span class="detail-value price">{createdBooking.price_display}</span>
					</div>

					<div class="detail-row">
						<span class="detail-label">Status</span>
						<span class="detail-value status-badge">{createdBooking.status}</span>
					</div>

					{#if createdBooking.notes}
						<div class="detail-row notes-row">
							<span class="detail-label">Notes</span>
							<span class="detail-value">{createdBooking.notes}</span>
						</div>
					{/if}
				</div>

				<div class="success-actions">
					<button type="button" class="primary-button" onclick={handleViewBookings}>
						View My Bookings
					</button>
					<button type="button" class="secondary-button" onclick={handleBookAnother}>
						Book Another Service
					</button>
				</div>
			</div>
		{:else if isLoading}
			<!-- Loading State -->
			<div class="loading-container">
				<div class="skeleton-card">
					<div class="skeleton-title"></div>
					<div class="skeleton-row"></div>
					<div class="skeleton-row"></div>
					<div class="skeleton-row"></div>
					<div class="skeleton-row"></div>
					<div class="skeleton-button"></div>
				</div>
			</div>
		{:else if error && !service}
			<!-- Error State -->
			<div class="error-container">
				<div class="error-icon">
					<svg fill="none" stroke="currentColor" viewBox="0 0 24 24" width="48" height="48">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
						/>
					</svg>
				</div>
				<h2 class="error-title">Unable to Load Booking</h2>
				<p class="error-message">{error}</p>
				<button type="button" class="primary-button" onclick={() => goto('/services')}>
					Back to Services
				</button>
			</div>
		{:else if service && location && startTime}
			<!-- Confirmation Form -->
			<h2 class="page-title">Confirm Your Booking</h2>
			<p class="page-subtitle">Please review your booking details before confirming</p>

			<div class="confirmation-card">
				<div class="summary-section">
					<h3 class="section-title">Booking Summary</h3>

					<div class="summary-item">
						<div class="summary-icon">
							<svg fill="none" stroke="currentColor" viewBox="0 0 24 24" width="24" height="24">
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="M21 13.255A23.931 23.931 0 0112 15c-3.183 0-6.22-.62-9-1.745M16 6V4a2 2 0 00-2-2h-4a2 2 0 00-2 2v2m4 6h.01M5 20h14a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"
								/>
							</svg>
						</div>
						<div class="summary-content">
							<span class="summary-label">Service</span>
							<span class="summary-value">{service.name}</span>
							{#if service.description}
								<span class="summary-description">{service.description}</span>
							{/if}
						</div>
					</div>

					<div class="summary-item">
						<div class="summary-icon">
							<svg fill="none" stroke="currentColor" viewBox="0 0 24 24" width="24" height="24">
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"
								/>
							</svg>
						</div>
						<div class="summary-content">
							<span class="summary-label">Date</span>
							<span class="summary-value">{formatDate(startTime)}</span>
						</div>
					</div>

					<div class="summary-item">
						<div class="summary-icon">
							<svg fill="none" stroke="currentColor" viewBox="0 0 24 24" width="24" height="24">
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
								/>
							</svg>
						</div>
						<div class="summary-content">
							<span class="summary-label">Time</span>
							<span class="summary-value">{formatTime(startTime)}</span>
							<span class="summary-description">Duration: {formatDuration(service.duration_minutes)}</span>
						</div>
					</div>

					<div class="summary-item">
						<div class="summary-icon">
							<svg fill="none" stroke="currentColor" viewBox="0 0 24 24" width="24" height="24">
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z"
								/>
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="M15 11a3 3 0 11-6 0 3 3 0 016 0z"
								/>
							</svg>
						</div>
						<div class="summary-content">
							<span class="summary-label">Location</span>
							<span class="summary-value">{location.name || 'Service Location'}</span>
							<span class="summary-description">{formatAddress(location)}</span>
						</div>
					</div>

					<div class="price-summary">
						<span class="price-label">Total Price</span>
						<span class="price-value">{service.price_display}</span>
					</div>
				</div>

				<div class="notes-section">
					<label for="notes" class="notes-label">
						Add Notes (optional)
					</label>
					<textarea
						id="notes"
						bind:value={notes}
						placeholder="Any special instructions or requests for your booking..."
						class="notes-input"
						rows="3"
					></textarea>
				</div>

				{#if error}
					<div class="form-error" role="alert">
						<svg fill="none" stroke="currentColor" viewBox="0 0 24 24" width="20" height="20">
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
							/>
						</svg>
						<span>{error}</span>
					</div>
				{/if}

				<div class="action-buttons">
					<button
						type="button"
						class="back-button"
						onclick={() => history.back()}
						disabled={isSubmitting}
					>
						Back
					</button>
					<button
						type="button"
						class="confirm-button"
						onclick={handleConfirmBooking}
						disabled={isSubmitting}
					>
						{#if isSubmitting}
							<span class="spinner"></span>
							Confirming...
						{:else}
							Confirm Booking
						{/if}
					</button>
				</div>
			</div>
		{/if}
	</main>
</div>

<style>
	.confirmation-page {
		min-height: 100vh;
		background-color: #f3f4f6;
	}

	.header {
		background-color: white;
		padding: 1rem;
		box-shadow: 0 1px 3px 0 rgb(0 0 0 / 0.1);
		display: flex;
		justify-content: center;
		align-items: center;
	}

	.logo {
		max-height: 2.5rem;
	}

	.company-name {
		font-size: 1.5rem;
		font-weight: 700;
		color: var(--color-primary, #3b82f6);
		margin: 0;
	}

	.main {
		max-width: 40rem;
		margin: 0 auto;
		padding: 2rem 1rem;
	}

	.page-title {
		font-size: 1.875rem;
		font-weight: 700;
		color: #111827;
		margin: 0 0 0.5rem;
		text-align: center;
	}

	.page-subtitle {
		color: #6b7280;
		text-align: center;
		margin: 0 0 2rem;
	}

	/* Confirmation Card */
	.confirmation-card {
		background-color: white;
		border-radius: 0.75rem;
		box-shadow: 0 1px 3px 0 rgb(0 0 0 / 0.1);
		overflow: hidden;
	}

	.summary-section {
		padding: 1.5rem;
	}

	.section-title {
		font-size: 1.125rem;
		font-weight: 600;
		color: #111827;
		margin: 0 0 1.25rem;
	}

	.summary-item {
		display: flex;
		gap: 1rem;
		padding: 1rem 0;
		border-bottom: 1px solid #e5e7eb;
	}

	.summary-item:last-of-type {
		border-bottom: none;
	}

	.summary-icon {
		flex-shrink: 0;
		width: 2.5rem;
		height: 2.5rem;
		background-color: #f3f4f6;
		border-radius: 0.5rem;
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--color-primary, #3b82f6);
	}

	.summary-content {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.summary-label {
		font-size: 0.75rem;
		font-weight: 500;
		color: #6b7280;
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.summary-value {
		font-size: 1rem;
		font-weight: 600;
		color: #111827;
	}

	.summary-description {
		font-size: 0.875rem;
		color: #6b7280;
	}

	.price-summary {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 1.25rem;
		background-color: #f9fafb;
		margin-top: 1rem;
		border-radius: 0.5rem;
	}

	.price-label {
		font-size: 1rem;
		font-weight: 600;
		color: #374151;
	}

	.price-value {
		font-size: 1.5rem;
		font-weight: 700;
		color: var(--color-primary, #3b82f6);
	}

	/* Notes Section */
	.notes-section {
		padding: 1.5rem;
		border-top: 1px solid #e5e7eb;
	}

	.notes-label {
		display: block;
		font-size: 0.875rem;
		font-weight: 500;
		color: #374151;
		margin-bottom: 0.5rem;
	}

	.notes-input {
		width: 100%;
		padding: 0.75rem;
		border: 1px solid #d1d5db;
		border-radius: 0.5rem;
		font-size: 1rem;
		resize: vertical;
		font-family: inherit;
	}

	.notes-input:focus {
		outline: none;
		border-color: var(--color-primary, #3b82f6);
		box-shadow: 0 0 0 3px rgb(59 130 246 / 0.2);
	}

	.notes-input::placeholder {
		color: #9ca3af;
	}

	/* Form Error */
	.form-error {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.75rem 1rem;
		margin: 0 1.5rem;
		background-color: #fef2f2;
		border: 1px solid #fecaca;
		border-radius: 0.5rem;
		color: #dc2626;
		font-size: 0.875rem;
	}

	/* Action Buttons */
	.action-buttons {
		display: flex;
		gap: 1rem;
		padding: 1.5rem;
		border-top: 1px solid #e5e7eb;
	}

	.back-button {
		flex: 1;
		padding: 0.875rem 1.5rem;
		background-color: white;
		color: #374151;
		border: 1px solid #d1d5db;
		border-radius: 0.5rem;
		font-size: 1rem;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.back-button:hover:not(:disabled) {
		background-color: #f3f4f6;
	}

	.back-button:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.confirm-button {
		flex: 2;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 0.5rem;
		padding: 0.875rem 1.5rem;
		background-color: var(--color-primary, #3b82f6);
		color: white;
		border: none;
		border-radius: 0.5rem;
		font-size: 1rem;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.confirm-button:hover:not(:disabled) {
		background-color: var(--color-secondary, #1e40af);
	}

	.confirm-button:focus {
		outline: none;
		box-shadow: 0 0 0 3px rgb(59 130 246 / 0.2);
	}

	.confirm-button:disabled {
		opacity: 0.7;
		cursor: not-allowed;
	}

	.spinner {
		width: 1rem;
		height: 1rem;
		border: 2px solid transparent;
		border-top-color: currentColor;
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	/* Success State */
	.success-container {
		text-align: center;
	}

	.success-icon {
		color: var(--color-accent, #10b981);
		margin-bottom: 1rem;
	}

	.success-title {
		font-size: 1.875rem;
		font-weight: 700;
		color: #111827;
		margin: 0 0 0.5rem;
	}

	.success-subtitle {
		color: #6b7280;
		margin: 0 0 2rem;
	}

	.booking-details-card {
		background-color: white;
		border-radius: 0.75rem;
		box-shadow: 0 1px 3px 0 rgb(0 0 0 / 0.1);
		padding: 1.5rem;
		text-align: left;
		margin-bottom: 2rem;
	}

	.details-title {
		font-size: 1rem;
		font-weight: 600;
		color: #374151;
		margin: 0 0 1rem;
		padding-bottom: 0.75rem;
		border-bottom: 1px solid #e5e7eb;
	}

	.detail-row {
		display: flex;
		justify-content: space-between;
		padding: 0.75rem 0;
		border-bottom: 1px solid #f3f4f6;
	}

	.detail-row:last-child {
		border-bottom: none;
	}

	.detail-row.notes-row {
		flex-direction: column;
		gap: 0.25rem;
	}

	.detail-label {
		font-size: 0.875rem;
		color: #6b7280;
	}

	.detail-value {
		font-size: 0.875rem;
		font-weight: 500;
		color: #111827;
	}

	.confirmation-number {
		font-family: monospace;
		font-weight: 600;
		color: var(--color-primary, #3b82f6);
	}

	.detail-value.price {
		font-weight: 700;
		color: var(--color-primary, #3b82f6);
	}

	.status-badge {
		display: inline-block;
		padding: 0.25rem 0.75rem;
		background-color: #fef3c7;
		color: #92400e;
		border-radius: 9999px;
		font-size: 0.75rem;
		font-weight: 500;
		text-transform: capitalize;
	}

	.success-actions {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.primary-button {
		width: 100%;
		padding: 0.875rem 1.5rem;
		background-color: var(--color-primary, #3b82f6);
		color: white;
		border: none;
		border-radius: 0.5rem;
		font-size: 1rem;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.primary-button:hover {
		background-color: var(--color-secondary, #1e40af);
	}

	.secondary-button {
		width: 100%;
		padding: 0.875rem 1.5rem;
		background-color: white;
		color: #374151;
		border: 1px solid #d1d5db;
		border-radius: 0.5rem;
		font-size: 1rem;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.secondary-button:hover {
		background-color: #f3f4f6;
	}

	/* Loading State */
	.loading-container {
		padding: 2rem 0;
	}

	.skeleton-card {
		background-color: white;
		border-radius: 0.75rem;
		padding: 1.5rem;
		box-shadow: 0 1px 3px 0 rgb(0 0 0 / 0.1);
	}

	.skeleton-title {
		height: 1.5rem;
		width: 40%;
		background-color: #e5e7eb;
		border-radius: 0.25rem;
		margin-bottom: 1.5rem;
		animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
	}

	.skeleton-row {
		height: 3.5rem;
		background-color: #e5e7eb;
		border-radius: 0.25rem;
		margin-bottom: 1rem;
		animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
	}

	.skeleton-button {
		height: 3rem;
		background-color: #e5e7eb;
		border-radius: 0.5rem;
		margin-top: 1.5rem;
		animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
	}

	@keyframes pulse {
		0%,
		100% {
			opacity: 1;
		}
		50% {
			opacity: 0.5;
		}
	}

	/* Error State */
	.error-container {
		text-align: center;
		padding: 3rem 1rem;
		background-color: white;
		border-radius: 0.75rem;
		box-shadow: 0 1px 3px 0 rgb(0 0 0 / 0.1);
	}

	.error-icon {
		color: #dc2626;
		margin-bottom: 1rem;
	}

	.error-title {
		font-size: 1.25rem;
		font-weight: 600;
		color: #111827;
		margin: 0 0 0.5rem;
	}

	.error-message {
		color: #6b7280;
		margin: 0 0 1.5rem;
	}

	@media (min-width: 640px) {
		.success-actions {
			flex-direction: row;
		}

		.primary-button,
		.secondary-button {
			width: auto;
			flex: 1;
		}
	}
</style>
