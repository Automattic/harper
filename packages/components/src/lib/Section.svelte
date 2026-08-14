<script lang="ts">
export let layout: 'single' | 'split' = 'single';
export let reverse = false;

const hasSubtitle = Boolean($$slots.subtitle);
const hasAside = Boolean($$slots.aside);

const { class: extraClass = '', ...restProps } = $$restProps;
</script>

<section
	{...restProps}
	class={`w-full px-4 md:px-6 ${extraClass}`.trim()}
>
	{#if layout === 'split'}
		<div class={`grid gap-8 md:grid-cols-2 ${hasAside ? 'md:items-start' : ''}`}>
			<div class={`space-y-4 ${reverse ? 'md:order-2' : ''}`}>
				{#if $$slots.title}
					<h3 class="font-semibold">
						<slot name="title" />
					</h3>
				{/if}
				{#if hasSubtitle}
					<p class="text-gray-600 dark:text-gray-300">
						<slot name="subtitle" />
					</p>
				{/if}
				{#if $$slots.default}
					<div class="space-y-3">
						<slot />
					</div>
				{/if}
			</div>
			{#if hasAside}
				<div class={`${reverse ? 'md:order-1' : ''}`}>
					<slot name="aside" />
				</div>
			{/if}
		</div>
	{:else}
		<div class="space-y-4">
			{#if $$slots.title}
				<h3 class="font-semibold">
					<slot name="title" />
				</h3>
			{/if}
			{#if hasSubtitle}
				<p class="text-gray-600 dark:text-gray-300">
					<slot name="subtitle" />
				</p>
			{/if}
			{#if $$slots.default}
				<div class="space-y-3">
					<slot />
				</div>
			{/if}
		</div>
	{/if}
</section>
