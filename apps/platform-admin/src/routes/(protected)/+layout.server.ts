import { redirect } from '@sveltejs/kit';
import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async ({ cookies, url }) => {
	const token = cookies.get('platform_token');

	if (!token) {
		throw redirect(303, `/login?redirect=${encodeURIComponent(url.pathname)}`);
	}

	return {
		token
	};
};
