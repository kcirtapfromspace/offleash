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

	let services = $state<Service[]>([]);
	let isLoading = $state(true);
	let error = $state<string | null>(null);

	const branding = $derived($page.data.branding);
	const cssVariables = $derived(generateCssVariables(branding));

	onMount(() => {
		fetchServices();
	});

	async function fetchServices() {
		try {
			isLoading = true;
			error = null;
			const data = await get<Service[]>('/services');
			services = data.filter((s) => s.is_active);
		} catch (err) {
			if (err instanceof ApiError) {
				error = err.message || 'Failed to load services';
			} else {
				error = 'An unexpected error occurred';
			}
		} finally {
			isLoading = false;
		}
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

	function formatPrice(cents: number): string {
		return `$${(cents / 100).toFixed(2)}`;
	}

	function handleServiceClick(service: Service) {
		goto(`/book/${service.id}`);
	}
</script>

<svelte:head>
	<title>Services - {branding.company_name}</title>
</svelte:head>

<div class="services-page" style={cssVariables}>
	<header class="header">
		{#if branding.logo_url}
			<img src={branding.logo_url} alt={branding.company_name} class="logo" />
		{:else}
			<h1 class="company-name">{branding.company_name}</h1>
		{/if}
	</header>

	<main class="main">
		<h2 class="page-title">Our Services</h2>
		<p class="page-subtitle">Select a service to book an appointment</p>

		{#if isLoading}
			<div class="loading-grid">
				{#each [1, 2, 3, 4, 5, 6] as _}
					<div class="skeleton-card">
						<div class="skeleton-title"></div>
						<div class="skeleton-description"></div>
						<div class="skeleton-meta"></div>
					</div>
				{/each}
			</div>
		{:else if error}
			<div class="error-message" role="alert">
				<p>{error}</p>
				<button onclick={fetchServices} class="retry-button">Try Again</button>
			</div>
		{:else if services.length === 0}
			<div class="empty-state">
				<div class="empty-icon">
					<svg fill="none" stroke="currentColor" viewBox="0 0 24 24" width="64" height="64">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="1.5"
							d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"
						/>
					</svg>
				</div>
				<h3>No Services Available</h3>
				<p>There are no services available at this time. Please check back later.</p>
			</div>
		{:else}
			<div class="services-grid">
				{#each services as service (service.id)}
					<button type="button" class="service-card" onclick={() => handleServiceClick(service)}>
						<h3 class="service-name">{service.name}</h3>
						{#if service.description}
							<p class="service-description">{service.description}</p>
						{/if}
						<div class="service-meta">
							<span class="service-duration">
								<svg fill="none" stroke="currentColor" viewBox="0 0 24 24" width="16" height="16">
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										stroke-width="2"
										d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
									/>
								</svg>
								{formatDuration(service.duration_minutes)}
							</span>
							<span class="service-price">
								{service.price_display || formatPrice(service.price_cents)}
							</span>
						</div>
					</button>
				{/each}
			</div>
		{/if}
	</main>
</div>

<style>
	.services-page {
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
		max-width: 72rem;
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

	.services-grid {
		display: grid;
		grid-template-columns: repeat(1, 1fr);
		gap: 1rem;
	}

	@media (min-width: 640px) {
		.services-grid {
			grid-template-columns: repeat(2, 1fr);
		}
	}

	@media (min-width: 1024px) {
		.services-grid {
			grid-template-columns: repeat(3, 1fr);
		}
	}

	.service-card {
		background-color: white;
		border: 2px solid transparent;
		border-radius: 0.75rem;
		padding: 1.5rem;
		text-align: left;
		cursor: pointer;
		transition: all 0.2s ease;
		box-shadow: 0 1px 3px 0 rgb(0 0 0 / 0.1);
		display: flex;
		flex-direction: column;
		width: 100%;
	}

	.service-card:hover {
		border-color: var(--color-primary, #3b82f6);
		box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1);
		transform: translateY(-2px);
	}

	.service-card:focus {
		outline: none;
		border-color: var(--color-primary, #3b82f6);
		box-shadow: 0 0 0 3px rgb(59 130 246 / 0.2);
	}

	.service-name {
		font-size: 1.125rem;
		font-weight: 600;
		color: #111827;
		margin: 0 0 0.5rem;
	}

	.service-description {
		color: #6b7280;
		font-size: 0.875rem;
		margin: 0 0 1rem;
		flex-grow: 1;
		display: -webkit-box;
		-webkit-line-clamp: 3;
		line-clamp: 3;
		-webkit-box-orient: vertical;
		overflow: hidden;
	}

	.service-meta {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding-top: 1rem;
		border-top: 1px solid #e5e7eb;
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

	/* Loading skeleton */
	.loading-grid {
		display: grid;
		grid-template-columns: repeat(1, 1fr);
		gap: 1rem;
	}

	@media (min-width: 640px) {
		.loading-grid {
			grid-template-columns: repeat(2, 1fr);
		}
	}

	@media (min-width: 1024px) {
		.loading-grid {
			grid-template-columns: repeat(3, 1fr);
		}
	}

	.skeleton-card {
		background-color: white;
		border-radius: 0.75rem;
		padding: 1.5rem;
		box-shadow: 0 1px 3px 0 rgb(0 0 0 / 0.1);
	}

	.skeleton-title {
		height: 1.5rem;
		width: 60%;
		background-color: #e5e7eb;
		border-radius: 0.25rem;
		margin-bottom: 0.75rem;
		animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
	}

	.skeleton-description {
		height: 1rem;
		width: 100%;
		background-color: #e5e7eb;
		border-radius: 0.25rem;
		margin-bottom: 0.5rem;
		animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
	}

	.skeleton-meta {
		height: 1rem;
		width: 40%;
		background-color: #e5e7eb;
		border-radius: 0.25rem;
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

	/* Error state */
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

	/* Empty state */
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
		margin: 0;
	}
</style>
