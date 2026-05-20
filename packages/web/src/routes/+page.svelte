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

onMount(() => {
	void (async () => {
		linter = await createEditorLinter();
	})();
});

</script>

<svelte:head>
	<title>Harper: The Private Grammar Checker</title>
	<meta
		name="description"
		content="Harper is the free, private, open-source grammar checker that runs on your device."
	/>
</svelte:head>

<div class="marketing-page">
	<MarketingHeader active="home" />

	<section class="hero">
		<div class="hero-inner">
			<HarperMark size={108} />
			<h1>Hi. I’m Harper.</h1>
			<p class="hero-sub">
				The <strong class="inline-block -rotate-1 bg-primary-100 p-1">Free</strong> Grammar Checker
				That Respects Your Privacy
			</p>
			<p class="hero-third">I make you look like a grammar genius.</p>
			<div class="hero-actions">
				<PillButton href="/get" size="lg">Get Harper</PillButton>
				<PillButton href={marketingLinks.github} kind="secondary" size="lg">Star on GitHub</PillButton>
			</div>
		</div>
	</section>

	<section class="try-editor" aria-labelledby="try-editor-title">
		<div class="section-inner wide">
			<div class="section-row">
				<h2 id="try-editor-title">Try Harper</h2>
				<a href="/editor">Open the full editor <Arrow /></a>
			</div>
			<div class="editor-frame">
				{#if browser && linter}
					<LazyEditor content={editorContent} {linter} />
				{:else}
					<div class="editor-loading">Loading Harper’s grammar engine...</div>
				{/if}
			</div>
		</div>
	</section>

	<section id="about" class="intro">
		<div class="section-inner narrow">
			<p class="lead">
				Harper is a free, open-source grammar checker designed to be just right. Think of it as
				the private alternative to Grammarly, built after years of dealing with the shortcomings
				of the competition.
			</p>
			<p>
				Harper catches the kinds of mistakes that matter: improper capitalization, misspelled
				words, awkward phrasing, and broken grammar. Your writing never leaves your computer.
			</p>
		</div>
	</section>

	<section class="integrations-callout">
		<div class="section-inner split">
			<div>
				<h2>One grammar checker.<br />Every place you write.</h2>
				<p>
					Harper is available as a language server, a JavaScript library, a Rust crate, browser
					extensions, editor extensions, and native apps. Pick the integration that matches your
					workflow or build your own.
				</p>
				<div class="button-row">
					<PillButton href="/get">See all integrations</PillButton>
					<PillButton href="/docs/about" kind="secondary">Read the docs</PillButton>
				</div>
			</div>
			<div class="integration-grid" aria-label="Featured Harper integrations">
				{#each featuredIntegrationIds as id}
					{@const integration = integrations.find((item) => item.id === id)}
					{#if integration}
						<a href={integration.href}>
							<IntegrationTile {integration} size={32} />
							<span>
								<strong>{integration.name}</strong>
								<small>{integration.desc}</small>
							</span>
						</a>
					{/if}
				{/each}
			</div>
		</div>
	</section>

	<section class="privacy-speed">
		<div class="section-inner wide">
			<PrivacySpeedCards />
		</div>
	</section>

	<section class="testimonials">
		<div class="section-inner wide">
			<div class="center-heading">
				<h2>Loved by writers, journalists, and devs.</h2>
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

	<section class="open-source">
		<div class="section-inner narrow">
			<HarperMark size={56} />
			<h2>Pay us a visit on GitHub.</h2>
			<p>
				Fork it, file an issue, add a rule, port it to a new editor. Harper is free software,
				and we’d love your help.
			</p>
			<div class="button-row center">
				<PillButton href={marketingLinks.github} size="lg">Star on GitHub</PillButton>
				<PillButton href="/docs/contributors/introduction" kind="secondary" size="lg">
					Contribute
				</PillButton>
			</div>
		</div>
	</section>

	<MarketingFooter />
</div>

<style>
.marketing-page {
	min-height: 100vh;
	background: var(--marketing-page-bg);
	color: var(--marketing-ink);
	font-family: inherit;
}

.section-inner {
	max-width: 68.75rem;
	margin: 0 auto;
	padding: 0 2.5rem;
}

.section-inner.wide {
	max-width: 73.75rem;
}

.section-inner.narrow {
	max-width: 45rem;
}

.hero {
	background: var(--marketing-cream);
	padding: 4.4rem 2.5rem 5rem;
	text-align: center;
}

.hero-inner {
	display: flex;
	max-width: 44rem;
	margin: 0 auto;
	flex-direction: column;
	align-items: center;
}

h1,
h2 {
	margin: 0;
	color: inherit;
	font-family: Domine, serif;
	font-weight: 650;
	letter-spacing: 0;
}

h1 {
	margin-top: 1.75rem;
	font-size: clamp(3.4rem, 8vw, 4rem);
	line-height: 1.02;
}

.hero-sub {
	margin: 1.35rem 0 0;
	font-family: Domine, serif;
	font-size: 1.38rem;
	line-height: 1.35;
}

.hero-third {
	margin: 0.75rem 0 0;
	color: var(--marketing-ink-3);
	font-family: Domine, serif;
	font-size: 1.12rem;
	font-style: italic;
}

.hero-actions,
.button-row {
	display: flex;
	flex-wrap: wrap;
	gap: 0.65rem;
	margin-top: 1.75rem;
}

.center {
	justify-content: center;
	align-items: center;
}

.try-editor {
	background: var(--marketing-cream);
	padding: 0.5rem 0 5.6rem;
}

.section-row {
	display: flex;
	align-items: baseline;
	justify-content: space-between;
	gap: 1rem;
	margin-bottom: 1.1rem;
}

.section-row h2 {
	font-size: 1.38rem;
	line-height: 1.3;
}

.section-row a {
	display: inline-flex;
	align-items: center;
	gap: 0.2rem;
	color: var(--marketing-amber);
	font-weight: 700;
	text-decoration: none;
}

.section-row a :global(svg) {
	width: 0.7rem;
	height: 0.7rem;
	fill: none;
	stroke: currentColor;
	stroke-linecap: round;
	stroke-linejoin: round;
	stroke-width: 1.5;
}

.editor-frame {
	height: 35rem;
	overflow: hidden;
	border: 0.5px solid var(--marketing-line-strong);
	border-radius: 0.9rem;
	background: #fbfaf6;
	box-shadow:
		0 30px 60px -24px rgba(28, 26, 22, 0.22),
		0 6px 14px rgba(28, 26, 22, 0.06),
		0 0 0 0.5px rgba(0, 0, 0, 0.04);
}

.editor-loading {
	display: flex;
	height: 100%;
	align-items: center;
	justify-content: center;
	color: var(--marketing-ink-3);
	font-family: "JetBrains Mono", monospace;
	font-size: 0.82rem;
}

.intro,
.integrations-callout {
	border-top: 0.5px solid var(--marketing-line);
	background: #fdfbf5;
	padding: 4.8rem 0;
}

.intro .lead {
	margin: 0;
	color: var(--marketing-ink);
	font-family: Domine, serif;
	font-size: clamp(1.6rem, 4vw, 1.75rem);
	font-weight: 550;
	line-height: 1.35;
}

.intro p:not(.lead),
.integrations-callout p,
.open-source p {
	color: var(--marketing-ink-2);
	font-size: 1rem;
	line-height: 1.65;
}

.intro p:not(.lead) {
	margin: 1.25rem 0 0;
}

.split {
	display: grid;
	grid-template-columns: minmax(0, 1fr) minmax(20rem, 1fr);
	align-items: center;
	gap: 3.5rem;
}

.split h2,
.center-heading h2 {
	margin-top: 0.75rem;
	font-size: clamp(2.2rem, 5vw, 2.5rem);
	line-height: 1.08;
}

.integration-grid {
	display: grid;
	grid-template-columns: repeat(2, minmax(10rem, 13.5rem));
	gap: 0.4rem;
	justify-content: center;
	border: 0.5px solid var(--marketing-line);
	border-radius: 1rem;
	background: #fff;
	padding: 1.1rem;
}

.integration-grid a {
	display: flex;
	align-items: center;
	gap: 0.75rem;
	border-radius: 0.65rem;
	color: var(--marketing-ink);
	padding: 0.65rem 0.75rem;
	text-decoration: none;
}

.integration-grid a:hover {
	background: rgba(28, 26, 22, 0.04);
}

.integration-grid span {
	display: flex;
	min-width: 0;
	flex-direction: column;
}

.integration-grid strong {
	overflow: hidden;
	font-size: 0.84rem;
	text-overflow: ellipsis;
	white-space: nowrap;
}

.integration-grid small {
	overflow: hidden;
	color: var(--marketing-ink-3);
	font-size: 0.72rem;
	text-overflow: ellipsis;
	white-space: nowrap;
}

.privacy-speed,
.testimonials {
	border-top: 0.5px solid var(--marketing-line);
	background: var(--marketing-cream);
	padding: 4.5rem 0;
}

.center-heading {
	margin-bottom: 2.75rem;
	text-align: center;
}

.open-source {
	border-top: 0.5px solid var(--marketing-line);
	background: var(--marketing-ink);
	color: #fbfaf6;
	padding: 5.6rem 0 6.25rem;
	text-align: center;
}

.open-source :global(.harper-mark) {
	margin: 0 auto 1.4rem;
	color: #fbe8c2;
}

.open-source h2 {
	margin-top: 0.75rem;
	font-size: clamp(2.5rem, 6vw, 3.25rem);
	line-height: 1.05;
}

.open-source p {
	color: rgba(251, 250, 246, 0.72);
}

.open-source .button-row {
	flex-direction: column;
	align-items: center;
}

@media (max-width: 880px) {
	.section-inner {
		padding-inline: 1rem;
	}

	.hero {
		padding-inline: 1rem;
	}

	.split {
		grid-template-columns: 1fr;
	}
}

@media (max-width: 620px) {
	.hero-actions,
	.button-row,
	.section-row {
		flex-direction: column;
		align-items: stretch;
	}

	.button-row.center {
		align-items: center;
	}

	.editor-frame {
		height: 40rem;
	}

	.integration-grid {
		grid-template-columns: 1fr;
	}
}
</style>
