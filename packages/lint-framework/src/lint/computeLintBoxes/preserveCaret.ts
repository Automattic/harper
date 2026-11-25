/** Map a UTF-16 caret offset through one replacement. */
export function mapCaretOffset(offset: number, start: number, end: number, length: number) {
	if (offset <= start) return offset;
	if (offset < end) return Math.min(offset, start + length);
	return offset + length - (end - start);
}

function getNodePath(root: Node, node: Node) {
	const path: { index: number; name: string }[] = [];
	while (node !== root) {
		const parent = node.parentNode!;
		path.push({
			index: Array.prototype.indexOf.call(parent.childNodes, node),
			name: node.nodeName,
		});
		node = parent;
	}
	return path.reverse();
}

function resolveNodePath(root: Node, path: ReturnType<typeof getNodePath>) {
	let node = root;
	for (const { index, name } of path) {
		const child = node.childNodes[index];
		if (child?.nodeName !== name) return;
		node = child;
	}
	return node;
}

/** Save a caret before the replacement temporarily selects its edit range. */
export function captureEditorCaret(editor: HTMLElement, edit: Range, replacement: string) {
	const doc = editor.ownerDocument;
	const host = doc.activeElement;
	const selection = doc.getSelection();
	const node = selection?.focusNode;
	if (
		!selection?.isCollapsed ||
		!node ||
		!(host instanceof HTMLElement) ||
		!host.isContentEditable ||
		!host.contains(node) ||
		!host.contains(edit.commonAncestorContainer)
	) {
		return;
	}

	// Use actual DOM endpoints for both positions. Lint offsets may count line breaks
	// differently from the DOM, and are not a coordinate system for restoring a caret.
	const prefix = doc.createRange();
	prefix.selectNodeContents(host);
	const offsetAt = (node: Node, offset: number) => {
		prefix.setEnd(node, offset);
		return prefix.toString().length;
	};
	const originalOffset = selection.focusOffset;
	const caret = offsetAt(node, originalOffset);
	const start = offsetAt(edit.startContainer, edit.startOffset);
	const end = offsetAt(edit.endContainer, edit.endOffset);
	const before = host.textContent ?? '';
	const expected = before.slice(0, start) + replacement + before.slice(end);
	const mapped = mapCaretOffset(caret, start, end, replacement.length);
	const path = getNodePath(host, node);
	const childNames = Array.from(node.childNodes, (child) => child.nodeName);

	const restoreAt = (target: Node, offset: number) => {
		const isText = target instanceof Text;
		const localOffset = isText ? offset - offsetAt(target, 0) : originalOffset;
		const length = isText ? target.length : target.childNodes.length;
		if (localOffset >= 0 && localOffset <= length && offsetAt(target, localOffset) === offset) {
			selection.collapse(target, localOffset);
			return true;
		}
		return false;
	};

	const hasFocus = () => host.isConnected && doc.activeElement === host;
	const restore = (offset: number) => {
		if (!hasFocus()) return;
		if (host.contains(node) && restoreAt(node, offset)) return;

		// A rebuild can discard the original node. Prefer the same DOM position
		// when its shape and mapped offset still match (including empty blocks).
		const rebuilt = resolveNodePath(host, path);
		if (
			rebuilt &&
			rebuilt.childNodes.length === childNames.length &&
			childNames.every((name, index) => rebuilt.childNodes[index]?.nodeName === name) &&
			restoreAt(rebuilt, offset)
		) {
			return;
		}

		// Text offsets can track split text nodes, but cannot identify an element
		// boundary such as a caret in an empty paragraph.
		if (!(node instanceof Text)) return;
		const walker = doc.createTreeWalker(host, NodeFilter.SHOW_TEXT);
		for (
			let text = walker.nextNode() as Text | null;
			text;
			text = walker.nextNode() as Text | null
		) {
			if (offset < text.length || (offset === text.length && originalOffset !== 0)) {
				selection.collapse(text, offset);
				return;
			}
			offset -= text.length;
		}
		selection.collapse(host, host.childNodes.length);
	};

	return (when?: 'after-editor-commit') => {
		if (!hasFocus()) return;
		const current = host.textContent;
		if (current !== before && current !== expected) return;
		if (current === before) restore(caret);
		if (when !== 'after-editor-commit' || expected === before) {
			if (current !== before) restore(mapped);
			return;
		}

		const win = doc.defaultView;
		if (!win) return;
		const abort = new AbortController();
		let frame: number | undefined;
		const stop = () => {
			observer.disconnect();
			clearTimeout(timeout);
			if (frame !== undefined) win.cancelAnimationFrame(frame);
			abort.abort();
		};
		const checkCommit = () => {
			const text = host.textContent;
			if (!hasFocus() || (text !== expected && (text !== before || frame !== undefined))) {
				stop();
				return;
			}
			if (text !== expected || frame !== undefined) return;

			restore(mapped);
			// The host can set its selection in the frame after committing text.
			// Reapply once after that frame, unless something cancels the restore.
			frame = win.requestAnimationFrame(() => {
				frame = win.requestAnimationFrame(() => {
					stop();
					if (host.textContent === expected) restore(mapped);
				});
			});
		};
		const observer = new MutationObserver(checkCommit);
		const timeout = setTimeout(stop, 1000);
		const options = { capture: true, signal: abort.signal };
		doc.addEventListener(
			'keydown',
			() => {
				// A key can arrive between the host's selection write and our next frame.
				// Finish the restore before the editor handles the key, not afterward.
				stop();
				if (host.textContent === expected) restore(mapped);
			},
			options,
		);
		doc.addEventListener('pointerdown', stop, options);
		host.addEventListener('blur', stop, options);
		win.addEventListener('blur', stop, { signal: abort.signal });
		observer.observe(host, { subtree: true, childList: true, characterData: true });
		checkCommit();
	};
}
