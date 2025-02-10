<script module lang="ts">
	import { LocalLinter, binary } from 'harper.js';

	export const prerender = true;
	export const frontmatter = {
		title: 'Rules'
	};

	let info: Record<string, string> = $state({});
	let titles: Record<string, string> = $state({});

	let linter = new LocalLinter({ binary });
	linter.getLintDescriptions().then(async (v) => {
		info = v;

		for (let key of Object.keys(info)) {
			titles[key] = await makeTitleCase(key);
			console.log(key, titles[key]);
		}
	});

	/** Make a snake case string title case. */
	function makeTitleCase(snakeCase: string): Promise<string> {
		return linter.toTitleCase(snakeCase.replaceAll('_', ' '));
	}
</script>

<p>This page is an incomplete list of the various grammatical rules Harper checks for.</p>

{#each Object.entries(info) as [name, description]}
	<h2>{titles[name]}</h2>
	<p>{description}</p>
	<p>This rule is also often referred to as <code>{name}</code>.</p>
{/each}
