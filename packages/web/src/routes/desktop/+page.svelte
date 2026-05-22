<script lang="ts">
import AppLogoTile from '$lib/marketing/AppLogoTile.svelte';
import DesktopMockups from '$lib/marketing/DesktopMockups.svelte';
import { compatibilityApps } from '$lib/marketing/data';
import HarperMark from '$lib/marketing/HarperMark.svelte';
import MarketingFooter from '$lib/marketing/MarketingFooter.svelte';
import MarketingHeader from '$lib/marketing/MarketingHeader.svelte';
import PillButton from '$lib/marketing/PillButton.svelte';
import PrivacySpeedCards from '$lib/marketing/PrivacySpeedCards.svelte';
import SimpleFAQ from '$lib/marketing/SimpleFAQ.svelte';

const faqs = [
	{
		q: 'Does Harper Desktop send my writing anywhere?',
		a: 'No. Like the rest of Harper, everything runs locally. No accounts, no telemetry, no snooping servers.',
	},
	{
		q: 'What are the system requirements?',
		a: 'macOS 14 Sonoma or later. Apple Silicon and Intel are both supported.',
	},
	{
		q: 'Is it really free?',
		a: 'Yes. Harper Desktop is free, like the rest of Harper. The source is on GitHub under Apache-2.0.',
	},
	{
		q: 'What about Windows and Linux?',
		a: 'Not yet. We started on macOS to get the system-wide integration right; other platforms are on the roadmap.',
	},
];
</script>

<svelte:head>
	<title>Harper Desktop</title>
	<meta
		name="description"
		content="Harper Desktop brings private grammar checking to text fields across macOS."
	/>
</svelte:head>

