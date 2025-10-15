<script lang="ts">
import { Button, Checkbox, Input, Label } from 'flowbite-svelte';

let { rule_id, feedback, example }: { rule_id: string; feedback: string; example: string } =
	$props();
</script>

<div class="p-5">
  <h1 class="text-2xl font-semibold">Report Problematic Lint</h1> 
  <p class="text-sm text-gray-600">If you've encountered an example of Harper producing an incorrect result, we'd love to know about it.</p>
  <form method="POST" class="mt-4 space-y-6" action="https://writewithharper.com/api/problematic-lints">
    <div class="space-y-3">
      <div class="flex items-baseline gap-2">
        <Label>What text caused (or should cause) feedback from Harper?</Label>
      </div>
      <Input name="example" bind:value={example} placeholder="Give us an example." />

      <Checkbox name="is_false_positive" value="true" hidden/>

      <div class="flex items-baseline gap-2">
        <Label>What rule caused (or should cause) feedback from Harper?</Label>
      </div>
      <Input name="rule_id" placeholder="We'd appreciate the specific rule ID, if applicable." bind:value={rule_id} />


      <div class="flex items-baseline gap-2">
        <Label>Additional Feedback</Label>
      </div>
      <Input name="feedback" placeholder="Anything you want to add?"  bind:value={feedback}/>

      <div class="flex items-center justify-between pt-2">
        <Button type="submit">Submit</Button>
      </div>
    </div>
  </form>
</div>
