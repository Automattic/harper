<script module lang="ts">
const CURSOR_RADIUS = 200;
const INTRO_SPREAD = 1800;
const INTRO_DURATION = 2200;
const RISE_DISTANCE = 46;
const INTRO_BLUR = 10;
const TEXT_FADE_RANGE = 140;
const GLYPHS = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz&?';
const FONT = '"Domine", Georgia, serif';

/** A pre-rendered glyph, with dimensions in CSS pixels rather than backing-store pixels. */
interface Sprite {
	image: HTMLCanvasElement;
	width: number;
	height: number;
}

/** One depth-sorted particle and its cached gray, text-blended, and introductory blurred glyphs. */
interface Letter {
	character: string;
	x: number;
	y: number;
	vx: number;
	vy: number;
	depth: number;
	size: number;
	weight: number;
	rotation: number;
	phase: number;
	delay: number;
	highlight: number;
	textBlend: number;
	sprites: Sprite[];
}

const random = (min: number, max: number) => min + Math.random() * (max - min);
const clamp = (value: number, min: number, max: number) => Math.max(min, Math.min(max, value));
const smooth = (value: number) => value * value * (3 - 2 * value);

/**
 * Mount the onboarding letter field using local element references. Call only in
 * the browser; the returned disposer cancels all work, including delayed font
 * initialization. Reduced motion draws a static field and hidden tabs stop rendering.
 * The canvas inherits its gray from --floating-letter-color, matching the deck's theme.
 */
