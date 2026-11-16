import type { Preview } from '@storybook/sveltekit';
import '../src/lib/styles.css';
import './preview.css';

const preview: Preview = {
	parameters: {
		controls: {
			matchers: {
				color: /(background|color)$/i,
				date: /Date$/i,
			},
		},
		darkMode: {
			darkClass: 'dark',
			lightClass: [],
			stylePreview: true,
		},
		a11y: {
			// 'todo' - show a11y violations in the test UI only
			// 'error' - fail CI on a11y violations
			// 'off' - skip a11y checks entirely
			test: 'todo',
		},

		layout: 'centered',
	},
};

export default preview;
