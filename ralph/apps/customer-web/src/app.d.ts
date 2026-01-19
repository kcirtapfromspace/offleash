// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces
import type { Branding } from '$lib/stores/branding';

declare global {
	namespace App {
		// interface Error {}
		// interface Locals {}
		interface PageData {
			branding: Branding;
		}
		// interface PageState {}
		// interface Platform {}
	}
}

export {};
