<script lang="ts">
import {
	type EditorFontFamily,
	type EditorFontSize,
	FONT_OPTIONS,
	FONT_SIZES,
	wordCount,
} from './editorDisplay.js';

export let text = '';
export let problemCount = 0;
export let fontFamily: EditorFontFamily = 'sans';
export let fontSize: EditorFontSize = 'default';
export let onFontFamilyChange: (fontFamily: EditorFontFamily) => void = () => {};
export let onFontSizeChange: (fontSize: EditorFontSize) => void = () => {};

const fontButtonClass =
	'inline-flex h-4 min-w-7 items-center justify-center rounded px-2 text-xs leading-none font-medium text-stone-500';

$: words = wordCount(text);
$: chars = text.length;
</script>

<footer
	class="flex min-h-[26px] flex-[0_0_26px] items-center gap-3.5 border-t-[0.5px] border-[rgba(28,26,22,0.09)] bg-[#f4f0e7] px-2 pr-2 pl-3.5 text-[11px] whitespace-nowrap text-stone-500 tabular-nums [font-family:'JetBrains_Mono',ui-monospace,'SF_Mono',Menlo,monospace] @max-[760px]:h-auto @max-[760px]:min-h-[30px] @max-[760px]:flex-wrap @max-[760px]:gap-y-1.5 @max-[760px]:py-[5px]"
	aria-label="Editor status"
>
	<div class="inline-flex items-center gap-2">
		<span class="inline-flex items-center gap-1.5">
			<span
				class={`h-1.5 w-1.5 rounded-full ${
					problemCount === 0 ? 'bg-emerald-600' : 'bg-amber-700'
				}`}
			></span>
			{#if problemCount === 0}
				All clear
			{:else}
				{problemCount} problem{problemCount === 1 ? '' : 's'}
			{/if}
		</span>
	</div>

	<span class="h-2.5 w-px bg-[rgba(28,26,22,0.14)]"></span>
	<span>{words} words</span>
	<span>{chars} chars</span>

	<span class="flex-1"></span>

	<div
		class="inline-flex h-[18px] items-center rounded-[5px] border-[0.5px] border-stone-200 bg-stone-200/60 p-px"
		aria-label="Font family"
	>
		{#each FONT_OPTIONS as option}
			<button
				type="button"
				class={`${fontButtonClass} ${
					fontFamily === option.value
						? 'bg-white font-bold text-stone-950 shadow-sm shadow-stone-950/10'
						: ''
				}`}
				style={`font-family: ${option.stack}`}
				title={option.label}
				aria-label={`Use ${option.label.toLowerCase()} font`}
				aria-pressed={fontFamily === option.value}
				on:click={() => onFontFamilyChange(option.value)}
			>
				{option.sample}
			</button>
		{/each}
	</div>

	<label
		class="relative inline-flex h-[18px] items-center rounded-[5px] border-[0.5px] border-stone-200 bg-stone-200/60 pr-[18px] pl-2 text-[11px] font-medium text-stone-950 after:absolute after:top-1/2 after:right-[5px] after:-translate-y-1/2 after:text-[9px] after:leading-none after:text-stone-500 after:content-['v'] after:[font-family:-apple-system,BlinkMacSystemFont,'SF_Pro_Text',sans-serif]"
	>
		<span>{fontSize === 'default' ? 'Default' : `${fontSize}px`}</span>
		<select
			value={fontSize}
			class="absolute inset-0 border-0 opacity-0"
			aria-label="Font size"
			on:change={(event) => {
				const value = event.currentTarget.value;
				onFontSizeChange(value === 'default' ? 'default' : Number(value));
			}}
		>
			<option value="default">Default</option>
			{#each FONT_SIZES as size}
				<option value={size}>{size}px</option>
			{/each}
		</select>
	</label>
</footer>
