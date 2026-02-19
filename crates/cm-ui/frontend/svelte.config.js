import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	preprocess: vitePreprocess(),
	compilerOptions: {
		warningFilter: (warning) => {
			// Suppress a11y label warnings for desktop app
			if (warning.code === 'a11y_label_has_associated_control') return false;
			return true;
		}
	},
	kit: {
		adapter: adapter({
			fallback: 'index.html'
		})
	}
};

export default config;
