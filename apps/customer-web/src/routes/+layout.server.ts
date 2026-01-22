import type { LayoutServerLoad } from './$types';
import type { Branding } from '$lib/stores/branding';
import { PUBLIC_API_URL } from '$env/static/public';

export const load: LayoutServerLoad = async ({ fetch, request }) => {
	// Extract tenant from host header
	const host = request.headers.get('host') || '';

	try {
		// Fetch branding from backend API
		const response = await fetch(`${PUBLIC_API_URL}/api/branding`, {
			headers: {
				Host: host
			}
		});

		if (response.ok) {
			const branding: Branding = await response.json();
			return { branding };
		}
	} catch {
		// Branding fetch failed, continue with defaults
	}

	// Return default branding if API call fails
	return {
		branding: {
			companyName: 'OFFLEASH',
			primaryColor: '#3b82f6',
			secondaryColor: '#6b7280',
			accentColor: '#10b981'
		} as Branding
	};
};
