<script lang="ts">
import Arrow from '$lib/components/Arrow.svelte';

type FaqItem = {
	q: string;
	a: string;
	href?: string;
};

export let items: FaqItem[] = [];
export let title = 'FAQs';
export let intro = '';
export let introHref = '';
export let introLinkText = '';
export let collapsible = false;
export let layout: 'grid' | 'narrow' = 'narrow';

$: innerClass =
	layout === 'grid'
		? 'mx-auto grid max-w-[68.75rem] grid-cols-1 gap-12 px-4 md:px-14 min-[881px]:grid-cols-[18.75rem_1fr]'
		: 'mx-auto max-w-180 px-4 text-center md:px-14';
</script>

<section class="border-t border-black/10 bg-cream py-20 dark:border-white/10 dark:bg-black">
	<div class={innerClass}>
		<div>
			<h2
				class="m-0 py-0 text-[clamp(2.2rem,5vw,2.5rem)] leading-[1.08] font-semibold tracking-normal text-black dark:text-white"
			>
				{title}
			</h2>
			{#if intro}
				<p class="!mt-4 !mb-0 text-base leading-relaxed text-black/70 dark:text-white/70">
					{intro}
					{#if introHref && introLinkText}
						<a
							class="inline-flex items-center gap-1 font-bold text-primary no-underline dark:text-primary-300 [&_path]:fill-none [&_path]:stroke-current [&_path]:stroke-[1.5] [&_path]:[stroke-linecap:round] [&_path]:[stroke-linejoin:round] [&_svg]:h-3 [&_svg]:w-3"
							href={introHref}>{introLinkText}<Arrow /></a
						>
					{/if}
				</p>
			{/if}
		</div>

		<div class={collapsible ? 'border-t border-black/10 dark:border-white/10' : ''}>
			{#if collapsible}
				{#each items as item}
					<details class="group border-b border-black/10 dark:border-white/10">
						<summary
							class="flex cursor-pointer list-none items-center justify-between py-4 font-bold text-black [&::-webkit-details-marker]:hidden dark:text-white"
						>
							<span>{item.q}</span>
							<span
								class="inline-flex size-6 items-center justify-center rounded-full bg-black/5 text-black/70 group-open:hidden dark:bg-white/10 dark:text-white/70"
								aria-hidden="true">+</span
							>
							<span
								class="hidden size-6 items-center justify-center rounded-full bg-black text-primary-100 group-open:inline-flex dark:bg-white dark:text-black"
								aria-hidden="true">-</span
							>
						</summary>
						<p class="!m-0 max-w-152 pb-5 text-base leading-relaxed text-black/70 dark:text-white/70">
							{#if item.href}
								<a class="font-bold text-primary no-underline dark:text-primary-300" href={item.href}
									>{item.a}</a
								>
							{:else}
								{item.a}
							{/if}
						</p>
					</details>
				{/each}
			{:else}
				{#each items as item}
					<div class="border-t border-black/10 py-5 text-left last:border-b dark:border-white/10">
						<h3 class="!m-0 py-0 text-base font-semibold tracking-normal text-black dark:text-white">
							{item.q}
						</h3>
						<p class="!mt-1.5 !mb-0 text-base leading-relaxed text-black/70 dark:text-white/70">
							{#if item.href}
								<a class="font-bold text-primary no-underline dark:text-primary-300" href={item.href}
									>{item.a}</a
								>
							{:else}
								{item.a}
							{/if}
						</p>
					</div>
				{/each}
			{/if}
		</div>
	</div>
</section>
