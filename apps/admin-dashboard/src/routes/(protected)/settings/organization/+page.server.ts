import type { PageServerLoad, Actions } from './$types';
import { env } from '$env/dynamic/private';
import { fail, redirect } from '@sveltejs/kit';

const API_URL = env.API_URL || 'http://localhost:8080';

interface PaymentConfig {
	business_model: string;
	fee_structure: string;
	billing_frequency: string;
	apple_pay_enabled: boolean;
	google_pay_enabled: boolean;
	preferred_provider: string | null;
}

interface PaymentProvider {
	id: string;
	provider_type: string;
	is_active: boolean;
	charges_enabled: boolean;
	payouts_enabled: boolean;
	created_at: string;
}

export const load: PageServerLoad = async ({ cookies, parent }) => {
	const parentData = await parent();
	const token = cookies.get('token');

	// Only owners can access organization settings
	const role = parentData.membership?.role;
	if (role !== 'owner') {
		throw redirect(303, '/settings');
	}

	let paymentConfig: PaymentConfig | null = null;
	let paymentProviders: PaymentProvider[] = [];

	if (token) {
		// Fetch payment configuration
		try {
			const configResponse = await fetch(`${API_URL}/admin/payment-config`, {
				headers: { Authorization: `Bearer ${token}` }
			});
			if (configResponse.ok) {
				paymentConfig = await configResponse.json();
			}
		} catch (error) {
			console.error('Error fetching payment config:', error);
		}

		// Fetch connected payment providers
		try {
			const providersResponse = await fetch(`${API_URL}/payment-providers`, {
				headers: { Authorization: `Bearer ${token}` }
			});
			if (providersResponse.ok) {
				paymentProviders = await providersResponse.json();
			}
		} catch (error) {
			console.error('Error fetching payment providers:', error);
		}
	}

	return {
		organizationName: parentData.membership?.organization_name || 'Unknown Organization',
		organizationSlug: parentData.membership?.organization_slug || '',
		paymentConfig,
		paymentProviders
	};
};

export const actions: Actions = {
	updatePaymentConfig: async ({ request, cookies }) => {
		const token = cookies.get('token');
		if (!token) {
			return fail(401, { error: 'Not authenticated' });
		}

		const formData = await request.formData();
		const businessModel = formData.get('businessModel')?.toString();
		const feeStructure = formData.get('feeStructure')?.toString();
		const billingFrequency = formData.get('billingFrequency')?.toString();
		const applePayEnabled = formData.get('applePayEnabled') === 'on';
		const googlePayEnabled = formData.get('googlePayEnabled') === 'on';
		const preferredProvider = formData.get('preferredProvider')?.toString() || null;

		try {
			const response = await fetch(`${API_URL}/admin/payment-config`, {
				method: 'PUT',
				headers: {
					Authorization: `Bearer ${token}`,
					'Content-Type': 'application/json'
				},
				body: JSON.stringify({
					business_model: businessModel,
					fee_structure: feeStructure,
					billing_frequency: billingFrequency,
					apple_pay_enabled: applePayEnabled,
					google_pay_enabled: googlePayEnabled,
					preferred_provider: preferredProvider
				})
			});

			if (!response.ok) {
				const error = await response
					.json()
					.catch(() => ({ message: 'Failed to update payment configuration' }));
				return fail(400, { configError: error.message || 'Failed to update payment configuration' });
			}

			return { configSuccess: true };
		} catch (error) {
			console.error('Error updating payment config:', error);
			return fail(500, { configError: 'Server error' });
		}
	},

	connectStripe: async ({ cookies }) => {
		const token = cookies.get('token');
		if (!token) {
			return fail(401, { error: 'Not authenticated' });
		}

		try {
			const response = await fetch(`${API_URL}/payment-providers/stripe/connect`, {
				headers: { Authorization: `Bearer ${token}` }
			});

			if (!response.ok) {
				const error = await response.json().catch(() => ({ message: 'Failed to initiate Stripe connection' }));
				return fail(400, { providerError: error.message });
			}

			const data = await response.json();
			// Return the OAuth URL to redirect to
			return { stripeConnectUrl: data.url };
		} catch (error) {
			console.error('Error connecting Stripe:', error);
			return fail(500, { providerError: 'Server error' });
		}
	},

	connectSquare: async ({ cookies }) => {
		const token = cookies.get('token');
		if (!token) {
			return fail(401, { error: 'Not authenticated' });
		}

		try {
			const response = await fetch(`${API_URL}/payment-providers/square/connect`, {
				headers: { Authorization: `Bearer ${token}` }
			});

			if (!response.ok) {
				const error = await response.json().catch(() => ({ message: 'Failed to initiate Square connection' }));
				return fail(400, { providerError: error.message });
			}

			const data = await response.json();
			// Return the OAuth URL to redirect to
			return { squareConnectUrl: data.url };
		} catch (error) {
			console.error('Error connecting Square:', error);
			return fail(500, { providerError: 'Server error' });
		}
	},

	disconnectProvider: async ({ request, cookies }) => {
		const token = cookies.get('token');
		if (!token) {
			return fail(401, { error: 'Not authenticated' });
		}

		const formData = await request.formData();
		const providerId = formData.get('providerId')?.toString();

		if (!providerId) {
			return fail(400, { providerError: 'Provider ID is required' });
		}

		try {
			const response = await fetch(`${API_URL}/payment-providers/${providerId}`, {
				method: 'DELETE',
				headers: { Authorization: `Bearer ${token}` }
			});

			if (!response.ok) {
				const error = await response.json().catch(() => ({ message: 'Failed to disconnect provider' }));
				return fail(400, { providerError: error.message });
			}

			return { providerSuccess: true };
		} catch (error) {
			console.error('Error disconnecting provider:', error);
			return fail(500, { providerError: 'Server error' });
		}
	},

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
				return fail(400, { deleteError: error.message || 'Failed to delete organization' });
			}

			// Clear cookies and redirect to login
			cookies.delete('token', { path: '/' });
			cookies.delete('user', { path: '/' });
			cookies.delete('membership', { path: '/' });
			cookies.delete('memberships', { path: '/' });
		} catch (error) {
			console.error('Error deleting organization:', error);
			return fail(500, { deleteError: 'Server error' });
		}

		// Redirect to login after deletion
		throw redirect(303, '/login?deleted=true');
	}
};
