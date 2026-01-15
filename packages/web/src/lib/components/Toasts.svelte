<script lang="ts">
import { Card } from 'components';

export type ToastTone = 'success' | 'error' | 'info';
export type Toast = {
	id: number;
	title: string;
	body?: string;
	tone: ToastTone;
};

export let toasts: Toast[] = [];
export let className: string | undefined = undefined;
</script>

<div class={`fixed bottom-24 right-6 z-20 flex w-[320px] flex-col gap-3 ${className ?? ''}`}>
	{#each toasts as toast (toast.id)}
		<Card
			className={`border px-4 py-3 text-sm shadow-xl ${
				toast.tone === 'success'
					? 'border-green-200 bg-green-50 text-green-900'
					: toast.tone === 'error'
						? 'border-red-200 bg-red-50 text-red-900'
						: 'border-black/10 bg-white text-black'
			}`}
		>
			<div class="text-sm font-semibold">{toast.title}</div>
			{#if toast.body}
				<div class="mt-1 text-xs leading-snug text-black/70">{toast.body}</div>
			{/if}
		</Card>
	{/each}
</div>
