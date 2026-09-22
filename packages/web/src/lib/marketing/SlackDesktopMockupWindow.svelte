<script lang="ts">
import MacWindow from './MacWindow.svelte';
import Squiggle from './Squiggle.svelte';
import Suggestion from './Suggestion.svelte';
</script>

<div class="absolute bottom-0 left-[-0.6rem] w-80 max-[620px]:static max-[620px]:w-full max-[620px]:transform-none" style="transform: rotate(-4deg);">
	<MacWindow title="Slack - #launches" dark>
		<div class="flex min-h-[9.6rem] gap-2.5 p-3 pr-3.5 pb-12 text-[0.82rem]">
			<div
				class="flex size-[1.65rem] shrink-0 items-center justify-center rounded-[0.45rem] bg-[#4a154b] text-xs font-extrabold text-white"
			>
				J
			</div>
			<p class="!m-0 leading-[1.55]">
				<b class="mr-1.5">Jordan</b><small class="text-white/45">2:14 PM</small><br />
				heads up: the deploy is taking <Squiggle>alot</Squiggle> longer
				<Squiggle>then</Squiggle> expected. might push to <Squiggle>tommorrow</Squiggle>.
			</p>
			<Suggestion word="alot" fix="a lot" />
		</div>
	</MacWindow>
</div>
