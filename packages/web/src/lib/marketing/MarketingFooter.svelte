<script lang="ts">
import { marketingLinks } from './data';
import HarperMark from './HarperMark.svelte';
</script>

<footer class="marketing-footer">
	<div class="top">
		<div>
			<div class="brand">
				<HarperMark size={26} />
				<strong>Harper</strong>
			</div>
			<p>The free, private, open-source grammar checker that lives on your device.</p>
			<div class="license"><span></span>Apache-2.0 · v1.0</div>
		</div>

		<div class="col">
			<h2>Get Harper</h2>
			<a href="/desktop">Harper Desktop</a>
			<a href={marketingLinks.chrome}>Chrome extension</a>
			<a href={marketingLinks.vscode}>VS Code</a>
			<a href="/docs/integrations/obsidian">Obsidian plugin</a>
			<a href="/get">All integrations</a>
		</div>

		<div class="col">
			<h2>Resources</h2>
			<a href="/docs/about">Documentation</a>
			<a href="/docs/rules">Rule reference</a>
			<a href="/docs/integrations/language-server">harper-ls</a>
			<a href="/docs/harperjs/introduction">harper.js</a>
			<a href={marketingLinks.crates}>harper-core</a>
		</div>

		<div class="col">
			<h2>Community</h2>
			<a href={marketingLinks.github}>GitHub</a>
			<a href={marketingLinks.discord}>Discord</a>
			<a href="/docs/contributors/introduction">Contributing</a>
		</div>
	</div>

	<div class="bottom">
		<span>An <a href="https://automattic.com/">Automattic</a> open-source project.</span>
		<div>
			<a href="/docs/about">Privacy</a>
			<a href={marketingLinks.github}>License</a>
		</div>
	</div>
</footer>

<style>
.marketing-footer {
	border-top: 0.5px solid var(--marketing-line);
	background: var(--marketing-cream);
	color: var(--marketing-ink-3);
	font-family: inherit;
}

.top {
	display: grid;
	grid-template-columns: 1.4fr 1fr 1fr 1fr;
	gap: 2.5rem;
	max-width: 77.5rem;
	margin: 0 auto;
	padding: 3rem 2.5rem 1.75rem;
}

.brand {
	display: flex;
	align-items: center;
	gap: 0.65rem;
	color: var(--marketing-ink);
}

.brand strong {
	font-size: 0.9rem;
}

p {
	max-width: 18rem;
	margin: 0.9rem 0 1rem;
	color: var(--marketing-ink-2);
	font-size: 0.84rem;
	line-height: 1.55;
}

.license {
	display: inline-flex;
	align-items: center;
	gap: 0.5rem;
	padding: 0.4rem 0.65rem;
	border: 0.5px solid var(--marketing-line);
	border-radius: 999px;
	background: #fff;
	color: var(--marketing-ink-2);
	font-family: "JetBrains Mono", monospace;
	font-size: 0.68rem;
}

.license span {
	width: 0.45rem;
	height: 0.45rem;
	border-radius: 999px;
	background: var(--marketing-grass);
}

.col {
	display: flex;
	flex-direction: column;
	gap: 0.65rem;
}

h2 {
	margin: 0 0 0.25rem;
	color: var(--marketing-amber);
	font-family: "JetBrains Mono", monospace;
	font-size: 0.66rem;
	font-weight: 800;
	letter-spacing: 0.14em;
	text-transform: uppercase;
}

a {
	color: var(--marketing-ink-2);
	font-size: 0.84rem;
	text-decoration: none;
}

a:hover {
	color: var(--marketing-ink);
	text-decoration: none;
}

.bottom {
	display: flex;
	justify-content: space-between;
	align-items: center;
	gap: 1rem;
	max-width: 77.5rem;
	margin: 0 auto;
	padding: 1.1rem 2.5rem;
	border-top: 0.5px solid var(--marketing-line);
	font-size: 0.78rem;
}

.bottom div {
	display: flex;
	gap: 1rem;
}

@media (max-width: 820px) {
	.top {
		grid-template-columns: 1fr 1fr;
		padding-inline: 1rem;
	}

	.bottom {
		flex-direction: column;
		align-items: flex-start;
		padding-inline: 1rem;
	}
}

@media (max-width: 560px) {
	.top {
		grid-template-columns: 1fr;
	}
}
</style>
