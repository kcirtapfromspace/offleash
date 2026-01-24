import { dev } from '$app/environment';
import type { Cookies } from '@sveltejs/kit';

/**
 * Get the cookie domain for sharing across subdomains.
 * In production, returns '.offleash.world' to share cookies across:
 * - offleash.world (customer)
 * - paperwork.offleash.world (admin)
 * - platform.offleash.world (platform)
 * - api.offleash.world (api)
 *
 * In development, returns undefined (uses current host).
 */
export function getCookieDomain(host: string): string | undefined {
	if (dev) {
		return undefined;
	}

	// Extract base domain from host (e.g., 'offleash.world' from 'paperwork.offleash.world')
	const hostname = host.split(':')[0]; // Remove port if present
	const parts = hostname.split('.');

	// If it's a subdomain of offleash.world, return .offleash.world
	if (parts.length >= 2) {
		const baseDomain = parts.slice(-2).join('.');
		if (baseDomain === 'offleash.world' || baseDomain === 'offleash.pro') {
			return `.${baseDomain}`;
		}
	}

	return undefined;
}

/**
 * Standard cookie options for auth tokens and user data.
 * Sets domain to share across subdomains in production.
 */
export function getAuthCookieOptions(host: string, httpOnly: boolean = true) {
	const domain = getCookieDomain(host);

	return {
		path: '/',
		httpOnly,
		secure: !dev,
		sameSite: 'lax' as const,
		maxAge: 60 * 60 * 24 * 7, // 7 days
		...(domain && { domain })
	};
}

/**
 * Set an auth cookie with proper domain for cross-subdomain sharing.
 */
export function setAuthCookie(
	cookies: Cookies,
	name: string,
	value: string,
	host: string,
	httpOnly: boolean = true
) {
	cookies.set(name, value, getAuthCookieOptions(host, httpOnly));
}

/**
 * Delete an auth cookie from all subdomains.
 */
export function deleteAuthCookie(cookies: Cookies, name: string, host: string) {
	const domain = getCookieDomain(host);
	cookies.delete(name, {
		path: '/',
		...(domain && { domain })
	});
}
