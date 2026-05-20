<script lang="ts">
import MacWindow from './MacWindow.svelte';
import Squiggle from './Squiggle.svelte';
import Suggestion from './Suggestion.svelte';
</script>

<div class="absolute top-[1.9rem] left-[4.4rem] z-3 w-[23.75rem] max-[620px]:static max-[620px]:w-full max-[620px]:transform-none" style="transform: rotate(1.5deg);">
	<MacWindow title="Mail - Compose">
		<div class="p-3.5 pb-12 text-[0.82rem]">
			<div class="mb-1.5 flex gap-1.5 text-[0.72rem] text-black/70">
				<span class="text-black/45">To:</span> design@example.com
			</div>
			<h3
				class="!mt-0 !mb-2.5 border-b border-black/10 pb-2 text-[0.84rem] font-semibold text-black"
			>
				Re: Q3 marketing review
			</h3>
			<p class="!m-0 leading-[1.55]">
				Thanks for sending these over, really <Squiggle>apreciate</Squiggle> the detail. I'll
				loop in the design team and we can have a draft back to you by
				<Squiggle color="#3a8dde">friday</Squiggle>.
			</p>
			<Suggestion word="apreciate" fix="appreciate" />
		</div>
	</MacWindow>
</div>
