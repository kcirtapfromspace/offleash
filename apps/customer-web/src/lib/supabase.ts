import { createBrowserClient, createServerClient } from '@supabase/ssr';
import { env } from '$env/dynamic/public';
import type { Cookies } from '@sveltejs/kit';

// Supabase is optional - if not configured, OAuth buttons won't work
const SUPABASE_URL = env.PUBLIC_SUPABASE_URL || '';
const SUPABASE_ANON_KEY = env.PUBLIC_SUPABASE_ANON_KEY || '';

export const isSupabaseConfigured = Boolean(SUPABASE_URL && SUPABASE_ANON_KEY);

// Browser client for client-side operations
export function createSupabaseBrowserClient() {
	if (!isSupabaseConfigured) {
		throw new Error('Supabase is not configured. Set PUBLIC_SUPABASE_URL and PUBLIC_SUPABASE_ANON_KEY.');
	}
	return createBrowserClient(SUPABASE_URL, SUPABASE_ANON_KEY);
}

// Server client for server-side operations (load functions, actions)
export function createSupabaseServerClient(cookies: Cookies) {
	if (!isSupabaseConfigured) {
		throw new Error('Supabase is not configured. Set PUBLIC_SUPABASE_URL and PUBLIC_SUPABASE_ANON_KEY.');
	}
	return createServerClient(SUPABASE_URL, SUPABASE_ANON_KEY, {
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