function createFloatingLetters(
	canvas: HTMLCanvasElement,
	content: HTMLElement,
	viewport: HTMLElement,
): () => void {
	const ctx = canvas.getContext('2d');
	if (!ctx) return () => {};

	const motion = matchMedia('(prefers-reduced-motion: reduce)');
	let width = 0;
	let height = 0;
	let dpr = 1;
	let letters: Letter[] = [];
	let color = '';
	let blendColor = '';
	let frameId = 0;
	let disposed = false;
	let started = false;
	let introStart = 0;
	let last = 0;
	let textRect = { left: 0, top: 0, right: 0, bottom: 0 };
	const pointer = { x: 0, y: 0, active: false };

	/** Measure in canvas-local CSS pixels so scrolling and embedded decks remain aligned. */
	function measureText() {
		const origin = canvas.getBoundingClientRect();
		const bounds = content.getBoundingClientRect();
		textRect = {
			left: bounds.left - origin.left,
			top: bounds.top - origin.top,
			right: bounds.right - origin.left,
			bottom: bounds.bottom - origin.top,
		};
		requestDraw();
	}

	function distanceToText(x: number, y: number) {
		return Math.hypot(
			Math.max(textRect.left - x, 0, x - textRect.right),
			Math.max(textRect.top - y, 0, y - textRect.bottom),
		);
	}

	/** Bake blur into an offscreen shadow for Safari; never render text or blur in the frame loop. */
	function makeSprite(letter: Letter, ink: string, blur: number): Sprite {
		const pad = Math.ceil(blur * 4 + 4);
		const width = Math.ceil(letter.size * 1.4 + pad * 2);
		const height = Math.ceil(letter.size * 1.5 + pad * 2);
		const image = document.createElement('canvas');
		image.width = Math.ceil(width * dpr);
		image.height = Math.ceil(height * dpr);
		const context = image.getContext('2d');
		if (context) {
			context.scale(dpr, dpr);
			context.font = `${letter.weight} ${letter.size}px ${FONT}`;
			context.textAlign = 'center';
			context.textBaseline = 'middle';
			context.fillStyle = ink;
			if (blur > 0.3) {
				const offset = 10000;
				context.shadowColor = ink;
				context.shadowBlur = blur * 2 * dpr;
				context.shadowOffsetX = offset * dpr;
				context.fillText(letter.character, width / 2 - offset, height / 2);
			} else {
				context.fillText(letter.character, width / 2, height / 2);
			}
		}
		return { image, width, height };
	}

	function buildSprites(letter: Letter) {
		const blur = (1 - letter.depth) * 1.2;
		letter.sprites = [
			makeSprite(letter, color, blur),
			makeSprite(letter, blendColor, blur),
			makeSprite(letter, color, INTRO_BLUR),
		];
	}

	function makeLetters(count: number) {
		letters = Array.from({ length: count }, (): Letter => {
			const depth = Math.random() ** 0.8;
			const letter: Letter = {
				character: GLYPHS[Math.floor(Math.random() * GLYPHS.length)],
				x: random(0, width),
				y: random(0, height),
				vx: random(-0.2, 0.2),
				vy: random(-0.2, 0.2),
				depth,
				size: Math.round(14 + depth * 34 + Math.random() * 10),
				weight: Math.random() < 0.3 ? 600 : 400,
				rotation: random(-0.4, 0.4),
				phase: random(0, Math.PI * 2),
				delay: Math.random() * INTRO_SPREAD,
				highlight: 0,
				textBlend: 0,
				sprites: [],
			};
			buildSprites(letter);
			return letter;
		}).sort((a, b) => a.depth - b.depth);
	}

	function resize() {
		const bounds = canvas.getBoundingClientRect();
		const oldWidth = width;
		const oldHeight = height;
		const oldDpr = dpr;
		width = bounds.width;
		height = bounds.height;
		dpr = Math.min(window.devicePixelRatio || 1, 2);
		canvas.width = Math.round(width * dpr);
		canvas.height = Math.round(height * dpr);
		const count = clamp(Math.round((width * height) / 8500), 40, 260);
		if (!letters.length || Math.abs(count - letters.length) > 20) {
			makeLetters(count);
		} else {
			for (const letter of letters) {
				letter.x *= width / (oldWidth || width || 1);
				letter.y *= height / (oldHeight || height || 1);
				if (dpr !== oldDpr) buildSprites(letter);
			}
		}
		measureText();
	}

	function stamp(sprite: Sprite, cos: number, sin: number, x: number, y: number, alpha: number) {
		if (alpha < 0.004 || !ctx) return;
		ctx.globalAlpha = alpha;
		ctx.setTransform(cos * dpr, sin * dpr, -sin * dpr, cos * dpr, x * dpr, y * dpr);
		ctx.drawImage(sprite.image, -sprite.width / 2, -sprite.height / 2, sprite.width, sprite.height);
	}

	/** Integrate the liquid field at a normalized 60Hz rate, then stamp depth-sorted sprites. */
	function frame(now: number) {
		frameId = 0;
		if (disposed || !ctx) return;
		const dt = Math.min((now - last) / 16.667, 3);
		last = now;
		ctx.setTransform(1, 0, 0, 1, 0, 0);
		ctx.globalAlpha = 1;
		ctx.clearRect(0, 0, canvas.width, canvas.height);

		for (const letter of letters) {
			const p = motion.matches
				? 1
				: clamp((now - introStart - letter.delay) / INTRO_DURATION, 0, 1);
			if (p <= 0) continue;
			const ease = 1 - (1 - p) ** 3;
			const distance = pointer.active
				? Math.hypot(letter.x - pointer.x, letter.y - pointer.y)
				: Number.POSITIVE_INFINITY;

			if (!motion.matches) {
				const fx =
					Math.sin(letter.y * 0.0042 + now * 0.00031 + letter.phase) +
					0.5 * Math.sin(letter.x * 0.0019 - now * 0.00017);
				const fy =
					Math.cos(letter.x * 0.0036 + now * 0.00026 + letter.phase * 0.7) +
					0.5 * Math.cos(letter.y * 0.0021 + now * 0.00013);
				const drift = 0.0065 * (0.45 + letter.depth * 0.55);
				letter.vx += fx * drift * dt;
				letter.vy += (fy * drift - 0.0012) * dt;
				if (distance < CURSOR_RADIUS && distance > 0.1) {
					const force = (1 - distance / CURSOR_RADIUS) * 0.035 * dt;
					letter.vx += ((letter.x - pointer.x) / distance) * force;
					letter.vy += ((letter.y - pointer.y) / distance) * force;
				}
				const damping = 0.975 ** dt;
				letter.vx *= damping;
				letter.vy *= damping;
				letter.x += letter.vx * dt * ease;
				letter.y += letter.vy * dt * ease;
				const margin = letter.size;
				if (letter.x < -margin) letter.x = width + margin;
				else if (letter.x > width + margin) letter.x = -margin;
				if (letter.y < -margin) letter.y = height + margin;
				else if (letter.y > height + margin) letter.y = -margin;
				const rotation = letter.vx * 0.9 + Math.sin(now * 0.0006 + letter.phase) * 0.18;
				letter.rotation += (rotation - letter.rotation) * 0.02 * dt;
			}

			const highlight = distance < CURSOR_RADIUS ? smooth(1 - distance / CURSOR_RADIUS) : 0;
			const textDistance = distanceToText(letter.x, letter.y);
			const textBlend =
				textDistance < TEXT_FADE_RANGE ? smooth(1 - textDistance / TEXT_FADE_RANGE) : 0;
			letter.highlight = motion.matches
				? 0
				: letter.highlight +
					(highlight - letter.highlight) * (highlight > letter.highlight ? 0.14 : 0.05) * dt;
			letter.textBlend = motion.matches
				? textBlend
				: letter.textBlend + (textBlend - letter.textBlend) * 0.06 * dt;
			const restAlpha = 0.12 + letter.depth * 0.38;
			let alpha = (restAlpha + (0.92 - restAlpha) * letter.highlight) * ease;
			alpha += (Math.max(alpha, 0.7 * ease) - alpha) * letter.textBlend;
			const scale = 0.4 + 0.6 * ease;
			const y = letter.y + (1 - ease) * RISE_DISTANCE * (0.5 + letter.depth);
			const cos = Math.cos(letter.rotation) * scale;
			const sin = Math.sin(letter.rotation) * scale;
			const sharp = p >= 1 ? 1 : ease * ease;
			if (sharp < 1) stamp(letter.sprites[2], cos, sin, letter.x, y, alpha * (1 - sharp));
			stamp(letter.sprites[0], cos, sin, letter.x, y, alpha * (1 - letter.textBlend) * sharp);
			stamp(letter.sprites[1], cos, sin, letter.x, y, alpha * letter.textBlend * sharp);
		}
		if (!motion.matches) requestDraw();
	}

	function requestDraw() {
		if (!disposed && started && !document.hidden && !frameId)
			frameId = requestAnimationFrame(frame);
	}

	function updatePointer(event: PointerEvent) {
		const origin = canvas.getBoundingClientRect();
		pointer.x = event.clientX - origin.left;
		pointer.y = event.clientY - origin.top;
		pointer.active = true;
	}

	function clearPointer() {
		pointer.active = false;
	}

	function releasePointer(event: PointerEvent) {
		if (event.pointerType === 'touch') clearPointer();
	}

	function refreshTheme() {
		const style = getComputedStyle(canvas);
		const nextColor = style.getPropertyValue('--floating-letter-color').trim() || '#4a463e';
		const nextBlendColor =
			style.getPropertyValue('--floating-letter-blend-color').trim() || '#ffffff';
		if (nextColor === color && nextBlendColor === blendColor) return;
		color = nextColor;
		blendColor = nextBlendColor;
		letters.forEach(buildSprites);
		requestDraw();
	}

	function refreshFonts() {
		letters.forEach(buildSprites);
		measureText();
	}

	function resume() {
		cancelAnimationFrame(frameId);
		frameId = 0;
		last = performance.now();
		clearPointer();
		requestDraw();
	}

	const resizeObserver = new ResizeObserver(() => {
		if (started) resize();
	});
	resizeObserver.observe(canvas);
	const contentObserver = new ResizeObserver(measureText);
	contentObserver.observe(content);
	const themeObserver = new MutationObserver(refreshTheme);
	for (let element: HTMLElement | null = canvas; element; element = element.parentElement) {
		themeObserver.observe(element, { attributes: true, attributeFilter: ['class'] });
	}
	viewport.addEventListener('pointermove', updatePointer, { passive: true });
	viewport.addEventListener('pointerdown', updatePointer, { passive: true });
	viewport.addEventListener('pointerleave', clearPointer);
	viewport.addEventListener('pointerup', releasePointer);
	viewport.addEventListener('scroll', measureText, { passive: true });
	window.addEventListener('blur', clearPointer);
	window.addEventListener('resize', resize);
	document.addEventListener('visibilitychange', resume);
	motion.addEventListener('change', resume);
	document.fonts.addEventListener('loadingdone', refreshFonts);

	function start() {
		if (disposed || started) return;
		started = true;
		clearTimeout(fontTimeout);
		refreshTheme();
		introStart = last = performance.now();
		resize();
	}
	const fontTimeout = window.setTimeout(start, 1200);
	void Promise.all([
		document.fonts.load('400 32px "Domine"'),
		document.fonts.load('600 32px "Domine"'),
	]).then(start, start);

	return () => {
		disposed = true;
		cancelAnimationFrame(frameId);
		clearTimeout(fontTimeout);
		resizeObserver.disconnect();
		contentObserver.disconnect();
		themeObserver.disconnect();
		viewport.removeEventListener('pointermove', updatePointer);
		viewport.removeEventListener('pointerdown', updatePointer);
		viewport.removeEventListener('pointerleave', clearPointer);
		viewport.removeEventListener('pointerup', releasePointer);
		viewport.removeEventListener('scroll', measureText);
		window.removeEventListener('blur', clearPointer);
		window.removeEventListener('resize', resize);
		document.removeEventListener('visibilitychange', resume);
		motion.removeEventListener('change', resume);
		document.fonts.removeEventListener('loadingdone', refreshFonts);
		letters = [];
	};
}
</script>

