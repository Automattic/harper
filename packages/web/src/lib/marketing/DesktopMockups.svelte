<script lang="ts">
import DiscordDesktopMockupWindow from './DiscordDesktopMockupWindow.svelte';
import MailDesktopMockupWindow from './MailDesktopMockupWindow.svelte';
import NotesDesktopMockupWindow from './NotesDesktopMockupWindow.svelte';
import SlackDesktopMockupWindow from './SlackDesktopMockupWindow.svelte';
</script>

<div class="stack" aria-label="Harper Desktop working in several macOS apps">
	<SlackDesktopMockupWindow />
	<MailDesktopMockupWindow />
	<NotesDesktopMockupWindow />
	<DiscordDesktopMockupWindow />
</div>

<style>
.stack {
	position: absolute;
	inset: 0;
}

@media (max-width: 1040px) {
	.stack {
		position: relative;
		height: 32rem;
		max-width: 42rem;
		margin: 0 auto;
	}
}

@media (max-width: 620px) {
	.stack {
		display: grid;
		height: auto;
		gap: 0.9rem;
	}
}
</style>
