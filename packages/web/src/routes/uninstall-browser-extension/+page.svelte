<script lang="ts">
import { Button, Card, Input, Isolate, SendIcon } from 'components';

const reasons = {
	confused: 'I was confused by how it worked',
	'unsupported-language': "It doesn't support my language",
	'slowed-down-browser': 'It slowed down my browser',
	'false-positive': 'It incorrectly flagged my text as an error',
	'false-negative': "It didn't identify enough errors in my text",
	'no-positives': "It didn't identify any errors in my text",
	other: 'Other',
};

let selected: string | undefined;
let otherText = '';

/** Replace the Other reason with its trimmed answer when nonempty; keep the reason otherwise. */
function handleFormData(e: FormDataEvent) {
	const fd = e.formData;
	if (fd.get('feedback') === 'other') {
		const v = (fd.get('other') || '').toString().trim();
		if (v) fd.set('feedback', v);
	}
}
</script>

<Isolate>
	<div class="flex min-h-screen justify-center bg-[#f6f1e6] px-4 py-8 font-['Atkinson_Hyperlegible',sans-serif] text-[#1c1a16] sm:py-[4.8rem] dark:bg-black dark:text-white">
		<form
			method="POST"
			action="/api/uninstall-feedback"
			class="w-full max-w-[38rem]"
			on:formdata={handleFormData}
		>
			<Card className="!p-0 overflow-hidden bg-white !shadow-[0_30px_60px_-24px_rgba(28,26,22,0.22)] dark:bg-gray-900">
				<!-- Unsplash photo by Ray Zhuang. -->
				<img
					src="/images/uninstall-kitten.webp"
					alt=""
					width="1216"
					height="811"
					class="block aspect-[3/2] h-auto w-full border-b-[0.5px] border-black/10 object-cover object-[50%_40%] brightness-125 dark:border-white/10"
				/>

				<div class="flex flex-col gap-7 p-5 sm:p-10">
					<div class="flex flex-col gap-3">
						<h1 class="!m-0 py-0 font-['Domine',serif] text-3xl font-[650] leading-[1.05] tracking-[-0.01em] sm:text-4xl lg:text-4xl">
							We're sorry to see you go.
						</h1>
						<p class="!m-0 font-['Domine',serif] text-[1.0625rem] font-[550] leading-normal text-[#4a463e] [text-wrap:pretty] dark:text-white/70">
							If you have a minute, would you mind telling us why you uninstalled Harper?
						</p>
					</div>

					<fieldset class="m-0 flex min-w-0 flex-col gap-2 border-0 p-0">
						<legend class="mb-3 p-0 text-[0.9375rem] font-bold">Why did you uninstall Harper?</legend>

						{#each Object.entries(reasons) as [value, label]}
							<label
								class="group flex cursor-pointer items-center gap-3 rounded-[0.65rem] border border-black/10 bg-white px-[0.9rem] py-[0.7rem] text-[0.9375rem] transition-[border-color,transform] duration-150 ease-out hover:-translate-y-px hover:border-[#b06a1b] has-[:checked]:translate-y-0 has-[:checked]:!border-[#f1920e] has-[:checked]:bg-[#fdf6ea] has-[:focus-visible]:ring-2 has-[:focus-visible]:ring-primary-300 motion-reduce:transform-none motion-reduce:transition-none dark:border-white/20 dark:bg-white/5 dark:hover:border-primary-300 dark:has-[:checked]:!bg-[#301d03]"
							>
								<input type="radio" name="feedback" {value} bind:group={selected} class="peer sr-only" />
								<span
									aria-hidden="true"
									class="flex size-[18px] flex-none items-center justify-center rounded-full border-[1.5px] border-[#ada79a] bg-white peer-checked:border-[#f1920e] dark:bg-gray-900"
								>
									<span class="size-2 rounded-full bg-[#f1920e] opacity-0 transition-opacity duration-[120ms] group-has-[:checked]:opacity-100 motion-reduce:transition-none"></span>
								</span>
								{label}
							</label>
						{/each}

						{#if selected === 'other'}
							<label for="other-answer" class="sr-only">Other reason</label>
							<Input id="other-answer" name="other" bind:value={otherText} placeholder="Your answer" class="block w-full dark:!bg-gray-900 dark:!text-white" />
						{/if}
					</fieldset>

					<div class="flex flex-col items-stretch gap-3 sm:flex-row sm:flex-wrap sm:items-center sm:justify-between">
						<Button type="submit" size="lg" disabled={!selected}>
							Send
							<SendIcon className="size-4 shrink-0" />
						</Button>
						<Button
							href="/get"
							color="white"
							size="lg"
							class="whitespace-normal !text-gray-900 hover:!no-underline dark:!text-white"
						>
							I made a mistake. I want to reinstall.
						</Button>
					</div>
				</div>
			</Card>
		</form>
	</div>
</Isolate>
