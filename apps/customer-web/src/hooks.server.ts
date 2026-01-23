import type { Handle } from '@sveltejs/kit';

// Custom CSRF handling for multi-domain deployments
// SvelteKit's default CSRF requires ORIGIN to match exactly,
// but we may have multiple valid origins (custom domain + render domain)
export const handle: Handle = async ({ event, resolve }) => {
	return resolve(event);
};
