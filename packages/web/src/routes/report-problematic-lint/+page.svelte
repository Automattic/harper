<script lang="ts">
import { Button, Card, Checkbox, Input, Label, Radio } from 'flowbite-svelte';
import Isolate from '$lib/components/Isolate.svelte';
</script>

<Isolate>
  <div class="flex flex-row justify-center items-center h-screen"> 
    <Card> 
      <h1 class="text-3xl font-semibold">Report Problematic Lint</h1> 
      <p class="text-sm text-gray-600">If you've encountered an example of Harper producing an incorrect result, we'd love to know about it.</p>
      <form method="POST" class="mt-4 space-y-6" action="/api/problematic-lints">
        <div class="space-y-3">
          <div class="flex items-baseline gap-2">
            <Label>What text caused (or should cause) feedback from Harper?</Label>
          </div>
          <Input name="example" placeholder="Give us an example." />

          <Checkbox name="is_false_positive">Is it a false positive? Otherwise, leave unchecked.</Checkbox>

          <div class="flex items-baseline gap-2">
            <Label>What rule caused (or should cause) feedback from Harper?</Label>
          </div>
          <Input name="rule_id" placeholder="We'd appreciate the specific rule ID, if applicable." />


          <div class="flex items-baseline gap-2">
            <Label>Additional Feedback</Label>
          </div>
          <Input name="feedback" placeholder="Anything you want to add?" />

          <div class="flex items-center justify-between pt-2">
            <Button type="submit">Submit</Button>
          </div>
        </div>
      </form>
    </Card> 
  </div>
</Isolate>
