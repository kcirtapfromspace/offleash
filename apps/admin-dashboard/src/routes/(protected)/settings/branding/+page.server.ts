import type { PageServerLoad, Actions } from './$types';
import { env } from '$env/dynamic/private';
import { fail, redirect } from '@sveltejs/kit';

const API_URL = env.API_URL || 'http://localhost:8080';

interface Branding {
	primary_color: string | null;
	secondary_color: string | null;
	logo_url: string | null;
	favicon_url: string | null;
	font_family: string | null;
}

export const load: PageServerLoad = async ({ cookies, parent }) => {
	const token = cookies.get('token');
	const parentData = await parent();

	// Only admins/owners can access branding
	const role = parentData.membership?.role;
	if (role !== 'admin' && role !== 'owner') {
		throw redirect(303, '/settings');
	}

	if (!token) {
		return { branding: null };
	}

	try {
		const response = await fetch(`${API_URL}/admin/branding`, {
			headers: {
				Authorization: `Bearer ${token}`
			}
		});

		if (!response.ok) {
			console.error('Failed to fetch branding:', response.status);
			return { branding: null };
		}

		const branding: Branding = await response.json();
		return { branding };
	} catch (error) {
		console.error('Error fetching branding:', error);
		return { branding: null };
	}
};

// Validate hex color format
function isValidHexColor(color: string): boolean {
	return /^#[0-9A-Fa-f]{6}$/.test(color);
}

export const actions: Actions = {
	update: async ({ request, cookies }) => {
		const token = cookies.get('token');
		if (!token) {
			return fail(401, { error: 'Not authenticated' });
		}

		const formData = await request.formData();
		const primaryColor = formData.get('primaryColor')?.toString() || '';
		const secondaryColor = formData.get('secondaryColor')?.toString() || '';
		const logoUrl = formData.get('logoUrl')?.toString() || '';
		const faviconUrl = formData.get('faviconUrl')?.toString() || '';
		const fontFamily = formData.get('fontFamily')?.toString() || '';

		// Build update payload (only include non-empty values)
		const payload: Record<string, string | null> = {};

		if (primaryColor) {
			if (!isValidHexColor(primaryColor)) {
				return fail(400, {
					error: 'Primary color must be a valid hex code (e.g., #10B981)',
					primaryColor,
					secondaryColor,
					logoUrl,
					faviconUrl,
					fontFamily
				});
			}
			payload.primary_color = primaryColor;
		}

		if (secondaryColor) {
			if (!isValidHexColor(secondaryColor)) {
				return fail(400, {
					error: 'Secondary color must be a valid hex code (e.g., #3B82F6)',
					primaryColor,
					secondaryColor,
					logoUrl,
					faviconUrl,
					fontFamily
				});
			}
			payload.secondary_color = secondaryColor;
		}

		if (logoUrl) {
			payload.logo_url = logoUrl;
		}

		if (faviconUrl) {
			payload.favicon_url = faviconUrl;
		}

		if (fontFamily) {
			payload.font_family = fontFamily;
		}

		try {
			const response = await fetch(`${API_URL}/admin/branding`, {
				method: 'PUT',
				headers: {
					Authorization: `Bearer ${token}`,
					'Content-Type': 'application/json'
				},
				body: JSON.stringify(payload)
			});

			if (!response.ok) {
				const error = await response.json().catch(() => ({ message: 'Failed to update branding' }));
				return fail(400, {
					error: error.message || 'Failed to update branding',
					primaryColor,
					secondaryColor,
					logoUrl,
					faviconUrl,
					fontFamily
				});
			}

			return { success: true };
		} catch (error) {
			console.error('Error updating branding:', error);
			return fail(500, {
				error: 'Server error',
				primaryColor,
				secondaryColor,
				logoUrl,
				faviconUrl,
				fontFamily
			});
		}
	}
};
