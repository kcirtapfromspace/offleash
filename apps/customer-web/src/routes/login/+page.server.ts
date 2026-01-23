import { redirect, fail } from '@sveltejs/kit';
import { dev } from '$app/environment';
import type { Actions, PageServerLoad } from './$types';
import { api, ApiError } from '$lib/api';

interface MembershipInfo {
	id: string;
	organization_id: string;
	organization_name: string;
	organization_slug: string;
	role: string;
	is_default: boolean;
}

interface LoginResponse {
	token: string;
	user: {
		id: string;
		email: string;
		first_name: string;
		last_name: string;
		role: string;
	};
	membership?: MembershipInfo;
	memberships?: MembershipInfo[];
}

export const load: PageServerLoad = async ({ cookies }) => {
	const token = cookies.get('token');
	if (token) {
		throw redirect(303, '/services');
	}
	return {};
};

export const actions: Actions = {
	default: async ({ request, cookies }) => {
		const data = await request.formData();
		const email = data.get('email')?.toString();
		const password = data.get('password')?.toString();

		if (!email || !password) {
			return fail(400, { error: 'Email and password are required', email });
		}

		try {
			// Use universal login (no org_slug required)
			const response = await api.post<LoginResponse>('/auth/login/universal', {
				email,
				password
			});

			// Store token
			cookies.set('token', response.token, {
				path: '/',
				httpOnly: true,
				secure: !dev,
				sameSite: 'lax',
				maxAge: 60 * 60 * 24 * 7 // 7 days
			});

			// Store user info
			cookies.set('user', JSON.stringify(response.user), {
				path: '/',
				httpOnly: false, // Allow client access for display
				secure: !dev,
				sameSite: 'lax',
				maxAge: 60 * 60 * 24 * 7
			});

			// Store current membership if available
			if (response.membership) {
				cookies.set('membership', JSON.stringify(response.membership), {
					path: '/',
					httpOnly: false,
					secure: !dev,
					sameSite: 'lax',
					maxAge: 60 * 60 * 24 * 7
				});
			}

			// Store all memberships for context switching
			if (response.memberships && response.memberships.length > 0) {
				cookies.set('memberships', JSON.stringify(response.memberships), {
					path: '/',
					httpOnly: false,
					secure: !dev,
					sameSite: 'lax',
					maxAge: 60 * 60 * 24 * 7
				});
			}

			throw redirect(303, '/services');
		} catch (err) {
			if (err instanceof ApiError) {
				if (err.status === 401) {
					return fail(401, { error: 'Invalid email or password', email });
				}
				return fail(err.status, {
					error: err.message || 'An error occurred. Please try again.',
					email
				});
			}
			// Re-throw redirects
			throw err;
		}
	}
};
