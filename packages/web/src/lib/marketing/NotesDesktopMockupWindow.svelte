<script lang="ts">
import MacWindow from './MacWindow.svelte';
import Squiggle from './Squiggle.svelte';
import Suggestion from './Suggestion.svelte';
</script>

<div class="pane notes">
	<MacWindow title="Notes">
		<div class="notes-body">
			<aside>
				<strong>Birthday dinner</strong>
				<span>Reserve table at Aug...</span>
			</aside>
			<section>
				<h3>Birthday dinner - saturday</h3>
				<small>Today at 11:08 AM</small>
				<p>
					Reserve table at Augusto's for 8 pm. Pick up a cake on the way,
					<Squiggle>noone</Squiggle> is allergic to nuts.
				</p>
				<Suggestion word="noone" fix="no one" />
			</section>
		</div>
	</MacWindow>
</div>

<style>
.pane {
	position: absolute;
}

.notes {
	right: -0.6rem;
	top: 0.6rem;
	z-index: 2;
	width: 17.5rem;
	transform: rotate(3.5deg);
}

.notes-body {
	display: flex;
	min-height: 11.5rem;
	background: #fffdf5;
}

.notes-body aside {
	display: flex;
	width: 5.75rem;
	flex-shrink: 0;
	flex-direction: column;
	gap: 0.2rem;
	border-right: 0.5px solid rgba(28, 26, 22, 0.08);
	background: #f6f0d8;
	padding: 0.6rem 0.4rem;
	font-size: 0.62rem;
}

.notes-body aside strong {
	border-radius: 0.3rem;
	background: #febc2e;
	padding: 0.35rem;
	color: #3a2a08;
}

.notes-body aside span {
	color: var(--marketing-ink-3);
}

.notes-body section {
	position: relative;
	padding: 0.7rem 0.9rem 3rem;
	font-size: 0.78rem;
}

h3 {
	margin: 0 0 0.65rem;
	padding-bottom: 0.5rem;
	border-bottom: 0.5px solid var(--marketing-line);
	color: var(--marketing-ink);
	font-size: 0.84rem;
}

.notes-body small {
	display: block;
	margin-bottom: 0.55rem;
	color: var(--marketing-ink-4);
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
