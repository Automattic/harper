import { resolve } from 'node:path';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [tailwindcss(), svelte()],
	build: {
		lib: {
			entry: resolve(__dirname, 'src/lib/bundle.ts'),
			formats: ['es'],
			fileName: () => 'index',
			cssFileName: 'harper-editor',
		},
		rollupOptions: {
			external: ['svelte', /^svelte\//, /^harper\.js(\/.*)?$/],
			output: {
				assetFileNames: (assetInfo) =>
					assetInfo.name === 'harper-editor.css'
						? 'harper-editor.css'
						: 'assets/[name]-[hash][extname]',
				chunkFileNames: 'chunks/[name]-[hash].js',
				entryFileNames: () => 'index.js',
			},
		},
	},
});
