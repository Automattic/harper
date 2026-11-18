<script module lang="ts">
import { defineMeta } from '@storybook/addon-svelte-csf';
import { expect, fn, spyOn, userEvent, waitFor } from 'storybook/test';
import type { ComponentProps } from 'svelte';
import FloatingLettersSlideDeck from './FloatingLettersSlideDeck.svelte';

const { Story } = defineMeta({
	title: 'Components/FloatingLettersSlideDeck',
	component: FloatingLettersSlideDeck,
	tags: ['autodocs'],
	parameters: { layout: 'fullscreen' },
	args: {
		title: 'Thanks for Installing Harper!',
		lede: 'Writing is hard. Writing well is harder. Harper makes it a little easier.',
		slideProgress: 0,
		onBack: fn(),
		onNext: fn(),
		nextDisabled: false,
		showAside: false,
	},
	argTypes: {
		slideProgress: { control: false },
		onBack: { control: false },
		onNext: { control: false },
	},
});
</script>

<script lang="ts">
let step = $state(0);
let helpOpen = $state(false);
let mounted = $state(true);
</script>

{#snippet template(args: ComponentProps<FloatingLettersSlideDeck>)}
	<FloatingLettersSlideDeck
		{...args}
		title={step === 0 ? args.title : step === 1 ? 'Keep writing' : 'All set!'}
		slideProgress={step / 2}
		onBack={() => { args.onBack(); step = Math.max(0, step - 1); }}
		onNext={() => { args.onNext(); step = Math.min(2, step + 1); }}
	>
		<div class="mt-6 w-full">
			<label for="deck-example">Try Harper here</label>
			<textarea id="deck-example" class="mt-2 block w-full rounded-lg border bg-white p-4 text-stone-900" rows="3">This is an test.</textarea>
			<button type="button" class="mt-4 underline" aria-expanded={helpOpen} onclick={() => helpOpen = !helpOpen}>Show help</button>
		</div>
		<div slot="aside" class="grid size-48 place-items-center rounded-lg bg-primary-100 text-primary-900">Aside content</div>
		<div slot="outside" class="absolute inset-x-0 top-full pt-4 pb-6" hidden={!helpOpen}>
			<div class="rounded-lg border bg-white p-6 text-stone-900">Supplementary help outside the deck.</div>
		</div>
	</FloatingLettersSlideDeck>
{/snippet}

<Story
	name="Navigation"
	{template}
	play={async ({ canvas, args }) => {
		await expect(canvas.queryByRole('button', { name: 'Back' })).not.toBeInTheDocument();
		await expect(canvas.getByRole('progressbar')).toHaveAttribute('aria-valuenow', '0');
		const next = canvas.getByRole('button', { name: 'Next' });
		await expect(next.querySelector('svg')).toHaveAttribute('aria-hidden', 'true');
		await expect(getComputedStyle(next).backgroundColor).toBe('rgb(193, 117, 11)');
		await expect(getComputedStyle(next).borderTopLeftRadius).toBe('999px');
		await userEvent.click(next);
		await expect(args.onNext).toHaveBeenCalledTimes(1);
		await waitFor(() => expect(canvas.getByRole('heading', { name: 'Keep writing' })).toHaveFocus());
		await expect(canvas.getByRole('progressbar')).toHaveAttribute('aria-valuenow', '50');
		const back = canvas.getByRole('button', { name: 'Back' });
		await expect(back.querySelector('svg')).toHaveAttribute('aria-hidden', 'true');
		await expect(getComputedStyle(back).backgroundColor).toBe('rgba(0, 0, 0, 0)');
		await expect(getComputedStyle(back).borderTopWidth).toBe('0px');
		await expect(getComputedStyle(back.querySelector('.back-label')!).textDecorationLine).toBe('underline');
		await userEvent.keyboard('{ArrowRight}');
		await waitFor(() => expect(canvas.getByRole('heading', { name: 'All set!' })).toHaveFocus());
		await expect(canvas.getByRole('progressbar')).toHaveAttribute('aria-valuenow', '100');
		await expect(canvas.queryByRole('button', { name: 'Next' })).not.toBeInTheDocument();
		await userEvent.keyboard('{Enter}');
		await expect(args.onNext).toHaveBeenCalledTimes(2);
		await userEvent.keyboard('{ArrowLeft}');
		await expect(args.onBack).toHaveBeenCalledTimes(1);
		await userEvent.click(canvas.getByRole('button', { name: 'Back' }));
		await waitFor(() => expect(canvas.getByRole('heading', { level: 1 })).toHaveFocus());
		await userEvent.keyboard('{ArrowLeft}');
		await expect(args.onBack).toHaveBeenCalledTimes(2);
	}}
/>

<Story
	name="Disabled Next"
	args={{ slideProgress: 0.5, nextDisabled: true }}
	play={async ({ canvas, args }) => {
		await expect(canvas.getByRole('button', { name: 'Next' })).toBeDisabled();
		canvas.getByRole('heading').focus();
		await userEvent.keyboard('{ArrowRight}{Enter}');
		await expect(args.onNext).not.toHaveBeenCalled();
		await userEvent.click(canvas.getByRole('button', { name: 'Back' }));
		await expect(args.onBack).toHaveBeenCalledTimes(1);
	}}
/>

<Story
	name="Interactive Content"
	{template}
	play={async ({ canvas, args }) => {
		await userEvent.click(canvas.getByRole('textbox'));
		await userEvent.keyboard('{ArrowRight}{ArrowLeft}{Enter}');
		await expect(args.onNext).not.toHaveBeenCalled();
		await expect(args.onBack).not.toHaveBeenCalled();
		await userEvent.click(canvas.getByRole('button', { name: 'Show help' }));
		await expect(canvas.getByText('Supplementary help outside the deck.')).toBeVisible();
		await userEvent.keyboard('{ArrowRight}{ArrowLeft}');
		await expect(args.onNext).not.toHaveBeenCalled();
		canvas.getByRole('heading').focus();
		await userEvent.keyboard('{Shift>}{ArrowRight}{/Shift}');
		await expect(args.onNext).not.toHaveBeenCalled();
		await userEvent.keyboard('{Enter}');
		await expect(args.onNext).toHaveBeenCalledTimes(1);
	}}
/>

<Story name="With Aside" args={{ showAside: true }} {template} />

<Story name="Dark">
	{#snippet template(args)}
		<div class="dark"><FloatingLettersSlideDeck {...args} /></div>
	{/snippet}
</Story>

<Story name="Long Content">
	{#snippet template(args)}
		<FloatingLettersSlideDeck {...args}>
			{#each Array.from({ length: 10 }, (_, i) => i) as paragraph}
				<p class="mt-6">Paragraph {paragraph + 1}: Long onboarding content remains scrollable on small screens without clipping the heading or navigation.</p>
			{/each}
		</FloatingLettersSlideDeck>
	{/snippet}
</Story>

<Story name="Last Slide" args={{ title: 'All set!', slideProgress: 1 }} />

<Story
	name="Centered Content"
	play={async ({ canvas }) => {
		const field = canvas.getByRole('textbox');
		const bounds = field.getBoundingClientRect();
		const center = bounds.left + bounds.width / 2;
		for (const element of [canvas.getByRole('button', { name: 'Launch app' }), canvas.getByTestId('centered-card')]) {
			const rect = element.getBoundingClientRect();
			await expect(Math.abs(rect.left + rect.width / 2 - center)).toBeLessThan(1);
		}
		await expect(bounds.width).toBeGreaterThan(canvas.getByTestId('centered-card').getBoundingClientRect().width);
	}}
>
	{#snippet template(args)}
		<FloatingLettersSlideDeck {...args}>
			<button type="button" class="mt-6 rounded bg-primary-600 px-6 py-3 text-white">Launch app</button>
			<div data-testid="centered-card" class="mt-6 w-40 rounded border p-4">Centered card</div>
			<textarea aria-label="Full-width editor" class="mt-6 w-full rounded border p-4" rows="2"></textarea>
		</FloatingLettersSlideDeck>
	{/snippet}
</Story>

<Story
	name="Mount and Unmount"
	play={async ({ canvas, canvasElement, args }) => {
		const field = canvasElement.querySelector('canvas')!;
		const draw = spyOn(field.getContext('2d')!, 'clearRect');
		try {
			await waitFor(() => expect(draw).toHaveBeenCalled());
			await userEvent.click(canvas.getByRole('button', { name: 'Toggle deck' }));
			await waitFor(() => expect(canvas.queryByRole('heading')).not.toBeInTheDocument());
			draw.mockClear();
			window.dispatchEvent(new Event('resize'));
			await new Promise(requestAnimationFrame);
			await new Promise(requestAnimationFrame);
			await expect(draw).not.toHaveBeenCalled();
			await userEvent.click(canvas.getByRole('button', { name: 'Toggle deck' }));
			await waitFor(() => expect(canvas.getByRole('heading')).toBeVisible());
			canvas.getByRole('heading').focus();
			await userEvent.keyboard('{ArrowRight}');
			await expect(args.onNext).toHaveBeenCalledTimes(1);
		} finally {
			draw.mockRestore();
		}
	}}
>
	{#snippet template(args)}
		<button type="button" class="fixed top-4 right-4 z-10 rounded bg-white p-2 text-black" onclick={() => mounted = !mounted}>Toggle deck</button>
		{#if mounted}<FloatingLettersSlideDeck {...args} />{/if}
	{/snippet}
</Story>

<Story
	name="Delayed Navigation"
	play={async ({ canvas, args }) => {
		await userEvent.click(canvas.getByRole('button', { name: 'Next' }));
		await waitFor(() => expect(canvas.getByRole('heading', { name: 'Saved step' })).toHaveFocus());
		await expect(args.onNext).toHaveBeenCalledTimes(1);
	}}
>
	{#snippet template(args)}
		<FloatingLettersSlideDeck
			{...args}
			title={step === 0 ? args.title : 'Saved step'}
			slideProgress={step / 2}
			onNext={() => {
				args.onNext();
				setTimeout(() => step = Math.min(2, step + 1), 50);
			}}
		/>
	{/snippet}
</Story>
