const NOTICE_MS = 2000;

let current: { host: HTMLElement; timer: number } | null = null;

/** Whether this frame holds the keyboard focus (and not one of its child frames). */
export function frameHasFocus(): boolean {
	const active = document.activeElement;
	return (
		document.hasFocus() &&
		!(active instanceof HTMLIFrameElement || active instanceof HTMLFrameElement)
	);
}

/**
 * Briefly show `text` at the top right corner of the focused element. Thunderbird's compose window
 * has no toolbar icon, so this is the only place a language switch can be seen there.
 */
export function showLanguageNotice(text: string): void {
	hideLanguageNotice();

	const host = document.createElement('harper-language-notice');
	// Outside <body>: in an editable body (e.g. an e-mail being written) it would become content.
	host.setAttribute('contenteditable', 'false');
	const shadow = host.attachShadow({ mode: 'closed' });

	const box = document.createElement('div');
	box.setAttribute('role', 'status');
	box.textContent = text;
	Object.assign(box.style, {
		position: 'fixed',
		zIndex: '2147483647',
		padding: '4px 10px',
		borderRadius: '6px',
		background: '#1f2937',
		color: '#ffffff',
		font: '13px/1.4 system-ui, sans-serif',
		boxShadow: '0 2px 8px rgba(0, 0, 0, 0.3)',
		pointerEvents: 'none',
		whiteSpace: 'nowrap',
	});
	shadow.append(box);
	document.documentElement.append(host);

	const anchor = noticeAnchor();
	const width = box.offsetWidth;
	box.style.top = `${Math.max(8, Math.min(anchor.top + 8, window.innerHeight - 40))}px`;
	box.style.left = `${Math.max(8, Math.min(anchor.right, window.innerWidth) - width - 8)}px`;

	current = { host, timer: window.setTimeout(hideLanguageNotice, NOTICE_MS) };
}

function hideLanguageNotice(): void {
	if (current == null) {
		return;
	}

	window.clearTimeout(current.timer);
	current.host.remove();
	current = null;
}

/** The rectangle of the focused element, or of the viewport when nothing in particular has focus. */
function noticeAnchor(): { top: number; right: number } {
	const active = document.activeElement;
	if (active == null || active === document.documentElement) {
		return { top: 0, right: window.innerWidth };
	}

	const rect = active.getBoundingClientRect();
	return { top: Math.max(0, rect.top), right: rect.right };
}
