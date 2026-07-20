<script lang="ts">
import Chart from 'chart.js/auto';
import { onMount } from 'svelte';

interface Props {
	data?: Record<string, number>;
	label: string;
	title: string;
}

let { data = {}, label, title }: Props = $props();

let chartCanvas = $state<HTMLCanvasElement>();
let chart: Chart | null = null;

let sortedEntries = $derived(Object.entries(data).toSorted(([_a, a], [_b, b]) => b - a));
let keys = $derived(sortedEntries.map(([a]) => a));
let values = $derived(sortedEntries.map(([, b]) => b));

onMount(() => {
	// Create a new Chart.js bar chart on mount
	chart = new Chart(chartCanvas!, {
		type: 'bar',
		data: {
			labels: keys,
			datasets: [
				{
					label,
					data: values,
					borderColor: 'rgba(80, 80, 80, 1)',
					borderWidth: 2,
					borderRadius: 6, // Rounded corners
					barPercentage: 0.6, // Thicker bars
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
					color: '#444', // Dark gray text
					font: {
						size: 18,
						weight: 'bold',
					},
				},
				legend: {
					display: false,
				},
			},
			scales: {
				x: {
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
				y: {
					beginAtZero: true,
					grid: {
						color: '#ddd',
					},
					ticks: {
						stepSize: 1,
						color: '#333',
						font: {
							size: 14,
						},
					},
				},
			},
		},
	});
});

// Update the chart data with new lint counts
function updateChart() {
	if (chart) {
		chart.data.labels = keys;
		chart.data.datasets[0].data = values;
		chart.update();
	}
}

// Whenever data changes, update the chart
$effect(() => {
	if (data) {
		updateChart();
	}
});
</script>


<style>
  /* Wrap the chart in a container to control layout and background */
  .chart-container {
    background: #f9f9f9;       /* Subtle off-white background */
    border: 1px solid #ccc;    /* Light gray border */
    border-radius: 8px;        /* Rounded corners */
    padding: 1rem;
    width: 100%;
    max-width: 700px;          /* Adjust as needed */
    height: 400px;             /* Fixed height for the chart area */
    margin: 0 auto;            /* Center horizontally */
  }

  canvas {
    width: 100%;
    height: 100%;
  }
</style>

<div class="chart-container">
  <canvas bind:this={chartCanvas}></canvas>
</div>
