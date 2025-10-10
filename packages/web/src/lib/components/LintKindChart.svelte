<script lang="ts">
import lintKindColor from '$lib/lintKindColor';
import Chart from 'chart.js/auto';
import { onMount } from 'svelte';

// Receive lint counts from the parent component
export let lintCounts: Record<string, number> = {};

let chartCanvas: HTMLCanvasElement;
let lintChart: Chart | null = null;

// Update the chart data with new lint counts
function updateChart() {
	if (lintChart) {
		lintChart.data.labels = Object.keys(lintCounts);
		lintChart.data.datasets[0].data = Object.values(lintCounts);
		lintChart.update();
	}
}

onMount(() => {
	// Create a new Chart.js bar chart on mount
	lintChart = new Chart(chartCanvas, {
		type: 'bar',
		data: {
			labels: Object.keys(lintCounts),
			datasets: [
				{
					label: 'Number of Corrections Applied',
					data: Object.values(lintCounts),
					backgroundColor: Object.keys(lintCounts).map(lintKindColor),
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
					text: 'Most Common Kinds of Corrections',
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

// Whenever lintCounts changes, update the chart
$: if (lintChart) {
	updateChart();
}
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
