import type { PageServerLoad } from './$types';
import { api } from '$lib/api';
import { redirect } from '@sveltejs/kit';

interface User {
	id: string;
	email: string;
	first_name: string;
	last_name: string;
	full_name: string;
	phone: string | null;
	role: string;
	timezone: string;
	created_at: string;
}

interface Booking {
	id: string;
	customer_id: string;
	service_name: string;
	status: string;
	scheduled_at: string;
}

interface CustomerWithStats extends User {
	total_bookings: number;
	last_booking?: string;
}

export const load: PageServerLoad = async ({ parent }) => {
	const { token, membership } = await parent();

	// Only admin/owner can access customer management
	if (membership?.role !== 'admin' && membership?.role !== 'owner') {
		throw redirect(303, '/dashboard');
	}

	try {
		// Fetch customers
		const customers = await api.get<User[]>('/users?role=customer', token);

		// Fetch bookings to calculate stats per customer
		let bookings: Booking[] = [];
		try {
			bookings = await api.get<Booking[]>('/bookings', token);
		} catch {
			// Bookings may not be accessible
		}

		// Calculate stats per customer
		const customersWithStats: CustomerWithStats[] = customers.map((customer) => {
			const customerBookings = bookings.filter((b) => b.customer_id === customer.id);
			const sortedBookings = customerBookings.sort(
				(a, b) => new Date(b.scheduled_at).getTime() - new Date(a.scheduled_at).getTime()
			);

			return {
				...customer,
				total_bookings: customerBookings.length,
				last_booking: sortedBookings[0]?.scheduled_at
			};
		});

		// Sort by most recent customer first
		customersWithStats.sort(
			(a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
		);

		return { customers: customersWithStats };
	} catch (error) {
		console.error('Failed to fetch customers:', error);
		return { customers: [], error: 'Failed to load customers' };
	}
};
