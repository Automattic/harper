import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';

const prod = process.env.APP_ENV === 'production';

export default defineConfig({
	ssr: {
		noExternal: prod ? ['mysql2', 'drizzle-orm', 'posthog-js', 'drizzle-zod', 'zod'] : [],
	},
	server: {
		port: 3000,
		fs: {
			allow: ['../harper.js', '../chrome-plugin', '../..'],
		},
	},
	plugins: [sveltekit(), tailwindcss()],
});
