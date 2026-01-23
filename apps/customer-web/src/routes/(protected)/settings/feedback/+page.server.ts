import type { PageServerLoad, Actions } from './$types';
import { env } from '$env/dynamic/private';
import { fail } from '@sveltejs/kit';

const API_URL = env.API_URL || 'http://localhost:3000';

export const load: PageServerLoad = async ({ locals }) => {
	return {
		user: locals.user
	};
};

export const actions: Actions = {
	default: async ({ request, cookies, locals }) => {
		const token = cookies.get('token');
		if (!token) {
			return fail(401, { error: 'Not authenticated' });
		}

		const formData = await request.formData();
		const feedbackType = formData.get('feedbackType')?.toString() as 'bug' | 'feature';
		const title = formData.get('title')?.toString();
		const description = formData.get('description')?.toString();

		if (!feedbackType || !title || !description) {
			return fail(400, { error: 'All fields are required' });
		}

		if (title.length < 5) {
			return fail(400, { error: 'Title must be at least 5 characters' });
		}

		if (description.length < 20) {
			return fail(400, { error: 'Description must be at least 20 characters' });
		}

		try {
			const response = await fetch(`${API_URL}/feedback`, {
				method: 'POST',
				headers: {
					'Authorization': `Bearer ${token}`,
					'Content-Type': 'application/json',
				},
				body: JSON.stringify({
					feedback_type: feedbackType,
					title,
					description,
				}),
			});

			if (!response.ok) {
				const error = await response.json().catch(() => ({ message: 'Failed to submit feedback' }));
				return fail(400, { error: error.message || 'Failed to submit feedback' });
			}

			const result = await response.json();

			return {
				success: true,
				feedbackType,
				issueUrl: result.issue_url,
				issueNumber: result.issue_number,
			};
		} catch (error) {
			console.error('Error submitting feedback:', error);
			return fail(500, { error: 'Server error. Please try again later.' });
		}
	},
};
