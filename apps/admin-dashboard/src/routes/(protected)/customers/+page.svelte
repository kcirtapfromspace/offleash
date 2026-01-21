<script lang="ts">
	import type { PageData } from './$types';

	export let data: PageData;

	let searchQuery = '';

	$: filteredCustomers = data.customers.filter((customer) => {
		const query = searchQuery.toLowerCase();
		return (
			customer.first_name.toLowerCase().includes(query) ||
			customer.last_name.toLowerCase().includes(query) ||
			customer.email.toLowerCase().includes(query) ||
			(customer.phone && customer.phone.includes(query))
		);
	});

	function formatDate(isoString: string): string {
		return new Date(isoString).toLocaleDateString('en-US', {
			month: 'short',
			day: 'numeric',
			year: 'numeric'
		});
	}

	function formatRelativeDate(isoString: string): string {
		const date = new Date(isoString);
		const now = new Date();
		const diffTime = now.getTime() - date.getTime();
		const diffDays = Math.floor(diffTime / (1000 * 60 * 60 * 24));

		if (diffDays === 0) return 'Today';
		if (diffDays === 1) return 'Yesterday';
		if (diffDays < 7) return `${diffDays} days ago`;
		if (diffDays < 30) return `${Math.floor(diffDays / 7)} weeks ago`;
		if (diffDays < 365) return `${Math.floor(diffDays / 30)} months ago`;
		return formatDate(isoString);
	}
</script>

<div class="p-6">
	<div class="mb-6">
		<h1 class="text-2xl font-bold text-gray-900">Customers</h1>
		<p class="text-gray-600">View and manage your customers</p>
	</div>

	{#if data.error}
		<div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg mb-6">
			{data.error}
		</div>
	{/if}

	<!-- Stats -->
	<div class="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
		<div class="bg-white border border-gray-200 rounded-xl p-4">
			<p class="text-sm font-medium text-gray-500">Total Customers</p>
			<p class="text-2xl font-bold text-gray-900">{data.customers.length}</p>
		</div>
		<div class="bg-white border border-gray-200 rounded-xl p-4">
			<p class="text-sm font-medium text-gray-500">With Bookings</p>
			<p class="text-2xl font-bold text-green-600">
				{data.customers.filter((c) => c.total_bookings > 0).length}
			</p>
		</div>
		<div class="bg-white border border-gray-200 rounded-xl p-4">
			<p class="text-sm font-medium text-gray-500">No Bookings Yet</p>
			<p class="text-2xl font-bold text-amber-600">
				{data.customers.filter((c) => c.total_bookings === 0).length}
			</p>
		</div>
		<div class="bg-white border border-gray-200 rounded-xl p-4">
			<p class="text-sm font-medium text-gray-500">Total Bookings</p>
			<p class="text-2xl font-bold text-purple-600">
				{data.customers.reduce((sum, c) => sum + c.total_bookings, 0)}
			</p>
		</div>
	</div>

	<!-- Search -->
	<div class="mb-6">
		<div class="relative">
			<svg
				class="absolute left-3 top-1/2 transform -translate-y-1/2 w-5 h-5 text-gray-400"
				fill="none"
				stroke="currentColor"
				viewBox="0 0 24 24"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
				/>
			</svg>
			<input
				type="text"
				bind:value={searchQuery}
				placeholder="Search by name, email, or phone..."
				class="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-purple-500"
			/>
		</div>
	</div>

	<!-- Customers Table -->
	{#if data.customers.length === 0}
		<div class="bg-white border border-gray-200 rounded-xl p-8 text-center">
			<svg class="w-12 h-12 mx-auto text-gray-400 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"
				/>
			</svg>
			<p class="text-gray-500">No customers yet</p>
			<p class="text-sm text-gray-400 mt-1">Customers will appear here when they sign up</p>
		</div>
	{:else if filteredCustomers.length === 0}
		<div class="bg-white border border-gray-200 rounded-xl p-8 text-center">
			<svg class="w-12 h-12 mx-auto text-gray-400 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
				/>
			</svg>
			<p class="text-gray-500">No customers match your search</p>
			<button on:click={() => (searchQuery = '')} class="text-sm text-purple-600 hover:text-purple-800 mt-2">
				Clear search
			</button>
		</div>
	{:else}
		<div class="bg-white border border-gray-200 rounded-xl overflow-hidden">
			<table class="min-w-full divide-y divide-gray-200">
				<thead class="bg-gray-50">
					<tr>
						<th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
							Customer
						</th>
						<th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
							Contact
						</th>
						<th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
							Bookings
						</th>
						<th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
							Last Activity
						</th>
						<th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
							Joined
						</th>
					</tr>
				</thead>
				<tbody class="bg-white divide-y divide-gray-200">
					{#each filteredCustomers as customer}
						<tr class="hover:bg-gray-50">
							<td class="px-6 py-4 whitespace-nowrap">
								<div class="flex items-center">
									<div class="w-10 h-10 bg-purple-100 rounded-full flex items-center justify-center mr-3">
										<span class="text-purple-600 font-semibold">
											{customer.first_name[0]}{customer.last_name[0]}
										</span>
									</div>
									<div>
										<div class="text-sm font-medium text-gray-900">
											{customer.first_name}
											{customer.last_name}
										</div>
									</div>
								</div>
							</td>
							<td class="px-6 py-4 whitespace-nowrap">
								<div class="text-sm text-gray-900">{customer.email}</div>
								{#if customer.phone}
									<div class="text-sm text-gray-500">{customer.phone}</div>
								{/if}
							</td>
							<td class="px-6 py-4 whitespace-nowrap">
								<span
									class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {customer.total_bookings >
									0
										? 'bg-green-100 text-green-800'
										: 'bg-gray-100 text-gray-600'}"
								>
									{customer.total_bookings}
									{customer.total_bookings === 1 ? 'booking' : 'bookings'}
								</span>
							</td>
							<td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
								{#if customer.last_booking}
									{formatRelativeDate(customer.last_booking)}
								{:else}
									<span class="text-gray-400">No bookings</span>
								{/if}
							</td>
							<td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
								{formatDate(customer.created_at)}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>

		<div class="mt-4 text-sm text-gray-500">
			Showing {filteredCustomers.length} of {data.customers.length} customers
		</div>
	{/if}
</div>
