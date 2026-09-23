import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import { version } from './package.json';

export default defineConfig({
	plugins: [sveltekit()],
	define: {
		__APP_VERSION__: JSON.stringify(version)
	},
	server: {
		host: '127.0.0.1',
		port: 5173,
		strictPort: true
	},
	build: {
		sourcemap: false
	}
});
