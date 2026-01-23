<script lang="ts">
	import { page } from '$app/state';
	import { enhance } from '$app/forms';
	import { goto, invalidateAll } from '$app/navigation';

	let { children, data } = $props();

	let showContextMenu = $state(false);
	let switchingContext = $state(false);

	const navItems = [
		{ path: '/services', label: 'Services' },
		{ path: '/locations', label: 'My Locations' },
		{ path: '/bookings', label: 'My Bookings' },
		{ path: '/settings', label: 'Settings' }
	];

	function isActive(path: string): boolean {
		return page.url.pathname === path || page.url.pathname.startsWith(path + '/');
	}

	const hasMultipleMemberships = $derived((data.memberships?.length ?? 0) > 1);
	const currentOrgName = $derived(data.membership?.organization_name ?? data.branding?.companyName ?? 'OFFLEASH');

	async function switchContext(membershipId: string) {
		if (switchingContext) return;
		switchingContext = true;
		showContextMenu = false;

		try {
			const response = await fetch('/api/switch-context', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ membership_id: membershipId })
			});

			if (response.ok) {
				// Refresh the page to load new context
				await invalidateAll();
				goto('/services');
			}
		} catch (e) {
			console.error('Failed to switch context:', e);
		} finally {
			switchingContext = false;
		}
	}
</script>

<div class="min-h-screen bg-gray-50">
	<nav class="bg-white shadow-sm">
		<div class="max-w-7xl mx-auto px-4">
			<div class="flex items-center justify-between h-16">
				<div class="flex items-center space-x-8">
					<!-- Organization name with context switcher -->
					<div class="relative">
						{#if hasMultipleMemberships}
							<button
								type="button"
								class="flex items-center space-x-2 text-xl font-bold hover:opacity-80"
								style="color: var(--color-primary)"
								onclick={() => (showContextMenu = !showContextMenu)}
							>
								<span>{currentOrgName}</span>
								<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
								</svg>
							</button>

							{#if showContextMenu}
								<!-- Backdrop -->
								<button
									type="button"
									class="fixed inset-0 z-10"
									onclick={() => (showContextMenu = false)}
									aria-label="Close menu"
								></button>

								<!-- Dropdown menu -->
								<div class="absolute left-0 mt-2 w-64 bg-white rounded-lg shadow-lg border z-20">
									<div class="py-2">
										<div class="px-4 py-2 text-xs font-semibold text-gray-500 uppercase">Switch Organization</div>
										{#each data.memberships ?? [] as membership}
											<button
												type="button"
												class="w-full px-4 py-3 text-left hover:bg-gray-50 flex items-center justify-between {membership.id === data.membership?.id ? 'bg-gray-50' : ''}"
												onclick={() => switchContext(membership.id)}
												disabled={switchingContext}
											>
												<div>
													<div class="font-medium text-gray-900">{membership.organization_name}</div>
													<div class="text-sm text-gray-500 capitalize">{membership.role}</div>
												</div>
												{#if membership.id === data.membership?.id}
													<svg class="w-5 h-5 text-green-500" fill="currentColor" viewBox="0 0 20 20">
														<path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
													</svg>
												{/if}
											</button>
										{/each}
									</div>
								</div>
							{/if}
						{:else}
							<a href="/services" class="text-xl font-bold" style="color: var(--color-primary)">
								{currentOrgName}
							</a>
						{/if}
					</div>

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

				<div class="flex items-center space-x-4">
					{#if data.user}
						<span class="text-sm text-gray-600 hidden sm:block">
							{data.user.first_name}
						</span>
					{/if}
					<form
						method="POST"
						action="/logout"
						use:enhance={() => {
							return async ({ result }) => {
								if (result.type === 'redirect') {
									// Force navigation to login page
									window.location.href = '/login';
								}
							};
						}}
					>
						<button
							type="submit"
							class="text-sm text-gray-600 hover:text-gray-900"
						>
							Logout
						</button>
					</form>
				</div>
			</div>
		</div>
	</nav>

	<main class="max-w-7xl mx-auto px-4 py-8">
		{@render children()}
	</main>
</div>
