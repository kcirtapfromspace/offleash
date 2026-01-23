import type { PageServerLoad, Actions } from './$types';
import { api, ApiError } from '$lib/api';
import { fail } from '@sveltejs/kit';

export interface Walker {
	id: string;
	email: string;
	first_name: string;
	last_name: string;
	full_name: string;
	role: string;
}

export interface CalendarEvent {
	id: string;
	user_id: string;
	title: string | null;
	description: string | null;
	start_time: string;
	end_time: string;
	all_day: boolean;
	event_type: 'booking' | 'block' | 'personal' | 'synced';
	calendar_connection_id: string | null;
	external_event_id: string | null;
	sync_status: string;
	recurrence_rule: string | null;
	color: string | null;
	is_blocking: boolean;
	created_at: string;
	updated_at: string;
	// Booking-specific fields (from join)
	customer_name?: string;
	service_name?: string;
	location_address?: string;
	walker_name?: string;
}

export interface Booking {
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

export interface WorkingHours {
	id: string;
	walker_id: string;
	day_of_week: number; // 0 = Sunday, 6 = Saturday
	day_name: string;
	start_time: string; // HH:MM format
	end_time: string; // HH:MM format
	is_active: boolean;
}

function getWeekStart(date: Date): Date {
	const d = new Date(date);
	const day = d.getDay();
	const diff = d.getDate() - day;
	return new Date(d.setDate(diff));
}

function getWeekEnd(date: Date): Date {
	const start = getWeekStart(date);
	const end = new Date(start);
	end.setDate(end.getDate() + 7);
	return end;
}

export const load: PageServerLoad = async ({ parent, url }) => {
	const { token, user } = await parent();

	// Get date from URL or default to today
	const dateParam = url.searchParams.get('date');
	const currentDate = dateParam ? new Date(dateParam) : new Date();

	// Get selected walker from URL (for admin view)
	const selectedWalkerId = url.searchParams.get('walker');

	// Calculate week boundaries
	const weekStart = getWeekStart(currentDate);
	const weekEnd = getWeekEnd(currentDate);

	// Format dates for API
	const startStr = weekStart.toISOString();
	const endStr = weekEnd.toISOString();

	try {
		// Fetch walkers list for admin dropdown
		const walkersResponse = await api.get<Walker[]>('/users?role=walker', token).catch(() => [] as Walker[]);

		// Determine which walker's data to show
		// If admin selected a walker, use that; otherwise use current user if they're a walker
		const viewingWalkerId = selectedWalkerId || (user.role === 'walker' ? user.id : walkersResponse[0]?.id);

		// Fetch calendar events, bookings, and working hours in parallel
		const [eventsResponse, bookingsResponse, workingHoursResponse] = await Promise.all([
			api.get<{ events: CalendarEvent[]; count: number }>(
				`/calendar/events?start=${encodeURIComponent(startStr)}&end=${encodeURIComponent(endStr)}`,
				token
			),
			api.get<Booking[]>('/bookings', token),
			viewingWalkerId
				? api.get<WorkingHours[]>(`/working-hours/${viewingWalkerId}`, token).catch(() => [] as WorkingHours[])
				: Promise.resolve([] as WorkingHours[])
		]);

		// Filter bookings to the current week and selected walker
		const weekBookings = bookingsResponse.filter(b => {
			const bookingStart = new Date(b.scheduled_start);
			const inWeek = bookingStart >= weekStart && bookingStart < weekEnd;
			// If viewing a specific walker, filter to their bookings
			const matchesWalker = !viewingWalkerId || b.walker_id === viewingWalkerId;
			return inWeek && matchesWalker;
		});

		// Transform bookings into calendar event format
		const bookingEvents: CalendarEvent[] = weekBookings.map(b => ({
			id: `booking-${b.id}`,
			user_id: b.walker_id,
			title: `${b.customer_name} - ${b.service_name}`,
			description: b.location_address,
			start_time: b.scheduled_start,
			end_time: b.scheduled_end,
			all_day: false,
			event_type: 'booking' as const,
			calendar_connection_id: null,
			external_event_id: null,
			sync_status: 'synced',
			recurrence_rule: null,
			color: getBookingColor(b.status),
			is_blocking: true,
			created_at: b.scheduled_start,
			updated_at: b.scheduled_start,
			customer_name: b.customer_name,
			service_name: b.service_name,
			location_address: b.location_address,
			walker_name: b.walker_name
		}));

		// Combine calendar events with booking events
		const allEvents = [...(eventsResponse.events || []), ...bookingEvents];

		return {
			events: allEvents,
			walkers: walkersResponse,
			selectedWalkerId: viewingWalkerId || null,
			workingHours: workingHoursResponse,
			weekStart: weekStart.toISOString(),
			weekEnd: weekEnd.toISOString(),
			currentDate: currentDate.toISOString(),
			isAdmin: user.role === 'admin'
		};
	} catch (error) {
		console.error('Failed to fetch calendar data:', error);
		return {
			events: [],
			walkers: [],
			selectedWalkerId: null,
			workingHours: [],
			weekStart: weekStart.toISOString(),
			weekEnd: weekEnd.toISOString(),
			currentDate: currentDate.toISOString(),
			isAdmin: user.role === 'admin',
			error: 'Failed to load calendar'
		};
	}
};

function getBookingColor(status: string): string {
	switch (status.toLowerCase()) {
		case 'pending':
			return '#fbbf24'; // yellow-400
		case 'confirmed':
			return '#3b82f6'; // blue-500
		case 'in_progress':
			return '#8b5cf6'; // purple-500
		case 'completed':
			return '#22c55e'; // green-500
		case 'cancelled':
			return '#ef4444'; // red-500
		default:
			return '#6b7280'; // gray-500
	}
}

export const actions: Actions = {
	createBlock: async ({ request, cookies }) => {
		const token = cookies.get('token');
		if (!token) {
			return fail(401, { error: 'Not authenticated' });
		}

		const formData = await request.formData();
		const title = formData.get('title')?.toString() || '';
		const startTime = formData.get('start_time')?.toString();
		const endTime = formData.get('end_time')?.toString();
		const isBlocking = formData.get('is_blocking') === 'true';

		if (!startTime || !endTime) {
			return fail(400, { error: 'Start and end time are required' });
		}

		try {
			await api.post(
				'/calendar/events',
				{
					title: title || null,
					start_time: startTime,
					end_time: endTime,
					event_type: 'block',
					is_blocking: isBlocking
				},
				token
			);
			return { success: true };
		} catch (err) {
			if (err instanceof ApiError) {
				return fail(err.status, { error: err.message || 'Failed to create block' });
			}
			throw err;
		}
	},

	deleteEvent: async ({ request, cookies }) => {
		const token = cookies.get('token');
		if (!token) {
			return fail(401, { error: 'Not authenticated' });
		}

		const formData = await request.formData();
		const eventId = formData.get('event_id')?.toString();

		if (!eventId) {
			return fail(400, { error: 'Event ID is required' });
		}

		try {
			await api.delete(`/calendar/events/${eventId}`, token);
			return { success: true };
		} catch (err) {
			if (err instanceof ApiError) {
				return fail(err.status, { error: err.message || 'Failed to delete event' });
			}
			throw err;
		}
	},

	createRecurringBlock: async ({ request, cookies }) => {
		const token = cookies.get('token');
		if (!token) {
			return fail(401, { error: 'Not authenticated' });
		}

		const formData = await request.formData();
		const title = formData.get('title')?.toString() || '';
		const startTime = formData.get('start_time')?.toString(); // HH:MM format
		const endTime = formData.get('end_time')?.toString(); // HH:MM format
		const isBlocking = formData.get('is_blocking') === 'true';
		const daysOfWeek = formData.getAll('days_of_week').map(d => parseInt(d.toString()));
		const weeksAhead = parseInt(formData.get('weeks_ahead')?.toString() || '52');
		const isIndefinite = formData.get('indefinite') === 'true';

		if (!startTime || !endTime) {
			return fail(400, { error: 'Start and end time are required' });
		}

		if (daysOfWeek.length === 0) {
			return fail(400, { error: 'Select at least one day of the week' });
		}

		try {
			// Generate dates for the next N weeks
			const today = new Date();
			const createdBlocks: string[] = [];
			const errors: string[] = [];

			// Build recurrence rule with indefinite flag
			const recurrenceRule = isIndefinite
				? `WEEKLY:${daysOfWeek.join(',')}:INDEFINITE`
				: `WEEKLY:${daysOfWeek.join(',')}:${weeksAhead}`;

			for (let week = 0; week < weeksAhead; week++) {
				for (const dayOfWeek of daysOfWeek) {
					// Calculate the date for this day of week
					const date = new Date(today);
					const currentDay = date.getDay();
					const daysUntil = (dayOfWeek - currentDay + 7) % 7 + (week * 7);
					date.setDate(date.getDate() + daysUntil);

					// Skip if date is in the past
					if (date < today && week === 0) continue;

					// Create full datetime strings
					const dateStr = date.toISOString().split('T')[0];
					const blockStart = `${dateStr}T${startTime}:00`;
					const blockEnd = `${dateStr}T${endTime}:00`;

					try {
						await api.post(
							'/calendar/events',
							{
								title: title || null,
								start_time: blockStart,
								end_time: blockEnd,
								event_type: 'block',
								is_blocking: isBlocking,
								recurrence_rule: recurrenceRule
							},
							token
						);
						createdBlocks.push(dateStr);
					} catch {
						errors.push(dateStr);
					}
				}
			}

			return {
				success: true,
				created: createdBlocks.length,
				errors: errors.length,
				indefinite: isIndefinite
			};
		} catch (err) {
			if (err instanceof ApiError) {
				return fail(err.status, { error: err.message || 'Failed to create recurring blocks' });
			}
			throw err;
		}
	}
};
