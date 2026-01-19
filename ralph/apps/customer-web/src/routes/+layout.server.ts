import type { LayoutServerLoad } from './$types';
import { defaultBranding, type Branding } from '$lib/stores/branding';

export const load: LayoutServerLoad = async ({ fetch }) => {
	let branding: Branding = defaultBranding;

	try {
		const response = await fetch('/api/branding');
		if (response.ok) {
			branding = await response.json();
		}
	} catch {
		// Use default branding if API is unavailable
	}

	return {
		branding
	};
};