<div class="min-h-screen bg-[#f6efe2] text-[#1c1a16] dark:bg-black dark:text-white">
	<MarketingHeader active="desktop" />

	<section class="relative overflow-hidden px-14 pt-3 pb-14 after:pointer-events-none after:absolute after:inset-[-20%_-10%] after:content-[''] after:bg-[radial-gradient(ellipse_at_70%_20%,#fbf3e6_0%,transparent_60%)] dark:after:opacity-10 max-[700px]:px-4">
		<div class="relative z-[1] mx-auto grid max-w-[77.5rem] grid-cols-[minmax(0,1fr)_minmax(32rem,1.2fr)] items-center gap-8 pt-[1.9rem] max-[1040px]:grid-cols-1">
			<div>
				<h1 class="!mt-[0.85rem] !mb-0 py-0 font-serif text-[clamp(3.2rem,6vw,3.9rem)] font-[650] leading-[1.02] tracking-normal text-inherit">
					Works in every app.
				</h1>
				<p class="!mt-5 !mb-0 max-w-[28.75rem] text-base leading-[1.6] text-[#4a463e] dark:text-white/70">
					Use Harper Desktop in the apps you use every day such as Mail, Slack, Notes, Pages,
					and Discord.
				</p>
				<div class="mt-6 flex flex-col items-start gap-[0.6rem] [&_svg]:size-[0.95rem] [&_svg]:fill-current">
					<PillButton href="/desktop/download" size="lg">
						<svg slot="icon" viewBox="0 0 24 24" aria-hidden="true">
							<path
								d="M17.05 12.4c-.04-3.1 2.53-4.6 2.65-4.66-1.45-2.1-3.69-2.4-4.49-2.43-1.91-.2-3.73 1.13-4.7 1.13-.97 0-2.46-1.1-4.06-1.07-2.09.03-4.02 1.22-5.09 3.08-2.17 3.76-.55 9.31 1.55 12.37 1.03 1.5 2.24 3.17 3.84 3.12 1.56-.06 2.14-1 4.02-1s2.42 1 4.06.97c1.68-.02 2.74-1.5 3.76-3.02 1.19-1.73 1.67-3.4 1.7-3.5-.04-.02-3.27-1.26-3.3-4.99zM14.05 3.95c.85-1.04 1.43-2.48 1.27-3.91-1.23.05-2.72.82-3.6 1.86-.79.92-1.49 2.4-1.3 3.81 1.37.1 2.78-.7 3.63-1.76z"
							/>
						</svg>
						Download for macOS
					</PillButton>
					<span class="text-xs text-[#807a6e] dark:text-white/55">macOS 14+ · Apple Silicon &amp; Intel</span>
				</div>
			</div>
			<div class="relative h-[33.75rem] max-[1040px]:h-auto">
				<DesktopMockups />
			</div>
		</div>
	</section>

	<section class="border-t-[0.5px] border-[rgba(28,26,22,0.1)] bg-[#fdfbf5] py-[4.5rem] pb-20 dark:border-white/10 dark:bg-black">
		<div class="mx-auto max-w-[68.75rem] px-14 max-[700px]:px-4">
			<div class="mx-auto max-w-[40rem] text-center">
				<h2 class="!mt-3 !mb-0 py-0 font-serif text-[clamp(2.1rem,5vw,2.45rem)] font-[650] leading-[1.1] tracking-normal text-inherit">
					Every text field. Same Harper.
				</h2>
				<p class="!mt-6 !mb-0 text-base leading-[1.6] text-[#4a463e] dark:text-white/70">
					If your cursor is blinking, Harper is watching. Harper Desktop hooks into macOS text
					inputs so the same suggestions follow you across the apps where you write.
				</p>
			</div>
			<div class="mt-11 grid grid-cols-6 justify-items-center gap-[1.4rem] max-[1040px]:grid-cols-4 max-[700px]:grid-cols-3 max-[700px]:gap-4" aria-label="Apps where Harper Desktop works">
				{#each compatibilityApps as app}
					<div class="flex flex-col items-center gap-2">
						<AppLogoTile id={app.id} size={60} />
						<strong class="text-xs text-[#4a463e] dark:text-white/70">{app.name}</strong>
					</div>
				{/each}
			</div>
			<p class="!mt-9 !mb-0 text-center text-[0.82rem] text-[#807a6e] dark:text-white/55">...and every other text field on macOS. If you can type in it, Harper helps.</p>
		</div>
	</section>

	<section class="relative overflow-hidden border-t-[0.5px] border-[rgba(28,26,22,0.1)] bg-[#fbfaf6] py-[4.5rem] dark:border-white/10 dark:bg-black">
		<div class="relative z-[1] mx-auto max-w-[68.75rem] px-14 max-[700px]:px-4">
			<PrivacySpeedCards desktop />
		</div>
	</section>

	<section class="border-t-[0.5px] border-[rgba(28,26,22,0.1)] bg-[#fdfbf5] py-[4.5rem] pb-20 dark:border-white/10 dark:bg-black">
		<div class="mx-auto grid max-w-[68.75rem] grid-cols-[minmax(0,1fr)_minmax(22rem,1.1fr)] items-center gap-14 px-14 max-[1040px]:grid-cols-1 max-[700px]:px-4">
			<div>
				<h2 class="!mt-3 !mb-0 py-0 font-serif text-[clamp(2.1rem,5vw,2.45rem)] font-[650] leading-[1.1] tracking-normal text-inherit">
					Tune the rules. Build your own dictionary.
				</h2>
				<p class="!mt-6 !mb-0 text-base leading-[1.6] text-[#4a463e] dark:text-white/70">
					Every rule in Harper is toggleable. Add team jargon, product names, and favorite
					phrases to your personal dictionary, and they stop being underlined everywhere on your
					Mac.
				</p>
				<ul class="mt-5 flex !list-none flex-col gap-[0.65rem] !p-0">
					<li class="flex gap-[0.65rem] leading-[1.5] text-[#4a463e] dark:text-white/70"><span class="mt-[0.55rem] size-[0.4rem] shrink-0 rounded-full bg-[#b06a1b]"></span><span><b class="text-[#1c1a16] dark:text-white">Pick your English.</b> British, American, Canadian, Australian, or Indian.</span></li>
					<li class="flex gap-[0.65rem] leading-[1.5] text-[#4a463e] dark:text-white/70"><span class="mt-[0.55rem] size-[0.4rem] shrink-0 rounded-full bg-[#b06a1b]"></span><span><b class="text-[#1c1a16] dark:text-white">Enable per app.</b> Grammar checking only where you want it.</span></li>
					<li class="flex gap-[0.65rem] leading-[1.5] text-[#4a463e] dark:text-white/70"><span class="mt-[0.55rem] size-[0.4rem] shrink-0 rounded-full bg-[#b06a1b]"></span><span><b class="text-[#1c1a16] dark:text-white">Make the rules.</b> Build custom rules using Weirpacks.</span></li>
				</ul>
			</div>
			<div class="overflow-hidden rounded-[0.85rem] border-[0.5px] border-[rgba(28,26,22,0.18)] bg-white shadow-[0_22px_50px_-18px_rgba(28,26,22,0.35),0_4px_10px_rgba(28,26,22,0.1)] dark:border-white/15 dark:bg-white/5">
				<div class="flex h-[1.65rem] items-center gap-[0.35rem] border-b-[0.5px] border-[rgba(28,26,22,0.08)] bg-[linear-gradient(180deg,#fafafa,#efece7)] px-[0.65rem]">
					<span class="size-[0.55rem] rounded-full bg-[#ff5f57]"></span><span class="size-[0.55rem] rounded-full bg-[#febc2e]"></span><span class="size-[0.55rem] rounded-full bg-[#28c840]"></span><strong class="mr-8 flex-1 text-center text-[0.68rem] text-[rgba(28,26,22,0.56)]">Harper - Settings</strong>
				</div>
				<img class="block h-auto w-full" src="/marketing/screenshots/settings-rules.png" alt="Harper Desktop settings showing rule toggles" />
			</div>
		</div>
	</section>

	<section class="relative overflow-hidden border-t-[0.5px] border-[rgba(28,26,22,0.1)] bg-[#fbfaf6] py-[4.5rem] after:pointer-events-none after:absolute after:inset-[-20%_-10%] after:content-[''] after:bg-[radial-gradient(ellipse_at_70%_20%,#fbf3e6_0%,transparent_60%)] dark:border-white/10 dark:bg-black dark:after:opacity-10">
		<div class="relative z-[1] mx-auto max-w-[45rem] px-14 text-center max-[700px]:px-4 [&_.harper-mark]:mx-auto">
			<HarperMark size={64} />
			<h2 class="!mt-3 !mb-0 py-0 font-serif text-[clamp(2.1rem,5vw,2.45rem)] font-[650] leading-[1.1] tracking-normal text-inherit">
				Free. Forever. Yours.
			</h2>
			<p class="!mt-6 !mb-[1.6rem] text-base leading-[1.6] text-[#4a463e] dark:text-white/70">
				Harper Desktop is free, like the rest of Harper. No account or credit card necessary.
			</p>
			<PillButton href="/desktop/download" size="lg">Download for macOS</PillButton>
			<div class='mt-4 text-xs text-[#807a6e] dark:text-white/55 [font-family:"JetBrains_Mono",monospace]'>macOS 14 Sonoma or later · Apple Silicon &amp; Intel</div>
		</div>
	</section>

	<SimpleFAQ items={faqs} title="FAQs" />

	<MarketingFooter />
</div>
