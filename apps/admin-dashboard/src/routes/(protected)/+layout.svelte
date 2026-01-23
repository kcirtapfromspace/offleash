<script lang="ts">
	import Navigation from '$lib/components/Navigation.svelte';

	let { children, data } = $props();
</script>

<div class="min-h-screen bg-gray-100">
	<Navigation
		user={data.user}
		membership={data.membership}
		memberships={data.memberships ?? []}
	/>
	<main>
		{@render children()}
	</main>
</div>