<script lang="ts">
import { onMount, tick } from 'svelte';
import Arrow from './icons/Arrow.svelte';

/**
 * A drop-in SlideDeck alternative with floating letters and a borderless layout.
 * Progress is parent-owned, from 0 (first slide) to 1 (last). The default slot
 * contains slide content; `aside` appears beside it on desktop when `showAside`
 * is true, and `outside` sits below the deck without affecting its centering.
 * `nextDisabled` gates both button and keyboard advancement. Focus follows an
 * actual progress change, including callbacks that update the parent later.
 * A `.dark` ancestor selects the dark palette. Reduced motion disables animation.
 */
export let title: string;
export let lede: string;
export let slideProgress: number;
export let onBack: () => void;
export let onNext: () => void;
export let nextDisabled = false;
export let showAside = false;

let canvas: HTMLCanvasElement;
let content: HTMLDivElement;
let viewport: HTMLDivElement;
let heading: HTMLElement;
let previousProgress = slideProgress;

$: focusStep(slideProgress);

onMount(() => {
	const dispose = createFloatingLetters(canvas, content, viewport);
	window.addEventListener('keydown', handleKeydown);
	return () => {
		window.removeEventListener('keydown', handleKeydown);
		dispose();
	};
});

/** Wait for the keyed slide to render before focusing, without stealing focus on mount. */
async function focusStep(progress: number) {
	if (progress === previousProgress) return;
	previousProgress = progress;
	await tick();
	heading?.focus();
}

