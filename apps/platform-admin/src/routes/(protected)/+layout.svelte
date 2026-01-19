<script lang="ts">
	import { page } from '$app/state';
	import { enhance } from '$app/forms';

	let { children } = $props();

	const navItems = [
		{ path: '/dashboard', label: 'Dashboard' },
		{ path: '/tenants', label: 'Tenants' }
	];

	function isActive(path: string): boolean {
		return page.url.pathname === path || page.url.pathname.startsWith(path + '/');
	}
</script>

<div class="min-h-screen bg-slate-900">
	<nav class="bg-slate-800 border-b border-slate-700">
		<div class="max-w-7xl mx-auto px-4">
			<div class="flex items-center justify-between h-16">
				<div class="flex items-center space-x-8">
					<span class="text-xl font-bold text-white">OFFLEASH</span>
					<span class="text-slate-400 text-sm">Platform Admin</span>
					<div class="flex space-x-4">
						{#each navItems as item}
							<a
								href={item.path}
								class="px-3 py-2 rounded-md text-sm font-medium {isActive(item.path)
									? 'bg-slate-700 text-white'
									: 'text-slate-300 hover:text-white hover:bg-slate-700'}"
							>
								{item.label}
							</a>
						{/each}
					</div>
				</div>
				<form method="POST" action="/logout" use:enhance>
					<button
						type="submit"
						class="text-sm text-slate-300 hover:text-white"
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
