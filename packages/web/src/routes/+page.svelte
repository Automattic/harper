<script module>
export const frontmatter = {
	home: false,
};
</script>

<script lang="ts">
import { browser } from '$app/environment';
import Arrow from '$lib/components/Arrow.svelte';
import TestimonialCollection from '$lib/components/TestimonialCollection.svelte';
import { createEditorLinter } from '$lib/createEditorLinter';
import FaqSection from '$lib/marketing/FaqSection.svelte';
import HarperMark from '$lib/marketing/HarperMark.svelte';
import IntegrationTile from '$lib/marketing/IntegrationTile.svelte';
import MarketingFooter from '$lib/marketing/MarketingFooter.svelte';
import MarketingHeader from '$lib/marketing/MarketingHeader.svelte';
import PillButton from '$lib/marketing/PillButton.svelte';
import PrivacySpeedCards from '$lib/marketing/PrivacySpeedCards.svelte';
import { featuredIntegrationIds, integrations, marketingLinks } from '$lib/marketing/data';
import { LazyEditor } from 'harper-editor';
import type { Linter } from 'harper.js';
import { loadLiveVersions, liveVersions } from '$lib/marketing/versions';
import { onMount } from 'svelte';
import demoText from '../../../../demo.md?raw';

const editorContent = demoText.trim();
let linter: Linter | null = null;

const testimonials = [
	{
		authorName: 'Rich Edmonds',
		authorSubtitle: 'Lead PC Hardware Editor, XDA Developers',
		testimonial:
			'Written in Rust, everything is processed in an instant and I find it neat to see the browser extension highlight words as I type, effectively checking per letter. And no account is required, allowing me to get up and running in no time.',
		source:
			'https://www.xda-developers.com/ditched-grammarly-for-this-amazing-open-source-alternative/',
	},
	{
		authorName: 'Justin Pot',
		authorSubtitle: 'Tech journalist, Lifehacker',
		testimonial:
			'Obsidian is my favorite productivity app, and Harper is a grammar checking tool that works well with it.',
		source:
			'https://lifehacker.com/tech/harper-offline-alternative-to-grammarly?test_uuid=02DN02BmbRCcASIX6xMQtY9&test_variant=B',
	},
	{
		authorName: 'Filip Cujanovic',
		authorSubtitle: 'Chrome Extension Review',
		testimonial:
			"Awesome extension! It’s privacy focused, that means that every check it done locally on your computer, there is no server where your data goes! And because of that it’s blazingly fast compared to Grammarly.",
		source:
			'https://chromewebstore.google.com/detail/private-grammar-checker-h/lodbfhdipoipcjmlebjbgmmgekckhpfb/reviews',
	},
	{
		authorName: 'Prakash Joshi Pax',
		authorSubtitle: 'Writer, Medium',
		testimonial: "What I loved about this tool is that it’s private, and open source and really fast.",
		source: 'https://beingpax.medium.com/9-new-obsidian-plugins-you-need-to-check-out-today-d55dba29bfb8',
	},
	{
		authorName: 'Tim Miller',
		authorSubtitle: 'Author, Obsidian Rocks',
		testimonial: 'Harper is great: it is discreet, fast, powerful, and private.',
		source: 'https://obsidian.rocks/resource-harper/',
	},
	{
		authorName: 'imbolc',
		authorSubtitle: 'Chrome Extension Review',
		testimonial: "I’ve been using Harper in Neovim for a long time and am glad to see it as an extension!",
		source:
			'https://chromewebstore.google.com/detail/private-grammar-checker-h/lodbfhdipoipcjmlebjbgmmgekckhpfb/reviews',
	},
	{
		authorName: 'Martijn Gribnau',
		authorSubtitle: 'Software Engineer',
		testimonial:
			'What a delightful way to check for flagrant spelling errors in markdown files. Thanks Harper authors!',
		source: 'https://gribnau.dev/posts/harper-cli/',
	},
	{
		authorName: 'Chloe Ferguson',
		authorSubtitle: 'Writer, We Are Founders',
		testimonial:
			'Harper excels at catching the kinds of mistakes that matter in technical writing – improper capitalization, misspelled words, and awkward phrasing that can make documentation unclear.',
		source:
			'https://www.wearefounders.uk/the-grammar-checker-that-actually-gets-developers-meet-harper/',
	},
	{
		authorName: 'Rogerio Taques',
		authorSubtitle: 'Chrome Extension Review',
		testimonial:
			"I’ve been using Harper instead of Grammarly for a few months already, and I can’t be happier! I can’t wait to see the great improvement when this tool reaches version 1.0.0! Great job! I hope that, eventually, it will also support languages other than English.",
		source:
			'https://chromewebstore.google.com/detail/private-grammar-checker-h/lodbfhdipoipcjmlebjbgmmgekckhpfb/reviews',
	},
];

