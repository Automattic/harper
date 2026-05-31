export const marketingLinks = {
	github: 'https://github.com/automattic/harper',
	discord: 'https://discord.gg/invite/JBqcAaKrzQ',
	chrome:
		'https://chromewebstore.google.com/detail/private-grammar-checking/lodbfhdipoipcjmlebjbgmmgekckhpfb?utm_source=harper-website&utm_medium=referral',
	firefox: 'https://addons.mozilla.org/en-US/firefox/addon/private-grammar-checker-harper/',
	vscode: 'https://marketplace.visualstudio.com/items?itemName=elijah-potter.harper',
	crates: 'https://crates.io/crates/harper-core',
};

export type Integration = {
	id: string;
	name: string;
	desc: string;
	href: string;
	category: string;
	categoryLabel: string;
	tag?: string;
	platform?: string;
	version?: string;
	cta: 'install' | 'docs';
	color?: string;
	fg?: string;
	initial?: string;
	community?: boolean;
};

export type IntegrationCategory = {
	id: string;
	label: string;
	blurb: string;
	items: Omit<Integration, 'category' | 'categoryLabel'>[];
};

export const integrationCategories: IntegrationCategory[] = [
	{
		id: 'apps',
		label: 'Apps',
		blurb: 'Use Harper in the apps you write in.',
		items: [
			{
				id: 'desktop',
				name: 'Harper Desktop',
				tag: 'BETA',
				desc: 'macOS, every text field, system-wide.',
				href: '/desktop',
				platform: 'macOS 14+',
				cta: 'install',
				color: '#1c1a16',
				fg: '#fbe8c2',
				initial: 'H',
			},
			{
				id: 'obsidian',
				name: 'Obsidian',
				desc: 'Inline checks in your vault.',
				href: '/docs/integrations/obsidian',
				platform: 'Obsidian plugin',
				cta: 'install',
			},
			{
				id: 'wordpress',
				name: 'WordPress',
				desc: 'Block-editor plugin.',
				href: '/docs/integrations/wordpress',
				platform: 'WordPress 6.2+',
				cta: 'install',
				color: '#21759b',
				fg: '#fff',
				initial: 'W',
			},
		],
	},
	{
		id: 'browsers',
		label: 'Browsers',
		blurb: 'Checks every textarea on the web.',
		items: [
			{
				id: 'chrome',
				name: 'Chrome',
				desc: 'Chrome Web Store extension.',
				href: marketingLinks.chrome,
				platform: 'Chrome, Edge, Brave',
				cta: 'install',
				color: '#4285f4',
				fg: '#fff',
				initial: 'C',
			},
			{
				id: 'firefox',
				name: 'Firefox',
				desc: 'AMO add-on.',
				href: marketingLinks.firefox,
				platform: 'Firefox 102+',
				cta: 'install',
				color: '#ff7139',
				fg: '#fff',
				initial: 'F',
			},
		],
	},
	{
		id: 'editors',
		label: 'Code editors',
		blurb: 'Real grammar checking for prose-in-code.',
		items: [
			{
				id: 'vscode',
				name: 'Visual Studio Code',
				desc: 'Marketplace extension.',
				href: marketingLinks.vscode,
				platform: 'VS Code, Cursor',
				cta: 'install',
				color: '#3e8cff',
				fg: '#fff',
				initial: 'V',
			},
			{
				id: 'neovim',
				name: 'Neovim',
				desc: 'LSP plugin.',
				href: '/docs/integrations/neovim',
				platform: 'Neovim 0.10+',
				cta: 'docs',
				color: '#57a143',
				fg: '#fff',
				initial: 'N',
			},
			{
				id: 'zed',
				name: 'Zed',
				desc: 'Native language server.',
				href: '/docs/integrations/zed',
				platform: 'Zed editor',
				cta: 'docs',
				color: '#ce5b1a',
				fg: '#fff',
				initial: 'Z',
				community: true,
			},
			{
				id: 'helix',
				name: 'Helix',
				desc: 'LSP integration.',
				href: '/docs/integrations/helix',
				platform: 'Helix editor',
				cta: 'docs',
				color: '#5b6e83',
				fg: '#fff',
				initial: 'H',
				community: true,
			},
			{
				id: 'emacs',
				name: 'Emacs',
				desc: 'flycheck-harper.',
				href: '/docs/integrations/emacs',
				platform: 'Emacs 28+',
				cta: 'docs',
				color: '#7356a5',
				fg: '#fff',
				initial: 'E',
				community: true,
			},
			{
				id: 'sublime',
				name: 'Sublime Text',
				desc: 'Package Control package.',
				href: '/docs/integrations/sublime-text',
				platform: 'Sublime Text 4',
				cta: 'docs',
				color: '#ff9800',
				fg: '#1c1a16',
				initial: 'S',
				community: true,
			},
		],
	},
	{
		id: 'developers',
		label: 'For developers',
		blurb: 'Embed Harper anywhere you ship code.',
		items: [
			{
				id: 'ls',
				name: 'Language server',
				desc: 'harper-ls, talks LSP.',
				href: '/docs/integrations/language-server',
				platform: 'LSP, any editor',
				cta: 'docs',
				color: '#1c1a16',
				fg: '#d0c7b3',
				initial: '{}',
			},
			{
				id: 'js',
				name: 'JavaScript library',
				desc: 'npm i harper.js',
				href: '/docs/harperjs/introduction',
				platform: 'Node, Bun, Browser',
				cta: 'docs',
				color: '#f7df1e',
				fg: '#1c1a16',
				initial: 'JS',
			},
			{
				id: 'rust',
				name: 'Rust crate',
				desc: 'crates.io / harper-core',
				href: marketingLinks.crates,
				platform: 'Rust crate',
				cta: 'docs',
				color: '#b06a1b',
				fg: '#fff',
				initial: 'R',
			},
		],
	},
];

export const integrations: Integration[] = integrationCategories.flatMap((category) =>
	category.items.map((item) => ({
		...item,
		category: category.id,
		categoryLabel: category.label,
	})),
);

export const featuredIntegrationIds = [
	'desktop',
	'chrome',
	'vscode',
	'obsidian',
	'firefox',
	'neovim',
	'wordpress',
	'zed',
];

export const compatibilityApps = [
	{ id: 'gmail', name: 'Gmail' },
	{ id: 'imessage', name: 'iMessage' },
	{ id: 'whatsapp', name: 'WhatsApp' },
	{ id: 'slack', name: 'Slack' },
	{ id: 'discord', name: 'Discord' },
	{ id: 'telegram', name: 'Telegram' },
	{ id: 'notion', name: 'Notion' },
	{ id: 'obsidian', name: 'Obsidian' },
	{ id: 'linear', name: 'Linear' },
	{ id: 'github', name: 'GitHub' },
	{ id: 'things', name: 'Things' },
	{ id: 'scrivener', name: 'Scrivener' },
];

export const docSearchConfig = {
	indexName: 'Documentation',
	appId: 'YIV4D9QMR0',
	apiKey: 'ff521ad7f129e4f4defe97dce3c923ad',
};
