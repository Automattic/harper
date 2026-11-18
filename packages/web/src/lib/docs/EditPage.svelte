<script lang="ts">
import { siteConfig } from './config';

/** Repository-relative route source, including +page.md or +page.svelte. */
interface Props {
	sourcePath: string;
}

let { sourcePath }: Props = $props();
const href = $derived(siteConfig.editLink.replace(':route', sourcePath));
</script>

<div class="mt-16 text-sm leading-[1.75]">
	<a class="text-(--docs-link)! hover:underline!" {href}>Edit this page on GitHub ↗</a>
</div>
