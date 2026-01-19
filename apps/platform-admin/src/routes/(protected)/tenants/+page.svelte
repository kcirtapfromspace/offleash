<script lang="ts">
	// Tenants list page
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
				<tr>
					<td colspan="5" class="px-6 py-4 text-center text-slate-400">
						No tenants found
					</td>
				</tr>
			</tbody>
		</table>
	</div>
</div>
