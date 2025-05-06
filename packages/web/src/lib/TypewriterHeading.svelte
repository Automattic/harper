<script lang="ts">
import { onDestroy, onMount } from 'svelte';
import Typed, { type TypedOptions } from 'typed.js';

/** Strings to rotate through */
export let items: string[] = [];

let el: HTMLSpanElement;
let typed: Typed;

onMount(() => {
	if (items.length === 0) return;

	typed = new Typed(el, {
		strings: items,
		typeSpeed: 40,
		backSpeed: 40,
		backDelay: 2000,
		loop: true,
		shuffle: true, // <‑‑ RANDOM order every cycle
		smartBackspace: true,
		cursorChar: '|',
	});
});

onDestroy(() => typed?.destroy());
</script>

<h2 class="text-center text-lg md:text-2xl lg:text-4xl"><span bind:this={el}></span></h2>
