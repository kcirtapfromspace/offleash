import { redirect } from '@sveltejs/kit';
import type { Actions } from './$types';

export const actions: Actions = {
	default: async ({ cookies }) => {
		cookies.delete('token', { path: '/' });
		cookies.delete('user', { path: '/' });
		cookies.delete('membership', { path: '/' });
		cookies.delete('memberships', { path: '/' });
		throw redirect(303, '/login');
	}
};
