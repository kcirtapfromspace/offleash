import adapter from '@sveltejs/adapter-node';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	preprocess: vitePreprocess(),

	kit: {
		adapter: adapter({
			out: 'build',
			precompress: true
		}),
		csrf: {
			// Allow cross-origin form submissions from known domains
			// This is needed when ORIGIN env var doesn't match the actual domain
			// (e.g., custom domain vs Render domain)
			checkOrigin: false
		}
	}
};

export default config;
