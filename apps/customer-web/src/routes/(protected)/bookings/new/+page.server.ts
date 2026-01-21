import type { PageServerLoad, Actions } from './$types';
import { api, ApiError } from '$lib/api';
import { fail, redirect } from '@sveltejs/kit';

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
	confidence: string;
}

interface AvailabilityResponse {
	walker_id: string;
	date: string;
	service_id: string;
	slots: TimeSlot[];
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
		let availability: { walkerId: string; walkerName: string; slots: TimeSlot[] }[] = [];

		if (selectedService && selectedDate && selectedLocationId) {
			// We need to get available walkers - let's try to fetch from a walkers endpoint
			// For demo purposes, we'll use the walker we created
			try {
				const availResponse = await api.get<AvailabilityResponse>(
					`/availability/b376c762-b772-4fde-963e-5dcaedd52626?date=${selectedDate}&service_id=${serviceId}&location_id=${selectedLocationId}`,
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

		if (!serviceId || !locationId || !walkerId || !startTime) {
			return fail(400, { error: 'Please complete all required fields' });
		}

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
