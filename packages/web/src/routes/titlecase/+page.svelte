<script>
	import { onMount } from 'svelte';
	import { Textarea } from 'flowbite-svelte';
	import Typed from 'typed.js';
	import { WorkerLinter } from 'harper.js';

	let textareaRef;
	let linter = new WorkerLinter();

	let text = $state('');

	$effect(() => {
		linter.toTitleCase(text).then((t) => (text = t));
	});

	onMount(() => {
		const typed = new Typed('#titleCaseInputField', {
			strings: [
				'Click Here to Write an Article Title',
				'Click Here to Write a Blog Title',
				'Click Here to Write a Social Media Post Title',
				'Click Here to Write a Newsletter Title',
				'Click Here to Write a Video Script Title',
				'Click Here to Write a Press Release Title',
				'Click Here to Brainstorm a New Ebook Title'
			],
			typeSpeed: 50,
			showCursor: false,
			attr: 'placeholder'
		});

		return () => typed.destroy();
	});
</script>

<h1>Title Case Converter</h1>

<div class="fixed left-0 top-0 w-screen h-screen bg-white dark:bg-black z-1000">
	<div class="max-w-4xl mx-auto shadow-md border-gray-300 dark:border-x h-full">
		<!-- Header -->
		<header class="border-b border-gray-300 p-4">
			<div class="flex justify-between items-center">
				<h1 class="text-3xl font-serif font-bold">The News, Written by You</h1>
				<span class="text-sm">July 4th 1776</span>
			</div>
			<div class="flex justify-between mt-2">
				<div class="text-xs">Vol. 123, No. 45</div>
				<div class="text-xs">Your trusted news source</div>
			</div>
		</header>

		<main class="p-4 md:p-6">
			<article class="mb-8">
				<Textarea
					bind:value={text}
					rows="1"
					class="heading-textarea font-serif text-2xl md:text-3xl font-bold border-none focus:ring-2 focus:ring-blue-200 bg-transparent p-0 resize-none overflow-hidden"
					id="titleCaseInputField"
				/>
				<div class="text-sm mb-3">By John Doe, Staff Writer</div>

				<p class="leading-relaxed">
					<a href="/">Harper</a> ships out-of-the box with everything you need to perform complex operations
					on English text at the edge. That includes converting text to title-case.
				</p>

				<p class="leading-relaxed">
					Just enter your text in the heading above and it'll be converted to title case following
					the <a href="https://www.chicagomanualofstyle.org/home.html">Chicago Style</a>. Your
					privacy means something. Keep your data where you want it: in your hands and on your
					device.
				</p>
			</article>
		</main>
	</div>
</div>
