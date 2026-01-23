import type { PageServerLoad, Actions } from './$types';
import { api, ApiError } from '$lib/api';
import { fail, redirect } from '@sveltejs/kit';
import { randomUUID } from 'crypto';

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
	address: string;
	city: string;
	state: string;
	zip_code: string;
	full_address: string;
	latitude: number;
	longitude: number;
	notes: string | null;
	is_default: boolean;
}

interface Walker {
	id: string;
	first_name: string;
	last_name: string;
	email: string;
}

interface TimeSlot {
	start: string;
	end: string;
	confidence?: string;
	travel_minutes?: number | null;
	travel_from?: string | null;
	is_tight?: boolean;
	warning?: string | null;
}

interface AvailabilityResponse {
	walker_id: string;
	date: string;
	service_id: string;
	slots: TimeSlot[];
}

// Enhanced availability response from /availability/slots endpoint
interface EnhancedAvailabilityResponse {
	date: string;
	walker_id: string;
	walker_name: string;
	slots: {
		start_time: string;
		end_time: string;
		travel_minutes: number | null;
		travel_from: string | null;
		is_tight: boolean;
		warning: string | null;
	}[];
	travel_buffer_minutes: number;
}

export const load: PageServerLoad = async ({ parent, url }) => {
	const { token } = await parent();
	const serviceId = url.searchParams.get('service');
	const selectedDate = url.searchParams.get('date');
	const selectedLocationId = url.searchParams.get('location');

	try {
		// Fetch services and locations in parallel
		const [services, locations] = await Promise.all([
			api.get<Service[]>('/services', token),
			api.get<Location[]>('/locations', token)
		]);

		const activeServices = services.filter((s) => s.is_active);
		const selectedService = serviceId ? activeServices.find((s) => s.id === serviceId) : null;

		// Fetch walkers (users with role=walker) - we'll need to add this endpoint or use a workaround
		// For now, we'll fetch availability which implicitly tells us which walkers are available
		let availability: {
			walkerId: string;
			walkerName: string;
			slots: TimeSlot[];
			travelBufferMinutes?: number;
		}[] = [];

		if (selectedService && selectedDate && selectedLocationId) {
			// Try the enhanced availability/slots endpoint first (with travel time info)
			// We need to fetch for each walker - for now, use a known walker ID
			const walkerIds = ['b376c762-b772-4fde-963e-5dcaedd52626'];

			for (const walkerId of walkerIds) {
				try {
					// Try enhanced endpoint with travel time
					const enhancedResponse = await api.get<EnhancedAvailabilityResponse>(
						`/availability/slots?walker_id=${walkerId}&date=${selectedDate}&service_id=${serviceId}&location_id=${selectedLocationId}`,
						token
					);

					if (enhancedResponse.slots.length > 0) {
						availability.push({
							walkerId: enhancedResponse.walker_id,
							walkerName: enhancedResponse.walker_name,
							travelBufferMinutes: enhancedResponse.travel_buffer_minutes,
							slots: enhancedResponse.slots.map(slot => ({
								start: slot.start_time,
								end: slot.end_time,
								travel_minutes: slot.travel_minutes,
								travel_from: slot.travel_from,
								is_tight: slot.is_tight,
								warning: slot.warning
							}))
						});
					}
				} catch (enhancedError) {
					// Fallback to basic availability endpoint
					console.warn('Enhanced availability not available, falling back:', enhancedError);
					try {
						const availResponse = await api.get<AvailabilityResponse>(
							`/availability/${walkerId}?date=${selectedDate}&service_id=${serviceId}&location_id=${selectedLocationId}`,
							token
						);
						if (availResponse.slots.length > 0) {
							availability.push({
								walkerId: availResponse.walker_id,
								walkerName: 'Alex Walker',
								slots: availResponse.slots
							});
						}
					} catch (e) {
						console.error('Failed to fetch availability:', e);
					}
				}
			}
		}

		return {
			services: activeServices,
			locations,
			selectedServiceId: serviceId,
			selectedService,
			selectedDate,
			selectedLocationId,
			availability
		};
	} catch (error) {
		console.error('Failed to load booking data:', error);
		return {
			services: [],
			locations: [],
			selectedServiceId: null,
			selectedService: null,
			selectedDate: null,
			selectedLocationId: null,
			availability: [],
			error: 'Failed to load booking data'
		};
	}
};

