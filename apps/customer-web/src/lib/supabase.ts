import { createBrowserClient, createServerClient } from '@supabase/ssr';
import { PUBLIC_SUPABASE_URL, PUBLIC_SUPABASE_ANON_KEY } from '$env/static/public';
import type { Cookies } from '@sveltejs/kit';

// Browser client for client-side operations
export function createSupabaseBrowserClient() {
	return createBrowserClient(PUBLIC_SUPABASE_URL, PUBLIC_SUPABASE_ANON_KEY);
}

// Server client for server-side operations (load functions, actions)
export function createSupabaseServerClient(cookies: Cookies) {
	return createServerClient(PUBLIC_SUPABASE_URL, PUBLIC_SUPABASE_ANON_KEY, {
		cookies: {
			getAll() {
				return cookies.getAll();
			},
			setAll(cookiesToSet) {
				cookiesToSet.forEach(({ name, value, options }) => {
					cookies.set(name, value, { ...options, path: '/' });
				});
			}
		}
	});
}

// OAuth providers we support
export type OAuthProvider = 'google' | 'apple';

// Sign in with OAuth provider
export async function signInWithOAuth(provider: OAuthProvider, redirectTo: string) {
	const supabase = createSupabaseBrowserClient();

	const { data, error } = await supabase.auth.signInWithOAuth({
		provider,
		options: {
			redirectTo,
			queryParams: provider === 'google' ? {
				access_type: 'offline',
				prompt: 'consent'
			} : undefined
		}
	});

	if (error) {
		throw error;
	}

	return data;
}

// Sign out
export async function signOut() {
	const supabase = createSupabaseBrowserClient();
	const { error } = await supabase.auth.signOut();
	if (error) {
		throw error;
	}
}

// Get current session
export async function getSession() {
	const supabase = createSupabaseBrowserClient();
	const { data: { session }, error } = await supabase.auth.getSession();
	if (error) {
		throw error;
	}
	return session;
}
