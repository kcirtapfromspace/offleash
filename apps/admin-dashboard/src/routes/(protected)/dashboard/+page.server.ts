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
	status: string;
	scheduled_start: string;
	scheduled_end: string;
	price_cents: number;
	price_display: string;
}

interface User {
	id: string;
	email: string;
	first_name: string;
	last_name: string;
	role: string;
	is_active: boolean;
}

export const load: PageServerLoad = async ({ parent }) => {
	const { token, membership, user } = await parent();

	// Check if user is admin/owner (can see full dashboard) or walker (limited view)
	const isAdmin = membership?.role === 'admin' || membership?.role === 'owner';

	try {
		// Walkers only fetch their own bookings, admins fetch all + users
		let bookings: Booking[] = [];
		let walkers: User[] = [];
		let customers: User[] = [];

		if (isAdmin) {
			// Admins can see everything
			const [bookingsData, users] = await Promise.all([
				api.get<Booking[]>('/bookings', token),
				api.get<User[]>('/users', token)
			]);
			bookings = bookingsData;
			walkers = users.filter((u) => u.role === 'walker');
			customers = users.filter((u) => u.role === 'customer');
		} else {
			// Walkers use the walker-specific endpoint (returns only their bookings)
			bookings = await api.get<Booking[]>('/bookings/walker', token);
		}

		const now = new Date();
		const todayStart = new Date(now.getFullYear(), now.getMonth(), now.getDate());
		const weekStart = new Date(todayStart);
		weekStart.setDate(weekStart.getDate() - 7);

		// Calculate metrics
		const todayBookings = bookings.filter((b) => {
			const start = new Date(b.scheduled_start);
			return start >= todayStart;
		});

		const weekBookings = bookings.filter((b) => {
			const start = new Date(b.scheduled_start);
			return start >= weekStart;
		});

		const completedBookings = bookings.filter((b) => b.status === 'completed');
		const pendingBookings = bookings.filter((b) => b.status === 'pending');
		const confirmedBookings = bookings.filter((b) => b.status === 'confirmed');

		// Revenue calculation (admin only)
		const totalRevenue = isAdmin ? completedBookings.reduce((sum, b) => sum + b.price_cents, 0) : 0;
		const weekRevenue = isAdmin
			? weekBookings.filter((b) => b.status === 'completed').reduce((sum, b) => sum + b.price_cents, 0)
			: 0;

		// Recent bookings (last 5)
		const recentBookings = [...bookings]
			.sort((a, b) => new Date(b.scheduled_start).getTime() - new Date(a.scheduled_start).getTime())
			.slice(0, 5);

		// Upcoming bookings
		const upcomingBookings = bookings
			.filter((b) => {
				const start = new Date(b.scheduled_start);
				return start >= now && ['pending', 'confirmed'].includes(b.status);
			})
			.sort((a, b) => new Date(a.scheduled_start).getTime() - new Date(b.scheduled_start).getTime())
			.slice(0, 5);

		// Count unique dogs/pets (using customer_id as proxy for now)
		const uniqueCustomers = new Set(bookings.map((b) => b.customer_id));

		return {
			isAdmin,
			metrics: {
				totalBookings: bookings.length,
				todayBookings: todayBookings.length,
				weekBookings: weekBookings.length,
				completedBookings: completedBookings.length,
				pendingBookings: pendingBookings.length,
				confirmedBookings: confirmedBookings.length,
				totalWalkers: walkers.length,
				activeWalkers: walkers.filter((w) => w.is_active).length,
				totalCustomers: customers.length,
				totalRevenue,
				weekRevenue,
				// Walker-specific metrics
				totalWalks: bookings.length,
				uniqueDogs: uniqueCustomers.size
			},
			recentBookings,
			upcomingBookings
		};
	} catch (error) {
		console.error('Failed to fetch dashboard data:', error);
		return {
			isAdmin,
			metrics: {
				totalBookings: 0,
				todayBookings: 0,
				weekBookings: 0,
				completedBookings: 0,
				pendingBookings: 0,
				confirmedBookings: 0,
				totalWalkers: 0,
				activeWalkers: 0,
				totalCustomers: 0,
				totalRevenue: 0,
				weekRevenue: 0,
				totalWalks: 0,
				uniqueDogs: 0
			},
			recentBookings: [],
			upcomingBookings: [],
			error: 'Failed to load dashboard data'
		};
	}
};
