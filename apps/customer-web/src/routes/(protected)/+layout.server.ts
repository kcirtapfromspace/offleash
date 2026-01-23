import { redirect } from '@sveltejs/kit';
import type { LayoutServerLoad } from './$types';
import { api } from '$lib/api';

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

export const load: LayoutServerLoad = async ({ cookies, url }) => {
	const token = cookies.get('token');

	if (!token) {
		throw redirect(303, `/login?redirect=${encodeURIComponent(url.pathname)}`);
	}

	// Try to get user and membership from cookies first
	let user: UserInfo | null = null;
	let membership: MembershipInfo | null = null;
	let memberships: MembershipInfo[] = [];

	try {
		const userCookie = cookies.get('user');
		if (userCookie) {
			user = JSON.parse(userCookie);
		}

		const membershipCookie = cookies.get('membership');
		if (membershipCookie) {
			membership = JSON.parse(membershipCookie);
		}

		const membershipsCookie = cookies.get('memberships');
		if (membershipsCookie) {
			memberships = JSON.parse(membershipsCookie);
		}
	} catch (e) {
		// Ignore parse errors, will fetch from API
	}

	// If no user info, fetch from API
	if (!user) {
		try {
			const response = await api.get<{ memberships: MembershipInfo[]; user_id: string; email: string; first_name: string; last_name: string }>('/contexts', token);
			memberships = response.memberships || [];
			// Find default membership
			membership = memberships.find((m) => m.is_default) || memberships[0] || null;
		} catch (e) {
			// Token might be invalid, redirect to login
			throw redirect(303, `/login?redirect=${encodeURIComponent(url.pathname)}`);
		}
	}

	return {
		token,
		user,
		membership,
		memberships
	};
};
