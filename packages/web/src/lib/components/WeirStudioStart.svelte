<script lang="ts">
import { Button, Card } from 'components';

export let onUpload: () => void;
export let onOpenExample: () => void;
export let onCreateEmpty: () => void;
export let onUploadChange: (event: Event) => void;
export let fileInputEl: HTMLInputElement | null = null;
</script>

<div class="absolute inset-0 z-30 flex items-center justify-center bg-[#fef4e7]/95">
	<Card className="w-[min(640px,90vw)] space-y-6 border-black/10 bg-white/95 p-8">
		<div class="space-y-2">
			<div class="text-xs font-semibold uppercase tracking-[0.25em] text-black/50">Weir Studio</div>
			<h2 class="text-2xl font-semibold text-black">Choose how to start</h2>
			<p class="text-sm text-black/60">
				Load a Weirpack to edit rules, run tests, and export updates.
			</p>
		</div>

		<div class="grid gap-3">
			<input
				class="hidden"
				type="file"
				accept=".weirpack,application/zip"
				bind:this={fileInputEl}
				on:change={onUploadChange}
			/>
			<Button color="dark" size="md" className="w-full justify-between" on:click={onUpload}>
				<span>Upload an existing Weirpack</span>
				<span class="text-xs uppercase tracking-[0.2em] text-white/60">Upload</span>
			</Button>
			<Button color="white" size="md" className="w-full justify-between" on:click={onOpenExample}>
				<span>Open example Weirpack</span>
				<span class="text-xs uppercase tracking-[0.2em] text-black/40">Example</span>
			</Button>
			<Button color="white" size="md" className="w-full justify-between" on:click={onCreateEmpty}>
				<span>Create an empty Weirpack</span>
				<span class="text-xs uppercase tracking-[0.2em] text-black/40">Empty</span>
			</Button>
		</div>
	</Card>
</div>
