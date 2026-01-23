import type { PageServerLoad, Actions } from './$types';
import { env } from '$env/dynamic/private';
import { fail } from '@sveltejs/kit';

const API_URL = env.API_URL || 'http://localhost:3000';

interface PaymentMethod {
	id: string;
	method_type: string;
	display_name: string;
	card_last_four: string | null;
	card_brand: string | null;
	card_exp_month: number | null;
	card_exp_year: number | null;
	nickname: string | null;
	is_default: boolean;
	is_expired: boolean;
	created_at: string;
}

export const load: PageServerLoad = async ({ cookies, locals }) => {
	const token = cookies.get('token');

	if (!token) {
		return { paymentMethods: [] };
	}

	try {
		const response = await fetch(`${API_URL}/payment-methods`, {
			headers: {
				Authorization: `Bearer ${token}`,
			},
		});

		if (!response.ok) {
			console.error('Failed to fetch payment methods:', response.status);
			return { paymentMethods: [] };
		}

		const paymentMethods = await response.json() as PaymentMethod[];
		return { paymentMethods };
	} catch (error) {
		console.error('Error fetching payment methods:', error);
		return { paymentMethods: [] };
	}
};

export const actions: Actions = {
	delete: async ({ request, cookies }) => {
		const token = cookies.get('token');
		if (!token) {
			return fail(401, { error: 'Not authenticated' });
		}

		const formData = await request.formData();
		const methodId = formData.get('methodId')?.toString();

		if (!methodId) {
			return fail(400, { error: 'Payment method ID is required' });
		}

		try {
			const response = await fetch(`${API_URL}/payment-methods/${methodId}`, {
				method: 'DELETE',
				headers: {
					Authorization: `Bearer ${token}`,
				},
			});

			if (!response.ok) {
				const error = await response.json().catch(() => ({ message: 'Failed to delete payment method' }));
				return fail(400, { error: error.message || 'Failed to delete payment method' });
			}

			return { success: true };
		} catch (error) {
			console.error('Error deleting payment method:', error);
			return fail(500, { error: 'Server error' });
		}
	},

	setDefault: async ({ request, cookies }) => {
		const token = cookies.get('token');
		if (!token) {
			return fail(401, { error: 'Not authenticated' });
		}

		const formData = await request.formData();
		const methodId = formData.get('methodId')?.toString();

		if (!methodId) {
			return fail(400, { error: 'Payment method ID is required' });
		}

		try {
			const response = await fetch(`${API_URL}/payment-methods/${methodId}/default`, {
				method: 'POST',
				headers: {
					Authorization: `Bearer ${token}`,
				},
			});

			if (!response.ok) {
				const error = await response.json().catch(() => ({ message: 'Failed to set default payment method' }));
				return fail(400, { error: error.message || 'Failed to set default payment method' });
			}

			return { success: true };
		} catch (error) {
			console.error('Error setting default payment method:', error);
			return fail(500, { error: 'Server error' });
		}
	},

	updateNickname: async ({ request, cookies }) => {
		const token = cookies.get('token');
		if (!token) {
			return fail(401, { error: 'Not authenticated' });
		}

		const formData = await request.formData();
		const methodId = formData.get('methodId')?.toString();
		const nickname = formData.get('nickname')?.toString() || null;

		if (!methodId) {
			return fail(400, { error: 'Payment method ID is required' });
		}

		try {
			const response = await fetch(`${API_URL}/payment-methods/${methodId}`, {
				method: 'PATCH',
				headers: {
					'Authorization': `Bearer ${token}`,
					'Content-Type': 'application/json',
				},
				body: JSON.stringify({ nickname }),
			});

			if (!response.ok) {
				const error = await response.json().catch(() => ({ message: 'Failed to update payment method' }));
				return fail(400, { error: error.message || 'Failed to update payment method' });
			}

			return { success: true };
		} catch (error) {
			console.error('Error updating payment method:', error);
			return fail(500, { error: 'Server error' });
		}
	},
};
