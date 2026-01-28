import type { PageServerLoad, Actions } from './$types';
import { env } from '$env/dynamic/private';
import { fail } from '@sveltejs/kit';

const API_URL = env.API_URL || 'http://localhost:8080';

interface UserProfile {
	id: string;
	email: string;
	role: string;
	first_name: string;
	last_name: string;
	full_name: string;
	phone: string | null;
	timezone: string;
	created_at: string;
}

export const load: PageServerLoad = async ({ cookies }) => {
	const token = cookies.get('token');

	if (!token) {
		return { profile: null };
	}

	try {
		const response = await fetch(`${API_URL}/users/me`, {
			headers: {
				Authorization: `Bearer ${token}`
			}
		});

		if (!response.ok) {
			console.error('Failed to fetch profile:', response.status);
			return { profile: null };
		}

		const profile: UserProfile = await response.json();
		return { profile };
	} catch (error) {
		console.error('Error fetching profile:', error);
		return { profile: null };
	}
};

export const actions: Actions = {
	update: async ({ request, cookies }) => {
		const token = cookies.get('token');
		if (!token) {
			return fail(401, { error: 'Not authenticated' });
		}

		const formData = await request.formData();
		const firstName = formData.get('firstName')?.toString() || '';
		const lastName = formData.get('lastName')?.toString() || '';
		const phone = formData.get('phone')?.toString() || '';
		const timezone = formData.get('timezone')?.toString() || 'America/Denver';

		// Validation
		if (!firstName.trim()) {
			return fail(400, { error: 'First name is required', firstName, lastName, phone, timezone });
		}
		if (!lastName.trim()) {
			return fail(400, { error: 'Last name is required', firstName, lastName, phone, timezone });
		}

		try {
			const response = await fetch(`${API_URL}/users/me`, {
				method: 'PUT',
				headers: {
					Authorization: `Bearer ${token}`,
					'Content-Type': 'application/json'
				},
				body: JSON.stringify({
					first_name: firstName.trim(),
					last_name: lastName.trim(),
					phone: phone.trim() || null,
					timezone
				})
			});

			if (!response.ok) {
				const error = await response.json().catch(() => ({ message: 'Failed to update profile' }));
				return fail(400, {
					error: error.message || 'Failed to update profile',
					firstName,
					lastName,
					phone,
					timezone
				});
			}

			return { success: true };
		} catch (error) {
			console.error('Error updating profile:', error);
			return fail(500, { error: 'Server error', firstName, lastName, phone, timezone });
		}
	}
};
