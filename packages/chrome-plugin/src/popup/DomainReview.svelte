<script lang="ts">
import { Button, Input, Label } from 'components';
import ProtocolClient from '../ProtocolClient';

let {
	domain: initialDomain,
	works: initialWorks,
	feedback: initialFeedback,
	onSubmit,
}: { domain: string; works: boolean; feedback: string; onSubmit: () => void } = $props();

// Local copies so the form can be edited without mutating props.
let domain = $state(initialDomain);
let works = $state(initialWorks);
let feedback = $state(initialFeedback);

let submitting = $state(false);
let successful = $state(false);
let failed = $state(false);

async function handleSubmit(event: SubmitEvent) {
	event.preventDefault();

	submitting = true;
	failed = false;

	const success = await ProtocolClient.postFormData(
		'https://writewithharper.com/api/domain-reviews',
		{
			domain,
			works: works ? 'yes' : 'no',
			feedback,
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

const segment =
	'flex-1 rounded-md px-3 py-1.5 text-sm font-medium transition-colors cursor-pointer';
</script>

<div class="px-5 py-4">
	<h1 class="text-xl font-semibold dark:text-white">Review this site</h1>
	<p class="mt-1 text-xs text-gray-500 dark:text-slate-400">
		Only what you enter below is sent to the Harper maintainer.
	</p>

	<form class="mt-3 space-y-3" onsubmit={handleSubmit}>
		<div class="space-y-1.5">
			<Label>Domain</Label>
			<Input
				name="domain"
				bind:value={domain}
				placeholder="example.com"
				class="dark:bg-slate-900 dark:border-slate-700"
			/>
		</div>

		<div class="space-y-1.5">
			<Label>Does Harper work well here?</Label>
			<div class="rounded-lg border border-gray-200 p-1 dark:border-slate-700 dark:bg-slate-900">
				<div
					class="relative flex w-full"
					role="radiogroup"
					aria-label="Does Harper work well on this domain?"
				>
					<!-- Animated Sliding Background -->
					<div
						class="absolute inset-y-0 left-0 w-1/2 rounded-md bg-primary transition-transform duration-300 ease-out"
						class:translate-x-0={works}
						class:translate-x-full={!works}
					></div>

					<button
						type="button"
						role="radio"
						aria-checked={works}
						class="relative z-10 {segment} {works ? 'text-black' : 'text-gray-500 hover:text-white dark:text-slate-400'}"
						onclick={() => (works = true)}
					>
						Yes
					</button>
					<button
						type="button"
						role="radio"
						aria-checked={!works}
						class="relative z-10 {segment} {!works ? 'text-black' : 'text-gray-500 hover:text-white dark:text-slate-400'}"
						onclick={() => (works = false)}
					>
						No
					</button>
				</div>
			</div>
		</div>

		<div class="space-y-1.5">
			<Label>{works ? 'Anything else? (optional)' : 'What went wrong?'}</Label>
			<textarea
				name="feedback"
				rows="2"
				bind:value={feedback}
				placeholder={works ? 'Optional notes' : 'e.g. underlines are misplaced, editor loses focus'}
				class="w-full resize-none rounded-md border border-gray-200 bg-transparent px-3 py-2 text-sm
					placeholder:text-gray-400 focus:outline-2 focus:outline-primary dark:border-slate-700 dark:bg-slate-900 dark:placeholder:text-slate-500"
			></textarea>
		</div>

		{#if failed}
			<p class="text-xs text-red-400" role="alert">Couldn't send your review. Check your connection and try again.</p>
		{/if}

		<Button
			class="w-full"
			state={successful ? 'success' : submitting ? 'loading' : 'idle'}
			type="submit"
			disabled={submitting || successful}
		>
			{successful ? 'Sent. Thank you!' : 'Send review'}
		</Button>
	</form>
</div>