<script lang="ts">
import MacWindow from './MacWindow.svelte';
import Squiggle from './Squiggle.svelte';
import Suggestion from './Suggestion.svelte';
</script>

<div class="pane mail">
	<MacWindow title="Mail - Compose">
		<div class="mail-body">
			<div class="meta"><span>To:</span> design@example.com</div>
			<h3>Re: Q3 marketing review</h3>
			<p>
				Thanks for sending these over, really <Squiggle>apreciate</Squiggle> the detail. I'll
				loop in the design team and we can have a draft back to you by
				<Squiggle color="#3a8dde">friday</Squiggle>.
			</p>
			<Suggestion word="apreciate" fix="appreciate" />
		</div>
	</MacWindow>
</div>

<style>
.pane {
	position: absolute;
}

.mail {
	left: 4.4rem;
	top: 1.9rem;
	z-index: 3;
	width: 23.75rem;
	transform: rotate(1.5deg);
}

.mail-body {
	padding: 0.8rem 0.9rem 3rem;
	font-size: 0.82rem;
}

.meta {
	display: flex;
	gap: 0.4rem;
	margin-bottom: 0.35rem;
	color: var(--marketing-ink-2);
	font-size: 0.72rem;
}

.meta span {
	color: var(--marketing-ink-4);
}

h3 {
	margin: 0 0 0.65rem;
	padding-bottom: 0.5rem;
	border-bottom: 0.5px solid var(--marketing-line);
	color: var(--marketing-ink);
	font-size: 0.84rem;
}

p {
	margin: 0;
	line-height: 1.55;
}

@media (max-width: 620px) {
	.pane {
		position: static;
		width: 100%;
		transform: none;
	}
}
</style>
