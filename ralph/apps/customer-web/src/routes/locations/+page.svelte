<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { get, post, put, del, getToken, ApiError } from '$lib/api';
	import { generateCssVariables } from '$lib/stores/branding';
	import { onMount } from 'svelte';

	interface Location {
		id: string;
		name: string;
		address: string;
		city: string;
		state: string;
		zip: string;
		notes: string | null;
		is_default: boolean;
	}

	interface FormErrors {
		name?: string;
		address?: string;
		city?: string;
		state?: string;
		zip?: string;
		general?: string;
	}

	let locations = $state<Location[]>([]);
	let isLoading = $state(true);
	let error = $state<string | null>(null);
	let isSubmitting = $state(false);

	// Form state
	let showForm = $state(false);
	let editingLocation = $state<Location | null>(null);
	let formName = $state('');
	let formAddress = $state('');
	let formCity = $state('');
	let formState = $state('');
	let formZip = $state('');
	let formNotes = $state('');
	let formErrors = $state<FormErrors>({});

	// Delete confirmation
	let deletingLocation = $state<Location | null>(null);

	const branding = $derived($page.data.branding);
	const cssVariables = $derived(generateCssVariables(branding));
	const isEditing = $derived(editingLocation !== null);

	onMount(() => {
		const token = getToken();
		if (!token) {
			goto('/login');
			return;
		}
		fetchLocations();
	});

	async function fetchLocations() {
		try {
			isLoading = true;
			error = null;
			const data = await get<Location[]>('/locations');
			locations = data;
		} catch (err) {
			if (err instanceof ApiError) {
				error = err.message || 'Failed to load locations';
			} else {
				error = 'An unexpected error occurred';
			}
		} finally {
			isLoading = false;
		}
	}

	function resetForm() {
		formName = '';
		formAddress = '';
		formCity = '';
		formState = '';
		formZip = '';
		formNotes = '';
		formErrors = {};
		editingLocation = null;
		showForm = false;
	}

	function openAddForm() {
		resetForm();
		showForm = true;
	}

	function openEditForm(location: Location) {
		formName = location.name;
		formAddress = location.address;
		formCity = location.city;
		formState = location.state;
		formZip = location.zip;
		formNotes = location.notes || '';
		formErrors = {};
		editingLocation = location;
		showForm = true;
	}

	function validateForm(): boolean {
		const newErrors: FormErrors = {};

		if (!formName.trim()) {
			newErrors.name = 'Name is required';
		}

		if (!formAddress.trim()) {
			newErrors.address = 'Address is required';
		}

		if (!formCity.trim()) {
			newErrors.city = 'City is required';
		}

		if (!formState.trim()) {
			newErrors.state = 'State is required';
		}

		if (!formZip.trim()) {
			newErrors.zip = 'ZIP code is required';
		} else if (!/^\d{5}(-\d{4})?$/.test(formZip.trim())) {
			newErrors.zip = 'Please enter a valid ZIP code';
		}

		formErrors = newErrors;
		return Object.keys(newErrors).length === 0;
	}

	async function handleSubmit(event: SubmitEvent) {
		event.preventDefault();

		if (!validateForm()) {
			return;
		}

		isSubmitting = true;
		formErrors = {};

		const locationData = {
			name: formName.trim(),
			address: formAddress.trim(),
			city: formCity.trim(),
			state: formState.trim(),
			zip: formZip.trim(),
			notes: formNotes.trim() || null
		};

		try {
			if (isEditing && editingLocation) {
				await put<Location>(`/locations/${editingLocation.id}`, locationData);
			} else {
				await post<Location>('/locations', locationData);
			}
			resetForm();
			await fetchLocations();
		} catch (err) {
			if (err instanceof ApiError) {
				formErrors = { general: err.message || 'Failed to save location' };
			} else {
				formErrors = { general: 'An unexpected error occurred' };
			}
		} finally {
			isSubmitting = false;
		}
	}

	async function handleSetDefault(location: Location) {
		try {
			await put<Location>(`/locations/${location.id}`, {
				...location,
				is_default: true
			});
			await fetchLocations();
		} catch (err) {
			if (err instanceof ApiError) {
				error = err.message || 'Failed to set default location';
			} else {
				error = 'An unexpected error occurred';
			}
		}
	}

	function confirmDelete(location: Location) {
		deletingLocation = location;
	}

	function cancelDelete() {
		deletingLocation = null;
	}

	async function handleDelete() {
		if (!deletingLocation) return;

		try {
			await del(`/locations/${deletingLocation.id}`);
			deletingLocation = null;
			await fetchLocations();
		} catch (err) {
			if (err instanceof ApiError) {
				error = err.message || 'Failed to delete location';
			} else {
				error = 'An unexpected error occurred';
			}
			deletingLocation = null;
		}
	}
