import { redirect, fail } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { api, ApiError } from '$lib/api';
import { setAuthCookie } from '$lib/cookies';
import { env } from '$env/dynamic/public';

interface MembershipInfo {
	id: string;
	organization_id: string;
	organization_name: string;
	organization_slug: string;
	role: string;
	is_default: boolean;
}

interface LoginResponse {
	token: string;
	user: {
		id: string;
		email: string;
		first_name: string;
		last_name: string;
		role: string;
	};
	membership?: MembershipInfo;
	memberships?: MembershipInfo[];
}

// Extract org slug from hostname (subdomain-based routing)
function getOrgSlugFromHost(host: string): string | null {
	const hostname = host.split(':')[0];
	const parts = hostname.split('.');
	if (parts.length >= 3) {
		const subdomain = parts[0];
		if (!['www', 'app', 'admin', 'platform', 'api'].includes(subdomain)) {
			return subdomain;
		}
	}
	return null;
}

// Trusted domains for external redirects (cross-app auth)
const TRUSTED_DOMAINS = [
	'paperwork.offleash.world',
	'platform.offleash.world',
	'offleash.world',
	'localhost'
];

// Validate if a redirect URL is to a trusted domain
function isTrustedRedirect(redirectUrl: string, currentOrigin: string): boolean {
	// Allow relative paths
	if (redirectUrl.startsWith('/')) {
		return true;
	}

	// Check if it's an absolute URL to a trusted domain
	try {
		const url = new URL(redirectUrl);
		return TRUSTED_DOMAINS.some(domain =>
			url.hostname === domain || url.hostname.endsWith('.' + domain)
		);
	} catch {
		// If URL parsing fails, treat as relative path
		return redirectUrl.startsWith('/');
	}
}

export const load: PageServerLoad = async ({ cookies, url }) => {
	const token = cookies.get('token');
	if (token) {
		// If authenticated user wants to become a walker, send them to walker onboarding
		const role = url.searchParams.get('role');
		const redirectTo = url.searchParams.get('redirect');

		if (role === 'walker') {
			// Check if user already has a walker membership
			const membershipsStr = cookies.get('memberships');
			if (membershipsStr) {
				try {
					const memberships = JSON.parse(membershipsStr) as MembershipInfo[];
					const hasWalkerMembership = memberships.some(
						(m) => m.role === 'walker' || m.role === 'owner' || m.role === 'admin'
					);
					if (!hasWalkerMembership) {
						// Customer wants to become a walker - go to walker onboarding
						throw redirect(303, '/onboarding/walker');
					}
				} catch {
					// If parsing fails, go to walker onboarding to be safe
					throw redirect(303, '/onboarding/walker');
				}
			} else {
				// No memberships cookie - go to walker onboarding
				throw redirect(303, '/onboarding/walker');
			}
		}

		// For other cases, respect the redirect param or go to services
		if (redirectTo) {
			if (isTrustedRedirect(redirectTo, url.origin)) {
				throw redirect(303, redirectTo);
			}
		}
		throw redirect(303, '/services');
	}
	// Pass role and redirect params to the page
	return {
		role: url.searchParams.get('role'),
		redirectTo: url.searchParams.get('redirect')
	};
};

export const actions: Actions = {
	default: async ({ request, cookies, url }) => {
		const data = await request.formData();
		const email = data.get('email')?.toString();
		const password = data.get('password')?.toString();
		const selectedRole = url.searchParams.get('role');
		const redirectTo = url.searchParams.get('redirect');

		if (!email || !password) {
			return fail(400, { error: 'Email and password are required', email });
		}

		try {
			// Use universal login (no org_slug required)
			const response = await api.post<LoginResponse>('/auth/login/universal', {
				email,
				password
			});

			const host = request.headers.get('host') || '';

			// Detect subdomain for auto-context selection
			const subdomainSlug = getOrgSlugFromHost(host);

			// Find membership matching subdomain (if on a business subdomain)
			let activeMembership = response.membership;
			if (subdomainSlug && response.memberships) {
				const subdomainMembership = response.memberships.find(
					(m) => m.organization_slug === subdomainSlug
				);
				if (subdomainMembership) {
					activeMembership = subdomainMembership;
				}
			}

			// If no active membership from subdomain, prefer customer role for customer-web
			if (!activeMembership && response.memberships && response.memberships.length > 0) {
				// First try to find a customer membership (preferred for customer-web)
				activeMembership = response.memberships.find(m => m.role === 'customer');
				// Fall back to default or first if no customer membership exists
				if (!activeMembership) {
					activeMembership = response.memberships.find(m => m.is_default) || response.memberships[0];
				}
			}

			// Switch context to get a token with org_id
			let finalToken = response.token;
			if (activeMembership) {
				try {
					console.log('Switching context to membership:', activeMembership.id);
					const switchResponse = await api.post<{ token: string; membership: MembershipInfo }>(
						'/contexts/switch',
						{ membership_id: activeMembership.id },
						response.token
					);
					finalToken = switchResponse.token;
					// Mark that token has org_id
					setAuthCookie(cookies, 'token_has_org_id', 'true', host, false);
					console.log('Context switch successful, got new token with org_id');
				} catch (switchErr) {
					// If context switch fails, continue with original token
					console.error('Failed to switch context after login:', switchErr);
					// Log more details
					if (switchErr instanceof Error) {
						console.error('Error details:', switchErr.message);
					}
				}
			} else {
				console.warn('No active membership found, cannot switch context');
			}

			// Store token (shared across subdomains)
			setAuthCookie(cookies, 'token', finalToken, host, true);

			// Store user info (shared across subdomains)
			setAuthCookie(cookies, 'user', JSON.stringify(response.user), host, false);

			// Store current membership (shared across subdomains)
			if (activeMembership) {
				setAuthCookie(cookies, 'membership', JSON.stringify(activeMembership), host, false);
			}

			// Store all memberships for context switching (shared across subdomains)
			if (response.memberships && response.memberships.length > 0) {
				setAuthCookie(cookies, 'memberships', JSON.stringify(response.memberships), host, false);
			}

			// Determine redirect destination
			let destination = '/services';

			// If user ended up in an admin role (no customer memberships), redirect to admin app
			if (activeMembership && ['walker', 'owner', 'admin'].includes(activeMembership.role)) {
				const adminUrl = env.PUBLIC_ADMIN_URL || 'https://paperwork.offleash.world';
				throw redirect(303, adminUrl);
			}

			if (selectedRole === 'walker') {
				// Check if user has any walker/owner/admin memberships
				const hasWalkerMembership = response.memberships?.some(
					(m) => m.role === 'walker' || m.role === 'owner' || m.role === 'admin'
				);

				if (!hasWalkerMembership) {
					// New walker with no business - go to onboarding
					destination = '/onboarding/walker';
				}
			} else if (redirectTo && isTrustedRedirect(redirectTo, url.origin)) {
				// Use provided redirect if it's valid (internal path or trusted external domain)
				destination = redirectTo;
			}

			throw redirect(303, destination);
		} catch (err) {
			if (err instanceof ApiError) {
				if (err.status === 401) {
					return fail(401, { error: 'Invalid email or password', email });
				}
				return fail(err.status, {
					error: err.message || 'An error occurred. Please try again.',
					email
				});
			}
			// Re-throw redirects
			throw err;
		}
	}
};
