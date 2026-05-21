<script lang="ts">
export let authorName: string;
export let authorSubtitle: string;
export let testimonial: string;
export let accent = false;

const baseFigureClasses =
	'm-0 flex min-h-64 flex-col justify-between gap-4 rounded-2xl border p-6 transition duration-150 ease-out group-hover:-translate-y-px group-hover:border-primary group-hover:shadow-[0_10px_24px_-16px_rgba(0,0,0,0.16)]';
const regularFigureClasses =
	'border-black/10 bg-white text-black dark:border-white/10 dark:bg-black dark:text-white';
const accentFigureClasses =
	'border-black bg-black text-white dark:border-white dark:bg-white dark:text-black';
const baseBlockquoteClasses =
	'm-0 border-0 bg-transparent p-0 not-italic text-inherit dark:border-0 dark:bg-transparent dark:text-inherit';
const regularBlockquoteClasses = 'text-sm leading-relaxed';
const accentBlockquoteClasses = 'font-serif text-[1.3125rem] font-bold leading-snug';
const baseCaptionClasses = 'flex items-center gap-3 border-t pt-3.5';
const regularCaptionClasses = 'border-black/10 dark:border-white/10';
const accentCaptionClasses = 'border-white/15 dark:border-black/15';
const avatarClasses =
	'flex size-8 items-center justify-center rounded-full bg-primary-100 text-xs font-extrabold text-primary dark:bg-primary-900 dark:text-primary-200';

$: figureClasses = `${baseFigureClasses} ${accent ? accentFigureClasses : regularFigureClasses}`;
$: blockquoteClasses = `${baseBlockquoteClasses} ${accent ? accentBlockquoteClasses : regularBlockquoteClasses}`;
$: captionClasses = `${baseCaptionClasses} ${accent ? accentCaptionClasses : regularCaptionClasses}`;

function initials(name: string) {
	return name
		.split(' ')
		.map((part) => part[0])
		.slice(0, 2)
		.join('');
}
</script>

<figure class={figureClasses}>
	<blockquote class={blockquoteClasses}>“{testimonial}”</blockquote>
	<figcaption class={captionClasses}>
		<span class={avatarClasses}>{initials(authorName)}</span>
		<div class="flex flex-col">
			<strong class="text-[0.82rem]">{authorName}</strong>
			<small class={accent ? 'text-xs text-white/65 dark:text-black/65' : 'text-xs text-black/50 dark:text-white/50'}>
				{authorSubtitle}
			</small>
		</div>
	</figcaption>
</figure>
