<script lang="ts">
import AppLogoTile from '$lib/marketing/AppLogoTile.svelte';
import DesktopMockups from '$lib/marketing/DesktopMockups.svelte';
import FaqSection from '$lib/marketing/FaqSection.svelte';
import { compatibilityApps } from '$lib/marketing/data';
import HarperMark from '$lib/marketing/HarperMark.svelte';
import MarketingFooter from '$lib/marketing/MarketingFooter.svelte';
import MarketingHeader from '$lib/marketing/MarketingHeader.svelte';
import PillButton from '$lib/marketing/PillButton.svelte';
import PrivacySpeedCards from '$lib/marketing/PrivacySpeedCards.svelte';
import SectionKicker from '$lib/marketing/SectionKicker.svelte';

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

<div class="marketing-page">
	<MarketingHeader active="desktop" />

	<section class="hero">
		<div class="hero-inner">
			<div class="copy">
				<SectionKicker>Harper Desktop</SectionKicker>
				<h1>Works in every app.</h1>
				<p>
					Use Harper in the apps you use every day such as Mail, Slack, Notes, Pages, and
					Discord.
				</p>
				<div class="cta">
					<PillButton disabled size="lg">
						<svg slot="icon" viewBox="0 0 24 24" aria-hidden="true">
							<path
								d="M17.05 12.4c-.04-3.1 2.53-4.6 2.65-4.66-1.45-2.1-3.69-2.4-4.49-2.43-1.91-.2-3.73 1.13-4.7 1.13-.97 0-2.46-1.1-4.06-1.07-2.09.03-4.02 1.22-5.09 3.08-2.17 3.76-.55 9.31 1.55 12.37 1.03 1.5 2.24 3.17 3.84 3.12 1.56-.06 2.14-1 4.02-1s2.42 1 4.06.97c1.68-.02 2.74-1.5 3.76-3.02 1.19-1.73 1.67-3.4 1.7-3.5-.04-.02-3.27-1.26-3.3-4.99zM14.05 3.95c.85-1.04 1.43-2.48 1.27-3.91-1.23.05-2.72.82-3.6 1.86-.79.92-1.49 2.4-1.3 3.81 1.37.1 2.78-.7 3.63-1.76z"
							/>
						</svg>
						Download for macOS
					</PillButton>
					<span>Beta coming soon · macOS 14+ · Apple Silicon &amp; Intel</span>
				</div>
			</div>
			<div class="scene">
				<DesktopMockups />
			</div>
		</div>
	</section>

	<section class="compatibility">
		<div class="section-inner">
			<div class="center-heading">
				<SectionKicker>Write in every app</SectionKicker>
				<h2>Every text field. Same Harper.</h2>
				<p>
					If your cursor is blinking, Harper is watching. Harper Desktop hooks into macOS text
					input so the same suggestions follow you across the apps where you write.
				</p>
			</div>
			<div class="app-grid" aria-label="Apps where Harper Desktop works">
				{#each compatibilityApps as app}
					<div>
						<AppLogoTile id={app.id} size={60} />
						<strong>{app.name}</strong>
					</div>
				{/each}
			</div>
			<p class="compat-note">...and every other text field on macOS. If you can type in it, Harper helps.</p>
		</div>
	</section>

	<section class="privacy-speed">
		<div class="section-inner">
			<PrivacySpeedCards desktop />
		</div>
	</section>

	<section class="settings">
		<div class="section-inner settings-grid">
			<div>
				<SectionKicker>Yours to shape</SectionKicker>
				<h2>Tune the rules. Build your own dictionary.</h2>
				<p>
					Every rule in Harper is toggleable. Add team jargon, product names, and favorite
					phrases to your personal dictionary, and they stop being underlined everywhere on your
					Mac.
				</p>
				<ul>
					<li><span></span><b>Pick your English.</b> British, American, Canadian, Australian, or Indian.</li>
					<li><span></span><b>Enable per app.</b> Grammar checking only where you want it.</li>
					<li><span></span><b>Make the rules.</b> Build custom rules using Weirpacks.</li>
				</ul>
			</div>
			<div class="screenshot-window">
				<div class="titlebar"><span></span><span></span><span></span><strong>Harper - Settings</strong></div>
				<img src="/marketing/screenshots/settings-rules.png" alt="Harper Desktop settings showing rule toggles" />
			</div>
		</div>
	</section>

	<section class="download">
		<div class="section-inner narrow">
			<HarperMark size={64} />
			<h2>Free. Forever. Yours.</h2>
			<p>
				Harper Desktop will be free, like the rest of Harper. No account or credit card necessary.
			</p>
			<PillButton disabled size="lg">Beta coming soon</PillButton>
			<div class="requirements">macOS 14 Sonoma or later · Apple Silicon &amp; Intel</div>
		</div>
	</section>

	<FaqSection items={faqs} title="FAQs" layout="narrow" />

	<MarketingFooter />
</div>

<style>
.marketing-page {
	min-height: 100vh;
	background: #f6efe2;
	color: var(--marketing-ink);
	font-family: var(--marketing-sans);
}

.section-inner {
	max-width: 68.75rem;
	margin: 0 auto;
	padding: 0 3.5rem;
}

.section-inner.narrow {
	max-width: 45rem;
	text-align: center;
}

.hero {
	position: relative;
	overflow: hidden;
	padding: 0.75rem 3.5rem 3.5rem;
}

.hero::after,
.download::after {
	position: absolute;
	inset: -20% -10%;
	content: '';
	background: radial-gradient(ellipse at 70% 20%, var(--marketing-amber-tint) 0%, transparent 60%);
	pointer-events: none;
}

.hero-inner {
	position: relative;
	z-index: 1;
	display: grid;
	grid-template-columns: minmax(0, 1fr) minmax(32rem, 1.2fr);
	align-items: center;
	gap: 2rem;
	max-width: 77.5rem;
	margin: 0 auto;
	padding-top: 1.9rem;
}

h1,
h2 {
	margin: 0;
	color: inherit;
	font-family: var(--marketing-display);
	font-weight: 650;
	letter-spacing: 0;
}

h1 {
	margin-top: 0.85rem;
	font-size: clamp(3.2rem, 6vw, 3.9rem);
	line-height: 1.02;
}

.copy p,
.center-heading p,
.settings p,
.download p {
	color: var(--marketing-ink-2);
	font-size: 1rem;
	line-height: 1.6;
}

.copy p {
	max-width: 28.75rem;
	margin: 1.25rem 0 0;
}

.cta {
	display: flex;
	flex-direction: column;
	align-items: flex-start;
	gap: 0.6rem;
	margin-top: 1.5rem;
}

.cta svg {
	width: 0.95rem;
	height: 0.95rem;
	fill: currentColor;
}

.cta span,
.requirements {
	color: var(--marketing-ink-3);
	font-size: 0.75rem;
}

.scene {
	position: relative;
	height: 33.75rem;
}

.compatibility,
.settings {
	border-top: 0.5px solid var(--marketing-line);
	background: #fdfbf5;
	padding: 4.5rem 0 5rem;
}

.privacy-speed,
.download {
	position: relative;
	overflow: hidden;
	border-top: 0.5px solid var(--marketing-line);
	background: var(--marketing-cream);
	padding: 4.5rem 0;
}

.privacy-speed > .section-inner,
.download > .section-inner {
	position: relative;
	z-index: 1;
}

.center-heading {
	max-width: 40rem;
	margin: 0 auto;
	text-align: center;
}

.center-heading h2,
.settings h2,
.download h2 {
	margin-top: 0.75rem;
	font-size: clamp(2.1rem, 5vw, 2.45rem);
	line-height: 1.1;
}

.app-grid {
	display: grid;
	grid-template-columns: repeat(6, minmax(0, 1fr));
	justify-items: center;
	gap: 1.4rem;
	margin-top: 2.75rem;
}

.app-grid div {
	display: flex;
	align-items: center;
	flex-direction: column;
	gap: 0.5rem;
}

.app-grid strong {
	color: var(--marketing-ink-2);
	font-size: 0.75rem;
}

.compat-note {
	margin: 2.25rem 0 0;
	color: var(--marketing-ink-3);
	font-size: 0.82rem;
	text-align: center;
}

.settings-grid {
	display: grid;
	grid-template-columns: minmax(0, 1fr) minmax(22rem, 1.1fr);
	align-items: center;
	gap: 3.5rem;
}

ul {
	display: flex;
	flex-direction: column;
	gap: 0.65rem;
	margin: 1.25rem 0 0;
	padding: 0;
	list-style: none;
}

li {
	display: flex;
	gap: 0.65rem;
	color: var(--marketing-ink-2);
	line-height: 1.5;
}

li span {
	width: 0.4rem;
	height: 0.4rem;
	flex-shrink: 0;
	border-radius: 999px;
	margin-top: 0.55rem;
	background: var(--marketing-amber);
}

li b {
	color: var(--marketing-ink);
}

.screenshot-window {
	overflow: hidden;
	border: 0.5px solid rgba(28, 26, 22, 0.18);
	border-radius: 0.85rem;
	background: #fff;
	box-shadow:
		0 22px 50px -18px rgba(28, 26, 22, 0.35),
		0 4px 10px rgba(28, 26, 22, 0.1);
}

.titlebar {
	display: flex;
	align-items: center;
	gap: 0.35rem;
	height: 1.65rem;
	border-bottom: 0.5px solid rgba(28, 26, 22, 0.08);
	background: linear-gradient(180deg, #fafafa, #efece7);
	padding: 0 0.65rem;
}

.titlebar span {
	width: 0.55rem;
	height: 0.55rem;
	border-radius: 999px;
}

.titlebar span:nth-child(1) {
	background: #ff5f57;
}

.titlebar span:nth-child(2) {
	background: #febc2e;
}

.titlebar span:nth-child(3) {
	background: #28c840;
}

.titlebar strong {
	flex: 1;
	margin-right: 2rem;
	color: rgba(28, 26, 22, 0.56);
	font-size: 0.68rem;
	text-align: center;
}

.screenshot-window img {
	display: block;
	width: 100%;
	height: auto;
}

.download :global(.harper-mark) {
	margin: 0 auto;
}

.download p {
	margin-bottom: 1.6rem;
}

.requirements {
	margin-top: 1rem;
	font-family: var(--marketing-mono);
}

@media (max-width: 1040px) {
	.hero-inner,
	.settings-grid {
		grid-template-columns: 1fr;
	}

	.scene {
		height: auto;
	}

	.app-grid {
		grid-template-columns: repeat(4, minmax(0, 1fr));
	}
}

@media (max-width: 700px) {
	.hero,
	.section-inner {
		padding-inline: 1rem;
	}

	.app-grid {
		grid-template-columns: repeat(3, minmax(0, 1fr));
		gap: 1rem;
	}
}
</style>
