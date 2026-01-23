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

	linkGoogle: async ({ request, cookies }) => {
		const token = cookies.get('token');
		if (!token) {
			return fail(401, { error: 'Not authenticated' });
		}

		const formData = await request.formData();
		const idToken = formData.get('id_token')?.toString();

		if (!idToken) {
			return fail(400, { error: 'Google token is required' });
		}

		try {
			const response = await fetch(`${API_URL}/users/me/identities/google`, {
				method: 'POST',
				headers: {
					'Authorization': `Bearer ${token}`,
					'Content-Type': 'application/json',
				},
				body: JSON.stringify({ id_token: idToken }),
			});

			if (!response.ok) {
				const error = await response.json().catch(() => ({ message: 'Failed to link Google account' }));
				return fail(400, { error: error.message || 'Failed to link Google account' });
			}

			return { success: true, message: 'Google account linked successfully' };
		} catch (error) {
			console.error('Error linking Google:', error);
			return fail(500, { error: 'Server error' });
		}
	},

	linkApple: async ({ request, cookies }) => {
		const token = cookies.get('token');
		if (!token) {
			return fail(401, { error: 'Not authenticated' });
		}

		const formData = await request.formData();
		const idToken = formData.get('id_token')?.toString();

		if (!idToken) {
			return fail(400, { error: 'Apple token is required' });
		}

		try {
			const response = await fetch(`${API_URL}/users/me/identities/apple`, {
				method: 'POST',
				headers: {
					'Authorization': `Bearer ${token}`,
					'Content-Type': 'application/json',
				},
				body: JSON.stringify({ id_token: idToken }),
			});

			if (!response.ok) {
				const error = await response.json().catch(() => ({ message: 'Failed to link Apple account' }));
				return fail(400, { error: error.message || 'Failed to link Apple account' });
			}

			return { success: true, message: 'Apple account linked successfully' };
		} catch (error) {
			console.error('Error linking Apple:', error);
			return fail(500, { error: 'Server error' });
		}
	},

	linkEmail: async ({ request, cookies }) => {
		const token = cookies.get('token');
		if (!token) {
			return fail(401, { error: 'Not authenticated' });
		}

		const formData = await request.formData();
		const password = formData.get('password')?.toString();
		const confirmPassword = formData.get('confirmPassword')?.toString();

		if (!password || !confirmPassword) {
			return fail(400, { error: 'Password and confirmation are required' });
		}

		if (password !== confirmPassword) {
			return fail(400, { error: 'Passwords do not match' });
		}

		if (password.length < 8) {
			return fail(400, { error: 'Password must be at least 8 characters' });
		}

		try {
			const response = await fetch(`${API_URL}/users/me/identities/email`, {
				method: 'POST',
				headers: {
					'Authorization': `Bearer ${token}`,
					'Content-Type': 'application/json',
				},
				body: JSON.stringify({ password }),
			});

			if (!response.ok) {
				const error = await response.json().catch(() => ({ message: 'Failed to set password' }));
				return fail(400, { error: error.message || 'Failed to set password' });
			}

			return { success: true, message: 'Password authentication added successfully' };
		} catch (error) {
			console.error('Error linking email:', error);
			return fail(500, { error: 'Server error' });
		}
	},

	changePassword: async ({ request, cookies }) => {
		const token = cookies.get('token');
		if (!token) {
			return fail(401, { error: 'Not authenticated' });
		}

		const formData = await request.formData();
		const currentPassword = formData.get('currentPassword')?.toString();
		const newPassword = formData.get('newPassword')?.toString();
		const confirmPassword = formData.get('confirmPassword')?.toString();

		if (!currentPassword || !newPassword || !confirmPassword) {
			return fail(400, { error: 'All password fields are required' });
		}

		if (newPassword !== confirmPassword) {
			return fail(400, { error: 'New passwords do not match' });
		}

		if (newPassword.length < 8) {
			return fail(400, { error: 'New password must be at least 8 characters' });
		}

		try {
			const response = await fetch(`${API_URL}/users/me/password`, {
				method: 'PUT',
				headers: {
					'Authorization': `Bearer ${token}`,
					'Content-Type': 'application/json',
				},
				body: JSON.stringify({
					current_password: currentPassword,
					new_password: newPassword,
				}),
			});

			if (!response.ok) {
				const error = await response.json().catch(() => ({ message: 'Failed to change password' }));
				return fail(400, { error: error.message || 'Failed to change password' });
			}

			return { success: true, message: 'Password changed successfully' };
		} catch (error) {
			console.error('Error changing password:', error);
			return fail(500, { error: 'Server error' });
		}
	},
};
