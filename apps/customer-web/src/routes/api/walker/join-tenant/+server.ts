import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { api, ApiError } from '$lib/api';

interface JoinTenantRequest {
	inviteToken: string;
}

interface JoinTenantResponse {
	success: boolean;
	tenantName?: string;
	message?: string;
}

export const POST: RequestHandler = async ({ request, cookies }) => {
	const token = cookies.get('token');

	if (!token) {
		return json({ error: 'Not authenticated' }, { status: 401 });
	}

	try {
		const body: JoinTenantRequest = await request.json();

		if (!body.inviteToken) {
			return json({ error: 'Invite token is required' }, { status: 400 });
		}

		const response = await api.post<JoinTenantResponse>(
			'/walker/join-tenant',
			body,
			token
		);

		return json(response);
	} catch (err) {
		if (err instanceof ApiError) {
			return json({
				success: false,
				error: err.message,
				message: err.message
			}, { status: err.status });
		}
		return json({
			success: false,
			error: 'Failed to join business',
			message: 'This invitation is invalid or has expired.'
		}, { status: 500 });
	}
};
