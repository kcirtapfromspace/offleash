import type { PageServerLoad, Actions } from './$types';
import { env } from '$env/dynamic/private';
import { fail } from '@sveltejs/kit';

const API_URL = env.API_URL || 'http://localhost:3000';

interface Identity {
	id: string;
	provider: string;
	provider_user_id: string;
	provider_email: string | null;
	created_at: string;
	can_unlink: boolean;
}

export const load: PageServerLoad = async ({ cookies }) => {
	const token = cookies.get('token');

	if (!token) {
		return { identities: [] };
	}

	try {
		const response = await fetch(`${API_URL}/users/me/identities`, {
			headers: {
				Authorization: `Bearer ${token}`,
			},
		});

		if (!response.ok) {
			console.error('Failed to fetch identities:', response.status);
			return { identities: [] };
		}

		const data = await response.json();
		return { identities: data.identities as Identity[] };
	} catch (error) {
		console.error('Error fetching identities:', error);
		return { identities: [] };
	}
};

export const actions: Actions = {
	unlink: async ({ request, cookies }) => {
		const token = cookies.get('token');
		if (!token) {
			return fail(401, { error: 'Not authenticated' });
		}

		const formData = await request.formData();
		const identityId = formData.get('identityId')?.toString();

		if (!identityId) {
			return fail(400, { error: 'Identity ID is required' });
		}

		try {
			const response = await fetch(`${API_URL}/users/me/identities/${identityId}`, {
				method: 'DELETE',
				headers: {
					Authorization: `Bearer ${token}`,
				},
			});

			if (!response.ok) {
				const error = await response.json().catch(() => ({ message: 'Failed to unlink account' }));
				return fail(400, { error: error.message || 'Failed to unlink account' });
			}

			return { success: true };
		} catch (error) {
			console.error('Error unlinking identity:', error);
			return fail(500, { error: 'Server error' });
		}
	},
};
