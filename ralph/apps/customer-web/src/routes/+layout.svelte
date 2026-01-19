<script lang="ts">
	import '../app.css';
	import favicon from '$lib/assets/favicon.svg';
	import { branding, generateCssVariables } from '$lib/stores/branding';

	let { data, children } = $props();

	$effect(() => {
		branding.set(data.branding);
	});

	const cssVariables = $derived(generateCssVariables(data.branding));
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<div style={cssVariables}>
	{@render children()}
</div>
