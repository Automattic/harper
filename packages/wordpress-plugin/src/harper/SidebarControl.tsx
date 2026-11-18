import { useCallback, useEffect, useMemo, useState } from 'react';
import { createPortal } from 'react-dom';
import DataBlock from './DataBlock';
import Highlighter from './Highlighter';
import SidebarTabContainer from './SidebarTabContainer';
import useLintBoxes from './useLintBoxes';

export default function SidebarControl() {
	const [documentContainer, setDocumentContainer] = useState<Element | null>(() =>
		DataBlock.getContainer(),
	);

	const [blocks, setBlocks] = useState<DataBlock[]>(() => DataBlock.getTerminalDataBlocks());

	const updateBlocks = useCallback(() => setBlocks(DataBlock.getTerminalDataBlocks()), []);

	useEffect(() => {
		/**
		 * Rediscover loaded or replaced editor containers. Iframe document mutations do not
		 * reach the parent observer, so iframe navigation also needs a captured load event.
		 */
		function syncContainer() {
			setDocumentContainer(DataBlock.getContainer());
		}

		const observer = new MutationObserver(syncContainer);
		observer.observe(document.documentElement, {
			childList: true,
			subtree: true,
			attributes: true,
			attributeFilter: ['class', 'name'],
		});
		document.addEventListener('load', syncContainer, true);
		syncContainer();

		return () => {
			observer.disconnect();
			document.removeEventListener('load', syncContainer, true);
		};
	}, []);

	useEffect(() => {
		updateBlocks();
		if (documentContainer === null) return;

		const observer = new MutationObserver(updateBlocks);

		observer.observe(documentContainer, {
			subtree: true,
			childList: true,
		});

		return () => observer.disconnect();
	}, [documentContainer, updateBlocks]);

	const richTexts = useMemo(() => blocks.flatMap((block) => block.getAllRichText()), [blocks]);

	const [lintBoxes, loadingLints] = useLintBoxes(richTexts);

	const highlights =
		documentContainer &&
		richTexts.map((richText, index) => {
			const boxes = lintBoxes[index] ?? [];
			return createPortal(
				<Highlighter richText={richText} key={richText.getTextContent()} lintBoxes={boxes} />,
				documentContainer,
			);
		});

	return (
		<>
			{highlights}
			<SidebarTabContainer lintBoxes={lintBoxes.flat()} loading={loadingLints} />
		</>
	);
}
