<script lang="ts">
import { Button, Checkbox, Input, Label } from 'components';
import ProtocolClient from '../ProtocolClient';

let {
	rule_id,
	feedback,
	example,
	onSubmit,
}: { rule_id: string; feedback: string; example: string; onSubmit: () => void } = $props();

let submitting = $state(false);
let successful = $state(false);
let failed = $state(false);

async function handleSubmit(event: SubmitEvent) {
	event.preventDefault();

	submitting = true;
	failed = false;

	const success = await ProtocolClient.postFormData(
		'https://writewithharper.com/api/problematic-lints',
		{
			example,
			rule_id,
			feedback,
			is_false_positive: 'true',
		},
	);

	submitting = false;

	if (success) {
		successful = true;
		setTimeout(onSubmit, 1200);
	} else {
		failed = true;
	}
}

// The `!` suffix is needed: the shared Input component ships its own orange-tinted
// background/border, which otherwise wins over these utilities. Colors deliberately avoid
// `dark:` variants (they don't apply reliably here): the tinted fill, inherited text color and
// translucent border read correctly on both light and dark popup backgrounds.
const inputClass =
	'w-full bg-slate-500/10! border-slate-500/40! text-inherit! caret-primary ' +
	'placeholder:text-primary/70! placeholder:opacity-100 ' +
	'focus:border-primary! focus:ring-1 focus:ring-primary';
const textareaClass = `min-h-14 flex-1 resize-none rounded-lg border px-3 py-2 text-sm leading-snug outline-none! transition-colors ${inputClass}`;
const labelClass = 'shrink-0 text-xs! font-medium! leading-none text-inherit!';
</script>

<form class="flex h-full min-h-0 flex-col" onsubmit={handleSubmit}>
	<!-- Fixed intro, so the title never scrolls under the header -->
	<div class="shrink-0 px-5 pt-3 pb-2.5">
		<h1 class="font-sans! text-xl font-semibold tracking-tight ">
			Report Problematic Lint
		</h1>
		<p class="mt-1 text-xs leading-snug opacity-70">
			Only the data you enter below will be sent to the Harper maintainer.
		</p>
	</div>

	<!-- Compact fields: sized to fit the popup without scrolling -->
	<div class="flex min-h-0 flex-1 flex-col gap-3 px-5 pb-3">
		<Checkbox name="is_false_positive" value="true" hidden />

		<div class="flex min-h-0 flex-1 flex-col gap-1.5">
			<Label class={labelClass}>What text caused (or should cause) feedback from Harper?</Label>
			<textarea
				name="example"
				bind:value={example}
				placeholder="Paste the text, plus any context that helps."
				class={textareaClass}
			></textarea>
		</div>

		<div class="flex min-h-0 flex-1 flex-col gap-1.5">
			<Label class={labelClass}>What rule caused (or should cause) feedback from Harper?</Label>
			<textarea
				name="rule_id"
				bind:value={rule_id}
				placeholder="Rule ID, and anything else worth knowing."
				class={textareaClass}
			></textarea>
		</div>

		<div class="flex shrink-0 flex-col gap-1.5">
			<Label class={labelClass}>Additional Feedback</Label>
			<Input
				name="feedback"
				bind:value={feedback}
				placeholder="Anything you want to add?"
				class={inputClass}
			/>
		</div>
	</div>

	{#if failed}
		<p class="shrink-0 px-5 pb-2 text-xs text-red-500 dark:text-red-400" role="alert">
			Couldn't send your report. Please try again.
		</p>
	{/if}

	<div class="shrink-0 px-5 pb-4">
		<Button
			class="w-full shrink-0 {successful ? '' : 'bg-primary! text-black! hover:bg-primary/85!'} ring-0! focus:ring-0! focus-visible:ring-0! focus:shadow-none! focus-visible:shadow-none! focus-visible:outline! focus-visible:outline-1! focus-visible:outline-offset-2! focus-visible:outline-primary!"
			state={successful ? 'success' : submitting ? 'loading' : 'idle'}
			type="submit"
			disabled={submitting || successful}
		>
			{successful ? 'Sent. Thank you!' : 'Send report'}
		</Button>
	</div>
</form>