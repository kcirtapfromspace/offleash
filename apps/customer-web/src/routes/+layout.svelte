<script lang="ts">
	import '../app.css';
	import favicon from '$lib/assets/favicon.svg';

	let { data, children } = $props();

	// Apply branding colors as CSS variables (using $derived for reactivity)
	const brandingStyle = $derived(
		data.branding
			? `--color-primary: ${data.branding.primaryColor}; --color-secondary: ${data.branding.secondaryColor}; --color-accent: ${data.branding.accentColor};`
			: ''
	);
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	{#if data.branding?.companyName}
		<title>{data.branding.companyName}</title>
	{/if}
</svelte:head>

<div style={brandingStyle}>
	{@render children()}
</div>
