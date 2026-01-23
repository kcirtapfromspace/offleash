import { writable, derived } from 'svelte/store';

export interface MembershipInfo {
	id: string;
	organization_id: string;
	organization_name: string;
	organization_slug: string;
	role: string;
	is_default: boolean;
}

export interface UserInfo {
	id: string;
	email: string;
	first_name: string;
	last_name: string;
	role: string;
}

// Current active membership
export const currentMembership = writable<MembershipInfo | null>(null);

// All user memberships (for context switching)
export const memberships = writable<MembershipInfo[]>([]);

// Current user info
export const user = writable<UserInfo | null>(null);

// Derived stores
export const isAuthenticated = derived(user, ($user) => $user !== null);

export const hasMultipleMemberships = derived(
	memberships,
	($memberships) => $memberships.length > 1
);

export const currentOrganization = derived(currentMembership, ($membership) =>
	$membership
		? {
				id: $membership.organization_id,
				name: $membership.organization_name,
				slug: $membership.organization_slug
			}
		: null
);

// Helper to initialize stores from cookies (call on client)
export function initFromCookies() {
	if (typeof document === 'undefined') return;

	const getCookie = (name: string): string | null => {
		const value = `; ${document.cookie}`;
		const parts = value.split(`; ${name}=`);
		if (parts.length === 2) {
			const cookieValue = parts.pop()?.split(';').shift();
			return cookieValue ? decodeURIComponent(cookieValue) : null;
		}
		return null;
	};

	try {
		const userCookie = getCookie('user');
		if (userCookie) {
			user.set(JSON.parse(userCookie));
		}

		const membershipCookie = getCookie('membership');
		if (membershipCookie) {
			currentMembership.set(JSON.parse(membershipCookie));
		}

		const membershipsCookie = getCookie('memberships');
		if (membershipsCookie) {
			memberships.set(JSON.parse(membershipsCookie));
		}
	} catch (e) {
		console.error('Failed to parse membership cookies:', e);
	}
}

// Clear all auth state
export function clearAuth() {
	user.set(null);
	currentMembership.set(null);
	memberships.set([]);
}
