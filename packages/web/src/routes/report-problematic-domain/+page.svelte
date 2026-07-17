<script lang="ts">
import { Button, Card, Input, Isolate, Label } from 'components';
</script>

<Isolate>
	<div class="flex h-screen flex-row items-center justify-center">
		<Card>
			<h1 class="text-3xl font-semibold">Report Problematic Domain</h1>
			<p class="text-sm text-gray-600">
				If Harper is not working correctly on a website, we'd love to know about it.
			</p>
			<form method="POST" class="mt-4 space-y-6" action="/api/problematic-domains">
				<div class="space-y-3">
					<div class="flex items-baseline gap-2">
						<Label>Which domain is causing problems?</Label>
					</div>
					<Input name="domain" placeholder="example.com" required />

					<div class="flex items-baseline gap-2">
						<Label>Additional Feedback</Label>
					</div>
					<Input
						name="feedback"
						placeholder="Tell us what went wrong."
						required
					/>

					<div class="flex items-center justify-between pt-2">
						<Button type="submit">Submit</Button>
					</div>
				</div>
			</form>
		</Card>
	</div>
</Isolate>
