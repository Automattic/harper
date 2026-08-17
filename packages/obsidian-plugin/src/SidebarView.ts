import { type IconName, ItemView, Menu, setIcon, type WorkspaceLeaf } from 'obsidian';
import type HarperPlugin from './index';
import { LINT_KIND_COLORS } from './lintKindColor';

export class SidebarView extends ItemView {
	constructor(leaf: WorkspaceLeaf, _plugin: HarperPlugin) {
		super(leaf);
	}

	getViewType() {
		return 'harper-sidebar-view';
	}

	getDisplayText() {
		return 'Harper Grammar';
	}

	getIcon(): IconName {
		return 'harper-logo';
	}

	async onOpen() {
		this.update();
	}

	update() {
		const container = this.containerEl;
		container.empty();

		container.style.padding = '0';
		container.style.display = 'flex';
		container.style.flexDirection = 'column';
		container.style.justifyContent = 'flex-start';

		const listContainer = document.createElement('div');

		// initial card
		const initialCard = createCard();
		const initialTitle = createTitle('Loading errors...');
		initialCard.appendChild(initialTitle);

		listContainer.appendChild(initialCard);

		listContainer.id = 'harper-error-list';
		listContainer.style.height = '100%';
		listContainer.style.overflowY = 'auto';

		container.appendChild(listContainer);

		this.registerEvent(
			this.app.workspace.on('harper:lint-updated', (errors: any[], editorView: any) => {
				listContainer.innerHTML = '';

				if (!errors || errors.length === 0) {
					const card = createCard();
					const title = createTitle('No grammar errors found!');
					card.appendChild(title);
					listContainer.appendChild(card);
					return;
				}

				errors.forEach((error) => {
					createErrorCard(error, editorView, listContainer);
				});
			}),
		);
	}

	async onClose() {}
}

function createCard(): HTMLDivElement {
	const card = document.createElement('div');
	card.style.display = 'flex';
	card.style.flexDirection = 'column';
	card.style.padding = '12px 16px';
	card.style.borderBottom = '1px solid var(--background-modifier-border)';
	card.style.gap = '8px';
	return card;
}

function createTitle(text: string, color?: string): HTMLSpanElement {
	const title = document.createElement('span');
	title.style.fontWeight = 'bold';
	if (color) {
		title.style.color = color;
	}
	title.textContent = text;
	return title;
}

function getSeverityColor(error: any) {
	// find card color
	let severityColor = '';

	if (error.markClass) {
		const classes = error.markClass.split(' ');

		const harperClass = classes.find((c: string) => c.startsWith('harper-lintRange-'));
		if (harperClass) {
			const lintKind = harperClass.replace('harper-lintRange-', '');
			if (LINT_KIND_COLORS?.[lintKind]) {
				severityColor = LINT_KIND_COLORS[lintKind];
			}
		}
	}
	return severityColor;
}

function createTitleDiv(error: any): HTMLDivElement {
	const titleDiv = document.createElement('div');
	titleDiv.style.display = 'flex';
	titleDiv.style.flexWrap = 'wrap';
	titleDiv.style.gap = '6px';
	titleDiv.style.marginTop = '4px';

	const title = createTitle(error.title || 'Harper Suggestion', getSeverityColor(error));

	// add the options ignore diagnostic and Disable Rule in a dropdown btn
	if (error.ignore || error.disable) {
		const btn = document.createElement('div');
		setIcon(btn, 'more-vertical');
		btn.style.cursor = 'var(--cursor)';
		btn.style.color = 'var(--text-muted)';
		btn.style.display = 'flex';
		btn.style.alignItems = 'center';
		btn.style.padding = '2px 4px';
		btn.style.borderRadius = 'var(--radius-s)';
		btn.style.marginLeft = 'auto';

		btn.onmouseover = () => {
			btn.style.backgroundColor = 'var(--background-modifier-hover)';
		};
		btn.onmouseout = () => {
			btn.style.backgroundColor = 'transparent';
		};

		btn.onclick = (e) => {
			const menu = new Menu();

			if (error.ignore) {
				menu.addItem((item) => {
					item
						.setTitle('Ignore Diagnostic')
						.setIcon('eye-off')
						.onClick(() => {
							error.ignore();
						});
				});
			}
			if (error.disable) {
				menu.addItem((item) => {
					item
						.setTitle('Disable Rule')
						.setIcon('ban')
						.onClick(() => {
							error.disable();
						});
				});
			}

			menu.showAtMouseEvent(e);
		};

		titleDiv.appendChild(title);
		titleDiv.appendChild(btn);
	}
	return titleDiv;
}

function getWordsArroundError(error: any, editorView: any) {
	const doc = editorView.state.doc;
	const problemText = doc.sliceString(error.from, error.to);

	// get 18 char before and after the word
	const rawPrefix = doc.sliceString(Math.max(0, error.from - 18), error.from);
	const rawSuffix = doc.sliceString(error.to, Math.min(doc.length, error.to + 18));
	// trim to be only 3 words before and after.
	let prefix = rawPrefix.split(/[.!?\n]/).pop() || '';
	const prefixWords = prefix
		.trim()
		.split(/\s+/)
		.filter((w) => w.length > 0);
	prefix = prefixWords.slice(-3).join(' ');
	if (prefix.length > 0) prefix += ' ';

	let suffix = rawSuffix.split(/[.!?\n]/)[0] || '';
	const suffixWords = suffix
		.trim()
		.split(/\s+/)
		.filter((w) => w.length > 0);
	suffix = suffixWords.slice(0, 3).join(' ');
	if (suffix.length > 0) suffix = ` ${suffix}`;

	// text container
	const textContainer = document.createElement('span');
	textContainer.style.fontSize = 'var(--font-ui-small)';

	if (prefix) {
		textContainer.appendChild(document.createTextNode(prefix));
	}

	const boldWord = document.createElement('strong');
	boldWord.textContent = problemText;
	boldWord.style.color = getSeverityColor(error);
	textContainer.appendChild(boldWord);

	if (suffix) {
		textContainer.appendChild(document.createTextNode(suffix));
	}
	return textContainer;
}

function getErrorActions(error: any, editorView: any): HTMLDivElement {
	if (error.actions && error.actions.length > 0) {
		const actionConst = document.createElement('div');
		actionConst.style.display = 'flex';
		actionConst.style.flexWrap = 'wrap';
		actionConst.style.gap = '6px';
		actionConst.style.marginTop = '4px';

		error.actions.forEach((action: any) => {
			const btn = document.createElement('button');
			btn.textContent = action.name;
			btn.title = action.title;

			btn.style.fontSize = 'var(--font-ui-smaller)';
			btn.style.cursor = 'var(--cursor)';

			btn.onclick = () => {
				action.apply(editorView, error.from, error.to);
			};

			actionConst.appendChild(btn);
		});
		return actionConst;
	}
	return document.createElement('div');
}

function createErrorCard(error: any, editorView: any, listContainer: HTMLDivElement) {
	try {
		const card = createCard();

		const titleDiv = createTitleDiv(error);
		const textContainer = getWordsArroundError(error, editorView);
		const actionConst = getErrorActions(error, editorView);

		card.appendChild(titleDiv);
		card.appendChild(textContainer);
		card.appendChild(actionConst);

		listContainer.appendChild(card);
	} catch (err) {
		console.error('Harper Sidebar failed to read: ', err, error);
	}
}
