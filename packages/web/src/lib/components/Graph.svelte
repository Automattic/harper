<script lang="ts">
import IntersectionObserver from 'svelte-intersection-observer';

let data = new Map<string, number>();
data.set('Harper', 10);
data.set('LanguageTool', 650);
data.set('Grammarly', 4000);

let maxW = 0;

for (let val of data.values()) {
	if (val > maxW) {
		maxW = val;
	}
}

let scaledData = new Map<string, number>();

for (let [key, val] of data.entries()) {
	scaledData.set(key, val / maxW);
}

let els: Record<string, HTMLElement> = {};

function expand(_node: HTMLElement, { width, duration }: { width: number; duration: number }) {
	return {
		duration,
		css: (t: number) => {
			return `width: ${width * 100 * t}%;`;
		},
	};
}
</script>

<div class="mt-auto flex flex-col gap-3">
	{#each scaledData as [name, width] (name)}
		<IntersectionObserver element={els[name]} let:intersecting>
			<div bind:this={els[name]}>
				{#if intersecting}
					<div
						class='grid grid-cols-[1fr_3.5rem] items-center gap-x-3 gap-y-2 text-xs [font-family:"JetBrains_Mono",monospace] min-[421px]:grid-cols-[6.2rem_1fr_4rem]'
					>
						<span class="whitespace-nowrap text-current">{name}</span>
						<b
							class="col-span-full row-start-2 h-2 overflow-hidden rounded-full bg-white/10 dark:bg-black/10 min-[421px]:col-auto min-[421px]:row-auto"
						>
							<i
								class={name === 'Harper'
									? 'block h-full rounded-full bg-primary'
									: 'block h-full rounded-full bg-white/20 dark:bg-black/20'}
								in:expand={{ width, duration: width * maxW }}
								style={`width: ${width * 100}%;`}
							></i>
						</b>
						<em class="text-right not-italic text-white/75 dark:text-black/75">{width * maxW} ms</em>
					</div>
				{/if}
			</div>
		</IntersectionObserver>
	{/each}
</div>
