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

interface WalkerProfile {
	id: string;
	user_id: string;
	bio: string | null;
	profile_photo_url: string | null;
	emergency_contact_name: string | null;
	emergency_contact_phone: string | null;
	emergency_contact_relationship: string | null;
	years_experience: number;
	specializations: Specialization[];
	created_at: string;
	updated_at: string;
}

interface Specialization {
	specialization: string;
	display_name: string;
	certified: boolean;
	certification_date: string | null;
	certification_expiry: string | null;
}

interface SpecializationOption {
	value: string;
	display_name: string;
}

interface PolygonPoint {
	lat: number;
	lng: number;
}

interface ServiceArea {
	id: string;
	walker_id: string;
	name: string;
	color: string;
	polygon: PolygonPoint[];
	is_active: boolean;
	priority: number;
	price_adjustment_percent: number;
	notes: string | null;
	created_at: string;
	updated_at: string;
}

export const load: PageServerLoad = async ({ params, parent }) => {
	const { token } = await parent();
	const walkerId = params.id;

	try {
		// Fetch walker user info
		const walker = await api.get<User>(`/users/${walkerId}`, token);

		if (walker.role !== 'walker') {
			throw redirect(302, '/walkers');
		}

		// Fetch working hours
		let working_hours: WorkingHours[] = [];
		try {
			working_hours = await api.get<WorkingHours[]>(`/working-hours/${walkerId}`, token);
		} catch {
			// Walker may not have working hours set yet
		}

		// Fetch profile
		let profile: WalkerProfile | null = null;
		try {
			profile = await api.get<WalkerProfile>(`/admin/walkers/${walkerId}/profile`, token);
		} catch {
			// Profile may not exist yet
		}

		// Fetch service areas
		let service_areas: ServiceArea[] = [];
		try {
			service_areas = await api.get<ServiceArea[]>(`/admin/walkers/${walkerId}/service-areas`, token);
		} catch {
			// No service areas yet
		}

		// Fetch available specializations
		let specialization_options: SpecializationOption[] = [];
		try {
			specialization_options = await api.get<SpecializationOption[]>('/walker/specializations', token);
		} catch {
			// Use default list
			specialization_options = [
				{ value: 'puppies', display_name: 'Puppies' },
				{ value: 'seniordogs', display_name: 'Senior Dogs' },
				{ value: 'largebreeds', display_name: 'Large Breeds' },
				{ value: 'smallbreeds', display_name: 'Small Breeds' },
				{ value: 'anxiousreactive', display_name: 'Anxious/Reactive Dogs' },
				{ value: 'multipledogs', display_name: 'Multiple Dogs' },
				{ value: 'petfirstaid', display_name: 'Pet First Aid Certified' },
				{ value: 'dogtraining', display_name: 'Dog Training' },
				{ value: 'catcare', display_name: 'Cat Care' },
				{ value: 'medicationadministration', display_name: 'Medication Administration' }
			];
		}

		return {
			walker,
			working_hours,
			profile,
			service_areas,
			specialization_options
		};
	} catch (error) {
		if (error instanceof Response) throw error;
		console.error('Failed to fetch walker:', error);
		throw redirect(302, '/walkers');
	}
};

