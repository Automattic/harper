<script module lang="ts">
import { defineMeta } from '@storybook/addon-svelte-csf';
import AutomatticLogo from './logos/AutomatticLogo.svelte';
import ChromeLogo from './logos/ChromeLogo.svelte';
import CodeLogo from './logos/CodeLogo.svelte';
import DiscordLogo from './logos/DiscordLogo.svelte';
import EdgeLogo from './logos/EdgeLogo.svelte';
import EmacsLogo from './logos/EmacsLogo.svelte';
import FirefoxLogo from './logos/FirefoxLogo.svelte';
import GitHubLogo from './logos/GitHubLogo.svelte';
import GmailLogo from './logos/GmailLogo.svelte';
import HelixLogo from './logos/HelixLogo.svelte';
import IMessageLogo from './logos/IMessageLogo.svelte';
import LinearLogo from './logos/LinearLogo.svelte';
import Logo from './logos/Logo.svelte';
import NeovimLogo from './logos/NeovimLogo.svelte';
import NotionLogo from './logos/NotionLogo.svelte';
import ObsidianLogo from './logos/ObsidianLogo.svelte';
import RedditLogo from './logos/RedditLogo.svelte';
import ScrivenerLogo from './logos/ScrivenerLogo.svelte';
import SlackLogo from './logos/SlackLogo.svelte';
import SublimeLogo from './logos/SublimeLogo.svelte';
import TelegramLogo from './logos/TelegramLogo.svelte';
import ThingsLogo from './logos/ThingsLogo.svelte';
import WhatsAppLogo from './logos/WhatsAppLogo.svelte';
import WordPressLogo from './logos/WordPressLogo.svelte';
import ZedLogo from './logos/ZedLogo.svelte';

const { Story } = defineMeta({
	title: 'Components/Logos',
	tags: ['autodocs'],
	parameters: {
		layout: 'padded',
	},
});

const logos = [
	{ name: 'Automattic', component: AutomatticLogo },
	{ name: 'Chrome', component: ChromeLogo },
	{ name: 'Discord', component: DiscordLogo },
	{ name: 'Edge', component: EdgeLogo },
	{ name: 'Emacs', component: EmacsLogo },
	{ name: 'Firefox', component: FirefoxLogo },
	{ name: 'GitHub', component: GitHubLogo },
	{ name: 'Gmail', component: GmailLogo },
	{ name: 'Harper', component: Logo },
	{ name: 'Helix', component: HelixLogo },
	{ name: 'iMessage', component: IMessageLogo },
	{ name: 'Linear', component: LinearLogo },
	{ name: 'Neovim', component: NeovimLogo },
	{ name: 'Notion', component: NotionLogo },
	{ name: 'Obsidian', component: ObsidianLogo },
	{ name: 'Reddit', component: RedditLogo },
	{ name: 'Scrivener', component: ScrivenerLogo },
	{ name: 'Slack', component: SlackLogo },
	{ name: 'Sublime Text', component: SublimeLogo },
	{ name: 'Telegram', component: TelegramLogo },
	{ name: 'Things', component: ThingsLogo },
	{ name: 'Visual Studio Code', component: CodeLogo },
	{ name: 'WhatsApp', component: WhatsAppLogo },
	{ name: 'WordPress', component: WordPressLogo },
	{ name: 'Zed', component: ZedLogo },
];
</script>

<Story name="All Logos" asChild>
	<div class="grid w-full grid-cols-5 gap-6">
		{#each logos as { name, component: LogoComponent } (name)}
			<figure class="flex flex-col items-center gap-3 rounded-lg border border-gray-200 p-4 dark:border-gray-700">
				<div class="h-20 w-full max-w-40" aria-hidden="true">
					<LogoComponent />
				</div>
				<figcaption class="text-center text-sm">{name}</figcaption>
			</figure>
		{/each}
	</div>
</Story>
