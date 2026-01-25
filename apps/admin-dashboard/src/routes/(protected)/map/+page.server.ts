import { error } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { api } from '$lib/api';

interface RouteStop {
	sequence: number;
	booking_id: string;
	customer_name: string;
	address: string;
	arrival_time: string;
	departure_time: string;
	travel_from_previous_minutes: number;
	service_duration_minutes: number;
}

interface RouteResponse {
	date: string;
	is_optimized: boolean;
	stops: RouteStop[];
	total_travel_minutes: number;
	total_distance_meters: number;
	savings_minutes: number;
}

interface BookingListItem {
	id: string;
	customer_id: string;
	customer_name: string;
	walker_id: string;
	walker_name: string;
	service_id: string;
	service_name: string;
	location_id: string;
	location_address: string;
	latitude: number | null;
	longitude: number | null;
	status: string;
	scheduled_start: string;
	scheduled_end: string;
	price_cents: number;
	price_display: string;
	notes: string | null;
	customer_phone: string | null;
}

export const load: PageServerLoad = async ({ parent, url }) => {
	const { token, membership, user } = await parent();

	if (!token || !membership) {
		throw error(401, 'Unauthorized');
	}

	// Get date from query param or use today
	const dateParam = url.searchParams.get('date');
	const today = new Date().toISOString().split('T')[0];
	const date = dateParam || today;

	// Determine walker ID - if user is a walker, use their ID; otherwise need to select
	const isWalker = membership.role === 'walker';
	const walkerId = isWalker ? user?.id : url.searchParams.get('walker_id');

	if (!walkerId) {
		// For admins/owners, we need to show a walker selector
		// For now, return empty state
		return {
			date,
			walkerId: null,
			walkerName: null,
			route: null,
			bookings: [],
			isWalker,
			error: isWalker ? null : 'Please select a walker to view their route'
		};
	}

	try {
		// Fetch optimized route
		const route = await api.get<RouteResponse>(
			`/walkers/${walkerId}/route?date=${date}`,
			token
		);

		// Fetch all bookings for the walker
		const allBookings = await api.get<BookingListItem[]>(
			'/bookings/walker',
			token
		);

		// Filter by date and active status
		const dateStart = new Date(date + 'T00:00:00');
		const dateEnd = new Date(date + 'T23:59:59');

		const bookings = allBookings.filter((b) => {
			const bookingDate = new Date(b.scheduled_start);
			const isOnDate = bookingDate >= dateStart && bookingDate <= dateEnd;
			const hasCoords = b.latitude && b.longitude;
			const isActive = ['pending', 'confirmed', 'in_progress'].includes(b.status);
			return isOnDate && hasCoords && isActive;
		}).map(b => ({
			id: b.id,
			customer_name: b.customer_name,
			service_name: b.service_name,
			status: b.status,
			scheduled_start: b.scheduled_start,
			scheduled_end: b.scheduled_end,
			address: b.location_address,
			latitude: b.latitude,
			longitude: b.longitude,
			notes: b.notes,
			customer_phone: b.customer_phone
		}));

		return {
			date,
			walkerId,
			walkerName: isWalker ? `${user?.first_name} ${user?.last_name}` : null,
			route,
			bookings,
			isWalker,
			error: null
		};
	} catch (err) {
		console.error('Failed to load route data:', err);
		return {
			date,
			walkerId,
			walkerName: null,
			route: null,
			bookings: [],
			isWalker,
			error: 'Failed to load route data'
		};
	}
};
