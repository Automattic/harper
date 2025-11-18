<script lang="ts">
import { Button, Card, Input, Label, Radio } from 'flowbite-svelte';
import Isolate from '$lib/components/Isolate.svelte';

const reasons = {
	confused: 'I was confused by how it worked',
	'unsupported-language': "It doesn't support my language",
	'slowed-down-browser': 'It slowed down my browser',
	'false-positive': 'It incorrectly flagged my text as an error',
	'false-negative': "It didn't identify enough errors in my text",
	'no-positives': "It didn't identify any errors in my text",
};

let otherSelected: string | number | undefined;
let otherText = '';

function handleFormData(e: FormDataEvent) {
	const fd = e.formData;
	if (fd.get('feedback') === 'other') {
		const v = (fd.get('other') || '').toString().trim();
		if (v) fd.set('feedback', v);
	}
}
</script>

<Isolate>
  <div class="flex flex-row justify-center items-center h-screen"> 
    <Card> 
      <h1 class="text-3xl font-semibold">Uninstalling Harper</h1> <p class="text-sm text-gray-600">We’re sorry to see you go. If you have a minute, would you mind telling us why you uninstalled our browser extension?</p>
      <form method="POST" class="mt-4 space-y-6" action="/api/uninstall-feedback" on:formdata={handleFormData}>
        <div class="space-y-3">
          <div class="flex items-baseline gap-2">
            <Label>Why did you uninstall Harper?</Label>
          </div>
      
          <div class="space-y-3">
            {#each Object.entries(reasons) as [k, r], i}
              <Radio value={k} name="feedback">{r}</Radio>
            {/each}
      
            <Radio name="feedback" value="other" bind:group={otherSelected}>Other</Radio>
            {#if otherSelected}
              <Input name="other" bind:value={otherText} placeholder="Your answer" />
            {/if}
        </div>
      
        <div class="flex items-center justify-between pt-2">
          <Button type="submit">Submit</Button>
        </div>
      </form>
    </Card> 
  </div>
</Isolate>

