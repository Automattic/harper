<script module>
export const frontmatter = {
	home: false,
};
</script>

<script lang="ts">
import ChromeLogo from '$lib/components/ChromeLogo.svelte';
import CodeLogo from '$lib/components/CodeLogo.svelte';
import LazyEditor from '$lib/components/LazyEditor.svelte';
import FirefoxLogo from '$lib/components/FirefoxLogo.svelte';
import GitHubLogo from '$lib/components/GitHubLogo.svelte';
import ObsidianLogo from '$lib/components/ObsidianLogo.svelte';
import Logo from '$lib/components/Logo.svelte';
import Graph from '$lib/components/Graph.svelte';
import Section from '$lib/components/Section.svelte';
import TestimonialCollection from '$lib/components/TestimonialCollection.svelte';
import EmacsLogo from '$lib/components/EmacsLogo.svelte';
import HelixLogo from '$lib/components/HelixLogo.svelte';
import NeovimLogo from '$lib/components/NeovimLogo.svelte';
import SublimeLogo from '$lib/components/SublimeLogo.svelte';
import WordPressLogo from '$lib/components/WordPressLogo.svelte';
import ZedLogo from '$lib/components/ZedLogo.svelte';
import EdgeLogo from '$lib/components/EdgeLogo.svelte';
import { Link } from 'components';
import { browser } from '$app/environment';
    import Testimonial from '$lib/components/Testimonial.svelte';

/**
 * @param {string} keyword
 */
function agentHas(keyword: string): boolean | undefined {
	if (!browser) {
		return false;
	}

	return navigator.userAgent.toLowerCase().includes(keyword.toLowerCase());
}


const testimonials = [
  {
    authorName: "Rich Edmonds",
    authorSubtitle: "Lead PC Hardware Editor, XDA Developers",
    testimonial: "Written in Rust, everything is processed in an instant and I find it neat to see the browser extension highlight words as I type, effectively checking per letter. And no account is required, allowing me to get up and running in no time.",
    source: "https://www.xda-developers.com/ditched-grammarly-for-this-amazing-open-source-alternative/"
  },
  {
    authorName: "Justin Pot",
    authorSubtitle: "Tech journalist, Lifehacker",
    testimonial: "Obsidian is my favorite productivity app, and Harper is a grammar checking tool that works well with it.",
    source: "https://lifehacker.com/tech/harper-offline-alternative-to-grammarly?test_uuid=02DN02BmbRCcASIX6xMQtY9&test_variant=B"
  },
  {
    authorName: "Filip Cujanovic",
    authorSubtitle: "Chrome Extension Review",
    testimonial: "Awesome extension! It's privacy focused, that means that every check it done locally on your computer, there is no server where your data goes! And because of that it's blazingly fast compared to Grammarly.",
    source: "https://chromewebstore.google.com/detail/private-grammar-checker-h/lodbfhdipoipcjmlebjbgmmgekckhpfb/reviews"
  },
  {
    authorName: "Tim Miller",
    authorSubtitle: "Author, Obsidian Rocks",
    testimonial: "Harper is great: it is discreet, fast, powerful, and private.",
    source: "https://obsidian.rocks/resource-harper/"
  },
  {
    authorName: "Prakash Joshi Pax",
    authorSubtitle: "Writer, Medium",
    testimonial: "What I loved about this tool is that it's private, and open source and really fast.",
    source: "https://beingpax.medium.com/9-new-obsidian-plugins-you-need-to-check-out-today-d55dba29bfb8"
  },
  {
    authorName: "imbolc",
    authorSubtitle: "Chrome Extension Review",
    testimonial: "I've been using Harper in Neovim for a long time and am glad to see it as an extension!",
    source: "https://chromewebstore.google.com/detail/private-grammar-checker-h/lodbfhdipoipcjmlebjbgmmgekckhpfb/reviews"

  },
  {
    authorName: "Martijn Gribnau",
    authorSubtitle: "Software Engineer",
    testimonial: "What a delightful way to check for flagrant spelling errors in markdown files. Thanks Harper authors!",
    source: "https://gribnau.dev/posts/harper-cli/"
  },
  {
    authorName: "Chloe Ferguson",
    authorSubtitle: "Writer, We Are Founders",
    testimonial: "Harper excels at catching the kinds of mistakes that matter in technical writing – improper capitalization, misspelled words, and awkward phrasing that can make documentation unclear.",
    source: "https://www.wearefounders.uk/the-grammar-checker-that-actually-gets-developers-meet-harper/"
  },
  {
    authorName: "Rogerio Taques",
    authorSubtitle: "Chrome Extension Review",
    testimonial: "I've been using Harper instead of Grammarly for a few months already, and I can't be happier! I can't wait to see the great improvement when this tool reaches version 1.0.0! Great job! I hope that, eventually, it will also support languages other than English.",
    source: "https://chromewebstore.google.com/detail/private-grammar-checker-h/lodbfhdipoipcjmlebjbgmmgekckhpfb/reviews"
  },
];
</script>

