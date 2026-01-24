import { redirect } from '@sveltejs/kit';
import type { LayoutServerLoad } from './$types';
import { api, ApiError } from '$lib/api';
import { setAuthCookie } from '$lib/cookies';

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

interface SessionResponse {
	user: UserInfo;
	membership?: MembershipInfo;
	memberships?: MembershipInfo[];
	org_id?: string;
}

export const load: LayoutServerLoad = async ({ cookies, url, request }) => {
	const token = cookies.get('token');

	if (!token) {
		throw redirect(303, `/login?redirect=${encodeURIComponent(url.pathname)}`);
	}

	// Try to get user and membership from cookies first (faster)
	let user: UserInfo | null = null;
	let membership: MembershipInfo | null = null;
	let memberships: MembershipInfo[] = [];
	let needsSessionFetch = false;

	try {
		const userCookie = cookies.get('user');
		if (userCookie) {
			user = JSON.parse(userCookie);
		} else {
			needsSessionFetch = true;
		}

		const membershipCookie = cookies.get('membership');
		if (membershipCookie) {
			membership = JSON.parse(membershipCookie);
		}

		const membershipsCookie = cookies.get('memberships');
		if (membershipsCookie) {
			memberships = JSON.parse(membershipsCookie);
		} else {
			needsSessionFetch = true;
		}
	} catch (e) {
		needsSessionFetch = true;
	}

	// If we're missing user or memberships info, fetch from API session endpoint
	// This handles cross-subdomain SSO where we have a token but no user cookies
	if (needsSessionFetch) {
		try {
			const session = await api.get<SessionResponse>('/auth/session', token);
			user = session.user;
			membership = session.membership || null;
			memberships = session.memberships || [];

			// Update cookies with session data for faster subsequent loads
			const host = request.headers.get('host') || '';
			setAuthCookie(cookies, 'user', JSON.stringify(user), host, false);
			if (membership) {
				setAuthCookie(cookies, 'membership', JSON.stringify(membership), host, false);
			}
			if (memberships.length > 0) {
				setAuthCookie(cookies, 'memberships', JSON.stringify(memberships), host, false);
			}
		} catch (err) {
			if (err instanceof ApiError && err.status === 401) {
				// Token is invalid or expired - redirect to login
				throw redirect(303, `/login?redirect=${encodeURIComponent(url.pathname)}`);
			}
			throw err;
		}
	}

	return {
		token,
		user,
		membership,
		memberships
	};
};
