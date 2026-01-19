<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { getToken } from '$lib/api';
	import { generateCssVariables } from '$lib/stores/branding';
	import { onMount } from 'svelte';

	const branding = $derived($page.data.branding);
	const cssVariables = $derived(generateCssVariables(branding));

	onMount(() => {
		const token = getToken();
		if (token) {
			goto('/services');
		}
	});
</script>

<svelte:head>
	<title>Welcome - {branding.company_name}</title>
</svelte:head>

<div class="welcome-page" style={cssVariables}>
	<div class="welcome-card">
		{#if branding.logo_url}
			<img src={branding.logo_url} alt={branding.company_name} class="logo" />
		{:else}
			<h1 class="company-name">{branding.company_name}</h1>
		{/if}

		<p class="tagline">Book your appointments with ease</p>

		<div class="actions">
			<a href="/login" class="button primary">Sign In</a>
			<a href="/register" class="button secondary">Create Account</a>
		</div>
	</div>
</div>

<style>
	.welcome-page {
		min-height: 100vh;
		display: flex;
		align-items: center;
		justify-content: center;
		background-color: #f3f4f6;
		padding: 1rem;
	}

	.welcome-card {
		background-color: white;
		padding: 3rem 2rem;
		border-radius: 0.75rem;
		box-shadow:
			0 4px 6px -1px rgb(0 0 0 / 0.1),
			0 2px 4px -2px rgb(0 0 0 / 0.1);
		text-align: center;
		max-width: 24rem;
		width: 100%;
	}

	.logo {
		max-height: 4rem;
		margin-bottom: 1.5rem;
	}

	.company-name {
		font-size: 2rem;
		font-weight: 700;
		color: var(--color-primary, #3b82f6);
		margin: 0 0 1.5rem;
	}

	.tagline {
		color: #6b7280;
		font-size: 1.125rem;
		margin: 0 0 2rem;
	}

	.actions {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.button {
		display: block;
		padding: 0.75rem 1.5rem;
		border-radius: 0.375rem;
		font-weight: 500;
		text-decoration: none;
		transition: all 0.2s ease;
	}

	.button.primary {
		background-color: var(--color-primary, #3b82f6);
		color: white;
	}

	.button.primary:hover {
		background-color: var(--color-secondary, #1e40af);
	}

	.button.secondary {
		background-color: white;
		color: var(--color-primary, #3b82f6);
		border: 1px solid var(--color-primary, #3b82f6);
	}

	.button.secondary:hover {
		background-color: #f3f4f6;
	}
</style>
