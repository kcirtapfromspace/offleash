<script lang="ts">
	import type { PageData } from './$types';

	export let data: PageData;

	function formatCurrency(cents: number): string {
		return new Intl.NumberFormat('en-US', {
			style: 'currency',
			currency: 'USD'
		}).format(cents / 100);
	}

	function formatDate(isoString: string): string {
		return new Date(isoString).toLocaleDateString('en-US', {
			month: 'short',
			day: 'numeric'
		});
	}

	function formatTime(isoString: string): string {
		return new Date(isoString).toLocaleTimeString('en-US', {
			hour: 'numeric',
			minute: '2-digit',
			hour12: true
		});
	}

	function getStatusColor(status: string): string {
		switch (status.toLowerCase()) {
			case 'pending':
				return 'bg-yellow-100 text-yellow-800';
			case 'confirmed':
				return 'bg-blue-100 text-blue-800';
			case 'in_progress':
				return 'bg-purple-100 text-purple-800';
			case 'completed':
				return 'bg-green-100 text-green-800';
			case 'cancelled':
				return 'bg-red-100 text-red-800';
			default:
				return 'bg-gray-100 text-gray-800';
		}
	}

	function formatStatus(status: string): string {
		return status.replace('_', ' ').replace(/\b\w/g, (l) => l.toUpperCase());
	}
</script>

