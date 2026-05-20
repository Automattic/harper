<script lang="ts">
import MacWindow from './MacWindow.svelte';
import Squiggle from './Squiggle.svelte';
import Suggestion from './Suggestion.svelte';
</script>

<div class="pane slack">
	<MacWindow title="Slack - #launches" dark>
		<div class="chat-body">
			<div class="avatar">J</div>
			<p>
				<b>Jordan</b><small>2:14 PM</small><br />
				heads up: the deploy is taking <Squiggle>alot</Squiggle> longer
				<Squiggle>then</Squiggle> expected. might push to <Squiggle>tommorrow</Squiggle>.
			</p>
			<Suggestion word="alot" fix="a lot" />
		</div>
	</MacWindow>
</div>

<style>
.pane {
	position: absolute;
}

.slack {
	left: -0.6rem;
	bottom: 0;
	width: 20rem;
	transform: rotate(-4deg);
}

.chat-body {
	display: flex;
	gap: 0.6rem;
	min-height: 9.6rem;
	padding: 0.75rem 0.85rem 3rem;
	font-size: 0.82rem;
}

.avatar {
	display: flex;
	flex-shrink: 0;
	align-items: center;
	justify-content: center;
	width: 1.65rem;
	height: 1.65rem;
	border-radius: 0.45rem;
	background: #4a154b;
	color: #fff;
	font-size: 0.75rem;
	font-weight: 800;
}

p {
	margin: 0;
	line-height: 1.55;
}

.chat-body b {
	margin-right: 0.4rem;
}

.chat-body small {
	color: var(--marketing-ink-4);
}

@media (max-width: 620px) {
	.pane {
		position: static;
		width: 100%;
		transform: none;
	}
}
</style>
