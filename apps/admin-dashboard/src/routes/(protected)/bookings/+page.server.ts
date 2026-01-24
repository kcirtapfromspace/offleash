import type { PageServerLoad, Actions } from './$types';
import { api, ApiError } from '$lib/api';
import { fail } from '@sveltejs/kit';

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
	const { token, membership } = await parent();
	const statusFilter = url.searchParams.get('status') || 'all';
	const searchQuery = url.searchParams.get('q') || '';

	// Check if user is admin/owner or walker
	const isAdmin = membership?.role === 'admin' || membership?.role === 'owner';

	try {
		// Walkers use the walker-specific endpoint, admins use the full bookings endpoint
		const bookings = await api.get<Booking[]>(isAdmin ? '/bookings' : '/bookings/walker', token);

		// Apply filters
		let filteredBookings = bookings;

		if (statusFilter !== 'all') {
			filteredBookings = filteredBookings.filter((b) => b.status === statusFilter);
		}

		if (searchQuery) {
			const query = searchQuery.toLowerCase();
			filteredBookings = filteredBookings.filter(
				(b) =>
					b.customer_name.toLowerCase().includes(query) ||
					b.walker_name.toLowerCase().includes(query) ||
					b.service_name.toLowerCase().includes(query)
			);
		}

		// Sort by date (newest first)
		filteredBookings.sort(
			(a, b) => new Date(b.scheduled_start).getTime() - new Date(a.scheduled_start).getTime()
		);

		// Get unique statuses for filter
		const statuses = [...new Set(bookings.map((b) => b.status))];

		return {
			bookings: filteredBookings,
			statuses,
			currentStatus: statusFilter,
			searchQuery
		};
	} catch (error) {
		console.error('Failed to fetch bookings:', error);
		// Include more error detail for debugging
		const errorMessage = error instanceof Error ? error.message : String(error);
		const apiError = error as { status?: number };
		const errorDetail = apiError.status
			? `API Error ${apiError.status}: ${errorMessage}`
			: `Error: ${errorMessage}`;

		return {
			bookings: [],
			statuses: [],
			currentStatus: statusFilter,
			searchQuery,
			error: errorDetail
		};
	}
};

export const actions: Actions = {
	updateStatus: async ({ request, cookies }) => {
		const token = cookies.get('token');
		if (!token) {
			return fail(401, { error: 'Not authenticated' });
		}

		const formData = await request.formData();
		const bookingId = formData.get('booking_id')?.toString();
		const status = formData.get('status')?.toString();

		if (!bookingId || !status) {
			return fail(400, { error: 'Missing required fields' });
		}

		try {
			await api.put(`/bookings/${bookingId}`, { status }, token);
			return { success: true };
		} catch (err) {
			if (err instanceof ApiError) {
				return fail(err.status, { error: err.message || 'Failed to update booking' });
			}
			throw err;
		}
	},

	cancel: async ({ request, cookies }) => {
		const token = cookies.get('token');
		if (!token) {
			return fail(401, { error: 'Not authenticated' });
		}

		const formData = await request.formData();
		const bookingId = formData.get('booking_id')?.toString();

		if (!bookingId) {
			return fail(400, { error: 'Missing booking ID' });
		}

		try {
			await api.post(`/bookings/${bookingId}/cancel`, {}, token);
			return { success: true };
		} catch (err) {
			if (err instanceof ApiError) {
				return fail(err.status, { error: err.message || 'Failed to cancel booking' });
			}
			throw err;
		}
	}
};
