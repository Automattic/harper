<script lang="ts">
import { onMount } from 'svelte';

let logoImg: HTMLImageElement | null = $state(null);

onMount(() => {
	logoImg = new Image();
	logoImg.src = '/harper-logo.svg';
});

let canvasElement: HTMLCanvasElement | null = $state(null);

$effect(() => {
	let ctx = canvasElement?.getContext('2d');
	if (ctx != null) {
		frameLoop(ctx);
	}
});

function frameLoop(ctx: CanvasRenderingContext2D) {
	render(ctx);
	requestAnimationFrame(() => frameLoop(ctx));
}

type ParticleState = {
	x: number;
	y: number;
	vx: number;
	vy: number;
	letter: string;
};

let particles: ParticleState[] = [];
let currentFacePosition = [0, 0];
let nextFacePosition = [0, 0];

/** Ensures that there are N number of particles by spawning new ones. */
function spawnParticles(ctx: CanvasRenderingContext2D, n: number) {
	let alphabet = 'abcdefghijklmnopqrstuvwxyz';

	let w = ctx.canvas.width;
	let h = ctx.canvas.height;

	let cx = w / 2;
	let cy = h / 2;

	let baseRadius = Math.max(w, h);

	for (let i = particles.length; i < n; i++) {
		let angle = Math.random() * Math.PI * 2;
		let letterIndex = Math.random() * alphabet.length;
		let sampleRadius = baseRadius * (1 + Math.random() * 2);

		particles.push({
			x: Math.cos(angle) * sampleRadius + cx,
			y: Math.sin(angle) * sampleRadius + cy,
			vx: 0,
			vy: 0,
			letter: alphabet.slice(letterIndex, letterIndex + 1),
		});
	}
}

function render(ctx: CanvasRenderingContext2D) {
	let rect = ctx.canvas.getBoundingClientRect();
	ctx.canvas.width = rect.width;
	ctx.canvas.height = rect.height;

	spawnParticles(ctx, 50);

	updateParticles(ctx);
	renderParticles(ctx);
	updateFacePosition();
	renderFace(ctx);
}

function renderParticles(ctx: CanvasRenderingContext2D) {
	ctx.textAlign = 'center';
	ctx.fillStyle = '#000';
	ctx.textBaseline = 'middle';
	ctx.font = getComputedStyle(ctx.canvas).font;

	let w = ctx.canvas.width;
	let h = ctx.canvas.height;

	let cx = w / 2;
	let cy = h / 2;

	for (let particle of particles) {
		const angle = Math.atan2(cy - particle.y, cx - particle.x) - Math.PI / 2;
		ctx.save();
		ctx.translate(particle.x, particle.y);
		ctx.rotate(angle);
		ctx.fillText(particle.letter, 0, 0);
		ctx.restore();
	}
}

function updateParticles(ctx: CanvasRenderingContext2D) {
	let w = ctx.canvas.width;
	let h = ctx.canvas.height;

	let cx = w / 2;
	let cy = h / 2;

	// Remove particles near the center
	let toRemove = [];
	for (let i = 0; i < particles.length; i++) {
		let particle = particles[i];

		let distFromCenter = Math.sqrt((cx - particle.x) ** 2 + (cy - particle.y) ** 2);
		if (distFromCenter < 50) {
			toRemove.push(i);
		}
	}
	particles = particles.filter((v, i) => !toRemove.includes(i));

	// Move according to velocity
	for (let particle of particles) {
		particle.x += particle.vx;
		particle.y += particle.vy;
	}

	// Accelerate towards the center.
	for (let particle of particles) {
		let dx = cx - particle.x;
		let dy = cy - particle.y;

		let mag = Math.sqrt(dx * dx + dy * dy);
		dx /= mag;
		dy /= mag;

		let c = 0.1;

		particle.vx += dx * c;
		particle.vy += dy * c;
	}
}

function lerp(from: number, to: number, t: number): number {
	return from + t * (to - from);
}

function updateFacePosition() {
	if (
		Math.sqrt(
			(currentFacePosition[0] - nextFacePosition[0]) ** 2 +
				(currentFacePosition[1] - nextFacePosition[1]) ** 2,
		) < 1
	) {
		currentFacePosition = nextFacePosition;
		nextFacePosition = [(Math.random() * 2 - 1) * 50, (Math.random() * 2 - 1) * 50];
	}

	let speed = 0.1;
	currentFacePosition = [
		lerp(currentFacePosition[0], nextFacePosition[0], speed),
		lerp(currentFacePosition[1], nextFacePosition[1], speed),
	];
}

function renderFace(ctx: CanvasRenderingContext2D) {
	let w = ctx.canvas.width;
	let h = ctx.canvas.height;

	let cx = w / 2;
	let cy = h / 2;

	ctx.save();
	ctx.translate(cx, cy);
	applyFacingTransform(ctx, currentFacePosition[0], currentFacePosition[1], 50);

	ctx.fillStyle = '#000';
	if (logoImg) {
		ctx.drawImage(logoImg, -50, -50, 100, 100);
	}

	ctx.restore();
}

function applyFacingTransform(
	ctx: CanvasRenderingContext2D,
	targetX: number,
	targetY: number,
	inset: number,
) {
	const dx = targetX;
	const dy = targetY;
	const length = Math.hypot(dx, dy, inset);
	const denominator = length * (length + inset);

	const xx = 1 - (dx * dx) / denominator;
	const xy = -(dx * dy) / denominator;
	const yy = 1 - (dy * dy) / denominator;

	ctx.transform(xx, xy, xy, yy, 0, 0);
}
</script>

<div class="w-full h-full relative">
<canvas class="block w-full h-full font-serif text-lg" bind:this={canvasElement}>
</canvas>
<h2 class="bottom-1/4 z-10 w-full text-center absolute text-black">Downloading Harper To Your Browser</h2>
  </div>
