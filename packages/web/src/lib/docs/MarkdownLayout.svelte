<script lang="ts">
import type { Snippet } from 'svelte';
import PageMetadata from '$lib/PageMetadata.svelte';
import type { MetadataInput } from '$lib/seo';

/** Markdown content. Embedded UI must use a data-docs-embed boundary to retain its own styles. */
interface Props extends MetadataInput {
	children: Snippet;
}

let { children, title, description }: Props = $props();
</script>

<PageMetadata metadata={{ title, description }} />

<div
	data-docs-content
	class="wrap-break-word
		[&_:is(h1,h2,h3,h4,h5,h6)]:docs-prose:relative [&_:is(h1,h2,h3,h4,h5,h6)]:docs-prose:text-(--docs-heading)! [&_:is(h1,h2,h3,h4,h5,h6)]:docs-prose:scroll-mt-[110px]
		[&_h1]:docs-prose:mt-0 [&_h1]:docs-prose:mb-5 [&_h1]:docs-prose:text-[32px]! [&_h1]:docs-prose:leading-[1.3]! [&_h1]:docs-prose:font-bold! [&_h1]:docs-prose:tracking-[-0.02em]!
		[&_h2]:docs-prose:mt-8 [&_h2]:docs-prose:mb-4 [&_h2]:docs-prose:border-t [&_h2]:docs-prose:border-(--docs-border) [&_h2]:docs-prose:pt-4 [&_h2]:docs-prose:text-2xl! [&_h2]:docs-prose:leading-[1.35]! [&_h2]:docs-prose:font-semibold! [&_h2]:docs-prose:tracking-[-0.01em]!
		[&_h3]:docs-prose:mt-8 [&_h3]:docs-prose:mb-3 [&_h3]:docs-prose:text-xl! [&_h3]:docs-prose:leading-[1.4]! [&_h3]:docs-prose:font-semibold!
		[&_h4]:docs-prose:mt-6 [&_h4]:docs-prose:mb-2 [&_h4]:docs-prose:text-lg! [&_h4]:docs-prose:leading-[1.4]! [&_h4]:docs-prose:font-semibold!
		[&_p]:docs-prose:my-4! [&_p]:docs-prose:leading-[1.75]! [&_ul]:docs-prose:my-4 [&_ul]:docs-prose:pl-5! [&_ul]:docs-prose:leading-[1.5] [&_ol]:docs-prose:my-4 [&_ol]:docs-prose:pl-5! [&_ol]:docs-prose:leading-[1.5] [&_li+li]:docs-prose:mt-2
		[&_a]:docs-prose:text-(--docs-link)! [&_a:hover]:docs-prose:underline [&_img]:docs-prose:max-w-full
		[&_code]:docs-prose:rounded-md [&_code]:docs-prose:bg-[#ececee] [&_code]:docs-prose:px-1.5 [&_code]:docs-prose:py-[3px] [&_code]:docs-prose:text-[0.875em] dark:[&_code]:docs-prose:bg-[#2e2e32]
		[&_pre]:docs-prose:my-0 [&_pre]:docs-prose:overflow-x-auto [&_pre]:docs-prose:rounded-lg [&_pre]:docs-prose:border [&_pre]:docs-prose:border-(--docs-border) [&_pre]:docs-prose:bg-[#fff] [&_pre]:docs-prose:px-[18px] [&_pre]:docs-prose:py-0 [&_pre]:docs-prose:text-sm [&_pre]:docs-prose:leading-[1.5] dark:[&_pre]:docs-prose:bg-[#1f1f23]
		[&_pre_code]:docs-prose:block [&_pre_code]:docs-prose:bg-transparent! [&_pre_code]:docs-prose:!p-0 [&_pre_code]:docs-prose:text-[1em]
		[&_blockquote]:docs-prose:my-5! [&_blockquote]:docs-prose:border-l-[3px]! [&_blockquote]:docs-prose:border-(--docs-border)! [&_blockquote]:docs-prose:px-4 [&_blockquote]:docs-prose:py-1 [&_blockquote]:docs-prose:text-(--docs-muted) [&_blockquote_p]:docs-prose:m-0!
		[&_table]:docs-prose:my-6 [&_table]:docs-prose:w-full [&_table]:docs-prose:border-collapse [&_th]:docs-prose:border-b-2 [&_th]:docs-prose:border-(--docs-border) [&_th]:docs-prose:text-left! [&_th]:docs-prose:font-semibold [&_th]:docs-prose:px-3 [&_th]:docs-prose:py-2 [&_td]:docs-prose:px-3 [&_td]:docs-prose:py-2 [&_tbody_tr]:docs-prose:border-t [&_tbody_tr]:docs-prose:border-(--docs-border) [&_tbody_tr:hover]:docs-prose:bg-(--docs-border)
		max-[479px]:[&_table]:docs-prose:block max-[479px]:[&_table]:docs-prose:overflow-x-auto
		[&_pre_:is(.hljs-comment,.hljs-quote)]:docs-prose:text-[#6a737d] dark:[&_pre_:is(.hljs-comment,.hljs-quote)]:docs-prose:text-[#8b949e]
		[&_pre_:is(.hljs-keyword,.hljs-selector-tag,.hljs-literal)]:docs-prose:text-[#a626a4] dark:[&_pre_:is(.hljs-keyword,.hljs-selector-tag,.hljs-literal)]:docs-prose:text-[#c678dd]
		[&_pre_:is(.hljs-string,.hljs-regexp,.hljs-addition)]:docs-prose:text-[#22863a] dark:[&_pre_:is(.hljs-string,.hljs-regexp,.hljs-addition)]:docs-prose:text-[#98c379]
		[&_pre_:is(.hljs-number,.hljs-symbol,.hljs-attr,.hljs-built_in)]:docs-prose:text-[#986801] dark:[&_pre_:is(.hljs-number,.hljs-symbol,.hljs-attr,.hljs-built_in)]:docs-prose:text-[#d19a66]
		[&_pre_:is(.hljs-title,.hljs-section)]:docs-prose:text-[#0366d6] dark:[&_pre_:is(.hljs-title,.hljs-section)]:docs-prose:text-[#61afef]
		[&_pre_:is(.hljs-tag,.hljs-name,.hljs-deletion)]:docs-prose:text-[#b31d28] dark:[&_pre_:is(.hljs-tag,.hljs-name,.hljs-deletion)]:docs-prose:text-[#e06c75]
		[&_pre_.hljs-emphasis]:docs-prose:italic [&_pre_.hljs-strong]:docs-prose:font-bold"
>
	{@render children()}
</div>
