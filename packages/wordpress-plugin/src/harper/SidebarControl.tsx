import { useCallback, useEffect, useMemo, useState } from 'react';
import { createPortal } from 'react-dom';
import DataBlock from './DataBlock';
import Highlighter from './Highlighter';
import SidebarTabContainer from './SidebarTabContainer';
import useLintBoxes from './useLintBoxes';

export default function SidebarControl() {
	const documentContainer = useMemo<Element>(() => DataBlock.getContainer(), []);

	const [blocks, setBlocks] = useState<DataBlock[]>(DataBlock.getTerminalDataBlocks());

	const updateBlocks = useCallback(() => setBlocks(DataBlock.getTerminalDataBlocks()), []);

	useEffect(updateBlocks, [updateBlocks]);

	useEffect(() => {
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
