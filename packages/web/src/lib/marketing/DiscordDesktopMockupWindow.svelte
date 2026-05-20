<script lang="ts">
import MacWindow from './MacWindow.svelte';
import Squiggle from './Squiggle.svelte';
import Suggestion from './Suggestion.svelte';
</script>

<div class="pane discord">
	<MacWindow title="Discord - #design-crit" dark>
		<div class="chat-body discord-body">
			<div class="avatar">R</div>
			<p>
				<b>riley</b><small>Today at 2:14 PM</small><br />
				not sure if its <Squiggle>worht</Squiggle> shipping yet, the copy still needs another
				<Squiggle>passs</Squiggle> imo
			</p>
			<Suggestion word="worht" fix="worth" />
		</div>
	</MacWindow>
</div>

<style>
.pane {
	position: absolute;
}

.discord {
	right: 0;
	bottom: 1.9rem;
	z-index: 4;
	width: 21.25rem;
	transform: rotate(-2deg);
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

.discord-body .avatar {
	border-radius: 999px;
	background: #5865f2;
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

.discord-body small {
	color: #8a8d93;
}

@media (max-width: 620px) {
	.pane {
		position: static;
		width: 100%;
		transform: none;
	}
}
</style>
