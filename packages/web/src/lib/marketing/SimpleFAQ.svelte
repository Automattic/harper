<script lang="ts">
type FaqItem = {
	q: string;
	a: string;
};

export let items: FaqItem[] = [];
export let title = 'FAQs';
</script>

<section class="border-t border-black/10 bg-cream py-20 dark:border-white/10 dark:bg-black">
	<div class="mx-auto max-w-180 px-4 text-center md:px-14">
		<h2
			class="m-0 py-0 text-[clamp(2.2rem,5vw,2.5rem)] leading-[1.08] font-semibold tracking-normal text-black dark:text-white"
		>
			{title}
		</h2>
		<div class="mt-8">
			{#each items as item}
				<div class="border-t border-black/10 py-5 text-left last:border-b dark:border-white/10">
					<h3 class="!m-0 py-0 text-base font-semibold tracking-normal text-black dark:text-white">
						{item.q}
					</h3>
					<p class="!mt-1.5 !mb-0 text-base leading-relaxed text-black/70 dark:text-white/70">
						{item.a}
					</p>
				</div>
			{/each}
		</div>
	</div>
</section>
