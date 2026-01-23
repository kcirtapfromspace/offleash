import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { api, ApiError } from '$lib/api';

interface CreateTenantRequest {
	name: string;
	slug: string;
}

interface CreateTenantResponse {
	id: string;
	name: string;
	slug: string;
}

export const POST: RequestHandler = async ({ request, cookies }) => {
	const token = cookies.get('token');

	if (!token) {
		return json({ error: 'Not authenticated' }, { status: 401 });
	}

	try {
		const body: CreateTenantRequest = await request.json();

		if (!body.name || !body.slug) {
			return json({ error: 'Name and slug are required' }, { status: 400 });
		}

		const response = await api.post<CreateTenantResponse>(
			'/walker/create-tenant',
			body,
			token
		);

		return json(response);
	} catch (err) {
		if (err instanceof ApiError) {
			return json({ error: err.message }, { status: err.status });
		}
		return json({ error: 'Failed to create business' }, { status: 500 });
	}
};
