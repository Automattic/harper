import { Animate, Spinner } from '@wordpress/components';
import type { IgnorableLintBox } from './Box';
import LintListItem from './LintListItem';

export default function LintList({
	lintBoxes,
	loading,
}: {
	lintBoxes: IgnorableLintBox[];
	loading: boolean;
}) {
	if (lintBoxes.length === 0) {
		return (
			<div className="harper-solved-cont">
				<Animate type="appear" options={{ origin: 'middle' }}>
					{({ className }) => (
						<div className={className ?? ''}>
							{loading ? (
								<Spinner
									style={{
										height: 'calc(4px * 20)',
										width: 'calc(4px * 20)',
									}}
								/>
							) : (
								<>
									<h2>LGTM 👍</h2>
									<p>Harper could not find any problems with your work.</p>
								</>
							)}
						</div>
					)}
				</Animate>
			</div>
		);
	}

	return (
		<>
			{lintBoxes
				.filter((box) => box.lint.suggestion_count() > 0)
				.map((box, index) => (
					<LintListItem key={index} box={box} />
				))}
		</>
	);
}
