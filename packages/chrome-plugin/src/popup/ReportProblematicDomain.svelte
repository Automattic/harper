<script lang="ts">
import { Button, Input, Label, Select } from 'components';
import ProtocolClient from '../ProtocolClient';

let {
	domain,
	works,
	feedback,
	onSubmit,
}: { domain: string; works: boolean; feedback: string; onSubmit: () => void } = $props();

let submitting = $state(false);

async function handleSubmit(event: SubmitEvent) {
	event.preventDefault();

	submitting = true;

	const success = await ProtocolClient.postFormData(
		'https://writewithharper.com/api/problematic-domains',
		{
			domain,
			works: works ? 'yes' : 'no',
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
				<Label class=" ">Which domain would you like to tell us about?</Label>
			</div>
			<Input
				name="domain"
				bind:value={domain}
				placeholder="example.com"
				class="dark:bg-slate-900 dark:border-slate-700 "
			/>
			<div class="flex items-baseline gap-2">
				<Label class=" ">Would you say that Harper works well on this domain?</Label>
			</div>
			<Select
				name="works"
				bind:value={works}
				class="dark:bg-slate-900 dark:border-slate-700 "
			>
        <option value={true}>Yes</option>
        <option value={false}>No</option>
        </Select>

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
