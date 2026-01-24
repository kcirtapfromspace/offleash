import { redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { deleteAuthCookie } from '$lib/cookies';

// Handle logout via GET request (direct navigation to /logout)
export const load: PageServerLoad = async ({ cookies, request }) => {
	const host = request.headers.get('host') || '';

	// Delete cookies with proper domain for cross-subdomain clearing
	deleteAuthCookie(cookies, 'token', host);
	deleteAuthCookie(cookies, 'user', host);
	deleteAuthCookie(cookies, 'membership', host);
	deleteAuthCookie(cookies, 'memberships', host);
	deleteAuthCookie(cookies, 'token_has_org_id', host);

	throw redirect(303, '/login');
};

// Handle logout via form POST
export const actions: Actions = {
	default: async ({ cookies, request }) => {
		const host = request.headers.get('host') || '';

		// Delete cookies with proper domain for cross-subdomain clearing
		deleteAuthCookie(cookies, 'token', host);
		deleteAuthCookie(cookies, 'user', host);
		deleteAuthCookie(cookies, 'membership', host);
		deleteAuthCookie(cookies, 'memberships', host);
		deleteAuthCookie(cookies, 'token_has_org_id', host);

		throw redirect(303, '/login');
	}
};
