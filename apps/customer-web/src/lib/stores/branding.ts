import { writable } from 'svelte/store';

export interface Branding {
	companyName: string;
	logoUrl?: string;
	primaryColor: string;
	secondaryColor: string;
	accentColor: string;
	supportEmail?: string;
	supportPhone?: string;
}

export const branding = writable<Branding | null>(null);
