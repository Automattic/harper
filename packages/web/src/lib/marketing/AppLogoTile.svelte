<script lang="ts">
import {
	DiscordLogo,
	GitHubLogo,
	GmailLogo,
	IMessageLogo,
	LinearLogo,
	NotionLogo,
	ObsidianLogo,
	RedditLogo,
	ScrivenerLogo,
	SlackLogo,
	TelegramLogo,
	ThingsLogo,
	WhatsAppLogo,
} from 'components';

export let id: string;
export let size = 40;

const tileClasses =
	'inline-flex shrink-0 items-center justify-center bg-white shadow-[0_0_0_0.5px_rgba(0,0,0,0.08),0_1px_2px_rgba(0,0,0,0.04)] dark:bg-white [&_svg]:block [&_svg]:h-[68%] [&_svg]:w-[68%]';

$: tileStyle = `width: ${size}px; height: ${size}px; border-radius: ${Math.round(size * 0.235)}px;`;
</script>

<span class={tileClasses} style={tileStyle} aria-hidden="true">
	{#if id === 'gmail'}
		<GmailLogo />
	{:else if id === 'imessage'}
		<IMessageLogo />
	{:else if id === 'whatsapp'}
		<WhatsAppLogo />
	{:else if id === 'slack'}
		<SlackLogo />
	{:else if id === 'discord'}
		<DiscordLogo />
	{:else if id === 'telegram'}
		<TelegramLogo />
	{:else if id === 'notion'}
		<NotionLogo />
	{:else if id === 'obsidian'}
		<ObsidianLogo />
	{:else if id === 'linear'}
		<LinearLogo />
	{:else if id === 'github'}
		<GitHubLogo />
	{:else if id === 'things'}
		<ThingsLogo />
	{:else if id === 'scrivener'}
		<ScrivenerLogo />
	{:else if id === 'reddit'}
		<RedditLogo />
	{/if}
</span>
