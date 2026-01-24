import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { api, ApiError } from '$lib/api';

interface TenantInfo {
	id: string;
	name: string;
	slug: string;
	custom_domain: string | null;
	status: string;
	subscription_tier: string;
	created_at: string;
	updated_at: string;
}

interface ListTenantsResponse {
	tenants: TenantInfo[];
	total: number;
	limit: number;
	offset: number;
}

export const load: PageServerLoad = async ({ cookies }) => {
	const token = cookies.get('platform_token');
	if (!token) {
		throw redirect(303, '/login');
	}

	try {
		const response = await api.get<ListTenantsResponse>('/admin/tenants', token);
		return {
			tenants: response.tenants,
			total: response.total
		};
	} catch (err) {
		if (err instanceof ApiError && err.status === 401) {
			throw redirect(303, '/login');
		}
		return {
			tenants: [],
			total: 0,
			error: err instanceof Error ? err.message : 'Failed to load tenants'
		};
	}
};
