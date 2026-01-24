<script lang="ts">
	import { page } from '$app/state';
	import { enhance } from '$app/forms';
	import { goto, invalidateAll } from '$app/navigation';
	import { env } from '$env/dynamic/public';

	interface MembershipInfo {
		id: string;
		organization_id: string;
		organization_name: string;
		organization_slug: string;
		role: string;
		is_default: boolean;
	}

	interface UserInfo {
		id: string;
		email: string;
		first_name: string;
		last_name: string;
		role: string;
	}

	interface Props {
		user: UserInfo | null;
		membership: MembershipInfo | null;
		memberships: MembershipInfo[];
	}

	let { user, membership, memberships }: Props = $props();

	let showContextMenu = $state(false);
	let switchingContext = $state(false);

	// Check if current user is admin/owner (can manage staff, services, customers)
	const isAdmin = $derived(membership?.role === 'admin' || membership?.role === 'owner');

	// Navigation items - walkers see a subset
	const allNavItems = [
		{ path: '/dashboard', label: 'Dashboard', roles: ['admin', 'owner', 'walker'] },
		{ path: '/calendar', label: 'Calendar', roles: ['admin', 'owner', 'walker'] },
		{ path: '/walkers', label: 'Staff', roles: ['admin', 'owner'] },
		{ path: '/services', label: 'Services', roles: ['admin', 'owner'] },
		{ path: '/bookings', label: 'Bookings', roles: ['admin', 'owner', 'walker'] },
		{ path: '/customers', label: 'Customers', roles: ['admin', 'owner'] },
		{ path: '/settings', label: 'Settings', roles: ['admin', 'owner', 'walker'] }
	];

	const navItems = $derived(
		allNavItems.filter((item) => item.roles.includes(membership?.role || ''))
	);

	function isActive(path: string): boolean {
		return page.url.pathname === path || page.url.pathname.startsWith(path + '/');
	}

	// Filter to only show admin/walker/owner memberships (not customer)
	const adminMemberships = $derived(
		memberships.filter(m => ['admin', 'owner', 'walker'].includes(m.role))
	);
	const hasMultipleMemberships = $derived(adminMemberships.length > 1);
	const currentOrgName = $derived(membership?.organization_name ?? 'Dashboard');
	const customerWebUrl = env.PUBLIC_CUSTOMER_URL || 'https://offleash.world';

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
				await invalidateAll();
				goto('/dashboard');
			}
		} catch (e) {
			console.error('Failed to switch context:', e);
		} finally {
			switchingContext = false;
		}
	}
</script>

<nav class="bg-gray-800 text-white p-4">
	<div class="flex items-center justify-between">
		<div class="flex items-center space-x-6">
			<!-- Organization switcher -->
			<div class="relative">
				{#if hasMultipleMemberships}
					<button
						type="button"
						class="flex items-center space-x-2 text-lg font-semibold hover:text-gray-300"
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
								{#each adminMemberships as mem}
									<button
										type="button"
										class="w-full px-4 py-3 text-left hover:bg-gray-50 flex items-center justify-between {mem.id === membership?.id ? 'bg-gray-50' : ''}"
										onclick={() => switchContext(mem.id)}
										disabled={switchingContext}
									>
										<div>
											<div class="font-medium text-gray-900">{mem.organization_name}</div>
											<div class="text-sm text-gray-500 capitalize">{mem.role}</div>
										</div>
										{#if mem.id === membership?.id}
											<svg class="w-5 h-5 text-green-500" fill="currentColor" viewBox="0 0 20 20">
												<path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
											</svg>
										{/if}
									</button>
								{/each}

								<!-- Divider and customer app link -->
								<div class="border-t border-gray-200 my-2"></div>
								<a
									href={customerWebUrl}
									class="w-full px-4 py-3 text-left hover:bg-gray-50 flex items-center text-gray-700"
								>
									<svg class="w-5 h-5 mr-3 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
									</svg>
									<span>Book a Walk</span>
									<svg class="w-4 h-4 ml-auto text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
									</svg>
								</a>
							</div>
						</div>
					{/if}
				{:else}
					<!-- Single membership - still show dropdown for customer app link -->
					<button
						type="button"
						class="flex items-center space-x-2 text-lg font-semibold hover:text-gray-300"
						onclick={() => (showContextMenu = !showContextMenu)}
					>
						<span>{currentOrgName}</span>
						<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
						</svg>
					</button>

					{#if showContextMenu}
						<button
							type="button"
							class="fixed inset-0 z-10"
							onclick={() => (showContextMenu = false)}
							aria-label="Close menu"
						></button>

						<div class="absolute left-0 mt-2 w-64 bg-white rounded-lg shadow-lg border z-20">
							<div class="py-2">
								<a
									href={customerWebUrl}
									class="w-full px-4 py-3 text-left hover:bg-gray-50 flex items-center text-gray-700"
								>
									<svg class="w-5 h-5 mr-3 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
									</svg>
									<span>Book a Walk</span>
									<svg class="w-4 h-4 ml-auto text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
									</svg>
								</a>
							</div>
						</div>
					{/if}
				{/if}
			</div>

			<!-- Nav items -->
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
		</div>

		<div class="flex items-center space-x-4">
			{#if user}
				<span class="text-sm text-gray-300">
					{user.first_name} {user.last_name}
				</span>
			{/if}
			<form method="POST" action="/logout" use:enhance>
				<button type="submit" class="px-3 py-2 rounded bg-red-600 hover:bg-red-700">
					Logout
				</button>
			</form>
		</div>
	</div>
</nav>
