<script lang="ts">
	interface Props {
		data: {
			tenants: Array<{
				id: string;
				name: string;
				slug: string;
				status: string;
				subscription_tier: string;
				created_at: string;
			}>;
			total: number;
			error?: string;
		};
	}
	let { data }: Props = $props();

	function formatDate(dateStr: string): string {
		return new Date(dateStr).toLocaleDateString('en-US', {
			year: 'numeric',
			month: 'short',
			day: 'numeric'
		});
	}

	function getStatusColor(status: string): string {
		switch (status) {
			case 'active':
				return 'bg-green-500/20 text-green-400';
			case 'suspended':
				return 'bg-red-500/20 text-red-400';
			case 'pending':
				return 'bg-yellow-500/20 text-yellow-400';
			default:
				return 'bg-slate-500/20 text-slate-400';
		}
	}
</script>

<div>
	<div class="flex items-center justify-between mb-6">
		<h1 class="text-2xl font-bold text-white">Tenants</h1>
		<a
			href="/tenants/new"
			class="px-4 py-2 bg-indigo-600 text-white rounded hover:bg-indigo-700"
		>
			Create Tenant
		</a>
	</div>

	{#if data.error}
		<div class="bg-red-500/20 border border-red-500 text-red-400 px-4 py-3 rounded mb-4">
			{data.error}
		</div>
	{/if}

	<div class="bg-slate-800 rounded-lg overflow-hidden">
		<table class="w-full">
			<thead>
				<tr class="border-b border-slate-700">
					<th class="px-6 py-3 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">
						Name
					</th>
					<th class="px-6 py-3 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">
						Slug
					</th>
					<th class="px-6 py-3 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">
						Plan
					</th>
					<th class="px-6 py-3 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">
						Status
					</th>
					<th class="px-6 py-3 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">
						Created
					</th>
				</tr>
			</thead>
			<tbody class="divide-y divide-slate-700">
				{#if data.tenants && data.tenants.length > 0}
					{#each data.tenants as tenant}
						<tr class="hover:bg-slate-700/50">
							<td class="px-6 py-4">
								<a href="/tenants/{tenant.id}" class="text-white hover:text-indigo-400">
									{tenant.name}
								</a>
							</td>
							<td class="px-6 py-4 text-slate-300">
								{tenant.slug}
							</td>
							<td class="px-6 py-4 text-slate-300 capitalize">
								{tenant.subscription_tier}
							</td>
							<td class="px-6 py-4">
								<span class="px-2 py-1 text-xs rounded {getStatusColor(tenant.status)}">
									{tenant.status}
								</span>
							</td>
							<td class="px-6 py-4 text-slate-400">
								{formatDate(tenant.created_at)}
							</td>
						</tr>
					{/each}
				{:else}
					<tr>
						<td colspan="5" class="px-6 py-4 text-center text-slate-400">
							No tenants found
						</td>
					</tr>
				{/if}
			</tbody>
		</table>
	</div>

	{#if data.total > 0}
		<div class="mt-4 text-sm text-slate-400">
			Showing {data.tenants.length} of {data.total} tenants
		</div>
	{/if}
</div>
