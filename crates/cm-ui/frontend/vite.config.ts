import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	server: {
		port: 5173,
		strictPort: true
	},
	// Prevent vite from obscuring Rust errors
	clearScreen: false,
	// Env variables with this prefix will be exposed to the frontend
	envPrefix: ['VITE_', 'TAURI_ENV_*'],
	test: {
		include: ['src/**/*.test.ts'],
		environment: 'node'
	}
});
