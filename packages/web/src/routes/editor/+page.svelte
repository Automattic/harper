<script>
import { page } from '$app/stores';
/// This page exists to be embedded via an `iframe`.
import Editor from '$lib/Editor.svelte';

let content = $page.url.searchParams.get('initialText') ?? '';
</script>

<div class="absolute top-0 left-0 w-full h-full z-[1000] bg-white dark:bg-black">
	<Editor {content}></Editor>
</div>
