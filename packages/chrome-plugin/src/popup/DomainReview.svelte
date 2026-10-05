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
	'flex-1 rounded-md px-3 py-1 text-sm font-semibold transition-colors cursor-pointer text-center';
</script>

<div class="flex h-full flex-col justify-between px-5 py-3">
    <div>
        <h1 class="text-lg font-semibold dark:text-white">Review this site</h1>
        <p class="mt-0.5 text-xs text-gray-500 dark:text-slate-400">
            Only what you enter below is sent to the Harper maintainer.
        </p>
    </div>

    <form class="mt-2.5 flex flex-1 flex-col justify-between overflow-hidden" onsubmit={handleSubmit}>
        <!-- Scrollable Inputs Area (Prevents pushing button out of bounds) -->
        <div class="space-y-2.5 overflow-y-auto pr-0.5 [scrollbar-width:none] [&::-webkit-scrollbar]:hidden">
            <div class="space-y-1">
                <Label class="text-xs dark:text-slate-200">Domain</Label>
                <Input
                    name="domain"
                    bind:value={domain}
                    placeholder="example.com"
                    class="w-full border-gray-200 !outline-none transition-colors focus:!border-primary focus:!ring-0 dark:border-slate-700 dark:bg-slate-900 dark:text-white"
                />
            </div>

            <div class="space-y-1">
                <Label class="text-xs dark:text-slate-200">Does Harper work well here?</Label>
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

            <div class="space-y-1">
                <Label class="text-xs dark:text-slate-200">{works ? 'Anything else? (optional)' : 'What went wrong?'}</Label>
                <textarea
                    name="feedback"
                    rows="4.5"
                    bind:value={feedback}
                    placeholder={works ? 'Optional notes' : 'e.g. underlines are misplaced, editor loses focus'}
                    class="w-full resize-none rounded-md border border-gray-200 bg-transparent px-3 py-1.5 text-sm text-gray-900 placeholder:text-gray-400 !outline-none transition-colors focus:!border-primary focus:!ring-0 dark:border-slate-700 dark:bg-slate-900 dark:text-white dark:placeholder:text-slate-500"
                ></textarea>
            </div>
        </div>

        {#if failed}
            <p class="py-1 text-xs text-red-400" role="alert">Couldn't send your review. Check your connection and try again.</p>
        {/if}

        <div class="pt-2">
            <Button
                class="w-full shrink-0"
                state={successful ? 'success' : submitting ? 'loading' : 'idle'}
                type="submit"
                disabled={submitting || successful}
            >
                {successful ? 'Sent. Thank you!' : 'Send review'}
            </Button>
        </div>
    </form>
</div>