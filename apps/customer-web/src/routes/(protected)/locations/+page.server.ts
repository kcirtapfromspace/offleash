import type { PageServerLoad, Actions } from './$types';
import { api, ApiError } from '$lib/api';
import { fail } from '@sveltejs/kit';

interface Location {
	id: string;
	name: string;
	address: string;
	city: string;
	state: string;
	zip_code: string;
	full_address: string;
	latitude: number;
	longitude: number;
	notes: string | null;
	is_default: boolean;
}

export const load: PageServerLoad = async ({ parent }) => {
	const { token } = await parent();

	try {
		const locations = await api.get<Location[]>('/locations', token);
		return { locations };
	} catch (error) {
		console.error('Failed to fetch locations:', error);
		return { locations: [], error: 'Failed to load locations' };
	}
};

export const actions: Actions = {
	add: async ({ request, cookies }) => {
		const token = cookies.get('token');
		if (!token) {
			return fail(401, { error: 'Not authenticated' });
		}

		const formData = await request.formData();
		const name = formData.get('name')?.toString();
		const address = formData.get('address')?.toString();
		const city = formData.get('city')?.toString();
		const state = formData.get('state')?.toString();
		const zip_code = formData.get('zip_code')?.toString();
		const notes = formData.get('notes')?.toString() || null;
		const is_default = formData.get('is_default') === 'on';

		// Simple validation
		if (!name || !address || !city || !state || !zip_code) {
			return fail(400, {
				error: 'All address fields are required',
				name,
				address,
				city,
				state,
				zip_code,
				notes
			});
		}

		try {
			// For demo, use fixed coordinates (in production, use geocoding API)
			await api.post(
				'/locations',
				{
					name,
					address,
					city,
					state,
					zip_code,
					latitude: 39.7392, // Denver coordinates as default
					longitude: -104.9903,
					notes,
					is_default
				},
				token
			);

			return { success: true };
		} catch (err) {
			if (err instanceof ApiError) {
				return fail(err.status, { error: err.message || 'Failed to add location' });
			}
			throw err;
		}
	}
};
