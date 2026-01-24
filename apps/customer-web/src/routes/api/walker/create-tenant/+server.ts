import { json } from '@sveltejs/kit';
import { dev } from '$app/environment';
import type { RequestHandler } from './$types';
import { api, ApiError } from '$lib/api';

interface CreateTenantRequest {
	business_name: string;
	slug: string;
}

interface CreateTenantResponse {
	success: boolean;
	organization_id: string;
	slug: string;
	token: string;
	user: {
		id: string;
		email: string;
		first_name: string;
		last_name: string;
	};
}

export const POST: RequestHandler = async ({ request, cookies }) => {
	const token = cookies.get('token');

	if (!token) {
		return json({ error: 'Not authenticated' }, { status: 401 });
	}

	try {
		const body: CreateTenantRequest = await request.json();

		if (!body.business_name || !body.slug) {
			return json({ error: 'Business name and slug are required' }, { status: 400 });
		}

		const response = await api.post<CreateTenantResponse>(
			'/walker/create-tenant',
			body,
			token
		);

		// Update token cookie with the new token that has org context
		if (response.token) {
			cookies.set('token', response.token, {
				path: '/',
				httpOnly: true,
				secure: !dev,
				sameSite: 'lax',
				maxAge: 60 * 60 * 24 * 7 // 7 days
			});
		}

		return json({
			success: response.success,
			slug: response.slug,
			organization_id: response.organization_id
		});
	} catch (err) {
		if (err instanceof ApiError) {
			return json({ error: err.message }, { status: err.status });
		}
		return json({ error: 'Failed to create business' }, { status: 500 });
	}
};
