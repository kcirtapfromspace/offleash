<script lang="ts">
	import { page } from '$app/state';
	import { enhance } from '$app/forms';

	const navItems = [
		{ path: '/dashboard', label: 'Dashboard' },
		{ path: '/walkers', label: 'Walkers' },
		{ path: '/services', label: 'Services' },
		{ path: '/bookings', label: 'Bookings' },
		{ path: '/customers', label: 'Customers' },
		{ path: '/settings', label: 'Settings' }
	];

	function isActive(path: string): boolean {
		return page.url.pathname === path || page.url.pathname.startsWith(path + '/');
	}
</script>

<nav class="bg-gray-800 text-white p-4">
	<div class="flex items-center justify-between">
		<div class="flex space-x-4">
			{#each navItems as item}
				<a
					href={item.path}
					class="px-3 py-2 rounded {isActive(item.path)
						? 'bg-gray-900'
						: 'hover:bg-gray-700'}"
				>
					{item.label}
				</a>
			{/each}
		</div>
		<form method="POST" action="/logout" use:enhance>
			<button type="submit" class="px-3 py-2 rounded bg-red-600 hover:bg-red-700">
				Logout
			</button>
		</form>
	</div>
</nav>
