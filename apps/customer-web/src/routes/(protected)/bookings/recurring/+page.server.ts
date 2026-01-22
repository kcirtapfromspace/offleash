import type { PageServerLoad } from './$types';
import { api } from '$lib/api';

interface RecurringBookingListItem {
	id: string;
	walker_id: string;
	walker_name: string;
	service_id: string;
	service_name: string;
	frequency: string;
	day_of_week_name: string;
	time_of_day: string;
	is_active: boolean;
	price_display: string;
	next_occurrence: string | null;
	total_bookings: number;
}

export const load: PageServerLoad = async ({ parent }) => {
	const { token } = await parent();

	try {
		const series = await api.get<RecurringBookingListItem[]>('/bookings/recurring', token);

		return {
			series
		};
	} catch (error) {
		console.error('Failed to load recurring bookings:', error);
		return {
			series: [],
			error: 'Failed to load recurring bookings'
		};
	}
};
