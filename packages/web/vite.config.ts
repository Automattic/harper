import { defaultTheme } from '@sveltepress/theme-default';
import { sveltepress } from '@sveltepress/vite';
import { defineConfig } from 'vite';
import topLevelAwait from 'vite-plugin-top-level-await';
import wasm from 'vite-plugin-wasm';

export default defineConfig({
	server: {
		port: 3000,
		fs: {
			allow: ['../harper.js/dist'],
		},
	},
	plugins: [
		sveltepress({
			siteConfig: {
				title: 'Harper',
				description: 'A Grammar Checker from Automattic',
			},
			theme: defaultTheme({
				editLink: 'https://github.com/automattic/harper/edit/master/packages/web/src/routes/:route',
				logo: '/circle-logo.png',
				github: 'https://github.com/automattic/harper',
				discord: 'https://discord.gg/invite/JBqcAaKrzQ',
				themeColor: {
					primary: '#818eae',
					dark: '#355280',
					gradient: {
						start: '#355280',
						end: '#818eae',
					},
				},
				navbar: [
					{ title: 'Documentation', to: '/docs/about' },
					{
						title: 'Visual Studio Code',
						to: 'https://marketplace.visualstudio.com/items?itemName=elijah-potter.harper',
					},
					{ title: 'Obsidian', to: '/docs/integrations/obsidian' },
					{
						title: 'Chrome Extension',
						to: 'https://chromewebstore.google.com/detail/private-grammar-checking/lodbfhdipoipcjmlebjbgmmgekckhpfb',
					},
				],
				sidebar: {
					'/docs/': [
						{
							items: [
								{
									title: 'About',
									to: '/docs/about',
								},
							],
						},
						{
							title: 'Integrations',
							items: [
								{
									title: 'Obsidian',
									to: '/docs/integrations/obsidian',
								},
								{
									title: 'Chrome Extension',
									to: '/docs/integrations/chrome-extension',
								},
								{
									title: 'Firefox Extension',
									to: '/docs/integrations/firefox-extension',
								},
								{
									title: 'WordPress',
									to: '/docs/integrations/wordpress',
								},
								{
									title: 'Language Server',
									to: '/docs/integrations/language-server',
								},
								{
									title: 'Visual Studio Code',
									to: '/docs/integrations/visual-studio-code',
								},
								{
									title: 'Neovim',
									to: '/docs/integrations/neovim',
								},
								{
									title: 'Helix',
									to: '/docs/integrations/helix',
								},
								{
									title: 'Emacs',
									to: '/docs/integrations/emacs',
								},
								{
									title: 'Zed',
									to: '/docs/integrations/zed',
								},
								{
									title: 'Sublime Text',
									to: '/docs/integrations/sublime-text',
								},
							],
						},
						{
							title: 'harper.js',
							items: [
								{
									title: 'Introduction',
									to: '/docs/harperjs/introduction',
								},
								{
									title: 'Linting',
									to: '/docs/harperjs/linting',
								},
								{
									title: 'Spans',
									to: '/docs/harperjs/spans',
								},
								{
									title: 'Configure Rules',
									to: '/docs/harperjs/configurerules',
								},
								{
									title: 'Node.js',
									to: '/docs/harperjs/node',
								},
								{
									title: 'CDN',
									to: '/docs/harperjs/CDN',
								},
								{
									title: 'API Reference',
									to: '/docs/harperjs/ref/index.html',
								},
							],
						},
						{
							title: 'Contributors',
							items: [
								{
									title: 'Introduction',
									to: '/docs/contributors/introduction',
								},
								{
									title: 'Environment',
									to: '/docs/contributors/environment',
								},
								{
									title: 'Committing',
									to: '/docs/contributors/committing',
								},
								{
									title: 'Architecture',
									to: '/docs/contributors/architecture',
								},
								{
									title: 'Dictionary',
									to: '/docs/contributors/dictionary',
								},
								{
									title: 'Test Suite',
									to: '/docs/contributors/tests',
								},
								{
									title: 'Author a Rule',
									to: '/docs/contributors/author-a-rule',
								},
								{
									title: 'Visual Studio Code',
									to: '/docs/contributors/visual-studio-code',
								},
								{
									title: 'Chrome Extension',
									to: '/docs/contributors/chrome-extension',
								},
								{
									title: 'WordPress',
									to: '/docs/contributors/wordpress',
								},
								{
									title: 'Obsidian',
									to: '/docs/contributors/obsidian',
								},
								{
									title: 'Reviewing Pull Requests',
									to: '/docs/contributors/review',
								},
								{
									title: 'Local Statistics',
									to: '/docs/contributors/local-stats',
								},
								{
									title: 'Brill Tagging',
									to: '/docs/contributors/brill',
								},
								{
									title: 'FAQ',
									to: '/docs/contributors/faq',
								},
							],
						},
						{
							title: 'Rust Reference',
							to: 'https://docs.rs/harper-core/latest/harper_core/',
						},
						{
							title: 'Rules',
							to: '/docs/rules',
						},
					],
				},
				highlighter: {
					languages: [
						'svelte',
						'sh',
						'js',
						'html',
						'ts',
						'md',
						'css',
						'scss',
						'toml',
						'rust',
						'lua',
						'json',
						'elisp',
					],
				},
			}),
		}),
		wasm(),
		topLevelAwait(),
	],
});
