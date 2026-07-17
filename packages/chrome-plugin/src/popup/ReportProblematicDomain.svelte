<script lang="ts">
import { Button, Input, Label } from 'components';
import ProtocolClient from '../ProtocolClient';

let { domain, feedback, onSubmit }: { domain: string; feedback: string; onSubmit: () => void } =
	$props();

let submitting = $state(false);

async function handleSubmit(event: SubmitEvent) {
	event.preventDefault();

	submitting = true;

	const success = await ProtocolClient.postFormData(
		'https://writewithharper.com/api/problematic-domains',
		{
			domain,
			feedback,
		},
	);

	submitting = false;

	if (success) {
		onSubmit();
	}
}
</script>

<div class="p-5">
	<h1 class="text-2xl font-semibold">Report Problematic Domain</h1>
	<p class="text-sm">
		Only the data you enter below will be sent to the Harper maintainer.
	</p>
	<form class="mt-4 space-y-6" onsubmit={handleSubmit}>
		<div class="space-y-3">
			<div class="flex items-baseline gap-2">
				<Label class=" ">Which domain is causing problems?</Label>
			</div>
			<Input
				name="domain"
				bind:value={domain}
				placeholder="example.com"
				class="dark:bg-slate-900 dark:border-slate-700 "
			/>

			<div class="flex items-baseline gap-2">
				<Label class=" ">Additional Feedback</Label>
			</div>
			<Input
				name="feedback"
				placeholder="Tell us what went wrong."
				bind:value={feedback}
				class="dark:bg-slate-900 dark:border-slate-700 "
			/>

			<div class="flex items-center justify-between pt-2">
				<Button type="submit" disabled={submitting}>Submit</Button>
			</div>
		</div>
	</form>
</div>
