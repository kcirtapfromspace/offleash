import { redirect } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { createSupabaseServerClient } from '$lib/supabase';

export const GET: RequestHandler = async ({ url, cookies }) => {
	const code = url.searchParams.get('code');
	const next = url.searchParams.get('next') ?? '/services';

	if (code) {
		const supabase = createSupabaseServerClient(cookies);
		const { error } = await supabase.auth.exchangeCodeForSession(code);

		if (!error) {
			// Get the session to extract user info
			const { data: { session } } = await supabase.auth.getSession();

			if (session) {
				// Store the Supabase access token as our app token
				// This will be used by the API for authentication
				cookies.set('token', session.access_token, {
					path: '/',
					httpOnly: true,
					secure: true,
					sameSite: 'lax',
					maxAge: 60 * 60 * 24 * 7 // 7 days
				});

				// Also store refresh token for session refresh
				if (session.refresh_token) {
					cookies.set('refresh_token', session.refresh_token, {
						path: '/',
						httpOnly: true,
						secure: true,
						sameSite: 'lax',
						maxAge: 60 * 60 * 24 * 30 // 30 days
					});
				}
			}

			throw redirect(303, next);
		}
	}

	// OAuth error - redirect to login with error
	throw redirect(303, '/login?error=oauth_failed');
};
