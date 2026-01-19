<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { get, del, getToken, ApiError } from '$lib/api';
	import { generateCssVariables } from '$lib/stores/branding';
	import { onMount } from 'svelte';

	interface Service {
		id: string;
		name: string;
	}

	interface Booking {
		id: string;
		service_id: string;
		service: Service;
		status: 'pending' | 'confirmed' | 'completed' | 'cancelled';
		scheduled_start: string;
		scheduled_end: string;
		location_id: string;
		price_cents: number;
		price_display: string;
		notes: string | null;
	}

	let bookings = $state<Booking[]>([]);
	let isLoading = $state(true);
	let error = $state<string | null>(null);
	let activeTab = $state<'upcoming' | 'past'>('upcoming');
	let cancellingId = $state<string | null>(null);
	let cancelError = $state<string | null>(null);

	const branding = $derived($page.data.branding);
	const cssVariables = $derived(generateCssVariables(branding));

	const now = $derived(new Date());
	const upcomingBookings = $derived(
		bookings.filter(
			(b) => new Date(b.scheduled_start) > now && b.status !== 'cancelled' && b.status !== 'completed'
		)
	);
	const pastBookings = $derived(
		bookings.filter(
			(b) => new Date(b.scheduled_start) <= now || b.status === 'cancelled' || b.status === 'completed'
		)
	);
	const displayedBookings = $derived(activeTab === 'upcoming' ? upcomingBookings : pastBookings);

	onMount(() => {
		const token = getToken();
		if (!token) {
			goto('/login');
			return;
		}
		fetchBookings();
	});

	async function fetchBookings() {
		try {
			isLoading = true;
			error = null;
			const data = await get<Booking[]>('/bookings');
			bookings = data;
		} catch (err) {
			if (err instanceof ApiError) {
				error = err.message || 'Failed to load bookings';
			} else {
				error = 'An unexpected error occurred';
			}
		} finally {
			isLoading = false;
		}
	}

	async function cancelBooking(bookingId: string) {
		try {
			cancellingId = bookingId;
			cancelError = null;
			await del(`/bookings/${bookingId}`);
			// Update the booking status locally
			bookings = bookings.map((b) => (b.id === bookingId ? { ...b, status: 'cancelled' as const } : b));
		} catch (err) {
			if (err instanceof ApiError) {
				cancelError = err.message || 'Failed to cancel booking';
			} else {
				cancelError = 'An unexpected error occurred';
			}
		} finally {
			cancellingId = null;
		}
	}

	function formatDate(dateString: string): string {
		const date = new Date(dateString);
		return date.toLocaleDateString('en-US', {
			weekday: 'short',
			month: 'short',
			day: 'numeric',
			year: 'numeric'
		});
	}

	function formatTime(dateString: string): string {
		const date = new Date(dateString);
		return date.toLocaleTimeString('en-US', {
			hour: 'numeric',
			minute: '2-digit',
			hour12: true
		});
	}

	function formatPrice(cents: number): string {
		return `$${(cents / 100).toFixed(2)}`;
	}

	function getStatusColor(status: string): string {
		switch (status) {
			case 'confirmed':
				return '#10b981';
			case 'pending':
				return '#f59e0b';
			case 'completed':
				return '#6b7280';
			case 'cancelled':
				return '#ef4444';
			default:
				return '#6b7280';
		}
	}

	function canCancel(booking: Booking): boolean {
		return (
			(booking.status === 'pending' || booking.status === 'confirmed') &&
			new Date(booking.scheduled_start) > new Date()
		);
	}
</script>

<svelte:head>
	<title>My Bookings - {branding.company_name}</title>
</svelte:head>

