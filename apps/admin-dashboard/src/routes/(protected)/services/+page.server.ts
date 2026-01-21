import type { PageServerLoad, Actions } from './$types';
import { api, ApiError } from '$lib/api';
import { fail } from '@sveltejs/kit';

interface Service {
	id: string;
	name: string;
	description: string | null;
	duration_minutes: number;
	price_cents: number;
	price_display: string;
	is_active: boolean;
}

export const load: PageServerLoad = async ({ parent }) => {
	const { token } = await parent();

	try {
		const services = await api.get<Service[]>('/services', token);
		return { services };
	} catch (error) {
		console.error('Failed to fetch services:', error);
		return { services: [], error: 'Failed to load services' };
	}
};

export const actions: Actions = {
	create: async ({ request, cookies }) => {
		const token = cookies.get('token');
		if (!token) {
			return fail(401, { error: 'Not authenticated' });
		}

		const formData = await request.formData();
		const name = formData.get('name')?.toString();
		const description = formData.get('description')?.toString() || null;
		const durationMinutes = parseInt(formData.get('duration_minutes')?.toString() || '0');
		const priceStr = formData.get('price')?.toString() || '0';

		if (!name) {
			return fail(400, { error: 'Service name is required' });
		}

		if (durationMinutes <= 0) {
			return fail(400, { error: 'Duration must be greater than 0' });
		}

		// Convert dollars to cents
		const priceCents = Math.round(parseFloat(priceStr) * 100);

		if (priceCents < 0) {
			return fail(400, { error: 'Price cannot be negative' });
		}

		try {
			await api.post(
				'/services',
				{
					name,
					description,
					duration_minutes: durationMinutes,
					base_price_cents: priceCents
				},
				token
			);

			return { success: true, message: 'Service created successfully' };
		} catch (err) {
			if (err instanceof ApiError) {
				return fail(err.status, { error: err.message || 'Failed to create service' });
			}
			throw err;
		}
	},

	update: async ({ request, cookies }) => {
		const token = cookies.get('token');
		if (!token) {
			return fail(401, { error: 'Not authenticated' });
		}

		const formData = await request.formData();
		const serviceId = formData.get('service_id')?.toString();
		const name = formData.get('name')?.toString();
		const description = formData.get('description')?.toString() || null;
		const durationMinutes = parseInt(formData.get('duration_minutes')?.toString() || '0');
		const priceStr = formData.get('price')?.toString() || '0';
		const isActive = formData.get('is_active') === 'true';

		if (!serviceId) {
			return fail(400, { error: 'Service ID is required' });
		}

		if (!name) {
			return fail(400, { error: 'Service name is required' });
		}

		// Convert dollars to cents
		const priceCents = Math.round(parseFloat(priceStr) * 100);

		try {
			await api.patch(
				`/services/${serviceId}`,
				{
					name,
					description,
					duration_minutes: durationMinutes,
					base_price_cents: priceCents,
					is_active: isActive
				},
				token
			);

			return { success: true, message: 'Service updated successfully' };
		} catch (err) {
			if (err instanceof ApiError) {
				return fail(err.status, { error: err.message || 'Failed to update service' });
			}
			throw err;
		}
	},

	toggleActive: async ({ request, cookies }) => {
		const token = cookies.get('token');
		if (!token) {
			return fail(401, { error: 'Not authenticated' });
		}

		const formData = await request.formData();
		const serviceId = formData.get('service_id')?.toString();
		const isActive = formData.get('is_active') === 'true';

		if (!serviceId) {
			return fail(400, { error: 'Service ID is required' });
		}

		try {
			await api.patch(
				`/services/${serviceId}`,
				{
					is_active: !isActive
				},
				token
			);

			return { success: true };
		} catch (err) {
			if (err instanceof ApiError) {
				return fail(err.status, { error: err.message || 'Failed to update service' });
			}
			throw err;
		}
	}
};
