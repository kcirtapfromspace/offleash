import type { LayoutServerLoad } from './$types';
import type { Branding } from '$lib/stores/branding';

export const load: LayoutServerLoad = async ({ fetch, request }) => {
	// Extract tenant from host header
	const host = request.headers.get('host') || '';

	try {
		// Fetch branding from API
		const response = await fetch('/api/branding', {
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