const faqs = [
	{
		q: 'Is Harper Free?',
		a: "Yes. Harper is free in every sense of the word. You don’t need a credit card to start using Harper, and the source code is freely available under the Apache-2.0 license.",
	},
	{
		q: 'How Does Harper Work?',
		a: "Harper watches your writing and provides instant suggestions when it notices a grammatical error. When you see an underline, it’s probably because Harper has something to say.",
	},
	{
		q: 'Does Harper Change The Meaning of My Words?',
		a: 'No. Harper will never intentionally suggest an edit that might change your meaning. Harper strives to never make it harder to express your creativity.',
	},
	{
		q: 'Is Harper Really Private?',
		a: 'Harper is the only widespread and comprehensive grammar checker that is truly private. Your data never leaves your device. Your writing should remain just that: yours.',
	},
	{
		q: 'How Do I Use or Integrate Harper?',
		a: "That depends on your use case. Do you want to use it within Obsidian? We have an Obsidian plugin. Do you want to use it within WordPress? We have a WordPress plugin. Do you want to use it within your Browser? We have a Chrome extension and a Firefox plugin. Do you want to use it within your code editor? We have documentation on how you can integrate with Visual Studio Code and its forks, Neovim, Helix, Emacs, Zed and Sublime Text. If you’re using a different code editor, then you can integrate directly with our language server, harper-ls. Do you want to integrate it in your web app or your JavaScript/TypeScript codebase? You can use harper.js. Do you want to integrate it in your Rust program or codebase? You can use harper-core.",
	},
	{
		q: 'What Human Languages Do You Support?',
		a: 'We currently only support English and its dialects British, American, Canadian, Australian, and Indian. Other languages are on the horizon, but we want our English support to be truly amazing before we diversify.',
	},
	{
		q: 'What Programming Languages Do You Support?',
		a: "For harper-ls and our code editor integrations, we support a wide variety of programming languages. You can view all of them over at the harper-ls documentation. We are entirely open to PRs that add support. If you just want to be able to run grammar checking on your code’s comments, you can use this PR as a model for what to do. For harper.js and those that use it under the hood like our Obsidian plugin, we support plaintext and/or Markdown.",
	},
	{
		q: 'Where Did the Name Harper Come From?',
		a: 'See this blog post.',
	},
	{
		q: 'Do I Need a GPU?',
		a: 'No. Harper runs on-device, no matter what. There are no special hardware requirements. No GPU, no additional memory, no fuss.',
	},
	{
		q: "What Do I Do If My Question Isn’t Here?",
		a: 'You can join our Discord and ask your questions there or you can start a discussion over at GitHub.',
	},
	{
		q: "Why Isn’t Harper Working in Gmail?",
		a: 'Harper will not run in Gmail unless the built-in grammar checker is disabled. If you wish to use Harper in Gmail, please disable the built-in grammar checker.',
	},
];

let liveFxVer = '';
let liveJsVer = '';
let liveRustVer = '';
let liveVscodeVer = '';

onMount(() => {
	void (async () => {
		linter = await createEditorLinter();
	})();

    void loadLiveVersions();
});

</script>

<svelte:head>
	<title>Harper: The Private Grammar Checker</title>
	<meta
		name="description"
		content="Harper is the free, private, open-source grammar checker that runs on your device."
	/>
</svelte:head>

