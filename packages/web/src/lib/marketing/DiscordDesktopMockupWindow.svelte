<script lang="ts">
import MacWindow from './MacWindow.svelte';
import Squiggle from './Squiggle.svelte';
import Suggestion from './Suggestion.svelte';
</script>

<div class="absolute right-0 bottom-[1.9rem] z-4 w-[21.25rem] max-[620px]:static max-[620px]:w-full max-[620px]:transform-none" style="transform: rotate(-2deg);">
	<MacWindow title="Discord - #design-crit" dark>
		<div class="flex min-h-[9.6rem] gap-2.5 p-3 pr-3.5 pb-12 text-[0.82rem]">
			<div
				class="flex size-[1.65rem] shrink-0 items-center justify-center rounded-full bg-[#5865f2] text-xs font-extrabold text-white"
			>
				R
			</div>
			<p class="!m-0 leading-[1.55]">
				<b class="mr-1.5">riley</b><small class="text-[#8a8d93]">Today at 2:14 PM</small><br />
				not sure if its <Squiggle>worht</Squiggle> shipping yet, the copy still needs another
				<Squiggle>passs</Squiggle> imo
			</p>
			<Suggestion word="worht" fix="worth" />
		</div>
	</MacWindow>
</div>
