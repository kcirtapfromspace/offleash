import { redirect, fail } from '@sveltejs/kit';
import type { Actions } from './$types';
import { api, ApiError } from '$lib/api';

interface CreateTenantResponse {
	organization: {
		id: string;
		name: string;
		slug: string;
	};
	tenant_database: {
		id: string;
		status: string;
	};
	admin_user: {
		id: string;
		email: string;
	};
}

export const actions: Actions = {
	default: async ({ request, cookies }) => {
		const token = cookies.get('platform_token');
		if (!token) {
			throw redirect(303, '/login');
		}

		const data = await request.formData();
		const name = data.get('name')?.toString();
		const slug = data.get('slug')?.toString();
		const adminEmail = data.get('adminEmail')?.toString();
		const adminPassword = data.get('adminPassword')?.toString();
		const subscriptionTier = data.get('subscriptionTier')?.toString() || 'starter';

		if (!name || !slug || !adminEmail || !adminPassword) {
			return fail(400, {
				error: 'All fields are required',
				name,
				slug,
				adminEmail
			});
		}

		// Validate slug format
		if (!/^[a-z0-9-]+$/.test(slug)) {
			return fail(400, {
				error: 'Slug must contain only lowercase letters, numbers, and hyphens',
				name,
				slug,
				adminEmail
			});
		}

		try {
			const response = await api.post<CreateTenantResponse>(
				'/platform/tenants',
				{
					name,
					slug,
					admin_email: adminEmail,
					admin_password: adminPassword
				},
				token
			);

			// Redirect to the new tenant's page
			throw redirect(303, `/tenants/${response.organization.id}`);
		} catch (err) {
			if (err instanceof ApiError) {
				return fail(err.status, {
					error: err.message || 'Failed to create tenant',
					name,
					slug,
					adminEmail
				});
			}
			// Re-throw redirects
			throw err;
		}
	}
};
