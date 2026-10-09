<script module lang="ts">
/** A timeseries observation with a valid Date and a finite numeric value. */
export interface TimeSeriesPoint {
	date: Date;
	value: number;
}
</script>

<script lang="ts">
import type { Point } from 'chart.js';
import Chart from 'chart.js/auto';
import { onMount } from 'svelte';

/** Props for a single-series chart. Dates are displayed in the viewer's local timezone. */
interface Props {
	/** Observations in any order; defaults to empty. Invalid dates and nonfinite values are omitted. */
	data?: TimeSeriesPoint[];
	title: string;
}

let { data = [], title }: Props = $props();

let chartCanvas = $state<HTMLCanvasElement>();
let chart: Chart<'line', Point[]> | null = null;
let points = $derived(toChartPoints(data));

const dateFormatter = new Intl.DateTimeFormat(undefined, {
	dateStyle: 'short',
	timeStyle: 'short',
});

/** Convert dates to milliseconds and sort chronologically without mutating input or merging duplicates. */
function toChartPoints(data: TimeSeriesPoint[]): Point[] {
	return data
		.map(({ date, value }) => ({ x: date.getTime(), y: value }))
		.filter(({ x, y }) => Number.isFinite(x) && Number.isFinite(y))
		.toSorted((a, b) => a.x - b.x);
}

onMount(() => {
	chart = new Chart(chartCanvas!, {
		type: 'line',
		data: {
			datasets: [
				{
					data: points,
					borderColor: 'rgba(80, 80, 80, 1)',
					backgroundColor: 'rgba(80, 80, 80, 1)',
					borderWidth: 2,
					pointRadius: 3,
					fill: false,
					tension: 0,
				},
			],
		},
		options: {
			responsive: true,
			maintainAspectRatio: false,
			plugins: {
				title: {
					display: true,
					text: title,
					color: '#444',
					font: {
						size: 18,
						weight: 'bold',
					},
				},
				legend: {
					display: false,
				},
				tooltip: {
					callbacks: {
						title: (items) =>
							items.length ? dateFormatter.format(items[0].parsed.x) : '',
					},
				},
			},
			scales: {
				x: {
					// Numeric timestamps preserve elapsed-time spacing without a date adapter.
					type: 'linear',
					grid: {
						color: '#ddd',
					},
					ticks: {
						callback: (value) => dateFormatter.format(Number(value)),
						maxTicksLimit: 6,
						color: '#333',
						font: {
							size: 14,
						},
					},
				},
				y: {
					beginAtZero: true,
					grid: {
						color: '#ddd',
					},
					ticks: {
						color: '#333',
						font: {
							size: 14,
						},
					},
				},
			},
		},
	});

	return () => {
		chart?.destroy();
		chart = null;
	};
});

/** Synchronize data and title on the existing chart rather than recreating its canvas resources. */
function updateChart(points: Point[], title: string) {
	if (!chart) return;

	chart.data.datasets[0].data = points;
	if (chart.options.plugins?.title) {
		chart.options.plugins.title.text = title;
	}
	chart.update();
}

$effect(() => {
	updateChart(points, title);
});
</script>

<style>
  .chart-container {
    background: #f9f9f9;
    border: 1px solid #ccc;
    border-radius: 8px;
    padding: 1rem;
    width: 100%;
    max-width: 700px;
    height: 400px;
    margin: 0 auto;
  }

  canvas {
    width: 100%;
    height: 100%;
  }
</style>

<div class="chart-container">
  <canvas bind:this={chartCanvas} aria-label={title}>{title}</canvas>
</div>