export const actions: Actions = {
	updateProfile: async ({ request, params, cookies }) => {
		const token = cookies.get('token');
		if (!token) {
			return fail(401, { error: 'Not authenticated' });
		}

		const formData = await request.formData();
		const bio = formData.get('bio')?.toString() || null;
		const profilePhotoUrl = formData.get('profile_photo_url')?.toString() || null;
		const emergencyContactName = formData.get('emergency_contact_name')?.toString() || null;
		const emergencyContactPhone = formData.get('emergency_contact_phone')?.toString() || null;
		const emergencyContactRelationship = formData.get('emergency_contact_relationship')?.toString() || null;
		const yearsExperience = parseInt(formData.get('years_experience')?.toString() || '0', 10);
		const specializationsJson = formData.get('specializations')?.toString();

		const specializations = specializationsJson ? JSON.parse(specializationsJson) : [];

		try {
			await api.put(
				`/admin/walkers/${params.id}/profile`,
				{
					bio,
					profile_photo_url: profilePhotoUrl,
					emergency_contact_name: emergencyContactName,
					emergency_contact_phone: emergencyContactPhone,
					emergency_contact_relationship: emergencyContactRelationship,
					years_experience: yearsExperience,
					specializations
				},
				token
			);

			return { success: true, message: 'Profile updated successfully' };
		} catch (err) {
			console.error('Failed to update profile:', err);
			if (err instanceof ApiError) {
				return fail(err.status, { error: err.message || 'Failed to update profile' });
			}
			return fail(500, { error: 'An unexpected error occurred' });
		}
	},

	updateWorkingHours: async ({ request, params, cookies }) => {
		const token = cookies.get('token');
		if (!token) {
			return fail(401, { error: 'Not authenticated' });
		}

		const formData = await request.formData();
		const scheduleJson = formData.get('schedule')?.toString();

		if (!scheduleJson) {
			return fail(400, { error: 'Missing schedule data' });
		}

		try {
			const schedule = JSON.parse(scheduleJson);
			await api.put(`/working-hours/${params.id}`, { schedule }, token);
			return { success: true, message: 'Working hours updated' };
		} catch (err) {
			if (err instanceof ApiError) {
				return fail(err.status, { error: err.message || 'Failed to update working hours' });
			}
			throw err;
		}
	},

	createServiceArea: async ({ request, params, cookies }) => {
		const token = cookies.get('token');
		if (!token) {
			return fail(401, { error: 'Not authenticated' });
		}

		const formData = await request.formData();
		const name = formData.get('name')?.toString();
		const color = formData.get('color')?.toString() || '#3B82F6';
		const polygonJson = formData.get('polygon')?.toString();
		const isActive = formData.get('is_active') === 'true';
		const priority = parseInt(formData.get('priority')?.toString() || '0', 10);
		const priceAdjustmentPercent = parseInt(formData.get('price_adjustment_percent')?.toString() || '0', 10);
		const notes = formData.get('notes')?.toString() || null;

		if (!name || !polygonJson) {
			return fail(400, { error: 'Name and polygon are required' });
		}

		try {
			const polygon = JSON.parse(polygonJson);
			if (polygon.length < 3) {
				return fail(400, { error: 'Polygon must have at least 3 points' });
			}

			await api.post(
				`/admin/walkers/${params.id}/service-areas`,
				{
					name,
					color,
					polygon,
					is_active: isActive,
					priority,
					price_adjustment_percent: priceAdjustmentPercent,
					notes
				},
				token
			);

			return { success: true, message: 'Service area created successfully' };
		} catch (err) {
			console.error('Failed to create service area:', err);
			if (err instanceof ApiError) {
				return fail(err.status, { error: err.message || 'Failed to create service area' });
			}
			return fail(500, { error: 'An unexpected error occurred' });
		}
	},

	updateServiceArea: async ({ request, cookies }) => {
		const token = cookies.get('token');
		if (!token) {
			return fail(401, { error: 'Not authenticated' });
		}

		const formData = await request.formData();
		const areaId = formData.get('area_id')?.toString();
		const name = formData.get('name')?.toString();
		const color = formData.get('color')?.toString();
		const polygonJson = formData.get('polygon')?.toString();
		const isActive = formData.get('is_active') === 'true';
		const priority = parseInt(formData.get('priority')?.toString() || '0', 10);
		const priceAdjustmentPercent = parseInt(formData.get('price_adjustment_percent')?.toString() || '0', 10);
		const notes = formData.get('notes')?.toString() || null;

		if (!areaId) {
			return fail(400, { error: 'Area ID is required' });
		}

		try {
			const polygon = polygonJson ? JSON.parse(polygonJson) : undefined;
			if (polygon && polygon.length < 3) {
				return fail(400, { error: 'Polygon must have at least 3 points' });
			}

			await api.put(
				`/admin/service-areas/${areaId}`,
				{
					name,
					color,
					polygon,
					is_active: isActive,
					priority,
					price_adjustment_percent: priceAdjustmentPercent,
					notes
				},
				token
			);

			return { success: true, message: 'Service area updated successfully' };
		} catch (err) {
			console.error('Failed to update service area:', err);
			if (err instanceof ApiError) {
				return fail(err.status, { error: err.message || 'Failed to update service area' });
			}
			return fail(500, { error: 'An unexpected error occurred' });
		}
	},

	deleteServiceArea: async ({ request, cookies }) => {
		const token = cookies.get('token');
		if (!token) {
			return fail(401, { error: 'Not authenticated' });
		}

		const formData = await request.formData();
		const areaId = formData.get('area_id')?.toString();

		if (!areaId) {
			return fail(400, { error: 'Area ID is required' });
		}

		try {
			await api.delete(`/admin/service-areas/${areaId}`, token);
			return { success: true, message: 'Service area deleted' };
		} catch (err) {
			console.error('Failed to delete service area:', err);
			if (err instanceof ApiError) {
				return fail(err.status, { error: err.message || 'Failed to delete service area' });
			}
			return fail(500, { error: 'An unexpected error occurred' });
		}
	}
};