<div class="p-6">
	<div class="mb-8">
		<h1 class="text-2xl font-bold text-gray-900">Dashboard</h1>
		<p class="text-gray-600">{data.isAdmin ? 'Overview of your dog walking business' : 'Your walk schedule and stats'}</p>
	</div>

	{#if data.error}
		<div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg mb-6">
			{data.error}
		</div>
	{/if}

	<!-- Metrics Grid -->
	<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
		<!-- Total Bookings/Walks -->
		<div class="bg-white rounded-xl border border-gray-200 p-6">
			<div class="flex items-center justify-between">
				<div>
					<p class="text-sm font-medium text-gray-500">{data.isAdmin ? 'Total Bookings' : 'My Walks'}</p>
					<p class="text-3xl font-bold text-gray-900 mt-1">{data.metrics.totalBookings}</p>
				</div>
				<div class="w-12 h-12 bg-blue-100 rounded-lg flex items-center justify-center">
					<svg class="w-6 h-6 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
					</svg>
				</div>
			</div>
			<p class="text-sm text-gray-500 mt-2">
				<span class="text-green-600 font-medium">{data.metrics.weekBookings}</span> this week
			</p>
		</div>

		{#if data.isAdmin}
			<!-- Revenue (Admin only) -->
			<div class="bg-white rounded-xl border border-gray-200 p-6">
				<div class="flex items-center justify-between">
					<div>
						<p class="text-sm font-medium text-gray-500">Total Revenue</p>
						<p class="text-3xl font-bold text-gray-900 mt-1">{formatCurrency(data.metrics.totalRevenue)}</p>
					</div>
					<div class="w-12 h-12 bg-green-100 rounded-lg flex items-center justify-center">
						<svg class="w-6 h-6 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
						</svg>
					</div>
				</div>
				<p class="text-sm text-gray-500 mt-2">
					<span class="text-green-600 font-medium">{formatCurrency(data.metrics.weekRevenue)}</span> this week
				</p>
			</div>

			<!-- Walkers (Admin only) -->
			<div class="bg-white rounded-xl border border-gray-200 p-6">
				<div class="flex items-center justify-between">
					<div>
						<p class="text-sm font-medium text-gray-500">Walkers</p>
						<p class="text-3xl font-bold text-gray-900 mt-1">{data.metrics.totalWalkers}</p>
					</div>
					<div class="w-12 h-12 bg-purple-100 rounded-lg flex items-center justify-center">
						<svg class="w-6 h-6 text-purple-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
						</svg>
					</div>
				</div>
				<p class="text-sm text-gray-500 mt-2">
					<span class="text-green-600 font-medium">{data.metrics.activeWalkers}</span> active
				</p>
			</div>

			<!-- Customers (Admin only) -->
			<div class="bg-white rounded-xl border border-gray-200 p-6">
				<div class="flex items-center justify-between">
					<div>
						<p class="text-sm font-medium text-gray-500">Customers</p>
						<p class="text-3xl font-bold text-gray-900 mt-1">{data.metrics.totalCustomers}</p>
					</div>
					<div class="w-12 h-12 bg-orange-100 rounded-lg flex items-center justify-center">
						<svg class="w-6 h-6 text-orange-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z" />
						</svg>
					</div>
				</div>
			</div>
		{:else}
			<!-- Dogs/Clients (Walker view) -->
			<div class="bg-white rounded-xl border border-gray-200 p-6">
				<div class="flex items-center justify-between">
					<div>
						<p class="text-sm font-medium text-gray-500">Dogs Walked</p>
						<p class="text-3xl font-bold text-gray-900 mt-1">{data.metrics.uniqueDogs}</p>
					</div>
					<div class="w-12 h-12 bg-orange-100 rounded-lg flex items-center justify-center">
						<svg class="w-6 h-6 text-orange-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z" />
						</svg>
					</div>
				</div>
				<p class="text-sm text-gray-500 mt-2">unique clients</p>
			</div>

			<!-- Today's Walks (Walker view) -->
			<div class="bg-white rounded-xl border border-gray-200 p-6">
				<div class="flex items-center justify-between">
					<div>
						<p class="text-sm font-medium text-gray-500">Today's Walks</p>
						<p class="text-3xl font-bold text-gray-900 mt-1">{data.metrics.todayBookings}</p>
					</div>
					<div class="w-12 h-12 bg-purple-100 rounded-lg flex items-center justify-center">
						<svg class="w-6 h-6 text-purple-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
						</svg>
					</div>
				</div>
				<p class="text-sm text-gray-500 mt-2">scheduled for today</p>
			</div>

			<!-- Completed (Walker view) -->
			<div class="bg-white rounded-xl border border-gray-200 p-6">
				<div class="flex items-center justify-between">
					<div>
						<p class="text-sm font-medium text-gray-500">Completed</p>
						<p class="text-3xl font-bold text-gray-900 mt-1">{data.metrics.completedBookings}</p>
					</div>
					<div class="w-12 h-12 bg-green-100 rounded-lg flex items-center justify-center">
						<svg class="w-6 h-6 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
						</svg>
					</div>
				</div>
				<p class="text-sm text-gray-500 mt-2">walks completed</p>
			</div>
		{/if}
	</div>

	<!-- Booking Status Summary -->
	<div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-8">
		<div class="bg-yellow-50 border border-yellow-200 rounded-lg p-4 flex items-center justify-between">
			<div>
				<p class="text-sm font-medium text-yellow-800">Pending</p>
				<p class="text-2xl font-bold text-yellow-900">{data.metrics.pendingBookings}</p>
			</div>
			<div class="w-10 h-10 bg-yellow-200 rounded-full flex items-center justify-center">
				<svg class="w-5 h-5 text-yellow-700" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
				</svg>
			</div>
		</div>
		<div class="bg-blue-50 border border-blue-200 rounded-lg p-4 flex items-center justify-between">
			<div>
				<p class="text-sm font-medium text-blue-800">Confirmed</p>
				<p class="text-2xl font-bold text-blue-900">{data.metrics.confirmedBookings}</p>
			</div>
			<div class="w-10 h-10 bg-blue-200 rounded-full flex items-center justify-center">
				<svg class="w-5 h-5 text-blue-700" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
				</svg>
			</div>
		</div>
		<div class="bg-green-50 border border-green-200 rounded-lg p-4 flex items-center justify-between">
			<div>
				<p class="text-sm font-medium text-green-800">Completed</p>
				<p class="text-2xl font-bold text-green-900">{data.metrics.completedBookings}</p>
			</div>
			<div class="w-10 h-10 bg-green-200 rounded-full flex items-center justify-center">
				<svg class="w-5 h-5 text-green-700" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
				</svg>
			</div>
		</div>
	</div>

	<!-- Two Column Layout -->
	<div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
		<!-- Upcoming Bookings -->
		<div class="bg-white rounded-xl border border-gray-200 overflow-hidden">
			<div class="px-6 py-4 border-b border-gray-200 flex items-center justify-between">
				<h2 class="text-lg font-semibold text-gray-900">Upcoming Bookings</h2>
				<a href="/bookings" class="text-sm text-blue-600 hover:underline">View All</a>
			</div>
			{#if data.upcomingBookings.length === 0}
				<div class="p-6 text-center text-gray-500">
					No upcoming bookings
				</div>
			{:else}
				<div class="divide-y divide-gray-100">
					{#each data.upcomingBookings as booking}
						<div class="px-6 py-4 hover:bg-gray-50">
							<div class="flex items-center justify-between mb-1">
								<p class="font-medium text-gray-900">{booking.service_name}</p>
								<span class="px-2 py-0.5 text-xs font-medium rounded-full {getStatusColor(booking.status)}">
									{formatStatus(booking.status)}
								</span>
							</div>
							<p class="text-sm text-gray-600">{booking.customer_name} with {booking.walker_name}</p>
							<p class="text-sm text-gray-500 mt-1">
								{formatDate(booking.scheduled_start)} at {formatTime(booking.scheduled_start)}
							</p>
						</div>
					{/each}
				</div>
			{/if}
		</div>

		<!-- Recent Activity -->
		<div class="bg-white rounded-xl border border-gray-200 overflow-hidden">
			<div class="px-6 py-4 border-b border-gray-200 flex items-center justify-between">
				<h2 class="text-lg font-semibold text-gray-900">Recent Bookings</h2>
				<a href="/bookings" class="text-sm text-blue-600 hover:underline">View All</a>
			</div>
			{#if data.recentBookings.length === 0}
				<div class="p-6 text-center text-gray-500">
					No recent bookings
				</div>
			{:else}
				<div class="divide-y divide-gray-100">
					{#each data.recentBookings as booking}
						<div class="px-6 py-4 hover:bg-gray-50">
							<div class="flex items-center justify-between mb-1">
								<p class="font-medium text-gray-900">{booking.service_name}</p>
								<span class="font-medium text-gray-900">{booking.price_display}</span>
							</div>
							<p class="text-sm text-gray-600">{booking.customer_name}</p>
							<div class="flex items-center justify-between mt-1">
								<p class="text-sm text-gray-500">
									{formatDate(booking.scheduled_start)}
								</p>
								<span class="px-2 py-0.5 text-xs font-medium rounded-full {getStatusColor(booking.status)}">
									{formatStatus(booking.status)}
								</span>
							</div>
						</div>
					{/each}
				</div>
			{/if}
		</div>
	</div>
</div>
