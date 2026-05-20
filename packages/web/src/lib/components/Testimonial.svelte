<script lang="ts">
export let authorName: string;
export let authorSubtitle: string;
export let testimonial: string;
export let accent = false;

function initials(name: string) {
	return name
		.split(' ')
		.map((part) => part[0])
		.slice(0, 2)
		.join('');
}
</script>

<figure class:accent>
	<blockquote>"{testimonial}"</blockquote>
	<figcaption>
		<span>{initials(authorName)}</span>
		<div>
			<strong>{authorName}</strong>
			<small>{authorSubtitle}</small>
		</div>
	</figcaption>
</figure>

<style>
figure {
	display: flex;
	min-height: 16rem;
	flex-direction: column;
	justify-content: space-between;
	gap: 1rem;
	margin: 0;
	border: 0.5px solid var(--marketing-line);
	border-radius: 1rem;
	background: #fff;
	padding: 1.5rem;
	transition:
		transform 120ms ease,
		box-shadow 120ms ease,
		border-color 120ms ease;
}

figure.accent {
	border-color: var(--marketing-ink);
	background: var(--marketing-ink);
	color: #fbfaf6;
}

blockquote {
	margin: 0;
	border: 0;
	background: transparent;
	padding: 0;
	color: inherit;
	font-size: 0.94rem;
	font-style: normal;
	line-height: 1.55;
	text-indent: 0;
}

figure.accent blockquote {
	font-family: var(--marketing-display);
	font-size: 1.35rem;
	line-height: 1.3;
}

figcaption {
	display: flex;
	align-items: center;
	gap: 0.75rem;
	border-top: 0.5px solid var(--marketing-line);
	padding-top: 0.9rem;
}

figure.accent figcaption {
	border-color: rgba(251, 250, 246, 0.16);
}

figcaption > span {
	display: flex;
	width: 2rem;
	height: 2rem;
	align-items: center;
	justify-content: center;
	border-radius: 999px;
	background: var(--marketing-amber-tint);
	color: var(--marketing-amber);
	font-size: 0.75rem;
	font-weight: 800;
}

figcaption div {
	display: flex;
	flex-direction: column;
}

figcaption strong {
	font-size: 0.82rem;
}

figcaption small {
	color: var(--marketing-ink-3);
	font-size: 0.72rem;
}

figure.accent figcaption small {
	color: rgba(251, 250, 246, 0.65);
}
</style>
