import { writable } from 'svelte/store';

export interface Branding {
	company_name: string;
	logo_url: string;
	primary_color: string;
	secondary_color: string;
	accent_color: string;
	support_email: string;
}

export const defaultBranding: Branding = {
	company_name: 'Offleash',
	logo_url: '',
	primary_color: '#3B82F6',
	secondary_color: '#1E40AF',
	accent_color: '#10B981',
	support_email: 'support@offleash.com'
};

export const branding = writable<Branding>(defaultBranding);

export function generateCssVariables(brandingData: Branding): string {
	return `
		--color-primary: ${brandingData.primary_color};
		--color-secondary: ${brandingData.secondary_color};
		--color-accent: ${brandingData.accent_color};
	`.trim();
}
