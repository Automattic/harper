import { Panel, PanelBody, TabPanel } from '@wordpress/components';
import type { IgnorableLintBox } from './Box';
import DialectSelectRow from './DialectSelectRow';
import LintList from './LintList';
import LintSettingList from './LintSettingList';

export default function SidebarTabContainer({
	lintBoxes,
	loading,
}: {
	lintBoxes: IgnorableLintBox[];
	loading: boolean;
}) {
	return (
		<TabPanel
			tabs={[
				{ name: 'errors', title: 'Errors' },
				{ name: 'settings', title: 'Settings' },
			]}
		>
			{(tab) => {
				switch (tab.name) {
					case 'errors':
						return <LintList lintBoxes={lintBoxes} loading={loading} />;
					case 'settings':
						return (
							<Panel>
								<PanelBody title="Rules">
									<DialectSelectRow />
									<LintSettingList />
								</PanelBody>
							</Panel>
						);
				}
			}}
		</TabPanel>
	);
}
