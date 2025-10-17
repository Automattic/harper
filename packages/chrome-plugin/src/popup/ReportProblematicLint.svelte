<script lang="ts">
import { Button, Checkbox, Input, Label } from 'flowbite-svelte';
import ProtocolClient from '../ProtocolClient';

let {
	rule_id,
	feedback,
	example,
	onSubmit,
}: { rule_id: string; feedback: string; example: string; onSubmit: () => void } = $props();

let submitting = $state(false);

async function handleSubmit(event: SubmitEvent) {
	event.preventDefault();

	submitting = true;

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
		onSubmit();
	}
}
</script>

<div class="p-5">
	<h1 class="text-2xl font-semibold">Report Problematic Lint</h1>
	<p class="text-sm text-gray-600">
		Only the data you enter below will be sent to the Harper maintainer.
	</p>
	<form class="mt-4 space-y-6" onsubmit={handleSubmit}>
		<div class="space-y-3">
			<div class="flex items-baseline gap-2">
				<Label>What text caused (or should cause) feedback from Harper?</Label>
			</div>
			<Input name="example" bind:value={example} placeholder="Give us an example." />

			<Checkbox name="is_false_positive" value="true" hidden />

			<div class="flex items-baseline gap-2">
				<Label>What rule caused (or should cause) feedback from Harper?</Label>
			</div>
			<Input
				name="rule_id"
				placeholder="We'd appreciate the specific rule ID, if applicable."
				bind:value={rule_id}
			/>

			<div class="flex items-baseline gap-2">
				<Label>Additional Feedback</Label>
			</div>
			<Input name="feedback" placeholder="Anything you want to add?" bind:value={feedback} />

			<div class="flex items-center justify-between pt-2">
				<Button type="submit" disabled={submitting}>Submit</Button>
			</div>
		</div>
	</form>
</div>