<div class="min-h-screen bg-[#f6f1e6] text-[#1c1a16] dark:bg-black dark:text-white">
	<MarketingHeader active="home" />

	<section class="bg-[#fbfaf6] px-10 pt-[4.4rem] pb-20 text-center dark:bg-black max-[880px]:px-4">
		<div class="mx-auto flex max-w-[44rem] flex-col items-center">
			<HarperMark size={108} />
			<h1 class="!mt-7 !mb-0 py-0 !font-serif text-[clamp(3.4rem,8vw,4rem)] font-[650] leading-[1.02] tracking-normal text-inherit">
				Hi. I’m Harper.
			</h1>
			<p class="!mt-[1.35rem] !mb-0 !font-serif text-[1.38rem] leading-[1.35]">
				The <strong class="inline-block -rotate-1 bg-primary-100 p-1 text-black">Free</strong> Grammar Checker
				That Respects Your Privacy
			</p>
			<p class="!mt-3 !mb-0 !font-serif text-[1.12rem] leading-[1.35] text-[#807a6e] italic dark:text-white/55">
				I make you look like a grammar genius.
			</p>
			<div class="mt-7 flex flex-wrap gap-[0.65rem] max-[620px]:flex-col max-[620px]:items-stretch">
				<PillButton href="/get" size="lg">Get Harper</PillButton>
				<PillButton href={marketingLinks.github} kind="secondary" size="lg">Star on GitHub</PillButton>
			</div>
		</div>
	</section>

	<section class="bg-[#fbfaf6] pt-2 pb-[5.6rem] dark:bg-black" aria-labelledby="try-editor-title">
		<div class="mx-auto max-w-[73.75rem] px-10 max-[880px]:px-4">
			<div class="mb-[1.1rem] flex items-baseline justify-between gap-4 max-[620px]:flex-col max-[620px]:items-stretch">
				<h2 id="try-editor-title" class="!m-0 py-0 !font-serif text-[1.38rem] font-semibold leading-[1.3] tracking-normal text-inherit">
					Try Harper
				</h2>
				<a
					class="inline-flex items-center gap-1 !text-[#b06a1b] font-bold no-underline hover:no-underline dark:!text-primary-300 [&_path]:fill-none [&_path]:stroke-current [&_path]:stroke-[1.5] [&_path]:[stroke-linecap:round] [&_path]:[stroke-linejoin:round] [&_svg]:size-[0.7rem]"
					href="/editor">Open the full editor <Arrow /></a
				>
			</div>
			<div class="h-[35rem] overflow-hidden rounded-[0.9rem] border-[0.5px] border-[rgba(28,26,22,0.16)] bg-[#fbfaf6] shadow-[0_30px_60px_-24px_rgba(28,26,22,0.22),0_6px_14px_rgba(28,26,22,0.06),0_0_0_0.5px_rgba(0,0,0,0.04)] dark:border-white/15 dark:bg-black max-[620px]:h-[40rem]">
				{#if browser && linter}
					<LazyEditor content={editorContent} {linter} />
				{:else}
					<div class='flex h-full items-center justify-center text-[0.82rem] text-[#807a6e] dark:text-white/55 [font-family:"JetBrains_Mono",monospace]'>
						Loading Harper’s grammar engine...
					</div>
				{/if}
			</div>
		</div>
	</section>

	<section id="about" class="border-t-[0.5px] border-[rgba(28,26,22,0.1)] bg-[#fdfbf5] py-[4.8rem] dark:border-white/10 dark:bg-black">
		<div class="mx-auto max-w-[45rem] px-10 max-[880px]:px-4">
			<p class="!m-0 !font-serif text-[clamp(1.6rem,4vw,1.75rem)] font-[550] leading-[1.35] text-[#1c1a16] dark:text-white">
				Harper is a free, open-source grammar checker designed to be just right. Think of it as
				the private alternative to Grammarly, built after years of dealing with the shortcomings
				of the competition.
			</p>
			<p class="!mt-5 !mb-0 text-base leading-[1.65] text-[#4a463e] dark:text-white/70">
				Harper catches the kinds of mistakes that matter: improper capitalization, misspelled
				words, awkward phrasing, and broken grammar. Your writing never leaves your computer.
			</p>
		</div>
	</section>

	<section class="border-t-[0.5px] border-[rgba(28,26,22,0.1)] bg-[#fdfbf5] py-[4.8rem] dark:border-white/10 dark:bg-black">
		<div class="mx-auto grid max-w-[68.75rem] grid-cols-[minmax(0,1fr)_minmax(20rem,1fr)] items-center gap-14 px-10 max-[880px]:grid-cols-1 max-[880px]:px-4">
			<div>
				<h2 class="!mt-3 !mb-0 py-0 !font-serif text-[clamp(2.2rem,5vw,2.5rem)] font-[650] leading-[1.08] tracking-normal text-inherit">
					One grammar checker.<br />Every place you write.
				</h2>
				<p class="!mt-6 !mb-0 text-base leading-[1.65] text-[#4a463e] dark:text-white/70">
					Harper is available as a language server, a JavaScript library, a Rust crate, browser
					extensions, editor extensions, and native apps. Pick the integration that matches your
					workflow or build your own.
				</p>
				<div class="mt-7 flex flex-wrap gap-[0.65rem] max-[620px]:flex-col max-[620px]:items-stretch">
					<PillButton href="/get">See all integrations</PillButton>
					<PillButton href="/docs/about" kind="secondary">Read the docs</PillButton>
				</div>
			</div>
			<div
				class="grid grid-cols-[repeat(2,minmax(10rem,13.5rem))] justify-center gap-[0.4rem] rounded-2xl border-[0.5px] border-[rgba(28,26,22,0.1)] bg-white p-[1.1rem] dark:border-white/10 dark:bg-white/5 max-[620px]:grid-cols-1"
				aria-label="Featured Harper integrations"
			>
				{#each featuredIntegrationIds as id}
					{@const integration = integrations.find((item) => item.id === id)}
					{#if integration}
						<a
							class="flex items-center gap-3 rounded-[0.65rem] px-3 py-[0.65rem] !text-[#1c1a16] no-underline hover:bg-black/[0.04] hover:no-underline dark:!text-white dark:hover:bg-white/10"
							href={integration.href}
						>
							<IntegrationTile {integration} size={32} />
							<span class="flex min-w-0 flex-col">
								<div class="flex items-center gap-1.5 overflow-hidden">
									<strong class="overflow-hidden text-ellipsis whitespace-nowrap text-[0.84rem]">{integration.name}</strong>
									{#if $liveVersions[integration.id]}
										<span class="inline-flex items-center rounded-full bg-[#f4f1ea] dark:bg-white/10 px-1.5 py-0.5 text-[0.65rem] font-mono font-medium text-[#6b6455] dark:text-white/80 border border-[#e4dfd3] dark:border-white/10 select-none">
											v{$liveVersions[integration.id]}
										</span>
									{/if}
								</div>
								<small class="overflow-hidden text-ellipsis whitespace-nowrap text-[0.72rem] text-[#807a6e] dark:text-white/55">
									{integration.desc}
								</small>
							</span>
						</a>
					{/if}
				{/each}
			</div>
		</div>
	</section>

	<section class="border-t-[0.5px] border-[rgba(28,26,22,0.1)] bg-[#fbfaf6] py-[4.5rem] dark:border-white/10 dark:bg-black">
		<div class="mx-auto max-w-[73.75rem] px-10 max-[880px]:px-4">
			<PrivacySpeedCards />
		</div>
	</section>

	<section class="border-t-[0.5px] border-[rgba(28,26,22,0.1)] bg-[#fbfaf6] py-[4.5rem] dark:border-white/10 dark:bg-black">
		<div class="mx-auto max-w-[73.75rem] px-10 max-[880px]:px-4">
			<div class="mb-11 text-center">
				<h2 class="!mt-3 !mb-0 py-0 !font-serif text-[clamp(2.2rem,5vw,2.5rem)] font-[650] leading-[1.08] tracking-normal text-inherit">
					Loved by writers, journalists, and devs.
				</h2>
			</div>
			<TestimonialCollection {testimonials} />
		</div>
	</section>

	<FaqSection
		items={faqs}
		title="Questions, answered."
		intro="Don’t see yours?"
		introHref={marketingLinks.discord}
		introLinkText="Ask on Discord"
		collapsible
		layout="grid"
	/>

	<section class="border-t-[0.5px] border-[rgba(28,26,22,0.1)] bg-[#1c1a16] py-[5.6rem] pb-[6.25rem] text-center text-[#fbfaf6] dark:border-white/10">
		<div class="mx-auto max-w-[45rem] px-10 max-[880px]:px-4 [&_.harper-mark]:mx-auto [&_.harper-mark]:mb-[1.4rem] [&_.harper-mark]:text-[#fbe8c2]">
			<HarperMark size={56} />
			<h2 class="!mt-3 !mb-0 py-0 !font-serif text-[clamp(2.5rem,6vw,3.25rem)] font-[650] leading-[1.05] tracking-normal text-inherit">
				Pay us a visit on GitHub.
			</h2>
			<p class="!mt-6 !mb-0 text-base leading-[1.65] text-[#fbfaf6]/70">
				Fork it, file an issue, add a rule, port it to a new editor. Harper is free software,
				and we’d love your help.
			</p>
			<div class="mt-7 flex flex-col flex-wrap items-center justify-center gap-[0.65rem]">
				<PillButton href={marketingLinks.github} size="lg">Star on GitHub</PillButton>
				<PillButton href="/docs/contributors/introduction" size="lg">Contribute</PillButton>
			</div>
		</div>
	</section>

	<MarketingFooter />
</div>
