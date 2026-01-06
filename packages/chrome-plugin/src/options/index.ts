import '../../app.css';
import { mount } from 'svelte';
import { setupTheme } from '../theme';
import App from './Options.svelte';

setupTheme();
const app = mount(App, {
	target: document.getElementById('app')!,
});

export default app;