</script>

<svelte:head>
	<title>My Locations - {branding.company_name}</title>
</svelte:head>

<div class="locations-page" style={cssVariables}>
	<header class="header">
		<div class="header-content">
			{#if branding.logo_url}
				<img src={branding.logo_url} alt={branding.company_name} class="logo" />
			{:else}
				<h1 class="company-name">{branding.company_name}</h1>
			{/if}
			<nav class="nav">
				<a href="/services" class="nav-link">Services</a>
				<a href="/locations" class="nav-link active">Locations</a>
			</nav>
		</div>
	</header>

	<main class="main">
		<div class="page-header">
			<div>
				<h2 class="page-title">My Locations</h2>
				<p class="page-subtitle">Manage your saved locations for booking</p>
			</div>
			{#if !showForm}
				<button type="button" class="add-button" onclick={openAddForm}>
					<svg fill="none" stroke="currentColor" viewBox="0 0 24 24" width="20" height="20">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M12 4v16m8-8H4"
						/>
					</svg>
					Add Location
				</button>
			{/if}
		</div>

		{#if showForm}
			<div class="form-card">
				<h3 class="form-title">{isEditing ? 'Edit Location' : 'Add New Location'}</h3>

				{#if formErrors.general}
					<div class="error-message" role="alert">
						{formErrors.general}
					</div>
				{/if}

				<form onsubmit={handleSubmit}>
					<div class="form-group">
						<label for="name">Location Name</label>
						<input
							type="text"
							id="name"
							bind:value={formName}
							required
							placeholder="Home, Office, etc."
							disabled={isSubmitting}
							class:input-error={formErrors.name}
						/>
						{#if formErrors.name}
							<span class="field-error">{formErrors.name}</span>
						{/if}
					</div>

					<div class="form-group">
						<label for="address">Street Address</label>
						<input
							type="text"
							id="address"
							bind:value={formAddress}
							required
							placeholder="123 Main St"
							disabled={isSubmitting}
							class:input-error={formErrors.address}
						/>
						{#if formErrors.address}
							<span class="field-error">{formErrors.address}</span>
						{/if}
					</div>

					<div class="form-row">
						<div class="form-group">
							<label for="city">City</label>
							<input
								type="text"
								id="city"
								bind:value={formCity}
								required
								placeholder="City"
								disabled={isSubmitting}
								class:input-error={formErrors.city}
							/>
							{#if formErrors.city}
								<span class="field-error">{formErrors.city}</span>
							{/if}
						</div>

						<div class="form-group form-group-small">
							<label for="state">State</label>
							<input
								type="text"
								id="state"
								bind:value={formState}
								required
								placeholder="CA"
								maxlength="2"
								disabled={isSubmitting}
								class:input-error={formErrors.state}
							/>
							{#if formErrors.state}
								<span class="field-error">{formErrors.state}</span>
							{/if}
						</div>

						<div class="form-group form-group-small">
							<label for="zip">ZIP Code</label>
							<input
								type="text"
								id="zip"
								bind:value={formZip}
								required
								placeholder="12345"
								disabled={isSubmitting}
								class:input-error={formErrors.zip}
							/>
							{#if formErrors.zip}
								<span class="field-error">{formErrors.zip}</span>
							{/if}
						</div>
					</div>

					<div class="form-group">
						<label for="notes">Notes (optional)</label>
						<textarea
							id="notes"
							bind:value={formNotes}
							placeholder="Gate code, parking instructions, etc."
							disabled={isSubmitting}
							rows="3"
						></textarea>
					</div>

					<div class="form-actions">
						<button type="button" class="cancel-button" onclick={resetForm} disabled={isSubmitting}>
							Cancel
						</button>
						<button type="submit" class="submit-button" disabled={isSubmitting}>
							{isSubmitting ? 'Saving...' : isEditing ? 'Update Location' : 'Save Location'}
						</button>
					</div>
				</form>
			</div>
		{/if}

		{#if isLoading}
			<div class="loading-grid">
				{#each [1, 2, 3] as _}
					<div class="skeleton-card">
						<div class="skeleton-title"></div>
						<div class="skeleton-address"></div>
						<div class="skeleton-meta"></div>
					</div>
				{/each}
			</div>
		{:else if error}
			<div class="error-alert" role="alert">
				<p>{error}</p>
				<button onclick={fetchLocations} class="retry-button">Try Again</button>
			</div>
		{:else if locations.length === 0 && !showForm}
			<div class="empty-state">
				<div class="empty-icon">
					<svg fill="none" stroke="currentColor" viewBox="0 0 24 24" width="64" height="64">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="1.5"
							d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z"
						/>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="1.5"
							d="M15 11a3 3 0 11-6 0 3 3 0 016 0z"
						/>
					</svg>
				</div>
				<h3>No Saved Locations</h3>
				<p>Add your first location to make booking easier.</p>
				<button type="button" class="add-button-empty" onclick={openAddForm}>
					Add Your First Location
				</button>
			</div>
		{:else}
			<div class="locations-grid">
				{#each locations as location (location.id)}
					<div class="location-card" class:is-default={location.is_default}>
						{#if location.is_default}
							<span class="default-badge">Default</span>
						{/if}
						<h3 class="location-name">{location.name}</h3>
						<p class="location-address">
							{location.address}<br />
							{location.city}, {location.state}
							{location.zip}
						</p>
						{#if location.notes}
							<p class="location-notes">{location.notes}</p>
						{/if}
						<div class="location-actions">
							{#if !location.is_default}
								<button
									type="button"
									class="action-button"
									onclick={() => handleSetDefault(location)}
								>
									Set as Default
								</button>
							{/if}
							<button type="button" class="action-button" onclick={() => openEditForm(location)}>
								Edit
							</button>
							<button
								type="button"
								class="action-button delete"
								onclick={() => confirmDelete(location)}
							>
								Delete
							</button>
						</div>
					</div>
				{/each}
			</div>
		{/if}
	</main>

	{#if deletingLocation}
		<div
			class="modal-overlay"
			onclick={cancelDelete}
			onkeydown={(e) => e.key === 'Escape' && cancelDelete()}
			role="button"
			tabindex="0"
			aria-label="Close modal"
		>
			<div
				class="modal"
				onclick={(e) => e.stopPropagation()}
				onkeydown={(e) => e.stopPropagation()}
				role="dialog"
				aria-modal="true"
				aria-labelledby="delete-modal-title"
				tabindex="-1"
			>
				<h3 class="modal-title" id="delete-modal-title">Delete Location</h3>
				<p class="modal-text">
					Are you sure you want to delete "{deletingLocation.name}"? This action cannot be undone.
				</p>
				<div class="modal-actions">
					<button type="button" class="cancel-button" onclick={cancelDelete}>Cancel</button>
					<button type="button" class="delete-button" onclick={handleDelete}>Delete</button>
				</div>
			</div>
		</div>
	{/if}
</div>

<style>
	.locations-page {
		min-height: 100vh;
		background-color: #f3f4f6;
	}

	.header {
		background-color: white;
		padding: 1rem;
		box-shadow: 0 1px 3px 0 rgb(0 0 0 / 0.1);
	}

	.header-content {
		max-width: 72rem;
		margin: 0 auto;
		display: flex;
		justify-content: space-between;
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

	.nav {
		display: flex;
		gap: 1.5rem;
	}

	.nav-link {
		color: #6b7280;
		text-decoration: none;
		font-weight: 500;
		padding: 0.5rem 0;
		border-bottom: 2px solid transparent;
	}

	.nav-link:hover {
		color: #374151;
	}

	.nav-link.active {
		color: var(--color-primary, #3b82f6);
		border-bottom-color: var(--color-primary, #3b82f6);
	}

	.main {
		max-width: 72rem;
		margin: 0 auto;
		padding: 2rem 1rem;
	}

	.page-header {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		margin-bottom: 2rem;
		flex-wrap: wrap;
		gap: 1rem;
	}

	.page-title {
		font-size: 1.875rem;
		font-weight: 700;
		color: #111827;
		margin: 0 0 0.25rem;
	}

	.page-subtitle {
		color: #6b7280;
		margin: 0;
	}

	.add-button {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.625rem 1rem;
		background-color: var(--color-primary, #3b82f6);
		color: white;
		border: none;
		border-radius: 0.375rem;
		font-weight: 500;
		cursor: pointer;
	}

	.add-button:hover {
		background-color: var(--color-secondary, #1e40af);
	}

	/* Form Card */
	.form-card {
		background-color: white;
		padding: 1.5rem;
		border-radius: 0.75rem;
		box-shadow: 0 1px 3px 0 rgb(0 0 0 / 0.1);
		margin-bottom: 2rem;
	}

	.form-title {
		font-size: 1.25rem;
		font-weight: 600;
		color: #111827;
		margin: 0 0 1.5rem;
	}

	.error-message {
		margin-bottom: 1rem;
		padding: 0.75rem;
		background-color: #fef2f2;
		border: 1px solid #fecaca;
		border-radius: 0.375rem;
		color: #dc2626;
		font-size: 0.875rem;
	}

	.form-group {
		margin-bottom: 1rem;
	}

	.form-group label {
		display: block;
		font-size: 0.875rem;
		font-weight: 500;
		color: #374151;
		margin-bottom: 0.25rem;
	}

	.form-group input,
	.form-group textarea {
		width: 100%;
		padding: 0.5rem 0.75rem;
		border: 1px solid #d1d5db;
		border-radius: 0.375rem;
		font-size: 1rem;
		box-sizing: border-box;
		font-family: inherit;
	}

	.form-group textarea {
		resize: vertical;
	}

	.form-group input:focus,
	.form-group textarea:focus {
		outline: none;
		border-color: var(--color-primary, #3b82f6);
		box-shadow: 0 0 0 2px rgb(59 130 246 / 0.2);
	}

	.form-group input:disabled,
	.form-group textarea:disabled {
		background-color: #f9fafb;
		cursor: not-allowed;
	}

	.form-group input.input-error {
		border-color: #ef4444;
	}

	.form-group input.input-error:focus {
		box-shadow: 0 0 0 2px rgb(239 68 68 / 0.2);
	}

	.field-error {
		display: block;
		font-size: 0.75rem;
		color: #dc2626;
		margin-top: 0.25rem;
	}

	.form-row {
		display: grid;
		grid-template-columns: 1fr auto auto;
		gap: 1rem;
	}

	@media (max-width: 640px) {
		.form-row {
			grid-template-columns: 1fr 1fr;
		}

		.form-row .form-group:first-child {
			grid-column: span 2;
		}
	}

	.form-group-small {
		min-width: 5rem;
	}

	.form-group-small input {
		text-transform: uppercase;
	}

	.form-actions {
		display: flex;
		gap: 0.75rem;
		justify-content: flex-end;
		margin-top: 1.5rem;
	}

	.cancel-button {
		padding: 0.625rem 1rem;
		background-color: white;
		color: #374151;
		border: 1px solid #d1d5db;
		border-radius: 0.375rem;
		font-weight: 500;
		cursor: pointer;
	}

	.cancel-button:hover:not(:disabled) {
		background-color: #f9fafb;
	}

	.cancel-button:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.submit-button {
		padding: 0.625rem 1rem;
		background-color: var(--color-primary, #3b82f6);
		color: white;
		border: none;
		border-radius: 0.375rem;
		font-weight: 500;
		cursor: pointer;
	}

	.submit-button:hover:not(:disabled) {
		background-color: var(--color-secondary, #1e40af);
	}

	.submit-button:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	/* Locations Grid */
	.locations-grid {
		display: grid;
		grid-template-columns: repeat(1, 1fr);
		gap: 1rem;
	}

	@media (min-width: 640px) {
		.locations-grid {
			grid-template-columns: repeat(2, 1fr);
		}
	}

	@media (min-width: 1024px) {
		.locations-grid {
			grid-template-columns: repeat(3, 1fr);
		}
	}

	.location-card {
		background-color: white;
		border: 2px solid transparent;
		border-radius: 0.75rem;
		padding: 1.5rem;
		box-shadow: 0 1px 3px 0 rgb(0 0 0 / 0.1);
		position: relative;
	}

	.location-card.is-default {
		border-color: var(--color-primary, #3b82f6);
	}

	.default-badge {
		position: absolute;
		top: 0.75rem;
		right: 0.75rem;
		background-color: var(--color-primary, #3b82f6);
		color: white;
		font-size: 0.75rem;
		font-weight: 500;
		padding: 0.25rem 0.5rem;
		border-radius: 9999px;
	}

	.location-name {
		font-size: 1.125rem;
		font-weight: 600;
		color: #111827;
		margin: 0 0 0.5rem;
	}

	.location-address {
		color: #6b7280;
		font-size: 0.875rem;
		margin: 0 0 0.75rem;
		line-height: 1.5;
	}

	.location-notes {
		color: #9ca3af;
		font-size: 0.875rem;
		margin: 0 0 1rem;
		font-style: italic;
	}

	.location-actions {
		display: flex;
		flex-wrap: wrap;
		gap: 0.5rem;
		padding-top: 1rem;
		border-top: 1px solid #e5e7eb;
	}

	.action-button {
		padding: 0.375rem 0.75rem;
		background-color: transparent;
		color: var(--color-primary, #3b82f6);
		border: 1px solid var(--color-primary, #3b82f6);
		border-radius: 0.25rem;
		font-size: 0.875rem;
		font-weight: 500;
		cursor: pointer;
	}

	.action-button:hover {
		background-color: rgb(59 130 246 / 0.1);
	}

	.action-button.delete {
		color: #dc2626;
		border-color: #dc2626;
	}

	.action-button.delete:hover {
		background-color: rgb(220 38 38 / 0.1);
	}

	/* Loading Skeleton */
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
		width: 50%;
		background-color: #e5e7eb;
		border-radius: 0.25rem;
		margin-bottom: 0.75rem;
		animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
	}

	.skeleton-address {
		height: 1rem;
		width: 80%;
		background-color: #e5e7eb;
		border-radius: 0.25rem;
		margin-bottom: 0.5rem;
		animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
	}

	.skeleton-meta {
		height: 1rem;
		width: 60%;
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

	/* Error Alert */
	.error-alert {
		background-color: #fef2f2;
		border: 1px solid #fecaca;
		border-radius: 0.5rem;
		padding: 2rem;
		text-align: center;
		color: #dc2626;
	}

	.error-alert p {
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

	/* Empty State */
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

	.add-button-empty {
		padding: 0.625rem 1.25rem;
		background-color: var(--color-primary, #3b82f6);
		color: white;
		border: none;
		border-radius: 0.375rem;
		font-weight: 500;
		cursor: pointer;
	}

	.add-button-empty:hover {
		background-color: var(--color-secondary, #1e40af);
	}

	/* Modal */
	.modal-overlay {
		position: fixed;
		inset: 0;
		background-color: rgb(0 0 0 / 0.5);
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 1rem;
		z-index: 50;
	}

	.modal {
		background-color: white;
		border-radius: 0.5rem;
		padding: 1.5rem;
		max-width: 24rem;
		width: 100%;
		box-shadow: 0 20px 25px -5px rgb(0 0 0 / 0.1);
	}

	.modal-title {
		font-size: 1.125rem;
		font-weight: 600;
		color: #111827;
		margin: 0 0 0.75rem;
	}

	.modal-text {
		color: #6b7280;
		font-size: 0.875rem;
		margin: 0 0 1.5rem;
	}

	.modal-actions {
		display: flex;
		gap: 0.75rem;
		justify-content: flex-end;
	}

	.delete-button {
		padding: 0.625rem 1rem;
		background-color: #dc2626;
		color: white;
		border: none;
		border-radius: 0.375rem;
		font-weight: 500;
		cursor: pointer;
	}

	.delete-button:hover {
		background-color: #b91c1c;
	}
</style>
