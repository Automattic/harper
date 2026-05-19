<script lang="ts">
import MacWindow from './MacWindow.svelte';
import Squiggle from './Squiggle.svelte';
import Suggestion from './Suggestion.svelte';
</script>

<div class="stack" aria-label="Harper Desktop working in several macOS apps">
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
</div>

<style>
.stack {
	position: absolute;
	inset: 0;
}

.pane {
	position: absolute;
}

.slack {
	left: -0.6rem;
	bottom: 0;
	width: 20rem;
	transform: rotate(-4deg);
}

.mail {
	left: 4.4rem;
	top: 1.9rem;
	z-index: 3;
	width: 23.75rem;
	transform: rotate(1.5deg);
}

.notes {
	right: -0.6rem;
	top: 0.6rem;
	z-index: 2;
	width: 17.5rem;
	transform: rotate(3.5deg);
}

.discord {
	right: 0;
	bottom: 1.9rem;
	z-index: 4;
	width: 21.25rem;
	transform: rotate(-2deg);
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

.chat-body b {
	margin-right: 0.4rem;
}

.chat-body small {
	color: var(--marketing-ink-4);
}

.discord-body small {
	color: #8a8d93;
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

.notes-body small {
	display: block;
	margin-bottom: 0.55rem;
	color: var(--marketing-ink-4);
}

@media (max-width: 1040px) {
	.stack {
		position: relative;
		height: 32rem;
		max-width: 42rem;
		margin: 0 auto;
	}
}

@media (max-width: 620px) {
	.stack {
		height: auto;
		display: grid;
		gap: 0.9rem;
	}

	.pane {
		position: static;
		width: 100%;
		transform: none;
	}
}
</style>
