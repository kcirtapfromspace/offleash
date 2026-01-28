import type { PageServerLoad, Actions } from './$types';
import { env } from '$env/dynamic/private';
import { fail, redirect } from '@sveltejs/kit';

const API_URL = env.API_URL || 'http://localhost:8080';

export const load: PageServerLoad = async ({ cookies, parent }) => {
	const parentData = await parent();

	// Only owners can access organization settings
	const role = parentData.membership?.role;
	if (role !== 'owner') {
		throw redirect(303, '/settings');
	}

	return {
		organizationName: parentData.membership?.organization_name || 'Unknown Organization',
		organizationSlug: parentData.membership?.organization_slug || ''
	};
};

export const actions: Actions = {
	delete: async ({ cookies }) => {
		const token = cookies.get('token');
		if (!token) {
			return fail(401, { error: 'Not authenticated' });
		}

		try {
			const response = await fetch(`${API_URL}/contexts/organization`, {
				method: 'DELETE',
				headers: {
					Authorization: `Bearer ${token}`
				}
			});

			if (!response.ok) {
				const error = await response
					.json()
					.catch(() => ({ message: 'Failed to delete organization' }));
				return fail(400, { error: error.message || 'Failed to delete organization' });
			}

			// Clear cookies and redirect to login
			cookies.delete('token', { path: '/' });
			cookies.delete('user', { path: '/' });
			cookies.delete('membership', { path: '/' });
			cookies.delete('memberships', { path: '/' });
		} catch (error) {
			console.error('Error deleting organization:', error);
			return fail(500, { error: 'Server error' });
		}

		// Redirect to login after deletion
		throw redirect(303, '/login?deleted=true');
	}
};
