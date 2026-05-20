<script lang="ts">
import IntersectionObserver from 'svelte-intersection-observer';

let data = new Map<string, number>();
data.set('Harper', 10);
data.set('LanguageTool', 650);
data.set('Grammarly', 4000);

let maxW = 0;

for (let val of data.values()) {
	if (val > maxW) {
		maxW = val;
	}
}

let scaledData = new Map<string, number>();

for (let [key, val] of data.entries()) {
	scaledData.set(key, val / maxW);
}

let els: Record<string, HTMLElement> = {};

function expand(_node: HTMLElement, { width, duration }: { width: number; duration: number }) {
	return {
		duration,
		css: (t: number) => {
			return `width: ${width * 100 * t}%;`;
		},
	};
}
</script>

<div class="graph">
	{#each scaledData as [name, width] (name)}
		<IntersectionObserver element={els[name]} let:intersecting>
			<div bind:this={els[name]}>
				{#if intersecting}
					<div class="row">
						<span>{name}</span>
						<b>
							<i
								class:harper={name === 'Harper'}
								in:expand={{ width, duration: width * maxW }}
								style={`width: ${width * 100}%;`}
							></i>
						</b>
						<em>{width * maxW} ms</em>
					</div>
				{/if}
			</div>
		</IntersectionObserver>
	{/each}
</div>

<style>
.graph {
	display: flex;
	flex-direction: column;
	gap: 0.75rem;
	margin-top: auto;
}

.row {
	display: grid;
	grid-template-columns: 6.2rem 1fr 4rem;
	align-items: center;
	gap: 0.65rem;
	font-family: "JetBrains Mono", monospace;
	font-size: 0.7rem;
}

.row span {
	color: inherit;
	white-space: nowrap;
}

.row b {
	height: 0.62rem;
	border-radius: 999px;
	background: rgba(251, 250, 246, 0.08);
	overflow: hidden;
}

.row i {
	display: block;
	height: 100%;
	border-radius: inherit;
	background: rgba(251, 250, 246, 0.18);
}

.row i.harper {
	background: var(--marketing-amber-soft);
}

.row em {
	color: rgba(251, 250, 246, 0.75);
	font-style: normal;
	text-align: right;
}

@media (max-width: 420px) {
	.row {
		grid-template-columns: 1fr 3.5rem;
		gap: 0.45rem 0.65rem;
	}

	.row b {
		grid-column: 1 / -1;
		grid-row: 2;
	}
}
</style>
