<script lang="ts">
    import AppLogoTile from "$lib/marketing/AppLogoTile.svelte";
    import IntegrationTile from "$lib/marketing/IntegrationTile.svelte";
    import { onMount } from "svelte";

let canvasElement: HTMLCanvasElement | null = $state(null);

$effect(() => {
	let ctx = canvasElement?.getContext('2d');
	if (ctx != null) {
		frameLoop(ctx);
	}
});

function frameLoop(ctx: CanvasRenderingContext2D){
  render(ctx);
  requestAnimationFrame(() => frameLoop(ctx))
}

type ParticleState = {
  x: number,
  y: number,
  vx: number,
  vy: number,
  letter: string
}

let particles: ParticleState[] = []

/** Ensures that there are N number of particles by spawning new ones. */
function spawnParticles(ctx: CanvasRenderingContext2D, n: number){
  let alphabet = "abcdefghijklmnopqrstuvwxyz";

  let w = ctx.canvas.width;
  let h = ctx.canvas.height;

  let cx = w / 2;
  let cy = h / 2;

  let baseRadius = Math.max(w, h);

   for (let i = particles.length; i < n; i++){
    let angle = Math.random() * Math.PI * 2;
    let letterIndex = Math.random() * alphabet.length;
let sampleRadius = baseRadius * (1 + Math.random() * 2);

    particles.push({
      x: Math.cos(angle) * sampleRadius + cx,
      y: Math.sin(angle) * sampleRadius + cy,
      vx: 0,
      vy: 0,
      letter: alphabet.slice(letterIndex, letterIndex + 1),
    })
  }
}

function render(ctx: CanvasRenderingContext2D) {
  let rect = ctx.canvas.getBoundingClientRect();
  ctx.canvas.width = rect.width;
  ctx.canvas.height = rect.height;

  let w = ctx.canvas.width;
  let h = ctx.canvas.height;

  spawnParticles(ctx, 50);

  updateParticles(ctx);
  renderParticles(ctx);
  renderNotifText(ctx, w, h);
}

function renderParticles(ctx: CanvasRenderingContext2D){
  ctx.textAlign = 'center';
  ctx.fillStyle = "#000"
  ctx.textBaseline = 'middle';
  ctx.font = getComputedStyle(ctx.canvas).font;

  let w = ctx.canvas.width;
  let h = ctx.canvas.height;

  let cx = w / 2;
  let cy = h / 2;

  for (let particle of particles){
    const angle = Math.atan2(cy - particle.y, cx - particle.x) - Math.PI / 2;
    ctx.save();
    ctx.translate(particle.x, particle.y);
    ctx.rotate(angle);
    ctx.fillText(particle.letter, 0, 0);
    ctx.restore();
  }
}

function updateParticles(ctx: CanvasRenderingContext2D){
  let w = ctx.canvas.width;
  let h = ctx.canvas.height;

  let cx = w / 2;
  let cy = h / 2;

  // Remove particles near the center
  let toRemove = []; 
  for (let i = 0; i < particles.length; i++){
    let particle = particles[i];

    let distFromCenter = Math.sqrt((cx - particle.x) ** 2 + (cy - particle.y) ** 2);
    if (distFromCenter < 50){
      toRemove.push(i)
    }
  }
  particles = particles.filter((v, i) => !toRemove.includes(i));

  // Move according to velocity
  for (let particle of particles){
    particle.x += particle.vx; 
    particle.y += particle.vy; 
  }

  // Accelerate towards the center.
  for (let particle of particles){
    let dx = (  cx- particle.x);
    let dy = (cy - particle.y );

    let mag = Math.sqrt(dx * dx + dy * dy);
    dx /= mag;
    dy /= mag;

    let c = 0.1;

    particle.vx += dx * c;
    particle.vy += dy * c;
  }
}

function renderNotifText(ctx: CanvasRenderingContext2D, width: number, height: number){
  ctx.textAlign = 'center';
  ctx.fillStyle = "#000"
   ctx.textBaseline = 'middle';
  ctx.font = getComputedStyle(ctx.canvas).font;
  ctx.fillText(`Downloading Harper${".".repeat(new Date().getSeconds() % 4)}`, width / 2, height * 2 / 3);
}
</script>

<canvas class="w-full h-full font-serif text-lg" bind:this={canvasElement}>

</canvas>
