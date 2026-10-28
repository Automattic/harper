<script module lang="ts">
import { defineMeta } from '@storybook/addon-svelte-csf';
import Arrow from './icons/Arrow.svelte';
import BookIcon from './icons/BookIcon.svelte';
import CheckIcon from './icons/CheckIcon.svelte';
import ChecklistIcon from './icons/ChecklistIcon.svelte';
import ChevronLeftIcon from './icons/ChevronLeftIcon.svelte';
import ChevronRightIcon from './icons/ChevronRightIcon.svelte';
import CloseIcon from './icons/CloseIcon.svelte';
import DownloadIcon from './icons/DownloadIcon.svelte';
import EditIcon from './icons/EditIcon.svelte';
import GearIcon from './icons/GearIcon.svelte';
import GridIcon from './icons/GridIcon.svelte';
import InfoIcon from './icons/InfoIcon.svelte';
import KeyIcon from './icons/KeyIcon.svelte';
import PackageIcon from './icons/PackageIcon.svelte';
import PlayIcon from './icons/PlayIcon.svelte';
import PlusIcon from './icons/PlusIcon.svelte';
import QuillIcon from './icons/QuillIcon.svelte';
import RocketIcon from './icons/RocketIcon.svelte';
import SearchIcon from './icons/SearchIcon.svelte';
import TrashIcon from './icons/TrashIcon.svelte';
import UploadIcon from './icons/UploadIcon.svelte';

const { Story } = defineMeta({
	title: 'Components/Icons',
	tags: ['autodocs'],
	parameters: {
		layout: 'padded',
	},
});

const icons = [
	{ name: 'Arrow', component: Arrow },
	{ name: 'Book', component: BookIcon },
	{ name: 'Check', component: CheckIcon },
	{ name: 'Checklist', component: ChecklistIcon },
	{ name: 'Chevron Left', component: ChevronLeftIcon },
	{ name: 'Chevron Right', component: ChevronRightIcon },
	{ name: 'Close', component: CloseIcon },
	{ name: 'Download', component: DownloadIcon },
	{ name: 'Edit', component: EditIcon },
	{ name: 'Gear', component: GearIcon },
	{ name: 'Grid', component: GridIcon },
	{ name: 'Info', component: InfoIcon },
	{ name: 'Key', component: KeyIcon },
	{ name: 'Package', component: PackageIcon },
	{ name: 'Play', component: PlayIcon },
	{ name: 'Plus', component: PlusIcon },
	{ name: 'Quill', component: QuillIcon },
	{ name: 'Rocket', component: RocketIcon },
	{ name: 'Search', component: SearchIcon },
	{ name: 'Trash', component: TrashIcon },
	{ name: 'Upload', component: UploadIcon },
];
</script>

<Story name="All Icons" asChild>
	<div class="grid w-full grid-cols-5 gap-6">
		{#each icons as { name, component: IconComponent } (name)}
			<figure class="flex flex-col items-center gap-3 rounded-lg border border-gray-200 p-4 dark:border-gray-700">
				<div class="size-10 [&>svg]:size-full" aria-hidden="true">
					<IconComponent />
				</div>
				<figcaption class="text-center text-sm">{name}</figcaption>
			</figure>
		{/each}
	</div>
</Story>