<main class="mx-auto flex w-full max-w-5xl flex-col gap-12 py-12">
	<div class="space-y-6 px-4 md:px-6">
		<div class="flex w-full flex-col items-center">
			<Logo width="200px" />
		</div>
		<div class="space-y-2 text-center">
			<h1 class="font-bold">Hi. I'm Harper.</h1>
			<h2>
				The <strong>Free</strong> Grammar Checker That Respects Your Privacy
			</h2>
		</div>

		<div
			class="md:flex md:flex-row grid grid-cols-2 items-center justify-evenly place-items-center gap-2 pt-2 text-center"
		>
			<Link
				href="https://github.com/automattic/harper"
				class="flex flex-row items-center [&>*]:m-2 skew-hover-left"
			>
				<GitHubLogo width="40px" height="40px" />GitHub
			</Link>

      {#if agentHas("firefox")}
	      <Link href="https://addons.mozilla.org/en-US/firefox/addon/private-grammar-checker-harper/" class="flex flex-row items-center [&>*]:m-2 skew-hover"
	      	><FirefoxLogo width="40px" height="40px" />Add to Firefox</Link
	      >
      {:else if agentHas("Edg")}
	      <Link href="https://microsoftedge.microsoft.com/addons/detail/private-grammar-checker-/ihjkkjfembmnjldmdchmadigpmapkpdh" class="flex flex-row items-center [&>*]:m-2 skew-hover-left"
	      	><EdgeLogo width="40px" height="40px" />Add to Edge</Link
	      >
      {:else}
	      <Link href="https://chromewebstore.google.com/detail/private-grammar-checking/lodbfhdipoipcjmlebjbgmmgekckhpfb" class="flex flex-row items-center [&>*]:m-2 skew-hover"
	      	><ChromeLogo width="40px" height="40px" />Add to Chrome</Link
	      >
      {/if}
			<Link
				href="https://marketplace.visualstudio.com/items?itemName=elijah-potter.harper"
				class="flex flex-row items-center [&>*]:m-2 skew-hover-left"
			>
				<CodeLogo width="40px" height="40px" />Install in VS Code
			</Link>
			<Link
				href="/docs/integrations/obsidian"
				class="flex flex-row items-center [&>*]:m-2 skew-hover"
			>
				<ObsidianLogo width="40px" height="40px" />Install in Obsidian
			</Link>
      <Link href="https://elijahpotter.dev" class="flex flex-row items-center [&>*]:m-2 skew-hover-left"
		><img
			width="40"
			height="40"
			src="/icons/profile.svg"
			alt="Author"
		/>Author</Link
	>
		</div>

		<div class="h-[800px] w-full">
      {#if browser}
			  <LazyEditor />
      {/if}
		</div>
	</div>

	<Section>
		<svelte:fragment slot="title">What is it?</svelte:fragment>
		<p>
			Harper is a free English grammar checker designed to be just right. You can think of it as an
			open-source alternative to Grammarly. I created it after years of dealing with the shortcomings
			of the competition.
		</p>
	</Section>

	<Section layout="split">
		<svelte:fragment slot="title">Private</svelte:fragment>
		<p>Harper is completely private, in every sense of the word.</p>
		<p>Since Harper runs on-device, your data doesn't go anywhere you don't want it to.</p>
		<p>
			That means you have 100% certainty we don't violate your copyright by training large language
			models.
		</p>
		<p>
			Harper also intentionally avoids including any kind of generative AI in any part of our processing pipeline.
		</p>
		<svelte:fragment slot="aside">
			<img
				src="/images/camera.webp"
				class="w-full rounded-xl object-cover shadow-sm"
				alt="Graffiti of a camera."
			/>
		</svelte:fragment>
	</Section>

	<Section layout="split" reverse>
		<svelte:fragment slot="title">Native Everywhere</svelte:fragment>
		<p>
			Harper is available as a
			<Link class="text-blue-600 dark:text-blue-400" href="/docs/integrations/language-server">language server</Link>,
			<Link class="text-blue-600 dark:text-blue-400" href="/docs/harperjs/introduction">JavaScript library</Link>
			through WebAssembly, and
			<Link class="text-blue-600 dark:text-blue-400" href="https://crates.io/crates/harper-core">Rust crate</Link>,
			so you can get fantastic grammar checking anywhere you work.
		</p>
		<p>
			That said, we take extra care to make sure the
			<Link class="text-blue-600 dark:text-blue-400" href="/docs/integrations/visual-studio-code">Visual Studio Code</Link>,
			<Link class="text-blue-600 dark:text-blue-400" href="/docs/integrations/neovim">Neovim</Link>,
			<Link class="text-blue-600 dark:text-blue-400" href="/docs/integrations/obsidian">Obsidian</Link>, and
			<Link class="text-blue-600 dark:text-blue-400" href="https://chromewebstore.google.com/detail/private-grammar-checking/lodbfhdipoipcjmlebjbgmmgekckhpfb">Chrome</Link>
			extensions are amazing.
		</p>
		<svelte:fragment slot="aside">
			<div class="grid gap-4 sm:grid-cols-2">
				<Link
					href="/docs/integrations/obsidian"
					class="flex items-center gap-3 rounded-lg border border-neutral-200 px-4 py-3 shadow-sm transition hover:shadow-md dark:border-neutral-800 skew-hover-left"
				>
					<ObsidianLogo width="40" height="40" />
					<span class="font-medium">Obsidian</span>
				</Link>
				<Link
					href="/docs/integrations/visual-studio-code"
					class="flex items-center gap-3 rounded-lg border border-neutral-200 px-4 py-3 shadow-sm transition hover:shadow-md dark:border-neutral-800 skew-hover"
				>
					<CodeLogo width="40" height="40" />
					<span class="font-medium">Visual Studio Code</span>
				</Link>
				<Link
					href="/docs/integrations/neovim"
					class="flex items-center gap-3 rounded-lg border border-neutral-200 px-4 py-3 shadow-sm transition hover:shadow-md dark:border-neutral-800 skew-hover"
				>
					<NeovimLogo width="40" height="40" />
					<span class="font-medium">Neovim</span>
				</Link>
				<Link
					href="https://chromewebstore.google.com/detail/private-grammar-checking/lodbfhdipoipcjmlebjbgmmgekckhpfb"
					class="flex items-center gap-3 rounded-lg border border-neutral-200 px-4 py-3 shadow-sm transition hover:shadow-md dark:border-neutral-800 skew-hover-left"
				>
					<ChromeLogo width="40" height="40" />
					<span class="font-medium">Chrome</span>
				</Link>
				<Link
					href="https://addons.mozilla.org/en-US/firefox/addon/private-grammar-checker-harper/"
					class="flex items-center gap-3 rounded-lg border border-neutral-200 px-4 py-3 shadow-sm transition hover:shadow-md dark:border-neutral-800 skew-hover"
				>
					<FirefoxLogo width="40" height="40" />
					<span class="font-medium">Firefox</span>
				</Link>
				<Link
					href="/docs/integrations/helix"
					class="flex items-center gap-3 rounded-lg border border-neutral-200 px-4 py-3 shadow-sm transition hover:shadow-md dark:border-neutral-800 skew-hover-left"
				>
					<HelixLogo width="40" height="40" />
					<span class="font-medium">Helix</span>
				</Link>
				<Link
					href="/docs/integrations/wordpress"
					class="flex items-center gap-3 rounded-lg border border-neutral-200 px-4 py-3 shadow-sm transition hover:shadow-md dark:border-neutral-800 skew-hover-left"
				>
					<WordPressLogo width="40" height="40" />
					<span class="font-medium">WordPress</span>
				</Link>
				<Link
					href="/docs/integrations/zed"
					class="flex items-center gap-3 rounded-lg border border-neutral-200 px-4 py-3 shadow-sm transition hover:shadow-md dark:border-neutral-800 skew-hover"
				>
					<ZedLogo width="40" height="40" />
					<span class="font-medium">Zed</span>
				</Link>
				<Link
					href="/docs/integrations/emacs"
					class="flex items-center gap-3 rounded-lg border border-neutral-200 px-4 py-3 shadow-sm transition hover:shadow-md dark:border-neutral-800 skew-hover-left"
				>
					<EmacsLogo width="40" height="40" />
					<span class="font-medium">Emacs</span>
				</Link>
				<Link
					href="/docs/integrations/sublime-text"
					class="flex items-center gap-3 rounded-lg border border-neutral-200 px-4 py-3 shadow-sm transition hover:shadow-md dark:border-neutral-800 skew-hover"
				>
					<SublimeLogo width="40" height="40" />
					<span class="font-medium">Sublime Text</span>
				</Link>
			</div>
		</svelte:fragment>
	</Section>

	<Section layout="split">
		<svelte:fragment slot="title">Wicked Fast</svelte:fragment>
		<p>
			Since Harper runs on your devices, it's able to serve up suggestions in under 10 milliseconds.
		</p>
		<p>No network request, no massive language models, no fuss.</p>
		<svelte:fragment slot="aside">
			<div class="rounded-xl border border-neutral-200 p-4 shadow-sm dark:border-neutral-800">
				<Graph />
			</div>
		</svelte:fragment>
	</Section>

	<Section>
		<svelte:fragment slot="title">Loved by Thousands</svelte:fragment>
		<TestimonialCollection testimonials={testimonials} />
	</Section>

	<Section id="faqs">
		<svelte:fragment slot="title">FAQs</svelte:fragment>
		<div class="space-y-4">
			<details class="group rounded-lg border border-neutral-200 bg-white p-4 shadow-sm open:shadow-md dark:border-neutral-800 dark:bg-neutral-900">
				<summary class="cursor-pointer font-semibold marker:text-neutral-400">
					Is Harper Free?
				</summary>
				<p class="mt-3">
					Yes. Harper is free in every sense of the word. You don't need a credit card to start using
					Harper, and the source code is freely available under the Apache-2.0 license.
				</p>
			</details>
			<details class="group rounded-lg border border-neutral-200 bg-white p-4 shadow-sm open:shadow-md dark:border-neutral-800 dark:bg-neutral-900">
				<summary class="cursor-pointer font-semibold marker:text-neutral-400">
					How Does Harper Work?
				</summary>
				<p class="mt-3">
					Harper watches your writing and provides instant suggestions when it notices a grammatical
					error. When you see an underline, it's probably because Harper has something to say.
				</p>
			</details>
			<details class="group rounded-lg border border-neutral-200 bg-white p-4 shadow-sm open:shadow-md dark:border-neutral-800 dark:bg-neutral-900">
				<summary class="cursor-pointer font-semibold marker:text-neutral-400">
					Does Harper Change The Meaning of My Words?
				</summary>
				<p class="mt-3">
					No. Harper will never intentionally suggest an edit that might change your meaning. Harper
					strives to never make it harder to express your creativity.
				</p>
			</details>
			<details class="group rounded-lg border border-neutral-200 bg-white p-4 shadow-sm open:shadow-md dark:border-neutral-800 dark:bg-neutral-900">
				<summary class="cursor-pointer font-semibold marker:text-neutral-400">
					Is Harper Really Private?
				</summary>
				<p class="mt-3">
					Harper is the only widespread and comprehensive grammar checker that is truly private. Your
					data never leaves your device. Your writing should remain just that: <strong>yours.</strong>
				</p>
			</details>
			<details class="group rounded-lg border border-neutral-200 bg-white p-4 shadow-sm open:shadow-md dark:border-neutral-800 dark:bg-neutral-900">
				<summary class="cursor-pointer font-semibold marker:text-neutral-400">
					How Do I Use or Integrate Harper?
				</summary>
				<div class="mt-3">
					<p>
						That depends on your use case. Do you want to use it within Obsidian? We have an
						<Link class="text-blue-600 dark:text-blue-400" href="/docs/integrations/obsidian">Obsidian plugin</Link>. Do you want to use it within WordPress? We have a
						<Link class="text-blue-600 dark:text-blue-400" href="/docs/integrations/wordpress">WordPress plugin</Link>. Do you want to use it within your Browser? We have a
						<Link class="text-blue-600 dark:text-blue-400" href="/docs/integrations/chrome-extension">Chrome extension</Link> and a
						<Link class="text-blue-600 dark:text-blue-400" href="/docs/integrations/firefox-extension">Firefox plugin</Link>. Do you want to use it within your code editor? We have documentation on how you can integrate with
						<Link class="text-blue-600 dark:text-blue-400" href="/docs/integrations/visual-studio-code">Visual Studio Code and its forks</Link>,
						<Link class="text-blue-600 dark:text-blue-400" href="/docs/integrations/neovim">Neovim</Link>,
						<Link class="text-blue-600 dark:text-blue-400" href="/docs/integrations/helix">Helix</Link>,
						<Link class="text-blue-600 dark:text-blue-400" href="/docs/integrations/emacs">Emacs</Link>,
						<Link class="text-blue-600 dark:text-blue-400" href="/docs/integrations/zed">Zed</Link> and
						<Link class="text-blue-600 dark:text-blue-400" href="/docs/integrations/sublime-text">Sublime Text</Link>. If you're using a different code editor, then you can integrate directly with our language server,
						<Link class="text-blue-600 dark:text-blue-400" href="/docs/integrations/language-server">harper-ls</Link>. Do you want to integrate it in your web app or your JavaScript/TypeScript codebase? You can use
						<Link class="text-blue-600 dark:text-blue-400" href="/docs/harperjs/introduction">harper.js</Link>. Do you want to integrate it in your Rust program or codebase? You can use
						<Link class="text-blue-600 dark:text-blue-400" href="https://crates.io/crates/harper-core">harper-core</Link>.
					</p>
				</div>
			</details>
			<details class="group rounded-lg border border-neutral-200 bg-white p-4 shadow-sm open:shadow-md dark:border-neutral-800 dark:bg-neutral-900">
				<summary class="cursor-pointer font-semibold marker:text-neutral-400">
					What Human Languages Do You Support?
				</summary>
				<p class="mt-3">
					We currently only support English and its dialects British, American, Canadian, and
					Australian. Other languages are on the horizon, but we want our English support to be truly
					amazing before we diversify.
				</p>
			</details>
			<details class="group rounded-lg border border-neutral-200 bg-white p-4 shadow-sm open:shadow-md dark:border-neutral-800 dark:bg-neutral-900">
				<summary class="cursor-pointer font-semibold marker:text-neutral-400">
					What Programming Languages Do You Support?
				</summary>
				<p class="mt-3">
					For <code>harper-ls</code> and our code editor integrations, we support a wide variety of
					programming languages. You can view all of them over at the
					<Link class="text-blue-600 dark:text-blue-400" href="/docs/integrations/language-server#Supported-Languages">harper-ls documentation</Link>.
					We are entirely open to PRs that add support. If you just want to be able to run grammar checking
					on your code's comments, you can use
					<Link class="text-blue-600 dark:text-blue-400" href="https://github.com/Automattic/harper/pull/332">this PR as a model for what to do</Link>.
				</p>
				<p class="mt-3">
					For <code>harper.js</code> and those that use it under the hood like our Obsidian plugin, we
					support plaintext and/or Markdown.
				</p>
			</details>
			<details class="group rounded-lg border border-neutral-200 bg-white p-4 shadow-sm open:shadow-md dark:border-neutral-800 dark:bg-neutral-900">
				<summary class="cursor-pointer font-semibold marker:text-neutral-400">
					Where Did the Name Harper Come From?
				</summary>
				<p class="mt-3">
					See <Link class="text-blue-600 dark:text-blue-400" href="https://elijahpotter.dev/articles/naming_harper">this blog post</Link>.
				</p>
			</details>
			<details class="group rounded-lg border border-neutral-200 bg-white p-4 shadow-sm open:shadow-md dark:border-neutral-800 dark:bg-neutral-900">
				<summary class="cursor-pointer font-semibold marker:text-neutral-400">
					Do I Need a GPU?
				</summary>
				<div class="mt-3">
					<p>No. Harper runs on-device, no matter what. There are no special hardware requirements. No GPU, no additional memory, no fuss.</p>
				</div>
			</details>
			<details class="group rounded-lg border border-neutral-200 bg-white p-4 shadow-sm open:shadow-md dark:border-neutral-800 dark:bg-neutral-900">
				<summary class="cursor-pointer font-semibold marker:text-neutral-400">
					What Do I Do If My Question Isn't Here?
				</summary>
				<p class="mt-3">
					You can join our
					<Link class="text-blue-600 dark:text-blue-400" href="https://discord.gg/invite/JBqcAaKrzQ">Discord</Link>
					and ask your questions there or you can start a discussion over at
					<Link class="text-blue-600 dark:text-blue-400" href="https://github.com/Automattic/harper/discussions">GitHub</Link>.
				</p>
			</details>
		</div>
	</Section>

	<Section>
		<svelte:fragment slot="title">Open Source</svelte:fragment>
		<p>Harper is completely open source under the Apache-2.0 license.</p>
		<p>
			Come pay us a visit on
			<Link class="text-blue-600 dark:text-blue-400" href="https://github.com/automattic/harper">GitHub</Link>.
		</p>
	</Section>
</main>
