<script>
/// This page exists to be embedded via an `iframe`.

import { page } from '$app/stores';
import Editor from '$lib/components/Editor.svelte';
import Isolate from '$lib/components/Isolate.svelte';

let content = $page.url.searchParams.get('initialText') ?? '';
</script>

<Isolate>
	<Editor {content}></Editor>
</Isolate>
