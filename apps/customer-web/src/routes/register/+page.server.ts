import { redirect, fail } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { api, ApiError } from '$lib/api';
import { setAuthCookie } from '$lib/cookies';

interface RegisterResponse {
	token: string;
}

// Extract org slug from hostname (subdomain-based routing)
function getOrgSlugFromHost(host: string): string | null {
	// Remove port if present
	const hostname = host.split(':')[0];

	// Check for subdomain pattern: {slug}.offleash.world or {slug}.offleash.pro
	const parts = hostname.split('.');
	if (parts.length >= 3) {
		const subdomain = parts[0];
		// Exclude common non-tenant subdomains
		if (!['www', 'app', 'admin', 'platform', 'api'].includes(subdomain)) {
			return subdomain;
		}
	}

	return null;
}

export const load: PageServerLoad = async ({ cookies, request }) => {
	const token = cookies.get('token');
	if (token) {
		throw redirect(303, '/services');
	}

	// Detect org from subdomain for branding
	const host = request.headers.get('host') || '';
	const orgSlug = getOrgSlugFromHost(host);

	return {
		orgSlug
	};
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

		// Detect org from subdomain
		const host = request.headers.get('host') || '';
		const orgSlug = getOrgSlugFromHost(host) || 'demo'; // Fallback to demo for main domain

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
				org_slug: orgSlug,
				first_name: firstName,
				last_name: lastName,
				email,
				phone,
				password,
				role: 'customer'
			});

			// Store token (shared across subdomains)
			setAuthCookie(cookies, 'token', response.token, host, true);

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
