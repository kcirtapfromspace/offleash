<script lang="ts">
	import Navigation from '$lib/components/Navigation.svelte';

	let { children } = $props();
</script>

<div class="min-h-screen bg-gray-100">
	<Navigation />
	<main>
		{@render children()}
	</main>
</div>
