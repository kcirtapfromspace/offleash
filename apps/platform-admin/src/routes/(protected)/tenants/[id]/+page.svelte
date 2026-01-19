<script lang="ts">
	import { page } from '$app/state';

	const tenantId = page.params.id;
</script>

<div>
	<div class="flex items-center justify-between mb-6">
		<h1 class="text-2xl font-bold text-white">Tenant Details</h1>
		<span class="text-slate-400">ID: {tenantId}</span>
	</div>

	<div class="bg-slate-800 rounded-lg p-6">
		<p class="text-slate-400">Tenant detail view placeholder</p>
	</div>
</div>
