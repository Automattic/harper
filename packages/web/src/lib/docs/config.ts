/** A navigation destination, either a local route or an external URL. */
export interface NavigationLink {
	title: string;
	to: string;
}

/** A sidebar section with an optional heading and ordered navigation links. */
export interface NavigationGroup {
	title?: string;
	items: NavigationLink[];
}

/** Sidebar entries support both grouped links and standalone destinations. */
export type SidebarEntry = NavigationLink | NavigationGroup;

export const siteConfig = {
	title: 'Harper: The Private Grammar Checker',
	description: 'The Free Grammar Checker That Respects Your Privacy',
	logo: '/circle-logo.png',
	github: 'https://github.com/automattic/harper',
	discord: 'https://discord.gg/invite/JBqcAaKrzQ',
	editLink: 'https://github.com/automattic/harper/edit/master/packages/web/src/routes/:route',
};

export const themeColor = {
	primary: '#f1920e',
	dark: '#301d03',
	gradient: {
		start: '#355280',
		end: '#818eae',
	},
};

export const navbar: NavigationLink[] = [
	{ title: 'Documentation', to: '/docs/about' },
	{ title: 'Get Harper', to: '/get' },
	{ title: 'Obsidian', to: '/docs/integrations/obsidian' },
	{
		title: 'Chrome Extension',
		to: 'https://chromewebstore.google.com/detail/private-grammar-checking/lodbfhdipoipcjmlebjbgmmgekckhpfb',
	},
];

export const sidebar: Record<string, SidebarEntry[]> = {
	'/docs/': [
		{ items: [{ title: 'About', to: '/docs/about' }] },
		{ items: [{ title: 'Weir', to: '/docs/weir' }] },
		{
			title: 'Integrations',
			items: [
				{ title: 'Obsidian', to: '/docs/integrations/obsidian' },
				{ title: 'Chrome Extension', to: '/docs/integrations/chrome-extension' },
				{ title: 'Firefox Extension', to: '/docs/integrations/firefox-extension' },
				{ title: 'WordPress', to: '/docs/integrations/wordpress' },
				{ title: 'Language Server', to: '/docs/integrations/language-server' },
				{ title: 'Visual Studio Code', to: '/docs/integrations/visual-studio-code' },
				{ title: 'Neovim', to: '/docs/integrations/neovim' },
				{ title: 'Helix', to: '/docs/integrations/helix' },
				{ title: 'Emacs', to: '/docs/integrations/emacs' },
				{ title: 'Zed', to: '/docs/integrations/zed' },
				{ title: 'Sublime Text', to: '/docs/integrations/sublime-text' },
			],
		},
		{
			title: 'harper.js',
			items: [
				{ title: 'Introduction', to: '/docs/harperjs/introduction' },
				{ title: 'Linting', to: '/docs/harperjs/linting' },
				{ title: 'Spans', to: '/docs/harperjs/spans' },
				{ title: 'Configure Rules', to: '/docs/harperjs/configurerules' },
				{ title: 'Node.js', to: '/docs/harperjs/node' },
				{ title: 'CDN', to: '/docs/harperjs/CDN' },
				{ title: 'API Reference', to: '/docs/harperjs/ref/index.html' },
			],
		},
		{
			title: 'Contributors',
			items: [
				{ title: 'Introduction', to: '/docs/contributors/introduction' },
				{ title: 'Environment', to: '/docs/contributors/environment' },
				{ title: 'Committing', to: '/docs/contributors/committing' },
				{ title: 'Architecture', to: '/docs/contributors/architecture' },
				{ title: 'Dictionary', to: '/docs/contributors/dictionary' },
				{ title: 'Testing Strategy', to: '/docs/contributors/testing-strategy' },
				{ title: 'Author a Rule', to: '/docs/contributors/author-a-rule' },
				{ title: 'Visual Studio Code', to: '/docs/contributors/visual-studio-code' },
				{ title: 'Chrome Extension', to: '/docs/contributors/chrome-extension' },
				{ title: 'Harper Desktop', to: '/docs/contributors/desktop' },
				{ title: 'Website', to: '/docs/contributors/website' },
				{ title: 'WordPress', to: '/docs/contributors/wordpress' },
				{ title: 'Obsidian', to: '/docs/contributors/obsidian' },
				{ title: 'Reviewing Pull Requests', to: '/docs/contributors/review' },
				{ title: 'Committer', to: '/docs/contributors/committer' },
				{ title: 'Local Statistics', to: '/docs/contributors/local-stats' },
				{ title: 'Brill Tagging', to: '/docs/contributors/brill' },
				{ title: 'FAQ', to: '/docs/contributors/faq' },
			],
		},
		{ title: 'Rust Reference', to: 'https://docs.rs/harper-core/latest/harper_core/' },
		{ title: 'Rules', to: '/docs/rules' },
	],
};

/** Public Algolia search credentials shared by documentation and marketing pages. */
export const docSearchConfig = {
	indexName: 'Documentation',
	appId: 'YIV4D9QMR0',
	apiKey: 'ff521ad7f129e4f4defe97dce3c923ad',
};