<div class="bookings-page" style={cssVariables}>
	<header class="header">
		{#if branding.logo_url}
			<img src={branding.logo_url} alt={branding.company_name} class="logo" />
		{:else}
			<h1 class="company-name">{branding.company_name}</h1>
		{/if}
	</header>

	<main class="main">
		<h2 class="page-title">My Bookings</h2>

		{#if isLoading}
			<div class="loading-container">
				{#each [1, 2, 3] as _}
					<div class="skeleton-card">
						<div class="skeleton-header">
							<div class="skeleton-title"></div>
							<div class="skeleton-badge"></div>
						</div>
						<div class="skeleton-details"></div>
						<div class="skeleton-details"></div>
						<div class="skeleton-button"></div>
					</div>
				{/each}
			</div>
		{:else if error}
			<div class="error-message" role="alert">
				<p>{error}</p>
				<button onclick={fetchBookings} class="retry-button">Try Again</button>
			</div>
		{:else if bookings.length === 0}
			<div class="empty-state">
				<div class="empty-icon">
					<svg fill="none" stroke="currentColor" viewBox="0 0 24 24" width="64" height="64">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="1.5"
							d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2"
						/>
					</svg>
				</div>
				<h3>No Bookings Yet</h3>
				<p>You haven't made any bookings yet. Start by browsing our services.</p>
				<a href="/services" class="book-service-link">Browse Services</a>
			</div>
		{:else}
			<div class="tabs">
				<button
					class="tab-button"
					class:active={activeTab === 'upcoming'}
					onclick={() => (activeTab = 'upcoming')}
				>
					Upcoming ({upcomingBookings.length})
				</button>
				<button
					class="tab-button"
					class:active={activeTab === 'past'}
					onclick={() => (activeTab = 'past')}
				>
					Past ({pastBookings.length})
				</button>
			</div>

			{#if displayedBookings.length === 0}
				<div class="empty-tab-state">
					<p>
						{activeTab === 'upcoming'
							? 'You have no upcoming bookings.'
							: 'You have no past bookings.'}
					</p>
					{#if activeTab === 'upcoming'}
						<a href="/services" class="book-service-link-small">Book a Service</a>
					{/if}
				</div>
			{:else}
				<div class="bookings-list">
					{#each displayedBookings as booking (booking.id)}
						<div class="booking-card">
							<div class="booking-header">
								<h3 class="service-name">{booking.service.name}</h3>
								<span
									class="status-badge"
									style="background-color: {getStatusColor(booking.status)}"
								>
									{booking.status.charAt(0).toUpperCase() + booking.status.slice(1)}
								</span>
							</div>

							<div class="booking-details">
								<div class="detail-item">
									<svg fill="none" stroke="currentColor" viewBox="0 0 24 24" width="18" height="18">
										<path
											stroke-linecap="round"
											stroke-linejoin="round"
											stroke-width="2"
											d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"
										/>
									</svg>
									<span class="detail-text">{formatDate(booking.scheduled_start)}</span>
								</div>
								<div class="detail-item">
									<svg fill="none" stroke="currentColor" viewBox="0 0 24 24" width="18" height="18">
										<path
											stroke-linecap="round"
											stroke-linejoin="round"
											stroke-width="2"
											d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
										/>
									</svg>
									<span class="detail-text">
										{formatTime(booking.scheduled_start)} - {formatTime(booking.scheduled_end)}
									</span>
								</div>
								<div class="detail-item">
									<svg fill="none" stroke="currentColor" viewBox="0 0 24 24" width="18" height="18">
										<path
											stroke-linecap="round"
											stroke-linejoin="round"
											stroke-width="2"
											d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
										/>
									</svg>
									<span class="detail-text price">
										{booking.price_display || formatPrice(booking.price_cents)}
									</span>
								</div>
							</div>

							{#if booking.notes}
								<div class="booking-notes">
									<span class="notes-label">Notes:</span>
									<p class="notes-text">{booking.notes}</p>
								</div>
							{/if}

							{#if canCancel(booking)}
								<div class="booking-actions">
									<button
										class="cancel-button"
										disabled={cancellingId === booking.id}
										onclick={() => cancelBooking(booking.id)}
									>
										{cancellingId === booking.id ? 'Cancelling...' : 'Cancel Booking'}
									</button>
								</div>
								{#if cancelError && cancellingId === booking.id}
									<p class="cancel-error">{cancelError}</p>
								{/if}
							{/if}
						</div>
					{/each}
				</div>
			{/if}
		{/if}
	</main>
</div>

<style>
	.bookings-page {
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
		max-width: 48rem;
		margin: 0 auto;
		padding: 2rem 1rem;
	}

	.page-title {
		font-size: 1.875rem;
		font-weight: 700;
		color: #111827;
		margin: 0 0 1.5rem;
		text-align: center;
	}

	/* Tabs */
	.tabs {
		display: flex;
		gap: 0;
		margin-bottom: 1.5rem;
		background-color: white;
		border-radius: 0.5rem;
		padding: 0.25rem;
		box-shadow: 0 1px 3px 0 rgb(0 0 0 / 0.1);
	}

	.tab-button {
		flex: 1;
		background: none;
		border: none;
		padding: 0.75rem 1rem;
		color: #6b7280;
		font-weight: 500;
		cursor: pointer;
		border-radius: 0.375rem;
		transition: all 0.2s ease;
	}

	.tab-button.active {
		background-color: var(--color-primary, #3b82f6);
		color: white;
	}

	.tab-button:hover:not(.active) {
		background-color: #f3f4f6;
		color: #374151;
	}

	/* Bookings List */
	.bookings-list {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.booking-card {
		background-color: white;
		border: 1px solid #e5e7eb;
		border-radius: 0.75rem;
		padding: 1.25rem;
		box-shadow: 0 1px 3px 0 rgb(0 0 0 / 0.1);
		transition: all 0.2s ease;
	}

	.booking-card:hover {
		box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1);
		border-color: var(--color-primary, #3b82f6);
	}

	.booking-header {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		margin-bottom: 1rem;
		gap: 0.75rem;
	}

	.service-name {
		font-size: 1.125rem;
		font-weight: 600;
		color: #111827;
		margin: 0;
		flex-grow: 1;
	}

	.status-badge {
		display: inline-block;
		padding: 0.25rem 0.75rem;
		border-radius: 9999px;
		color: white;
		font-size: 0.75rem;
		font-weight: 600;
		text-transform: uppercase;
		white-space: nowrap;
		flex-shrink: 0;
	}

	.booking-details {
		display: flex;
		flex-direction: column;
		gap: 0.625rem;
		padding-bottom: 1rem;
		border-bottom: 1px solid #e5e7eb;
	}

	.detail-item {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		color: #6b7280;
	}

	.detail-item svg {
		flex-shrink: 0;
		color: var(--color-accent, #10b981);
	}

	.detail-text {
		font-size: 0.875rem;
	}

	.detail-text.price {
		font-weight: 600;
		color: var(--color-primary, #3b82f6);
	}

	.booking-notes {
		margin-top: 1rem;
		padding: 0.75rem;
		background-color: #f9fafb;
		border-radius: 0.375rem;
		border-left: 3px solid var(--color-accent, #10b981);
	}

	.notes-label {
		font-size: 0.75rem;
		font-weight: 600;
		color: #6b7280;
		text-transform: uppercase;
	}

	.notes-text {
		margin: 0.25rem 0 0;
		font-size: 0.875rem;
		color: #374151;
	}

	.booking-actions {
		margin-top: 1rem;
		padding-top: 1rem;
		border-top: 1px solid #e5e7eb;
	}

	.cancel-button {
		background-color: #fee2e2;
		color: #dc2626;
		padding: 0.5rem 1rem;
		border: 1px solid #fecaca;
		border-radius: 0.375rem;
		font-weight: 500;
		font-size: 0.875rem;
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.cancel-button:hover:not(:disabled) {
		background-color: #fecaca;
		border-color: #f87171;
	}

	.cancel-button:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.cancel-error {
		margin: 0.5rem 0 0;
		color: #dc2626;
		font-size: 0.875rem;
	}

	/* Loading State */
	.loading-container {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.skeleton-card {
		background-color: white;
		border-radius: 0.75rem;
		padding: 1.25rem;
		box-shadow: 0 1px 3px 0 rgb(0 0 0 / 0.1);
	}

	.skeleton-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 1rem;
	}

	.skeleton-title {
		height: 1.5rem;
		width: 50%;
		background-color: #e5e7eb;
		border-radius: 0.25rem;
		animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
	}

	.skeleton-badge {
		height: 1.5rem;
		width: 5rem;
		background-color: #e5e7eb;
		border-radius: 9999px;
		animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
	}

	.skeleton-details {
		height: 1rem;
		width: 70%;
		background-color: #e5e7eb;
		border-radius: 0.25rem;
		margin-bottom: 0.5rem;
		animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
	}

	.skeleton-button {
		height: 2.25rem;
		width: 8rem;
		background-color: #e5e7eb;
		border-radius: 0.375rem;
		margin-top: 1rem;
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

	/* Empty States */
	.empty-state {
		text-align: center;
		padding: 3rem 1rem;
		background-color: white;
		border-radius: 0.75rem;
		box-shadow: 0 1px 3px 0 rgb(0 0 0 / 0.1);
	}

	.empty-icon {
		color: #9ca3af;
		margin-bottom: 1rem;
	}

	.empty-state h3 {
		font-size: 1.25rem;
		font-weight: 600;
		color: #374151;
		margin: 0 0 0.5rem;
	}

	.empty-state p {
		color: #6b7280;
		margin: 0 0 1.5rem;
	}

	.book-service-link {
		display: inline-block;
		background-color: var(--color-primary, #3b82f6);
		color: white;
		padding: 0.75rem 1.5rem;
		border-radius: 0.5rem;
		text-decoration: none;
		font-weight: 500;
		transition: all 0.2s ease;
	}

	.book-service-link:hover {
		background-color: var(--color-secondary, #1e40af);
	}

	.empty-tab-state {
		text-align: center;
		padding: 2rem 1rem;
		background-color: white;
		border-radius: 0.75rem;
		box-shadow: 0 1px 3px 0 rgb(0 0 0 / 0.1);
	}

	.empty-tab-state p {
		color: #6b7280;
		margin: 0 0 1rem;
	}

	.book-service-link-small {
		display: inline-block;
		background-color: var(--color-primary, #3b82f6);
		color: white;
		padding: 0.5rem 1rem;
		border-radius: 0.375rem;
		text-decoration: none;
		font-weight: 500;
		font-size: 0.875rem;
		transition: all 0.2s ease;
	}

	.book-service-link-small:hover {
		background-color: var(--color-secondary, #1e40af);
	}

	@media (min-width: 640px) {
		.booking-details {
			flex-direction: row;
			flex-wrap: wrap;
			gap: 1.5rem;
		}
	}
</style>
