<script lang="ts">
import MacWindow from './MacWindow.svelte';
import Squiggle from './Squiggle.svelte';
import Suggestion from './Suggestion.svelte';
</script>

<div class="absolute top-2.5 right-[-0.6rem] z-2 w-70 max-[620px]:static max-[620px]:w-full max-[620px]:transform-none" style="transform: rotate(3.5deg);">
	<MacWindow title="Notes">
		<div class="flex min-h-46 bg-[#fffdf5] text-black">
			<aside
				class="flex w-23 shrink-0 flex-col gap-1 border-r border-black/10 bg-[#f6f0d8] p-2 text-[0.62rem]"
			>
				<strong class="rounded-sm bg-[#febc2e] p-1.5 text-[#3a2a08]">Birthday dinner</strong>
				<span class="text-black/50">Reserve table at Aug...</span>
			</aside>
			<section class="relative p-3 pb-12 text-[0.78rem]">
				<h3
					class="!mt-0 !mb-2.5 border-b border-black/10 pb-2 text-[0.84rem] font-semibold text-black"
				>
					Birthday dinner - saturday
				</h3>
				<small class="mb-2 block text-black/45">Today at 11:08 AM</small>
				<p class="!m-0 leading-[1.55]">
					Reserve table at Augusto's for 8 pm. Pick up a cake on the way,
					<Squiggle>noone</Squiggle> is allergic to nuts.
				</p>
				<Suggestion word="noone" fix="no one" />
			</section>
		</div>
	</MacWindow>
</div>
