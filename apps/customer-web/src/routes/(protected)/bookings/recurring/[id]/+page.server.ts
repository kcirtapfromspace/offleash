import type { PageServerLoad, Actions } from './$types';
import { api, ApiError } from '$lib/api';
import { fail, redirect } from '@sveltejs/kit';

interface RecurringBookingSeriesResponse {
	id: string;
	customer_id: string;
	walker_id: string;
	service_id: string;
	location_id: string;
	frequency: string;
	day_of_week: number;
	day_of_week_name: string;
	time_of_day: string;
	timezone: string;
	end_date: string | null;
	total_occurrences: number | null;
	is_active: boolean;
	price_cents_per_booking: number;
	price_display: string;
	default_notes: string | null;
	created_at: string;
}

interface SeriesBookingItem {
	id: string;
	occurrence_number: number;
	scheduled_start: string;
	scheduled_end: string;
	status: string;
	price_display: string;
}

interface RecurringSeriesDetailResponse {
	series: RecurringBookingSeriesResponse;
	walker_name: string;
	service_name: string;
	location_address: string;
	bookings: SeriesBookingItem[];
}

export const load: PageServerLoad = async ({ parent, params }) => {
	const { token } = await parent();

	try {
		const data = await api.get<RecurringSeriesDetailResponse>(
			`/bookings/recurring/${params.id}`,
			token
		);

		return {
			...data
		};
	} catch (error) {
		console.error('Failed to load recurring series:', error);
		if (error instanceof ApiError && error.status === 404) {
			throw redirect(303, '/bookings/recurring');
		}
		return {
			series: null,
			walker_name: '',
			service_name: '',
			location_address: '',
			bookings: [],
			error: 'Failed to load recurring series'
		};
	}
};

export const actions: Actions = {
	cancel: async ({ request, cookies, params }) => {
		const token = cookies.get('token');
		if (!token) {
			throw redirect(303, '/login');
		}

		const formData = await request.formData();
		const scope = formData.get('scope')?.toString() || 'all_future';

		try {
			await api.post(
				`/bookings/recurring/${params.id}/cancel`,
				{ scope },
				token
			);

			throw redirect(303, '/bookings/recurring');
		} catch (err) {
			if (err instanceof ApiError) {
				return fail(err.status, { error: err.message || 'Failed to cancel series' });
			}
			throw err;
		}
	}
};