interface RecurringConflict {
	date: string;
	reason: string;
}

interface RecurringResponse {
	series: { id: string } | null;
	bookings_created: number;
	total_planned: number;
	conflicts: RecurringConflict[];
	preview_dates: string[];
}

export const actions: Actions = {
	book: async ({ request, cookies }) => {
		const token = cookies.get('token');
		if (!token) {
			throw redirect(303, '/login');
		}

		const formData = await request.formData();
		const serviceId = formData.get('service_id')?.toString();
		const locationId = formData.get('location_id')?.toString();
		const walkerId = formData.get('walker_id')?.toString();
		const startTime = formData.get('start_time')?.toString();
		const notes = formData.get('notes')?.toString();
		const isRecurring = formData.get('is_recurring') === 'on';

		if (!serviceId || !locationId || !walkerId || !startTime) {
			return fail(400, { error: 'Please complete all required fields' });
		}

		// Handle recurring bookings
		if (isRecurring) {
			const frequency = formData.get('recurring_frequency')?.toString() || 'weekly';
			const endConditionType = formData.get('end_condition_type')?.toString() || 'occurrences';
			const endOccurrences = parseInt(formData.get('end_occurrences')?.toString() || '12');
			const endDate = formData.get('end_date')?.toString();

			// Extract date and time from startTime (ISO format)
			const startDateTime = new Date(startTime);
			const startDateStr = startDateTime.toISOString().split('T')[0];
			const timeOfDay = startDateTime.toTimeString().slice(0, 5); // HH:MM

			const endCondition =
				endConditionType === 'occurrences'
					? { type: 'occurrences' as const, value: endOccurrences }
					: { type: 'date' as const, value: endDate };

			// Generate idempotency key to prevent duplicate submissions
			const idempotencyKey = randomUUID();

			try {
				const result = await api.post<RecurringResponse>(
					'/bookings/recurring',
					{
						walker_id: walkerId,
						service_id: serviceId,
						location_id: locationId,
						frequency,
						start_date: startDateStr,
						time_of_day: timeOfDay,
						end_condition: endCondition,
						notes: notes || null,
						preview_only: false
					},
					token,
					{ 'X-Idempotency-Key': idempotencyKey }
				);

				if (result.series) {
					// Return success with conflict info for display before redirect
					if (result.conflicts.length > 0) {
						return {
							success: true,
							seriesId: result.series.id,
							bookingsCreated: result.bookings_created,
							totalPlanned: result.total_planned,
							conflicts: result.conflicts,
							message: `Created ${result.bookings_created} of ${result.total_planned} bookings. ${result.conflicts.length} dates had conflicts.`
						};
					}
					throw redirect(303, `/bookings/recurring/${result.series.id}`);
				} else {
					return fail(400, { error: 'Failed to create recurring series' });
				}
			} catch (err) {
				if (err instanceof ApiError) {
					return fail(err.status, {
						error: err.message || 'Failed to create recurring booking',
						errorType: 'api_error'
					});
				}
				throw err;
			}
		}

		// Handle single booking
		try {
			const booking = await api.post<{ id: string }>(
				'/bookings',
				{
					service_id: serviceId,
					location_id: locationId,
					walker_id: walkerId,
					start_time: startTime,
					notes: notes || null
				},
				token
			);

			throw redirect(303, `/bookings/${booking.id}`);
		} catch (err) {
			if (err instanceof ApiError) {
				return fail(err.status, { error: err.message || 'Failed to create booking' });
			}
			throw err;
		}
	}
};
