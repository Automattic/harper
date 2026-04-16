<script>
/// This page exists to be embedded via an `iframe`.

import { Editor } from 'harper-editor';
import { page } from '$app/stores';
import Isolate from '$lib/components/Isolate.svelte';

let content = $page.url.searchParams.get('initialText') ?? '';
</script>

<Isolate>
	<Editor {content}></Editor>
</Isolate>