function changeStep(direction: -1 | 1) {
	if (direction === -1) {
		if (slideProgress > 0) onBack();
	} else if (slideProgress < 1 && !nextDisabled) onNext();
}

/** Preserve SlideDeck shortcuts without intercepting typing, controls, or shadow-root editors. */
function handleKeydown(event: KeyboardEvent) {
	if (
		event.defaultPrevented ||
		event.isComposing ||
		event.repeat ||
		event.altKey ||
		event.ctrlKey ||
		event.metaKey ||
		event.shiftKey ||
		event
			.composedPath()
			.some(
				(target) =>
					target instanceof Element &&
					target.closest(
						'input, textarea, select, button, a, summary, video, [contenteditable]:not([contenteditable="false"]), [role="button"], [role="textbox"]',
					),
			)
	)
		return;

	if (event.key === 'ArrowRight' || event.key === 'Enter') {
		event.preventDefault();
		changeStep(1);
	} else if (event.key === 'ArrowLeft') {
		event.preventDefault();
		changeStep(-1);
	}
}
</script>

<main class="floating-deck">
	<canvas bind:this={canvas} class="letters" aria-hidden="true"></canvas>
	<div
		class="progress"
		role="progressbar"
		aria-label="Slide deck progress"
		aria-valuemin={0}
		aria-valuemax={100}
		aria-valuenow={Math.round(slideProgress * 100)}
	>
		<span style:width={`${slideProgress * 100}%`}></span>
	</div>
	<div bind:this={viewport} class="viewport">
		<div bind:this={content} class="deck-content" class:with-aside={showAside}>
			{#key slideProgress}
				<section class="step" aria-labelledby="step-heading">
					<div class="slide-layout" class:with-aside={showAside}>
						<div class="copy">
							<svelte:element
								this={slideProgress === 0 ? 'h1' : 'h2'}
								bind:this={heading}
								tabindex="-1"
								id="step-heading"
								class="title"
							>
								{title}
							</svelte:element>
							<p class="lede">{lede}</p>
							<div class="extra"><slot /></div>
						</div>
						{#if showAside}
							<div class="aside"><slot name="aside" /></div>
						{/if}
					</div>
					<nav class="navigation" aria-label="Slide navigation">
						{#if slideProgress > 0}
							<button type="button" class="back" onclick={() => changeStep(-1)}>
								<span class="back-arrow"><Arrow width="20" height="20" /></span>
								<span class="back-label">Back</span>
							</button>
						{/if}
						{#if slideProgress < 1}
							<button type="button" class="next" disabled={nextDisabled} onclick={() => changeStep(1)}>
								<span>Next</span>
								<Arrow width="20" height="20" />
							</button>
						{/if}
					</nav>
				</section>
			{/key}
			<slot name="outside" />
		</div>
	</div>
</main>

<style>
.floating-deck {
	--deck-bg-top: #fbfaf7;
	--deck-bg-deep: #f1ede5;
	--deck-ink: #1c1a16;
	--deck-ink-soft: #4a463e;
	--deck-track: rgb(28 26 22 / 9%);
	--deck-focus: #2a6bd8;
	--floating-letter-color: #4a463e;
	--floating-letter-blend-color: #ffffff;
	position: relative;
	isolation: isolate;
	height: 100dvh;
	overflow: hidden;
	color: var(--deck-ink);
	color-scheme: light;
	font-family: "Atkinson Hyperlegible", system-ui, sans-serif;
	background: radial-gradient(120% 90% at 50% 0%, var(--deck-bg-top) 0%, var(--deck-bg-deep) 100%);
}

:global(.dark) .floating-deck {
	--deck-bg-top: #1c1917;
	--deck-bg-deep: #0c0a09;
	--deck-ink: #f5f5f4;
	--deck-ink-soft: #d6d3d1;
	--deck-track: rgb(255 255 255 / 10%);
	--deck-focus: #f7be6e;
	--floating-letter-color: #d6d3d1;
	--floating-letter-blend-color: #0c0a09;
	color-scheme: dark;
}

.letters {
	position: absolute;
	inset: 0;
	width: 100%;
	height: 100%;
	pointer-events: none;
}

.progress {
	position: absolute;
	inset: 0 0 auto;
	z-index: 2;
	padding-top: env(safe-area-inset-top, 0px);
	background: var(--deck-track);
}

.progress span {
	display: block;
	height: 3px;
	background: #c1750b;
	transition: width 400ms ease;
}

.viewport {
	position: relative;
	z-index: 1;
	display: grid;
	place-items: center;
	box-sizing: border-box;
	height: 100%;
	overflow-y: auto;
	scrollbar-gutter: stable both-edges;
	padding: max(1.5rem, env(safe-area-inset-top, 0px)) max(1.5rem, env(safe-area-inset-right, 0px)) max(1.5rem, env(safe-area-inset-bottom, 0px)) max(1.5rem, env(safe-area-inset-left, 0px));
}

.deck-content {
	position: relative;
	width: 100%;
	max-width: 34rem;
	margin-block: auto;
}

.step {
	max-width: none;
	text-align: center;
	animation: step-in 420ms cubic-bezier(.2, .7, .2, 1);
}

.copy { min-width: 0; }

.title {
	margin: 0 auto 0.85rem;
	padding: 0;
	max-width: 16ch;
	font-family: Domine, Georgia, "Times New Roman", serif;
	font-weight: 600;
	font-size: clamp(2.1rem, 6vw, 3.8rem);
	line-height: 1.05;
	letter-spacing: normal;
	text-wrap: pretty;
}

.title:focus { outline: none; }

.lede {
	margin: 0 auto;
	max-width: 38ch;
	color: var(--deck-ink-soft);
	font-family: Domine, Georgia, "Times New Roman", serif;
	font-weight: 500;
	font-size: 1.25rem;
	line-height: 1.6;
	text-wrap: pretty;
}

.extra {
	display: flex;
	flex-direction: column;
	align-items: center;
}

.extra > :global(*) { max-width: 100%; }
.extra:empty { display: none; }

.aside {
	display: flex;
	align-items: center;
	justify-content: center;
	min-width: 0;
	margin-top: 2.5rem;
}

.navigation {
	display: flex;
	justify-content: center;
	gap: 0.75rem;
	margin-top: 2.25rem;
}

.navigation button {
	display: inline-flex;
	align-items: center;
	justify-content: center;
	gap: 0.4rem;
	padding: 0.8rem 1.6rem;
	border: 1px solid transparent;
	border-radius: 999px;
	font: inherit;
	font-weight: 600;
	font-size: 1rem;
	cursor: pointer;
}

.navigation .back {
	padding-inline: 0.5rem;
	border: 0;
	border-radius: 0.25rem;
	color: var(--deck-ink-soft);
	background: transparent;
}

.back-arrow {
	display: inline-flex;
	flex-shrink: 0;
	transform: rotate(180deg);
}

.back-label {
	text-decoration: underline;
	text-underline-offset: 3px;
}

.navigation .back:hover { color: var(--deck-ink); }
.back:hover .back-label { text-decoration-thickness: 2px; }

.next { color: #ffffff; background: #c1750b; }
.next:hover:not(:disabled) { background: #915808; }
.navigation button:disabled { opacity: 0.5; cursor: not-allowed; }
.navigation button:focus-visible { outline: 2px solid var(--deck-focus); outline-offset: 3px; }

@media (min-width: 768px) {
	.deck-content.with-aside { max-width: 64rem; }
	.slide-layout.with-aside {
		display: grid;
		grid-template-columns: minmax(0, 1fr) minmax(0, 16rem);
		align-items: center;
		gap: 2.5rem;
	}
	.aside { margin-top: 0; }
}

@keyframes step-in {
	from { opacity: 0; }
	to { opacity: 1; }
}

@media (prefers-reduced-motion: reduce) {
	.step { animation: none; }
	.progress span { transition: none; }
}
</style>
