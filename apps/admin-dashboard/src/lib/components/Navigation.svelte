<script lang="ts">
	import { page } from '$app/state';
	import { enhance } from '$app/forms';
	import { goto, invalidateAll } from '$app/navigation';

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

	const navItems = [
		{ path: '/dashboard', label: 'Dashboard' },
		{ path: '/calendar', label: 'Calendar' },
		{ path: '/walkers', label: 'Walkers' },
		{ path: '/services', label: 'Services' },
		{ path: '/bookings', label: 'Bookings' },
		{ path: '/customers', label: 'Customers' },
		{ path: '/settings', label: 'Settings' }
	];

	function isActive(path: string): boolean {
		return page.url.pathname === path || page.url.pathname.startsWith(path + '/');
	}

	const hasMultipleMemberships = $derived(memberships.length > 1);
	const currentOrgName = $derived(membership?.organization_name ?? 'Admin Dashboard');

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
								{#each memberships as mem}
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
							</div>
						</div>
					{/if}
				{:else}
					<span class="text-lg font-semibold">{currentOrgName}</span>
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
