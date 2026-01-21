import type { PageServerLoad, Actions } from './$types';
import { api, ApiError } from '$lib/api';
import { fail, redirect } from '@sveltejs/kit';

interface Booking {
	id: string;
	customer_id: string;
	walker_id: string;
	service_id: string;
	location_id: string;
	status: string;
	scheduled_start: string;
	scheduled_end: string;
	price_cents: number;
	price_display: string;
	notes: string | null;
}

interface Service {
	id: string;
	name: string;
	description: string | null;
	duration_minutes: number;
	price_cents: number;
	price_display: string;
	is_active: boolean;
}

interface Location {
	id: string;
	name: string;
	full_address: string;
}

export const load: PageServerLoad = async ({ parent, params }) => {
	const { token } = await parent();

	try {
		const booking = await api.get<Booking>(`/bookings/${params.id}`, token);

		// Fetch related service and location details
		const [services, locations] = await Promise.all([
			api.get<Service[]>('/services', token),
			api.get<Location[]>('/locations', token)
		]);

		const service = services.find((s) => s.id === booking.service_id);
		const location = locations.find((l) => l.id === booking.location_id);

		return {
			booking,
			service,
			location,
			walkerName: 'Alex Walker' // TODO: Fetch walker details
		};
	} catch (error) {
		console.error('Failed to fetch booking:', error);
		throw redirect(303, '/bookings');
	}
};

export const actions: Actions = {
	cancel: async ({ params, cookies }) => {
		const token = cookies.get('token');
		if (!token) {
			throw redirect(303, '/login');
		}

		try {
			await api.post(`/bookings/${params.id}/cancel`, {}, token);
			return { success: true, message: 'Booking cancelled successfully' };
		} catch (err) {
			if (err instanceof ApiError) {
				return fail(err.status, { error: err.message || 'Failed to cancel booking' });
			}
			throw err;
		}
	}
};
