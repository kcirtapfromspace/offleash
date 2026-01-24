import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ parent }) => {
	// Get data from parent layout (includes membership info)
	const parentData = await parent();
	return {
		membership: parentData.membership
	};
};
