import type { PageServerLoad, Actions } from './$types';
import { api, ApiError } from '$lib/api';
import { fail, redirect } from '@sveltejs/kit';

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

interface WorkingHours {
	id: string;
	walker_id: string;
	day_of_week: number;
	day_name: string;
	start_time: string;
	end_time: string;
	is_active: boolean;
}

interface WalkerWithStats extends User {
	is_active: boolean;
	bookings_count?: number;
	completed_bookings?: number;
	working_hours?: WorkingHours[];
}

export const load: PageServerLoad = async ({ parent }) => {
	const { token, membership } = await parent();

	// Only admin/owner can access staff management
	if (membership?.role !== 'admin' && membership?.role !== 'owner') {
		throw redirect(303, '/dashboard');
	}

	try {
		// Fetch users filtered by role=walker
		const walkers = await api.get<User[]>('/users?role=walker', token);

		// Fetch working hours for each walker
		const walkersWithHours: WalkerWithStats[] = await Promise.all(
			walkers.map(async (walker) => {
				let working_hours: WorkingHours[] = [];
				try {
					working_hours = await api.get<WorkingHours[]>(`/working-hours/${walker.id}`, token);
				} catch {
					// Walker may not have working hours set yet
				}
				return {
					...walker,
					is_active: true, // Users are active by default
					working_hours
				};
			})
		);

		// Sort by name
		walkersWithHours.sort((a, b) =>
			`${a.first_name} ${a.last_name}`.localeCompare(`${b.first_name} ${b.last_name}`)
		);

		return { walkers: walkersWithHours };
	} catch (error) {
		console.error('Failed to fetch walkers:', error);
		return { walkers: [], error: 'Failed to load walkers' };
	}
};

export const actions: Actions = {
	createWalker: async ({ request, cookies }) => {
		const token = cookies.get('token');
		if (!token) {
			return fail(401, { error: 'Not authenticated' });
		}

		const formData = await request.formData();
		const email = formData.get('email')?.toString();
		const password = formData.get('password')?.toString();
		const firstName = formData.get('first_name')?.toString();
		const lastName = formData.get('last_name')?.toString();
		const phone = formData.get('phone')?.toString() || null;

		if (!email || !password || !firstName || !lastName) {
			return fail(400, { error: 'All fields are required' });
		}

		try {
			// Use admin endpoint to create walker (requires authentication)
			await api.post(
				'/admin/walkers',
				{
					email,
					password,
					first_name: firstName,
					last_name: lastName,
					phone
				},
				token
			);

			return { success: true, message: 'Walker created successfully' };
		} catch (err) {
			console.error('Failed to create walker:', err);
			if (err instanceof ApiError) {
				return fail(err.status, { error: err.message || 'Failed to create walker' });
			}
			return fail(500, { error: 'An unexpected error occurred' });
		}
	},

	updateWorkingHours: async ({ request, cookies }) => {
		const token = cookies.get('token');
		if (!token) {
			return fail(401, { error: 'Not authenticated' });
		}

		const formData = await request.formData();
		const walkerId = formData.get('walker_id')?.toString();
		const scheduleJson = formData.get('schedule')?.toString();

		if (!walkerId || !scheduleJson) {
			return fail(400, { error: 'Missing required fields' });
		}

		try {
			const schedule = JSON.parse(scheduleJson);
			await api.put(`/working-hours/${walkerId}`, { schedule }, token);
			return { success: true, message: 'Working hours updated' };
		} catch (err) {
			if (err instanceof ApiError) {
				return fail(err.status, { error: err.message || 'Failed to update working hours' });
			}
			throw err;
		}
	}
};
