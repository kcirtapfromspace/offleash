<script lang="ts">
	import { page } from '$app/state';
	import { enhance } from '$app/forms';

	let { children, data } = $props();

	const navItems = [
		{ path: '/services', label: 'Services' },
		{ path: '/locations', label: 'My Locations' },
		{ path: '/bookings', label: 'My Bookings' },
		{ path: '/profile', label: 'Profile' }
	];

	function isActive(path: string): boolean {
		return page.url.pathname === path || page.url.pathname.startsWith(path + '/');
	}
</script>

<div class="min-h-screen bg-gray-50">
	<nav class="bg-white shadow-sm">
		<div class="max-w-7xl mx-auto px-4">
			<div class="flex items-center justify-between h-16">
				<div class="flex items-center space-x-8">
					<a href="/services" class="text-xl font-bold" style="color: var(--color-primary)">
						{data.branding?.companyName ?? 'OFFLEASH'}
					</a>
					<div class="hidden md:flex space-x-4">
						{#each navItems as item}
							<a
								href={item.path}
								class="px-3 py-2 rounded-md text-sm font-medium {isActive(item.path)
									? 'bg-gray-100'
									: 'text-gray-600 hover:text-gray-900 hover:bg-gray-50'}"
								style={isActive(item.path) ? 'color: var(--color-primary)' : ''}
							>
								{item.label}
							</a>
						{/each}
					</div>
				</div>
				<form method="POST" action="/logout" use:enhance>
					<button
						type="submit"
						class="text-sm text-gray-600 hover:text-gray-900"
					>
						Logout
					</button>
				</form>
			</div>
		</div>
	</nav>

	<main class="max-w-7xl mx-auto px-4 py-8">
		{@render children()}
	</main>
</div>
