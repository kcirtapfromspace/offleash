import type { PageServerLoad, Actions } from './$types';
import { env } from '$env/dynamic/private';
import { fail } from '@sveltejs/kit';

const API_URL = env.API_URL || 'http://localhost:3000';

interface Pet {
	id: string;
	name: string;
	species: string;
	breed: string | null;
	date_of_birth: string | null;
	age_years: number | null;
	weight_lbs: number | null;
	gender: string | null;
	color: string | null;
	microchip_id: string | null;
	is_spayed_neutered: boolean | null;
	vaccination_status: string | null;
	temperament: string | null;
	special_needs: string | null;
	emergency_contact_name: string | null;
	emergency_contact_phone: string | null;
	vet_name: string | null;
	vet_phone: string | null;
	photo_url: string | null;
	notes: string | null;
	created_at: string;
}

interface User {
	id: string;
	email: string;
	first_name: string;
	last_name: string;
	phone: string | null;
}

export const load: PageServerLoad = async ({ cookies, locals }) => {
	const token = cookies.get('token');

	if (!token) {
		return { pets: [], user: null };
	}

	try {
		const [petsResponse, userResponse] = await Promise.all([
			fetch(`${API_URL}/pets`, {
				headers: { Authorization: `Bearer ${token}` },
			}),
			fetch(`${API_URL}/users/me`, {
				headers: { Authorization: `Bearer ${token}` },
			}),
		]);

		const pets = petsResponse.ok ? (await petsResponse.json()) as Pet[] : [];
		const user = userResponse.ok ? (await userResponse.json()) as User : null;

		return { pets, user };
	} catch (error) {
		console.error('Error loading profile:', error);
		return { pets: [], user: null };
	}
};

export const actions: Actions = {
	updateProfile: async ({ request, cookies }) => {
		const token = cookies.get('token');
		if (!token) {
			return fail(401, { error: 'Not authenticated' });
		}

		const formData = await request.formData();
		const first_name = formData.get('first_name')?.toString();
		const last_name = formData.get('last_name')?.toString();
		const phone = formData.get('phone')?.toString() || null;

		if (!first_name || !last_name) {
			return fail(400, { error: 'First and last name are required' });
		}

		try {
			const response = await fetch(`${API_URL}/users/me`, {
				method: 'PUT',
				headers: {
					'Authorization': `Bearer ${token}`,
					'Content-Type': 'application/json',
				},
				body: JSON.stringify({ first_name, last_name, phone }),
			});

			if (!response.ok) {
				const error = await response.json().catch(() => ({ message: 'Failed to update profile' }));
				return fail(400, { error: error.message || 'Failed to update profile' });
			}

			return { success: true, message: 'Profile updated successfully' };
		} catch (error) {
			console.error('Error updating profile:', error);
			return fail(500, { error: 'Server error' });
		}
	},

	createPet: async ({ request, cookies }) => {
		const token = cookies.get('token');
		if (!token) {
			return fail(401, { error: 'Not authenticated' });
		}

		const formData = await request.formData();
		const name = formData.get('name')?.toString();
		const breed = formData.get('breed')?.toString() || null;
		const date_of_birth = formData.get('date_of_birth')?.toString() || null;
		const weight_lbs = formData.get('weight_lbs')?.toString();
		const gender = formData.get('gender')?.toString() || null;
		const color = formData.get('color')?.toString() || null;
		const is_spayed_neutered = formData.get('is_spayed_neutered') === 'true';
		const temperament = formData.get('temperament')?.toString() || null;
		const special_needs = formData.get('special_needs')?.toString() || null;
		const vet_name = formData.get('vet_name')?.toString() || null;
		const vet_phone = formData.get('vet_phone')?.toString() || null;
		const notes = formData.get('notes')?.toString() || null;

		if (!name) {
			return fail(400, { error: 'Pet name is required' });
		}

		try {
			const response = await fetch(`${API_URL}/pets`, {
				method: 'POST',
				headers: {
					'Authorization': `Bearer ${token}`,
					'Content-Type': 'application/json',
				},
				body: JSON.stringify({
					name,
					species: 'dog',
					breed,
					date_of_birth,
					weight_lbs: weight_lbs ? parseFloat(weight_lbs) : null,
					gender,
					color,
					is_spayed_neutered,
					temperament,
					special_needs,
					vet_name,
					vet_phone,
					notes,
				}),
			});

			if (!response.ok) {
				const error = await response.json().catch(() => ({ message: 'Failed to add pet' }));
				return fail(400, { error: error.message || 'Failed to add pet' });
			}

			return { success: true, message: 'Pet added successfully' };
		} catch (error) {
			console.error('Error creating pet:', error);
			return fail(500, { error: 'Server error' });
		}
	},

	updatePet: async ({ request, cookies }) => {
		const token = cookies.get('token');
		if (!token) {
			return fail(401, { error: 'Not authenticated' });
		}

		const formData = await request.formData();
		const petId = formData.get('petId')?.toString();
		const name = formData.get('name')?.toString();
		const breed = formData.get('breed')?.toString() || null;
		const date_of_birth = formData.get('date_of_birth')?.toString() || null;
		const weight_lbs = formData.get('weight_lbs')?.toString();
		const gender = formData.get('gender')?.toString() || null;
		const color = formData.get('color')?.toString() || null;
		const is_spayed_neutered = formData.get('is_spayed_neutered') === 'true';
		const temperament = formData.get('temperament')?.toString() || null;
		const special_needs = formData.get('special_needs')?.toString() || null;
		const vet_name = formData.get('vet_name')?.toString() || null;
		const vet_phone = formData.get('vet_phone')?.toString() || null;
		const notes = formData.get('notes')?.toString() || null;

		if (!petId || !name) {
			return fail(400, { error: 'Pet ID and name are required' });
		}

		try {
			const response = await fetch(`${API_URL}/pets/${petId}`, {
				method: 'PUT',
				headers: {
					'Authorization': `Bearer ${token}`,
					'Content-Type': 'application/json',
				},
				body: JSON.stringify({
					name,
					breed,
					date_of_birth,
					weight_lbs: weight_lbs ? parseFloat(weight_lbs) : null,
					gender,
					color,
					is_spayed_neutered,
					temperament,
					special_needs,
					vet_name,
					vet_phone,
					notes,
				}),
			});

			if (!response.ok) {
				const error = await response.json().catch(() => ({ message: 'Failed to update pet' }));
				return fail(400, { error: error.message || 'Failed to update pet' });
			}

			return { success: true, message: 'Pet updated successfully' };
		} catch (error) {
			console.error('Error updating pet:', error);
			return fail(500, { error: 'Server error' });
		}
	},

	deletePet: async ({ request, cookies }) => {
		const token = cookies.get('token');
		if (!token) {
			return fail(401, { error: 'Not authenticated' });
		}

		const formData = await request.formData();
		const petId = formData.get('petId')?.toString();

		if (!petId) {
			return fail(400, { error: 'Pet ID is required' });
		}

		try {
			const response = await fetch(`${API_URL}/pets/${petId}`, {
				method: 'DELETE',
				headers: {
					Authorization: `Bearer ${token}`,
				},
			});

			if (!response.ok) {
				const error = await response.json().catch(() => ({ message: 'Failed to delete pet' }));
				return fail(400, { error: error.message || 'Failed to delete pet' });
			}

			return { success: true, message: 'Pet removed successfully' };
		} catch (error) {
			console.error('Error deleting pet:', error);
			return fail(500, { error: 'Server error' });
		}
	},
};
