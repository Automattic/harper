<script lang="ts">
import DiscordDesktopMockupWindow from './DiscordDesktopMockupWindow.svelte';
import MailDesktopMockupWindow from './MailDesktopMockupWindow.svelte';
import NotesDesktopMockupWindow from './NotesDesktopMockupWindow.svelte';
import SlackDesktopMockupWindow from './SlackDesktopMockupWindow.svelte';
</script>

<div
	class="absolute inset-0 max-[1040px]:relative max-[1040px]:mx-auto max-[1040px]:h-128 max-[1040px]:max-w-168 max-[620px]:grid max-[620px]:h-auto max-[620px]:gap-3.5"
	aria-label="Harper Desktop working in several macOS apps"
>
	<SlackDesktopMockupWindow />
	<MailDesktopMockupWindow />
	<NotesDesktopMockupWindow />
	<DiscordDesktopMockupWindow />
</div>
