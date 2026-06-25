<script lang="ts">
import DiscordLogo from '$lib/components/DiscordLogo.svelte';
import GitHubLogo from '$lib/components/GitHubLogo.svelte';
import GmailLogo from '$lib/components/GmailLogo.svelte';
import IMessageLogo from '$lib/components/IMessageLogo.svelte';
import LinearLogo from '$lib/components/LinearLogo.svelte';
import NotionLogo from '$lib/components/NotionLogo.svelte';
import ObsidianLogo from '$lib/components/ObsidianLogo.svelte';
import RedditLogo from '$lib/components/RedditLogo.svelte';
import ScrivenerLogo from '$lib/components/ScrivenerLogo.svelte';
import SlackLogo from '$lib/components/SlackLogo.svelte';
import TelegramLogo from '$lib/components/TelegramLogo.svelte';
import ThingsLogo from '$lib/components/ThingsLogo.svelte';
import WhatsAppLogo from '$lib/components/WhatsAppLogo.svelte';

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
