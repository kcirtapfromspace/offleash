import type { PageServerLoad } from './$types';
import { api } from '$lib/api';

interface Booking {
	id: string;
	customer_id: string;
	customer_name: string;
	walker_id: string;
	walker_name: string;
	service_id: string;
	service_name: string;
	location_id: string;
	location_address: string;
	status: string;
	scheduled_start: string;
	scheduled_end: string;
	price_cents: number;
	price_display: string;
	notes: string | null;
}

export const load: PageServerLoad = async ({ parent, url }) => {
	const { token } = await parent();
	const filter = url.searchParams.get('filter') || 'upcoming';

	try {
		// Fetch bookings - the API may filter by authenticated user
		const bookings = await api.get<Booking[]>('/bookings', token);

		const now = new Date();

		// Filter based on upcoming vs past
		const filteredBookings = bookings.filter((b) => {
			const startTime = new Date(b.scheduled_start);
			if (filter === 'upcoming') {
				return startTime >= now && b.status !== 'cancelled';
			} else {
				return startTime < now || b.status === 'cancelled' || b.status === 'completed';
			}
		});

		// Sort by date
		filteredBookings.sort((a, b) => {
			const dateA = new Date(a.scheduled_start).getTime();
			const dateB = new Date(b.scheduled_start).getTime();
			return filter === 'upcoming' ? dateA - dateB : dateB - dateA;
		});

		return {
			bookings: filteredBookings,
			filter
		};
	} catch (error) {
		console.error('Failed to fetch bookings:', error);
		return {
			bookings: [],
			filter,
			error: 'Failed to load bookings'
		};
	}
};
