<script lang="ts">
export let title: string;
export let dark = false;

const windowBaseClasses =
	'relative overflow-hidden rounded-xl border shadow-[0_22px_50px_-18px_rgba(0,0,0,0.35),0_4px_10px_rgba(0,0,0,0.1),0_0_0_0.5px_rgba(0,0,0,0.04)]';
$: windowClasses = dark
	? `${windowBaseClasses} border-white/10 bg-[#2b2d31] text-[#dcddde]`
	: `${windowBaseClasses} border-black/20 bg-white text-black`;
$: titlebarClasses = dark
	? 'flex h-[1.65rem] items-center gap-2 border-b border-white/10 bg-[#1e1f22] px-2.5'
	: 'flex h-[1.65rem] items-center gap-2 border-b border-black/10 bg-linear-to-b from-[#fafafa] to-[#efece7] px-2.5';
$: titleClasses = dark
	? 'mr-[2.1rem] flex-1 text-center text-[0.68rem] text-white/70'
	: 'mr-[2.1rem] flex-1 text-center text-[0.68rem] text-black/55';
</script>

<div class={windowClasses}>
	<div class={titlebarClasses}>
		<div class="flex gap-1.5">
			<span class="size-[0.55rem] rounded-full bg-[#ff5f57]"></span>
			<span class="size-[0.55rem] rounded-full bg-[#febc2e]"></span>
			<span class="size-[0.55rem] rounded-full bg-[#28c840]"></span>
		</div>
		<strong class={titleClasses}>{title}</strong>
	</div>
	<div class="relative min-h-40"><slot /></div>
</div>
