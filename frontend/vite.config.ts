import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vitest/config';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	server: {
		port: 5173,
		strictPort: true,
		proxy: {
			'/api': 'http://localhost:3001'
		}
	},
	clearScreen: false,
	envPrefix: ['VITE_'],
	test: {
		include: ['src/**/*.test.ts'],
		environment: 'node'
	}
});
