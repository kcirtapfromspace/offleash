import { redirect, fail } from '@sveltejs/kit';
import { dev } from '$app/environment';
import type { Actions, PageServerLoad } from './$types';
import { api, ApiError } from '$lib/api';

interface RegisterResponse {
	token: string;
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
		const firstName = data.get('firstName')?.toString();
		const lastName = data.get('lastName')?.toString();
		const email = data.get('email')?.toString();
		const phone = data.get('phone')?.toString();
		const password = data.get('password')?.toString();
		const confirmPassword = data.get('confirmPassword')?.toString();

		if (!firstName || !lastName || !email || !password) {
			return fail(400, {
				error: 'All required fields must be filled',
				firstName,
				lastName,
				email,
				phone
			});
		}

		if (password !== confirmPassword) {
			return fail(400, {
				error: 'Passwords do not match',
				firstName,
				lastName,
				email,
				phone
			});
		}

		try {
			const response = await api.post<RegisterResponse>('/auth/register', {
				org_slug: 'demo',
				first_name: firstName,
				last_name: lastName,
				email,
				phone,
				password,
				role: 'customer'
			});

			cookies.set('token', response.token, {
				path: '/',
				httpOnly: true,
				secure: !dev,
				sameSite: 'lax',
				maxAge: 60 * 60 * 24 * 7 // 7 days
			});

			throw redirect(303, '/services');
		} catch (err) {
			if (err instanceof ApiError) {
				return fail(err.status, {
					error: err.message || 'Registration failed. Please try again.',
					firstName,
					lastName,
					email,
					phone
				});
			}
			// Re-throw redirects
			throw err;
		}
	}
};
