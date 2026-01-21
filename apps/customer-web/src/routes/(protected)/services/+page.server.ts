import type { PageServerLoad } from './$types';
import { api } from '$lib/api';

interface Service {
	id: string;
	name: string;
	description: string | null;
	duration_minutes: number;
	price_cents: number;
	price_display: string;
	is_active: boolean;
}

export const load: PageServerLoad = async ({ parent }) => {
	const { token } = await parent();

	try {
		const services = await api.get<Service[]>('/services', token);
		return {
			services: services.filter((s) => s.is_active)
		};
	} catch (error) {
		console.error('Failed to fetch services:', error);
		return {
			services: [],
			error: 'Failed to load services'
		};
	}
};
